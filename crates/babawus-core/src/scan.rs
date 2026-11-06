//! The single pass over the source that finds what a backup has to do.
//!
//! See "What is copied", "Exclusions" and "Names that cannot be stored" in Requirements.md.

use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::copy::{Skip, differs, source_skip};
use crate::exclude::Exclusions;
use crate::fs::{FileKind, FileSystem, Metadata};
use crate::names::{MAX_PATH, fold, name_problem};
use crate::versions::Format;

/// What an account wants backed up.
#[derive(Clone, Debug)]
pub struct Selection {
    pub home: PathBuf,
    /// Selected files and folders, relative to the home folder.
    pub selected: Vec<PathBuf>,
    pub exclusions: Exclusions,
}

/// Where the backup goes.
pub struct Destination<'a> {
    pub medium: &'a dyn FileSystem,
    /// The root folder of the medium.
    pub root: PathBuf,
    /// The folder for this account, relative to the root. See [`account_folder`].
    pub account: PathBuf,
    pub format: Format,
}

/// `backup/<computer folder>/<username>`, with the user name in lower case, because exFAT does
/// not distinguish case and both file systems must behave the same.
pub fn account_folder(computer_folder: &str, username: &str) -> PathBuf {
    let username = username.to_lowercase();
    Path::new("backup").join(computer_folder).join(username)
}

/// Something to back up. Paths are relative to the home folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    File {
        relative: PathBuf,
        len: u64,
        /// An unchanged file is still visited, because its version folder may need pruning.
        needs_copy: bool,
    },
    Folder {
        relative: PathBuf,
    },
}

/// A file or folder that can't be backed up. A skipped folder is one entry, with the number of
/// files in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    pub relative: PathBuf,
    pub reason: Skip,
    pub files: u64,
}

/// What a scan found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scan {
    pub entries: Vec<Entry>,
    pub skipped: Vec<Skipped>,
    pub files_scanned: u64,
    pub files_to_copy: u64,
    pub bytes_to_copy: u64,
}

/// Scans the selection. Returns `None` if `cancel` was set.
pub fn scan(
    source: &dyn FileSystem,
    selection: &Selection,
    destination: &Destination,
    cancel: &AtomicBool,
) -> Option<Scan> {
    let mut selected = selection.selected.clone();
    selected.sort();
    let mut scanner = Scanner {
        source,
        selection,
        destination,
        cancel,
        seen: HashSet::new(),
        claimed: HashSet::new(),
        result: Scan::default(),
    };
    for relative in selected {
        scanner.selected(&relative)?;
    }
    Some(scanner.result)
}

struct Scanner<'a> {
    source: &'a dyn FileSystem,
    selection: &'a Selection,
    destination: &'a Destination<'a>,
    cancel: &'a AtomicBool,
    /// Paths already handled, because selections can overlap.
    seen: HashSet<PathBuf>,
    /// On exFAT, the paths taken on the medium, in the form that ignores case.
    claimed: HashSet<String>,
    result: Scan,
}

impl Scanner<'_> {
    fn source_path(&self, relative: &Path) -> PathBuf {
        self.selection.home.join(relative)
    }

    fn medium_path(&self, relative: &Path) -> PathBuf {
        let destination = self.destination;
        destination.root.join(&destination.account).join(relative)
    }

    fn cancelled(&self) -> Option<()> {
        (!self.cancel.load(Ordering::Relaxed)).then_some(())
    }

    fn skip(&mut self, relative: &Path, reason: Skip, files: u64) {
        self.result.skipped.push(Skipped {
            relative: relative.to_owned(),
            reason,
            files,
        });
    }

    /// Handles one selected file or folder. Returns `None` if the scan was cancelled.
    fn selected(&mut self, relative: &Path) -> Option<()> {
        self.cancelled()?;
        let inside_home = relative
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
        if !inside_home {
            self.skip(relative, Skip::OutsideHome, 0);
            return Some(());
        }
        if self.selection.exclusions.excludes(relative) {
            return Some(());
        }
        let metadata = match self.source.metadata(&self.source_path(relative)) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.skip(relative, Skip::SelectionMissing, 0);
                return Some(());
            }
            Err(_) => {
                self.skip(relative, Skip::Unreadable, 0);
                return Some(());
            }
        };
        if metadata.kind == FileKind::Other {
            return Some(());
        }
        let format = self.destination.format;
        if let Some(reason) = relative.iter().find_map(|part| name_problem(part, format)) {
            let files = self.count_files(relative, &metadata, metadata.device);
            self.skip(relative, reason, files);
            return Some(());
        }
        // A selected folder may be on any file system. Only what is below it has to match.
        self.visit(relative, &metadata, metadata.device)
    }

    /// Handles a file or folder that has passed the exclusions and name rules.
    fn visit(&mut self, relative: &Path, metadata: &Metadata, device: u64) -> Option<()> {
        if !self.seen.insert(relative.to_owned()) {
            return Some(());
        }
        if self.destination.format == Format::ExFat
            && !self.claimed.insert(fold(&relative.to_string_lossy()))
        {
            let files = self.count_files(relative, metadata, device);
            self.skip(relative, Skip::CaseClash, files);
            return Some(());
        }
        if self.medium_path(relative).as_os_str().len() > MAX_PATH {
            let files = self.count_files(relative, metadata, device);
            self.skip(relative, Skip::PathTooLong, files);
            return Some(());
        }
        match metadata.kind {
            FileKind::File => self.file(relative, metadata),
            FileKind::Dir => return self.folder(relative, device),
            FileKind::Other => {}
        }
        Some(())
    }

    fn file(&mut self, relative: &Path, metadata: &Metadata) {
        let copy = self
            .destination
            .medium
            .metadata(&self.medium_path(relative));
        let needs_copy = match copy {
            Ok(copy) => differs(metadata, &copy, self.destination.format),
            Err(_) => true,
        };
        self.result.files_scanned += 1;
        if needs_copy {
            self.result.files_to_copy += 1;
            self.result.bytes_to_copy += metadata.len;
        }
        self.result.entries.push(Entry::File {
            relative: relative.to_owned(),
            len: metadata.len,
            needs_copy,
        });
    }

    fn folder(&mut self, relative: &Path, device: u64) -> Option<()> {
        let names = match self.source.read_dir(&self.source_path(relative)) {
            Ok(names) => names,
            Err(error) => {
                self.skip(relative, source_skip(&error), 0);
                return Some(());
            }
        };
        self.result.entries.push(Entry::Folder {
            relative: relative.to_owned(),
        });
        for name in names {
            self.cancelled()?;
            let child = relative.join(&name);
            if self.selection.exclusions.excludes(&child) {
                continue;
            }
            let metadata = match self.source.metadata(&self.source_path(&child)) {
                Ok(metadata) => metadata,
                Err(error) => {
                    self.skip(&child, source_skip(&error), 0);
                    continue;
                }
            };
            let other_device = metadata.kind == FileKind::Dir && metadata.device != device;
            if metadata.kind == FileKind::Other || other_device {
                continue;
            }
            if let Some(reason) = name_problem(&name, self.destination.format) {
                let files = self.count_files(&child, &metadata, device);
                self.skip(&child, reason, files);
                continue;
            }
            self.visit(&child, &metadata, device)?;
        }
        Some(())
    }

    /// The number of files in a skipped folder, or 1 for a skipped file.
    fn count_files(&self, relative: &Path, metadata: &Metadata, device: u64) -> u64 {
        match metadata.kind {
            FileKind::File => 1,
            FileKind::Other => 0,
            FileKind::Dir => {
                let names = self.source.read_dir(&self.source_path(relative));
                let children = names
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| relative.join(name));
                children
                    .filter(|child| !self.selection.exclusions.excludes(child))
                    .filter_map(|child| {
                        let metadata = self.source.metadata(&self.source_path(&child)).ok()?;
                        let other_device =
                            metadata.kind == FileKind::Dir && metadata.device != device;
                        (!other_device).then(|| self.count_files(&child, &metadata, device))
                    })
                    .sum()
            }
        }
    }
}

#[cfg(test)]
mod tests;
