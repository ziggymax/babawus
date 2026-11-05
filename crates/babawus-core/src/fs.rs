//! The small file system interface that all logic runs against.
//!
//! [`RealFs`] is the real thing. [`fake::FakeFs`] keeps everything in memory and can inject
//! faults. `tests/fs_contract.rs` runs the same tests against both, and on real exFAT and Ext4,
//! so that the fake can't drift away from how real file systems behave.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::Path;
use std::time::SystemTime;

pub mod fake;
mod real;

pub use real::RealFs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    File,
    Dir,
    /// Symbolic links and special files. Babawus treats these alike: it neither follows nor
    /// copies them.
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub kind: FileKind,
    pub len: u64,
    pub modified: SystemTime,
    pub mode: u32,
    /// Identifies the file system that holds the file, to tell when a folder is on another one.
    pub device: u64,
}

pub trait FileSystem: Send + Sync {
    /// The metadata of `path` itself. Symbolic links are not followed.
    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    /// The names in the folder `path`, sorted by their bytes.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>>;

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read + Send>>;

    /// Creates a file in a single operation that fails if it already exists.
    fn create_new(&self, path: &Path) -> io::Result<Box<dyn WriteFile>>;

    /// Opens a file for adding to its end, and creates it if needed.
    fn open_append(&self, path: &Path) -> io::Result<Box<dyn WriteFile>>;

    fn create_dir(&self, path: &Path) -> io::Result<()>;

    /// Moves `from` to `to`, replacing a file that is already there.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn remove_file(&self, path: &Path) -> io::Result<()>;

    /// Sets the mode exactly, whatever the umask. Only for Ext4: exFAT stores no permissions.
    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()>;

    /// The bytes an ordinary user can still write to the file system that holds `path`.
    fn free_space(&self, path: &Path) -> io::Result<u64>;

    /// Flushes everything on the file system that holds `path`, including renames and folder
    /// changes, which flushing a file does not cover.
    fn sync_medium(&self, path: &Path) -> io::Result<()>;
}

/// A file that is open for writing.
pub trait WriteFile: Send {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()>;

    /// Sets the modification time. Do this before [`sync`](Self::sync), so that it is flushed too.
    fn set_modified(&mut self, time: SystemTime) -> io::Result<()>;

    /// Flushes this file's content to the medium.
    fn sync(&mut self) -> io::Result<()>;
}
