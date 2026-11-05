//! Backing up one file: the safe copy, the versioning steps and pruning.
//!
//! See "Handling of file versioning" and "Writing a new copy safely" in Requirements.md.

use std::ffi::{OsStr, OsString};
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use crate::fs::{FileKind, FileSystem, Metadata, WriteFile};
use crate::versions::{Format, fits_on, older_version, version_folder, versions_to_remove};

/// Each chunk is flushed before the next, so that progress shows what has reached the medium.
pub const CHUNK_SIZE: usize = 4 * 1024 * 1024;

/// Modification times are compared with this tolerance, because exFAT may store them with less
/// precision than the source file system.
const TIME_TOLERANCE: Duration = Duration::from_secs(2);

/// exFAT can only store times from 1980 to 2107, in seconds since 1970. The driver silently moves
/// any other time to the nearest it can store.
const EXFAT_EARLIEST: u64 = 315_532_800;
const EXFAT_LATEST: u64 = 4_354_819_199;

const FILE_MODE: u32 = 0o666;
const DIR_MODE: u32 = 0o777;

/// Why a file was skipped. The run goes on with the next file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    Unreadable,
    NotWritable,
    TypeConflict,
    ChangedDuringCopy,
    VersionNameTooLong,
    /// Any other error the file system reported while writing.
    WriteError,
}

/// Why the whole run has to stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    MediumFull,
    MediumGone,
    CannotWrite,
}

/// What pruning a version folder did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pruned {
    pub removed: u64,
    /// Some older versions could not be removed. The next run tries again.
    pub failed: bool,
}

/// What became of a visited file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    Unchanged(Pruned),
    Copied(Pruned),
    Skipped(Skip),
    Cancelled,
}

/// The time a file with modification time `time` gets on the medium.
fn stored_time(time: SystemTime, format: Format) -> SystemTime {
    let seconds = |seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds);
    match format {
        Format::ExFat => time.clamp(seconds(EXFAT_EARLIEST), seconds(EXFAT_LATEST)),
        Format::Ext4 => time,
    }
}

/// Whether the source file differs from its newest copy on the medium.
fn differs(source: &Metadata, copy: &Metadata, format: Format) -> bool {
    let source_time = stored_time(source.modified, format);
    let difference = source_time
        .duration_since(copy.modified)
        .or_else(|_| copy.modified.duration_since(source_time))
        .unwrap_or(Duration::MAX);
    source.len != copy.len || difference > TIME_TOLERANCE
}

enum Copy {
    Done(PathBuf),
    Changed,
    Failed(Skip),
    Cancelled,
}

/// Backs up files to one medium during one run.
pub struct FileCopier<'a> {
    pub source: &'a dyn FileSystem,
    pub medium: &'a dyn FileSystem,
    /// The root folder of the medium.
    pub root: PathBuf,
    pub format: Format,
    /// The number of older versions to keep of each file.
    pub keep: usize,
    /// The current time as `YYYY-MM-DD_HHMMSS`, in local time.
    pub now: &'a dyn Fn() -> String,
    pub cancel: &'a AtomicBool,
    next_temp: u64,
}

impl<'a> FileCopier<'a> {
    pub fn new(
        source: &'a dyn FileSystem,
        medium: &'a dyn FileSystem,
        root: PathBuf,
        format: Format,
        keep: usize,
        now: &'a dyn Fn() -> String,
        cancel: &'a AtomicBool,
    ) -> Self {
        Self {
            source,
            medium,
            root,
            format,
            keep,
            now,
            cancel,
            next_temp: 0,
        }
    }

    fn temp_folder(&self) -> PathBuf {
        self.root.join(".babawus/tmp")
    }

    /// Empties the temporary folder, at the start of a run.
    pub fn empty_temp_folder(&self) -> Result<(), Stop> {
        let folder = self.temp_folder();
        match self.medium.create_dir(&folder) {
            Ok(()) => self
                .set_mode(&folder, DIR_MODE)
                .map_err(|error| self.stop(&error))?,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(self.stop(&error)),
        }
        let entries = self
            .medium
            .read_dir(&folder)
            .map_err(|error| self.stop(&error))?;
        for entry in entries {
            let path = folder.join(entry);
            self.medium
                .remove_file(&path)
                .map_err(|error| self.stop(&error))?;
        }
        Ok(())
    }

    /// Flushes everything to the medium, at the end of a run and before the verdict.
    pub fn flush_medium(&self) -> Result<(), Stop> {
        self.medium
            .sync_medium(&self.root)
            .map_err(|_| Stop::MediumGone)
    }

    /// Backs up the file `source` to `target`, which is relative to the root of the medium, and
    /// prunes its version folder. `progress` gets the number of bytes of this file that have
    /// reached the medium, and starts again from 0 if the copy has to be repeated.
    pub fn visit(
        &mut self,
        source: &Path,
        target: &Path,
        progress: &mut dyn FnMut(u64),
    ) -> Result<Visit, Stop> {
        let target = self.root.join(target);
        let Ok(source_metadata) = self.source.metadata(source) else {
            return Ok(Visit::Skipped(Skip::Unreadable));
        };
        let existing = match self.medium.metadata(&target) {
            Ok(metadata) if metadata.kind != FileKind::File => {
                return Ok(Visit::Skipped(Skip::TypeConflict));
            }
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) if error.kind() == ErrorKind::NotADirectory => None,
            Err(error) => return self.skip_or_stop(&error).map(Visit::Skipped),
        };
        if let Some(copy) = &existing
            && !differs(&source_metadata, copy, self.format)
        {
            return self.prune(&target).map(Visit::Unchanged);
        }
        if existing.is_some() && self.version_plan(&target)?.is_none() {
            return Ok(Visit::Skipped(Skip::VersionNameTooLong));
        }
        if self.cancel.load(Ordering::Relaxed) {
            return Ok(Visit::Cancelled);
        }
        let free = self.medium.free_space(&self.root);
        if free.map_err(|error| self.stop(&error))? < source_metadata.len {
            return Err(Stop::MediumFull);
        }
        if let Some(skip) = self.ensure_parent_folders(&target)? {
            return Ok(Visit::Skipped(skip));
        }

        let temp = match self.copy_to_temp(source, &source_metadata, progress)? {
            Copy::Done(temp) => temp,
            Copy::Changed => {
                progress(0);
                let Ok(metadata) = self.source.metadata(source) else {
                    return Ok(Visit::Skipped(Skip::Unreadable));
                };
                match self.copy_to_temp(source, &metadata, progress)? {
                    Copy::Done(temp) => temp,
                    Copy::Changed => return Ok(Visit::Skipped(Skip::ChangedDuringCopy)),
                    Copy::Failed(skip) => return Ok(Visit::Skipped(skip)),
                    Copy::Cancelled => return Ok(Visit::Cancelled),
                }
            }
            Copy::Failed(skip) => return Ok(Visit::Skipped(skip)),
            Copy::Cancelled => return Ok(Visit::Cancelled),
        };

        if existing.is_some()
            && let Some(skip) = self.move_to_version_folder(&target)?
        {
            self.discard(&temp);
            return Ok(Visit::Skipped(skip));
        }
        if let Err(error) = self.medium.rename(&temp, &target) {
            self.discard(&temp);
            return self.skip_or_stop(&error).map(Visit::Skipped);
        }
        self.prune(&target).map(Visit::Copied)
    }

    /// Copies the source to a new temporary file, in flushed chunks.
    fn copy_to_temp(
        &mut self,
        source: &Path,
        metadata: &Metadata,
        progress: &mut dyn FnMut(u64),
    ) -> Result<Copy, Stop> {
        let Ok(mut reader) = self.source.open(source) else {
            return Ok(Copy::Failed(Skip::Unreadable));
        };
        self.next_temp += 1;
        let temp = self.temp_folder().join(format!("copy-{}", self.next_temp));
        let mut file = match self.medium.create_new(&temp) {
            Ok(file) => file,
            Err(error) => return self.temp_failure(&error),
        };
        let result = self.write_chunks(&temp, &mut *reader, &mut *file, metadata, progress);
        drop(file);
        let copy = match result {
            Ok(Copy::Done(_)) => match self.source.metadata(source) {
                Ok(after) if after.len == metadata.len && after.modified == metadata.modified => {
                    return Ok(Copy::Done(temp));
                }
                Ok(_) => Ok(Copy::Changed),
                Err(_) => Ok(Copy::Failed(Skip::Unreadable)),
            },
            Ok(copy) => Ok(copy),
            Err(error) => self.temp_failure(&error),
        };
        self.discard(&temp);
        copy
    }

    /// Returns an error if writing to the medium fails.
    fn write_chunks(
        &self,
        temp: &Path,
        reader: &mut dyn Read,
        file: &mut dyn WriteFile,
        metadata: &Metadata,
        progress: &mut dyn FnMut(u64),
    ) -> io::Result<Copy> {
        self.set_mode(temp, FILE_MODE)?;
        let mut chunk = vec![0; CHUNK_SIZE];
        let mut flushed = 0;
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Ok(Copy::Cancelled);
            }
            let Ok(length) = fill(reader, &mut chunk) else {
                return Ok(Copy::Failed(Skip::Unreadable));
            };
            file.write_all(&chunk[..length])?;
            // A short chunk is the last one. Setting the time now means that a small file
            // needs only one flush.
            let last = length < CHUNK_SIZE;
            if last {
                file.set_modified(metadata.modified)?;
            }
            file.sync()?;
            flushed += length as u64;
            if length > 0 {
                progress(flushed);
            }
            if last {
                return Ok(Copy::Done(temp.to_owned()));
            }
        }
    }

    /// Creates the folders above `target`. If the source has turned a file into a folder, the
    /// old file on the medium becomes an ordinary older version.
    fn ensure_parent_folders(&self, target: &Path) -> Result<Option<Skip>, Stop> {
        let Some(parent) = target.parent() else {
            return Ok(None);
        };
        let relative = parent.strip_prefix(&self.root).unwrap_or(parent);
        let mut folder = self.root.clone();
        for part in relative {
            folder.push(part);
            match self.medium.metadata(&folder) {
                Ok(metadata) if metadata.kind == FileKind::Dir => continue,
                Ok(_) => {
                    if let Some(skip) = self.move_to_version_folder(&folder)? {
                        return Ok(Some(skip));
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return self.skip_or_stop(&error).map(Some),
            }
            if let Err(error) = self.create_folder(&folder) {
                return self.skip_or_stop(&error).map(Some);
            }
        }
        Ok(None)
    }

    fn create_folder(&self, folder: &Path) -> io::Result<()> {
        self.medium.create_dir(folder)?;
        self.set_mode(folder, DIR_MODE)
    }

    /// Where the newest copy at `target` would go, or `None` if the names are too long.
    fn version_plan(&self, target: &Path) -> Result<Option<(PathBuf, OsString)>, Stop> {
        let name = file_name(target);
        let folder_name = version_folder(name);
        if !fits_on(&folder_name, self.format) {
            return Ok(None);
        }
        let folder = target.with_file_name(folder_name);
        let timestamp = (self.now)();
        for suffix in 1.. {
            let version = older_version(name, &timestamp, suffix);
            if !fits_on(&version, self.format) {
                return Ok(None);
            }
            match self.medium.metadata(&folder.join(&version)) {
                Ok(_) => continue,
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    return Ok(Some((folder, version)));
                }
                Err(error) => return Err(self.stop(&error)),
            }
        }
        unreachable!("the loop only ends by returning")
    }

    /// Moves the newest copy at `target` into its version folder.
    fn move_to_version_folder(&self, target: &Path) -> Result<Option<Skip>, Stop> {
        let Some((folder, version)) = self.version_plan(target)? else {
            return Ok(Some(Skip::VersionNameTooLong));
        };
        match self.medium.metadata(&folder) {
            Ok(metadata) if metadata.kind == FileKind::Dir => {}
            Ok(_) => return Ok(Some(Skip::TypeConflict)),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if let Err(error) = self.create_folder(&folder) {
                    return self.skip_or_stop(&error).map(Some);
                }
            }
            Err(error) => return self.skip_or_stop(&error).map(Some),
        }
        match self.medium.rename(target, &folder.join(version)) {
            Ok(()) => Ok(None),
            Err(error) => self.skip_or_stop(&error).map(Some),
        }
    }

    /// Removes the oldest versions of the file at `target`, until the limit is reached.
    fn prune(&self, target: &Path) -> Result<Pruned, Stop> {
        let name = file_name(target);
        let folder = target.with_file_name(version_folder(name));
        let entries = match self.medium.read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if is_no_folder(&error) => return Ok(Pruned::default()),
            Err(error) => return self.prune_failed(&error, Pruned::default()),
        };
        let mut pruned = Pruned::default();
        for version in versions_to_remove(&entries, name, self.keep) {
            if let Err(error) = self.medium.remove_file(&folder.join(version)) {
                return self.prune_failed(&error, pruned);
            }
            pruned.removed += 1;
        }
        Ok(pruned)
    }

    fn prune_failed(&self, error: &io::Error, pruned: Pruned) -> Result<Pruned, Stop> {
        self.skip_or_stop(error)?;
        Ok(Pruned {
            failed: true,
            ..pruned
        })
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        match self.format {
            Format::Ext4 => self.medium.set_mode(path, mode),
            Format::ExFat => Ok(()),
        }
    }

    /// Removes a temporary file. If this fails, the next run empties the folder anyway.
    fn discard(&self, temp: &Path) {
        let _ = self.medium.remove_file(temp);
    }

    fn is_gone(&self, error: &io::Error) -> bool {
        matches!(
            error.raw_os_error(),
            Some(libc::EIO | libc::ENODEV | libc::ENXIO)
        ) || self.medium.metadata(&self.root).is_err()
    }

    /// Decides what an error on the medium means for this file.
    fn skip_or_stop(&self, error: &io::Error) -> Result<Skip, Stop> {
        if self.is_gone(error) {
            return Err(Stop::MediumGone);
        }
        match error.kind() {
            ErrorKind::StorageFull => Err(Stop::MediumFull),
            ErrorKind::ReadOnlyFilesystem => Err(Stop::CannotWrite),
            ErrorKind::PermissionDenied => Ok(Skip::NotWritable),
            _ => Ok(Skip::WriteError),
        }
    }

    /// An error that always ends the run.
    fn stop(&self, error: &io::Error) -> Stop {
        self.skip_or_stop(error).err().unwrap_or(Stop::CannotWrite)
    }

    /// An error while writing to `.babawus/tmp/`. If that folder can't be written, nothing can
    /// be backed up, and if it is only this file, the run goes on.
    fn temp_failure(&self, error: &io::Error) -> Result<Copy, Stop> {
        match self.skip_or_stop(error)? {
            Skip::NotWritable => Err(Stop::CannotWrite),
            skip => Ok(Copy::Failed(skip)),
        }
    }
}

fn is_no_folder(error: &io::Error) -> bool {
    matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory)
}

fn file_name(path: &Path) -> &OsStr {
    path.file_name().expect("a backed up file has a name")
}

/// Reads until `chunk` is full or the file ends, and returns the number of bytes read.
fn fill(reader: &mut dyn Read, chunk: &mut [u8]) -> io::Result<usize> {
    let mut length = 0;
    while length < chunk.len() {
        match reader.read(&mut chunk[length..]) {
            Ok(0) => break,
            Ok(read) => length += read,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(length)
}

#[cfg(test)]
mod tests;
