//! A whole backup, from taking the lock to the verdict.
//!
//! See "Start a backup on a specific medium", "Get a clear verdict", "Finishing a run" and
//! "Responsiveness" in Requirements.md.
//!
//! [`Backup::run`] does the work and reports to a function. [`Backup::start`] does the same on a
//! worker thread, for the UI.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::SystemTime;

use crate::config::{Config, Loaded, load, save};
use crate::copy::{FileCopier, Pruned, Skip, Visit, stop_for};
use crate::fs::FileSystem;
use crate::lock::{Holder, LockError, MediumLock, ThisProcess};
use crate::log::{PRUNE_FAILED, RunLog, Runner, Stats, account_name, new_run_id};
use crate::scan::{Destination, Entry, Scan, Selection, Skipped, account_folder, scan};
use crate::time::{log_timestamp, version_timestamp};
use crate::verdict::{Reason, Verdict, worst};
use crate::versions::Format;

/// What a run tells the UI while it works.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    /// The run has started, and is finding out what to do.
    Scanning,
    /// What the run is going to copy.
    Overview { files: u64, bytes: u64 },
    /// What is left. Only bytes that have reached the medium count as done.
    Copying { files_left: u64, bytes_left: u64 },
    /// Everything is being flushed to the medium, which can take several seconds.
    Finishing,
}

/// How a run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub reason: Reason,
    pub stats: Stats,
    /// What could not be backed up.
    pub skipped: Vec<Skipped>,
    /// Files whose older versions could not all be removed. These don't change the verdict.
    pub prune_failures: Vec<PathBuf>,
}

impl Outcome {
    pub fn verdict(&self) -> Verdict {
        self.reason.verdict()
    }

    /// A run that ended before it had a log.
    fn early(reason: Reason) -> Self {
        Self {
            reason,
            stats: Stats::default(),
            skipped: Vec::new(),
            prune_failures: Vec::new(),
        }
    }
}

/// What came of asking for a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ended {
    /// Another Babawus holds the medium's lock, or is just taking it, which is when there is no
    /// holder yet. Nothing was done, so there is no verdict.
    Busy(Option<Holder>),
    Finished(Outcome),
}

/// A message from a run on a worker thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Progress(Progress),
    /// The last message. The medium's lock has already been released.
    Ended(Ended),
}

/// A backup of one account to one medium.
#[derive(Clone)]
pub struct Backup {
    pub medium: Arc<dyn FileSystem>,
    /// The root folder of the medium.
    pub root: PathBuf,
    pub format: Format,
    /// The computer's own file system, with the source files.
    pub system: Arc<dyn FileSystem>,
    /// If this computer has no computer ID, `computer_id` is empty, and it counts as not added.
    pub this: ThisProcess,
    pub username: String,
    pub hostname: String,
    pub home: PathBuf,
    pub now: Arc<dyn Fn() -> SystemTime + Send + Sync>,
}

/// Where an account's backup goes, and what it holds.
struct Plan {
    folder: PathBuf,
    selection: Selection,
}

impl Backup {
    /// The account's name in the log and the lock, and its plan if it has been added.
    fn plan(&self, config: &Config) -> (String, Option<Plan>) {
        let id = &self.this.computer_id;
        let Some((computer, account)) = config.account(id, &self.username) else {
            let computer = config.computer(id);
            let folder = computer.map_or(&self.hostname, |computer| &computer.folder);
            return (account_name(&self.username, folder), None);
        };
        let plan = Plan {
            folder: account_folder(&computer.folder, &self.username),
            selection: account.selection(&self.home),
        };
        (account_name(&self.username, &computer.folder), Some(plan))
    }

    /// Runs the backup. `cancel` can be set at any time, and `progress` hears about every step.
    ///
    /// The configuration is read before taking the lock, so that a medium that isn't in order is
    /// not touched at all: it may need a newer Babawus. Nothing can change it in between, because
    /// only one Babawus runs on a computer, and that one is here.
    pub fn run(&self, cancel: &AtomicBool, progress: &mut dyn FnMut(Progress)) -> Ended {
        progress(Progress::Scanning);
        let medium = &*self.medium;
        let early = |reason| Ended::Finished(Outcome::early(reason));
        let stopped = |error: &_| early(stop_for(medium, &self.root, error).into());
        let mut config = match load(medium, &self.root) {
            Ok(Loaded::Ready(config)) => config,
            // Where the medium was mounted may have gone with it, which also looks like no file.
            Ok(Loaded::Missing) if medium.metadata(&self.root).is_err() => {
                return early(Reason::MediumRemoved);
            }
            Ok(Loaded::Missing) => return early(Reason::NotAdded),
            Ok(Loaded::Damaged(_) | Loaded::NewerFormat) => return early(Reason::ConfigDamaged),
            Err(error) => return stopped(&error),
        };
        let (account, plan) = self.plan(&config);
        let run_id = new_run_id();
        let lock = MediumLock::take(
            medium,
            &self.root,
            self.format,
            &*self.system,
            &self.this,
            &run_id,
            &account,
        );
        let lock = match lock {
            Ok(lock) => lock,
            Err(LockError::Held(holder)) => return Ended::Busy(holder),
            Err(LockError::Io(error)) => return stopped(&error),
        };

        let version_now = || version_timestamp((self.now)());
        let log_now = || log_timestamp((self.now)());
        let copier = FileCopier::new(
            &*self.system,
            medium,
            self.root.clone(),
            self.format,
            config.older_versions as usize,
            &version_now,
            cancel,
        );
        if let Err(stop) = copier.empty_temp_folder() {
            return early(stop.into());
        }
        let log = RunLog::start(
            medium,
            &self.root,
            self.format,
            run_id,
            &Runner {
                account: &account,
                hostname: &self.hostname,
            },
            &log_now,
            None,
        );
        let log = match log {
            Ok(log) => log,
            Err(error) => return stopped(&error),
        };

        // The hostname is for display only, so this must not change the verdict. If the medium
        // has a problem, the next step finds out, and the next run tries again.
        if config.note_hostname(&self.this.computer_id, &self.hostname) {
            let _ = save(medium, &self.root, self.format, &config);
        }

        let mut pass = Pass {
            cancel,
            progress,
            home: &self.home,
            log,
            stats: Stats::default(),
            skipped: Vec::new(),
            prune_failures: Vec::new(),
            files_left: 0,
            bytes_left: 0,
        };
        let stop = match &plan {
            Some(plan) => pass.back_up(self, plan, copier),
            None => Some(Reason::NotAdded),
        };
        let outcome = pass.finish(self, stop);
        // Only now: the lock is removed after the end entry has been written.
        drop(lock);
        Ended::Finished(outcome)
    }

    /// Runs the backup on a worker thread.
    pub fn start(self) -> RunningBackup {
        let (sender, messages) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        thread::spawn(move || {
            let ended = self.run(&flag, &mut |progress| {
                // The UI may have gone away. The run still finishes properly.
                let _ = sender.send(Message::Progress(progress));
            });
            let _ = sender.send(Message::Ended(ended));
        });
        RunningBackup { messages, cancel }
    }
}

/// A backup on a worker thread.
pub struct RunningBackup {
    /// Ends with [`Message::Ended`]. If it stops without one, the worker has panicked, which
    /// leaves the same state as any other interruption.
    pub messages: Receiver<Message>,
    cancel: Arc<AtomicBool>,
}

impl RunningBackup {
    /// Asks the run to stop. It ends with the verdict "Backup not finished".
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// The part of a run that has a log.
struct Pass<'a> {
    cancel: &'a AtomicBool,
    progress: &'a mut dyn FnMut(Progress),
    home: &'a Path,
    log: RunLog<'a>,
    stats: Stats,
    skipped: Vec<Skipped>,
    prune_failures: Vec<PathBuf>,
    files_left: u64,
    bytes_left: u64,
}

impl Pass<'_> {
    /// Scans and copies. Returns what stopped the run early, if anything did.
    fn back_up(&mut self, backup: &Backup, plan: &Plan, mut copier: FileCopier) -> Option<Reason> {
        let destination = Destination {
            medium: &*backup.medium,
            root: backup.root.clone(),
            account: plan.folder.clone(),
            format: backup.format,
        };
        let Some(found) = scan(&*backup.system, &plan.selection, &destination, self.cancel) else {
            return Some(Reason::Cancelled);
        };
        self.overview(&found);
        for entry in &found.entries {
            if self.cancel.load(Ordering::Relaxed) {
                return Some(Reason::Cancelled);
            }
            if let Err(reason) = self.entry(&mut copier, &plan.folder, entry) {
                return Some(reason);
            }
        }
        None
    }

    fn overview(&mut self, found: &Scan) {
        self.stats.scanned_files = found.files_scanned;
        self.files_left = found.files_to_copy;
        self.bytes_left = found.bytes_to_copy;
        (self.progress)(Progress::Overview {
            files: found.files_to_copy,
            bytes: found.bytes_to_copy,
        });
        for skipped in &found.skipped {
            self.skip(&skipped.relative, skipped.reason, skipped.files);
        }
    }

    /// Backs up one file or folder. An error ends the run.
    fn entry(
        &mut self,
        copier: &mut FileCopier,
        folder: &Path,
        entry: &Entry,
    ) -> Result<(), Reason> {
        let (relative, len, needs_copy) = match entry {
            Entry::Folder { relative } => {
                if let Some(skip) = copier.visit_folder(&folder.join(relative))? {
                    self.skip(relative, skip, 0);
                }
                return Ok(());
            }
            Entry::File {
                relative,
                len,
                needs_copy,
            } => (relative, *len, *needs_copy),
        };
        let mut flushed = 0;
        let (files_left, bytes_left) = (self.files_left, self.bytes_left);
        let progress = &mut self.progress;
        let visit = copier.visit(
            &self.home.join(relative),
            &folder.join(relative),
            &mut |bytes| {
                flushed = bytes;
                progress(Progress::Copying {
                    files_left,
                    bytes_left: bytes_left.saturating_sub(bytes),
                });
            },
        )?;
        match visit {
            Visit::Unchanged(pruned) => self.pruned(relative, pruned),
            Visit::Copied(pruned) => {
                self.stats.copied_files += 1;
                self.stats.copied_bytes += flushed;
                self.pruned(relative, pruned);
            }
            Visit::Skipped(skip) => self.skip(relative, skip, 1),
            Visit::Cancelled => return Err(Reason::Cancelled),
        }
        if needs_copy {
            self.files_left = self.files_left.saturating_sub(1);
            self.bytes_left = self.bytes_left.saturating_sub(len);
            (self.progress)(Progress::Copying {
                files_left: self.files_left,
                bytes_left: self.bytes_left,
            });
        }
        Ok(())
    }

    /// Writes an anomaly to the log. If that fails, the run goes on: if the medium has a problem,
    /// the next step finds out, and if it is only the log, the end entry does.
    fn log_error(&mut self, code: &str, path: &Path, files: u64) {
        let _ = self.log.error(code, path, files);
    }

    fn skip(&mut self, relative: &Path, reason: Skip, files: u64) {
        self.log_error(reason.code(), relative, files);
        self.stats.skipped_files += files;
        self.skipped.push(Skipped {
            relative: relative.to_owned(),
            reason,
            files,
        });
    }

    fn pruned(&mut self, relative: &Path, pruned: Pruned) {
        self.stats.versions_removed += pruned.removed;
        if pruned.failed {
            self.log_error(PRUNE_FAILED, relative, 1);
            self.prune_failures.push(relative.to_owned());
        }
    }

    /// Flushes the medium, and then writes the end entry. An end entry that can't be written
    /// counts as a failed flush, because a later run will report this run as not finished.
    fn finish(mut self, backup: &Backup, stop: Option<Reason>) -> Outcome {
        (self.progress)(Progress::Finishing);
        let skipped = (!self.skipped.is_empty()).then_some(Reason::FilesSkipped);
        let mut reason = worst(stop.into_iter().chain(skipped));
        if backup.medium.sync_medium(&backup.root).is_err() {
            reason = worst([reason, Reason::FlushFailed]);
        }
        self.stats.free_bytes = backup.medium.free_space(&backup.root).unwrap_or(0);
        self.stats.errors = self.log.errors();
        if self.log.end(reason, self.stats).is_err() {
            reason = worst([reason, Reason::FlushFailed]);
        }
        Outcome {
            reason,
            stats: self.stats,
            skipped: self.skipped,
            prune_failures: self.prune_failures,
        }
    }
}

#[cfg(test)]
mod tests;
