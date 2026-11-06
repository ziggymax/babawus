//! The lock for one Babawus per computer, between two real users in a folder like `/run/lock/`.
//!
//! These use sudo, so a plain `cargo test` ignores them.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use babawus_core::computer_lock::{ComputerLock, Taken};
use babawus_core::log::new_run_id;

const OTHER_USER: &str = "nobody";
const TAKE_FILE: &str = "BABAWUS_TEST_TAKE_FILE";

fn sudo(arguments: &[&str]) -> Command {
    let mut command = Command::new("sudo");
    command.arg("-n").args(arguments);
    command
}

fn run_sudo(arguments: &[&str]) {
    assert!(sudo(arguments).status().unwrap().success());
}

/// A folder with the same owner and mode as `/run/lock/`, removed afterwards.
struct SharedFolder(PathBuf);

impl SharedFolder {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("babawus-shared-{}", new_run_id()));
        let text = path.to_str().unwrap();
        run_sudo(&["install", "-d", "-m", "1777", "-o", "root", text]);
        Self(path)
    }

    fn lock_file(&self) -> PathBuf {
        self.0.join("babawus.lock")
    }
}

impl Drop for SharedFolder {
    fn drop(&mut self) {
        run_sudo(&["rm", "-rf", self.0.to_str().unwrap()]);
    }
}

/// Puts a file that belongs to the other user in place, as a Babawus of theirs would leave it.
fn other_users_file(path: &Path, mode: &str) {
    let text = path.to_str().unwrap();
    run_sudo(&["install", "-m", mode, "-o", OTHER_USER, "/dev/null", text]);
    let write = format!("printf {OTHER_USER} > {text}");
    run_sudo(&["-u", OTHER_USER, "sh", "-c", &write]);
}

/// Has the other user hold the lock, until the process is ended.
fn hold_as_other_user(path: &Path) -> Child {
    let mut holder = sudo(&["-u", OTHER_USER, "flock", "-n"])
        .arg(path)
        .args(["sh", "-c", "echo held; exec sleep 60"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = [0; 5];
    let stdout = holder.stdout.as_mut().unwrap();
    std::io::Read::read_exact(stdout, &mut line).unwrap();
    assert_eq!(&line, b"held\n");
    holder
}

#[test]
#[ignore = "uses sudo"]
fn a_lock_file_left_by_another_user_can_be_taken() {
    let folder = SharedFolder::new();
    let path = folder.lock_file();
    other_users_file(&path, "666");

    let taken = ComputerLock::take(&path, "usera").unwrap();

    assert!(matches!(taken, Taken::Ours(_)));
    assert_eq!(fs::read_to_string(&path).unwrap(), "usera");
}

#[test]
#[ignore = "uses sudo"]
fn a_lock_held_by_another_user_gives_their_name() {
    let folder = SharedFolder::new();
    let path = folder.lock_file();
    other_users_file(&path, "666");
    let _holder = KillOnDrop(hold_as_other_user(&path));

    let taken = ComputerLock::take(&path, "usera").unwrap();

    assert!(matches!(taken, Taken::AlreadyOpen(user) if user == OTHER_USER));
}

#[test]
#[ignore = "uses sudo"]
fn a_lock_file_that_only_its_owner_can_write_still_works() {
    let folder = SharedFolder::new();
    let path = folder.lock_file();
    other_users_file(&path, "644");

    let taken = ComputerLock::take(&path, "usera").unwrap();

    assert!(matches!(taken, Taken::Ours(_)));
}

#[test]
#[ignore = "uses sudo"]
fn the_other_user_can_take_a_file_this_user_left() {
    let folder = SharedFolder::new();
    let path = folder.lock_file();
    drop(ComputerLock::take(&path, "usera").unwrap());

    // The test program may be in a home folder the other user can't enter.
    let program = folder.0.join("test-program");
    fs::copy(std::env::current_exe().unwrap(), &program).unwrap();
    let status = sudo(&["-u", OTHER_USER, "env"])
        .arg(format!("{TAKE_FILE}={}", path.display()))
        .arg(&program)
        .args(["takes_the_lock_when_run_as_the_other_user", "--exact"])
        .stdout(Stdio::null())
        .status()
        .unwrap();

    assert!(status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), OTHER_USER);
}

/// The other user's half of the test above.
#[test]
fn takes_the_lock_when_run_as_the_other_user() {
    if let Some(path) = std::env::var_os(TAKE_FILE) {
        let taken = ComputerLock::take(Path::new(&path), OTHER_USER).unwrap();
        assert!(matches!(taken, Taken::Ours(_)));
    }
}

#[test]
fn every_user_can_use_a_new_file_whatever_the_umask() {
    let folder = std::env::temp_dir().join(format!("babawus-umask-{}", new_run_id()));
    fs::create_dir(&folder).unwrap();
    let path = folder.join("babawus.lock");
    // SAFETY: this only sets the umask. Nothing else here depends on it, because the other tests
    // in this program get their modes from `install`, or from `take`, which sets its own.
    let old_umask = unsafe { libc::umask(0o077) };

    let lock = ComputerLock::take(&path, "usera").unwrap();

    // SAFETY: as above.
    unsafe { libc::umask(old_umask) };
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    drop(lock);
    fs::remove_dir_all(&folder).unwrap();
    assert_eq!(mode & 0o777, 0o666);
}

/// The holder runs under sudo, so it is ended through its process group's sudo.
struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let pattern = "^sleep 60$";
        let _ = sudo(&["pkill", "-u", OTHER_USER, "-f", pattern]).status();
        let _ = self.0.wait();
    }
}
