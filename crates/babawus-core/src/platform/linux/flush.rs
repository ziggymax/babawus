//! Flushing and locking on Linux: flushing a medium, free space, file permissions, the lock that
//! allows one Babawus per computer, and whether the process that holds a medium's lock is still
//! running. See "Platform layer" and "One Babawus at a time" in Requirements.md.
//!
//! The computer lock uses the real file system, because the point of it is what the system does: it releases
//! the lock when the process ends, even after a crash. The lock file stays behind, and the system
//! empties `/run/lock/` when it starts.
//!
//! All users share the file, and `/run/lock/` is a shared folder. There, the system refuses to
//! open another user's file with the option to create it (`fs.protected_regular`), so the file is
//! opened first, and only created if it isn't there.

use std::ffi::CString;
use std::fs::{self, File, OpenOptions, Permissions, TryLockError};
use std::io::{self, ErrorKind, Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::fs::FileSystem;
use crate::medium::read_file;

/// The name of this process's program.
pub(crate) const OWN_PROGRAM_FILE: &str = "/proc/self/comm";

/// Sets the permissions of the file or folder `path` to `mode`, such as `0o666`.
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    fs::set_permissions(path, Permissions::from_mode(mode))
}

/// The permissions of a file or folder, such as `0o666`.
pub fn mode(metadata: &fs::Metadata) -> u32 {
    metadata.mode() & 0o7777
}

/// The device that holds a file or folder, which tells one file system from another.
pub fn device(metadata: &fs::Metadata) -> u64 {
    metadata.dev()
}

/// The space that a user can still use on the file system that holds `path`, in bytes.
pub fn free_space(path: &Path) -> io::Result<u64> {
    let path = CString::new(path.as_os_str().as_encoded_bytes())?;
    let mut stats = MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is a valid C string, and `stats` is valid for writes.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `statvfs` succeeded, so it has filled in `stats`.
    let stats = unsafe { stats.assume_init() };
    Ok(stats.f_bavail * stats.f_frsize)
}

/// Writes out everything that is still in memory for the file system that holds `path`.
pub fn flush_file_system(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    // SAFETY: `file` keeps the file descriptor open during the call.
    if unsafe { libc::syncfs(file.as_raw_fd()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Whether `error` says that the medium has gone, for example because it was pulled out.
pub fn is_medium_gone(error: &io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(libc::EIO | libc::ENODEV | libc::ENXIO)
    )
}

/// The name of this process's program, read from `system`, the computer's own file system.
pub fn own_program(system: &dyn FileSystem) -> io::Result<String> {
    read_line(system, Path::new(OWN_PROGRAM_FILE))
}

/// The name of the program that the process `process_id` runs, read from `system`, the
/// computer's own file system. Without a readable name, the process has ended.
pub fn program_of(system: &dyn FileSystem, process_id: u32) -> io::Result<String> {
    read_line(system, &program_file(process_id))
}

/// The file that holds the name of the program that the process `process_id` runs.
pub(crate) fn program_file(process_id: u32) -> PathBuf {
    PathBuf::from(format!("/proc/{process_id}/comm"))
}

/// A file's contents without the line ending.
fn read_line(system: &dyn FileSystem, path: &Path) -> io::Result<String> {
    let contents = String::from_utf8(read_file(system, path)?)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    Ok(contents.trim_end().to_owned())
}

const MODE: u32 = 0o666;

/// Another user could have created the file again in between, but not forever.
const ATTEMPTS: usize = 10;

/// Holds the lock until this is dropped, or the process ends.
#[derive(Debug)]
pub struct ComputerLock {
    _file: File,
}

/// What came of trying to take the lock.
#[derive(Debug)]
pub enum Taken {
    Ours(ComputerLock),
    /// Another Babawus has the lock, for this user. The name is empty in the tiny moment before
    /// that Babawus has written it.
    AlreadyOpen(String),
}

fn options() -> OpenOptions {
    let mut options = OpenOptions::new();
    // Anyone can put a link in a shared folder.
    options.custom_flags(libc::O_NOFOLLOW);
    options
}

/// Opens the lock file, and creates it if needed.
fn open(path: &Path) -> io::Result<File> {
    let mut last_error = None;
    for _ in 0..ATTEMPTS {
        match options().read(true).write(true).open(path) {
            Ok(file) => return Ok(file),
            // A file that isn't writable, for example one that is just being created, can still
            // be locked.
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                return options().read(true).open(path);
            }
            Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
            Err(error) => last_error = Some(error),
        }
        let created = options()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(MODE)
            .open(path);
        match created {
            Ok(file) => {
                // The mode above is limited by the umask.
                file.set_permissions(Permissions::from_mode(MODE))?;
                return Ok(file);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or_else(|| ErrorKind::NotFound.into()))
}

/// Replaces the file's contents with `username`. A file that can't be written still holds the
/// lock, and all that is lost is the name in another user's message.
fn record(mut file: &File, username: &str) -> io::Result<()> {
    file.set_len(0)?;
    file.write_all(username.as_bytes())
}

fn recorded(mut file: &File) -> io::Result<String> {
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(String::from_utf8_lossy(&contents).into_owned())
}

impl ComputerLock {
    /// Tries to take the lock in the file `path` for `username`.
    pub fn take(path: &Path, username: &str) -> io::Result<Taken> {
        let file = open(path)?;
        if !file.metadata()?.is_file() {
            return Err(ErrorKind::InvalidInput.into());
        }
        match file.try_lock() {
            Ok(()) => {
                let _ = record(&file, username);
                Ok(Taken::Ours(Self { _file: file }))
            }
            Err(TryLockError::WouldBlock) => Ok(Taken::AlreadyOpen(recorded(&file)?)),
            Err(TryLockError::Error(error)) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    use super::*;
    use crate::log::new_run_id;

    /// A folder that is removed afterwards.
    struct Folder(PathBuf);

    impl Folder {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("babawus-lock-{}", new_run_id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn lock_file(&self) -> PathBuf {
            self.0.join("babawus.lock")
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn take(path: &Path, username: &str) -> Taken {
        ComputerLock::take(path, username).unwrap()
    }

    /// Takes a lock that was just released, waiting up to a second for it. Another test that
    /// starts a program forks this process, and until that program has started, the fork still
    /// has every open file, so the released lock can be held a moment longer. That made this
    /// test fail now and then on CI, which runs those tests in parallel (T-061).
    fn take_released(path: &Path, username: &str) -> Taken {
        for _ in 0..100 {
            match take(path, username) {
                Taken::AlreadyOpen(_) => std::thread::sleep(std::time::Duration::from_millis(10)),
                ours => return ours,
            }
        }
        take(path, username)
    }

    #[test]
    fn the_first_babawus_gets_the_lock() {
        let folder = Folder::new();

        assert!(matches!(take(&folder.lock_file(), "usera"), Taken::Ours(_)));
    }

    #[test]
    fn a_second_babawus_learns_who_has_the_lock() {
        let folder = Folder::new();
        let _first = take(&folder.lock_file(), "usera");

        let second = take(&folder.lock_file(), "userb");

        assert!(matches!(second, Taken::AlreadyOpen(user) if user == "usera"));
    }

    #[test]
    fn a_released_lock_can_be_taken_and_records_only_the_new_user() {
        let folder = Folder::new();
        let path = folder.lock_file();
        drop(take(&path, "a-long-user-name"));

        let lock = take_released(&path, "userb");

        assert!(matches!(lock, Taken::Ours(_)));
        assert_eq!(fs::read_to_string(&path).unwrap(), "userb");
    }

    #[test]
    fn a_file_that_cannot_be_written_can_still_be_locked() {
        let folder = Folder::new();
        let path = folder.lock_file();
        fs::write(&path, "old").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o444)).unwrap();

        let first = take(&path, "usera");
        let second = take(&path, "userb");

        assert!(matches!(first, Taken::Ours(_)));
        assert!(matches!(second, Taken::AlreadyOpen(_)));
    }

    #[test]
    fn a_link_in_place_of_the_file_is_not_followed() {
        let folder = Folder::new();
        let target = folder.0.join("precious");
        fs::write(&target, "precious").unwrap();
        symlink(&target, folder.lock_file()).unwrap();

        assert!(ComputerLock::take(&folder.lock_file(), "usera").is_err());

        assert_eq!(fs::read_to_string(&target).unwrap(), "precious");
    }

    #[test]
    fn a_folder_in_place_of_the_file_is_an_error() {
        let folder = Folder::new();
        fs::create_dir(folder.lock_file()).unwrap();

        assert!(ComputerLock::take(&folder.lock_file(), "usera").is_err());
    }

    #[test]
    fn a_pipe_in_place_of_the_file_is_an_error() {
        let folder = Folder::new();
        let path = folder.lock_file();
        let status = std::process::Command::new("mkfifo").arg(&path).status();
        assert!(status.unwrap().success());

        assert!(ComputerLock::take(&path, "usera").is_err());
    }

    #[test]
    fn a_missing_folder_is_an_error() {
        let path = Path::new("/nonexistent/babawus.lock");

        assert!(ComputerLock::take(path, "usera").is_err());
    }

    #[test]
    fn only_the_errors_of_a_pulled_out_medium_say_it_has_gone() {
        for code in [libc::EIO, libc::ENODEV, libc::ENXIO] {
            assert!(
                is_medium_gone(&io::Error::from_raw_os_error(code)),
                "{code}"
            );
        }
        assert!(!is_medium_gone(&io::Error::from_raw_os_error(libc::ENOSPC)));
        assert!(!is_medium_gone(&ErrorKind::NotFound.into()));
    }
}
