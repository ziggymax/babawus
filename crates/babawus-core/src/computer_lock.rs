//! The lock that allows one Babawus per computer. See "One Babawus at a time" in Requirements.md.
//!
//! This uses the real file system, because the point of it is what the system does: it releases
//! the lock when the process ends, even after a crash. The lock file stays behind, and the system
//! empties `/run/lock/` when it starts.
//!
//! All users share the file, and `/run/lock/` is a shared folder. There, the system refuses to
//! open another user's file with the option to create it (`fs.protected_regular`), so the file is
//! opened first, and only created if it isn't there.

use std::fs::{File, OpenOptions, Permissions, TryLockError};
use std::io::{self, ErrorKind, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

/// The lock file of a real Babawus.
pub const COMPUTER_LOCK_FILE: &str = "/run/lock/babawus.lock";

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
}
