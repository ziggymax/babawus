//! The medium's lock. See "One Babawus at a time" in Requirements.md.
//!
//! A lock that isn't live never stops anyone, so nothing here needs flushing: if an interruption
//! loses the lock, brings back a removed one or leaves half of one, the next action just takes
//! over.
//!
//! Between creating the lock file and writing to it, the lock looks left over. Only another
//! Babawus on this computer could see that, and the lock in `/run/lock/` normally rules that out.
//! In case that lock can't be used, a new lock is read back, and only counts if it is still ours.

use std::collections::BTreeSet;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::fs::FileSystem;
use crate::medium::{FILE_MODE, LOCK_FILE, read_file, set_mode};
use crate::platform;
use crate::versions::Format;

/// The runs of this process that hold a lock. A lock with this process's ID and any other run ID
/// is left over, for example because the medium was pulled during a backup.
static HELD_RUNS: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

fn held_runs() -> MutexGuard<'static, BTreeSet<String>> {
    HELD_RUNS.lock().unwrap_or_else(|error| error.into_inner())
}

/// What a lock needs to know about this process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThisProcess {
    pub computer_id: String,
    pub boot_id: String,
    pub process_id: u32,
    /// The name of the running program, to tell Babawus from a process that got a reused ID.
    pub program: String,
}

impl ThisProcess {
    /// Reads the details from `system`, which is the computer's own file system.
    pub fn read(system: &dyn FileSystem, computer_id: &str) -> io::Result<Self> {
        Ok(Self {
            computer_id: computer_id.to_owned(),
            boot_id: platform::boot_id(system)?,
            process_id: std::process::id(),
            program: platform::own_program(system)?,
        })
    }

    fn is_live(&self, system: &dyn FileSystem, holder: &Holder) -> bool {
        holder.computer_id == self.computer_id
            && holder.boot_id == self.boot_id
            && if holder.process_id == self.process_id {
                held_runs().contains(&holder.run_id)
            } else {
                self.runs_this_program(system, holder.process_id)
            }
    }

    /// Whether `process_id` is a running process of this program. Without a readable name, it
    /// has ended.
    fn runs_this_program(&self, system: &dyn FileSystem, process_id: u32) -> bool {
        platform::program_of(system, process_id).is_ok_and(|program| program == self.program)
    }
}

/// What a lock file records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holder {
    pub run_id: String,
    /// `username@computer folder`. See [`account_name`](crate::log::account_name).
    pub account: String,
    pub computer_id: String,
    pub boot_id: String,
    pub process_id: u32,
}

#[derive(Debug)]
pub enum LockError {
    /// A live lock is in the way. Without a holder, another Babawus is just taking the lock.
    Held(Option<Holder>),
    Io(io::Error),
}

impl From<io::Error> for LockError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// The holder of the lock file. A file that can't be understood has none.
fn read_holder(medium: &dyn FileSystem, root: &Path) -> io::Result<Option<Holder>> {
    match read_file(medium, &root.join(LOCK_FILE)) {
        Ok(data) => Ok(str::from_utf8(&data)
            .ok()
            .and_then(|text| toml::from_str(text).ok())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// The holder of the medium's lock, if that lock is live. This only reads, so it is safe to use
/// for the media list.
pub fn live_holder(
    medium: &dyn FileSystem,
    root: &Path,
    system: &dyn FileSystem,
    this: &ThisProcess,
) -> io::Result<Option<Holder>> {
    let holder = read_holder(medium, root)?;
    Ok(holder.filter(|holder| this.is_live(system, holder)))
}

/// The lock on a medium, held until this is dropped.
pub struct MediumLock<'a> {
    medium: &'a dyn FileSystem,
    root: &'a Path,
    run_id: String,
}

impl<'a> MediumLock<'a> {
    /// Takes the medium's lock, and removes a lock that isn't live. `system` is the computer's
    /// own file system.
    pub fn take(
        medium: &'a dyn FileSystem,
        root: &'a Path,
        format: Format,
        system: &dyn FileSystem,
        this: &ThisProcess,
        run_id: &str,
        account: &str,
    ) -> Result<Self, LockError> {
        if let Some(holder) = live_holder(medium, root, system, this)? {
            return Err(LockError::Held(Some(holder)));
        }
        let path = root.join(LOCK_FILE);
        match medium.remove_file(&path) {
            Err(error) if error.kind() != ErrorKind::NotFound => return Err(error.into()),
            _ => {}
        }
        let holder = Holder {
            run_id: run_id.to_owned(),
            account: account.to_owned(),
            computer_id: this.computer_id.clone(),
            boot_id: this.boot_id.clone(),
            process_id: this.process_id,
        };
        let text = toml::to_string(&holder).map_err(io::Error::other)?;
        let mut file = medium.create_new(&path)?;
        let written = set_mode(medium, format, &path, FILE_MODE)
            .and_then(|()| file.write_all(text.as_bytes()));
        if let Err(error) = written {
            // If this fails too, what is left isn't a live lock.
            let _ = medium.remove_file(&path);
            return Err(error.into());
        }
        drop(file);
        match read_holder(medium, root)? {
            Some(found) if found.run_id == run_id => {}
            // The file is no longer ours, so it stays as it is.
            other => return Err(LockError::Held(other)),
        }
        held_runs().insert(run_id.to_owned());
        Ok(Self {
            medium,
            root,
            run_id: run_id.to_owned(),
        })
    }
}

impl Drop for MediumLock<'_> {
    /// Removes the lock. If that fails, for example because the medium is gone, what stays behind
    /// is no longer live.
    fn drop(&mut self) {
        let holder = read_holder(self.medium, self.root);
        if holder.is_ok_and(|holder| holder.is_some_and(|holder| holder.run_id == self.run_id)) {
            let _ = self.medium.remove_file(&self.root.join(LOCK_FILE));
        }
        held_runs().remove(&self.run_id);
    }
}

#[cfg(test)]
mod tests;
