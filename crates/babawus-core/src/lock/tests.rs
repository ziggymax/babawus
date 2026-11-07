use std::ffi::OsString;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

use super::*;
use crate::fs::WriteFile;
use crate::fs::fake::FakeFs;
use crate::fs::{Metadata, RealFs};
use crate::log::new_run_id;

const COMPUTER: &str = "0123456789abcdef0123456789abcdef";
const OTHER_COMPUTER: &str = "fedcba9876543210fedcba9876543210";
const BOOT: &str = "11111111-2222-3333-4444-555555555555";
const OTHER_BOOT: &str = "66666666-7777-8888-9999-000000000000";
const PROGRAM: &str = "babawus";
const OWN_PROCESS: u32 = 1000;
const OTHER_PROCESS: u32 = 2000;
const ENDED_PROCESS: u32 = 3000;
const REUSED_PROCESS: u32 = 4000;
const ACCOUNT: &str = "usera@mint";
const SLEEP_IN_CHILD: &str = "BABAWUS_TEST_SLEEP";

fn root() -> PathBuf {
    PathBuf::from("/medium")
}

fn lock_path() -> PathBuf {
    root().join(LOCK_FILE)
}

fn this() -> ThisProcess {
    ThisProcess {
        computer_id: COMPUTER.to_owned(),
        boot_id: BOOT.to_owned(),
        process_id: OWN_PROCESS,
        program: PROGRAM.to_owned(),
    }
}

fn holder(process_id: u32) -> Holder {
    Holder {
        run_id: new_run_id(),
        account: "userb@mint".to_owned(),
        computer_id: COMPUTER.to_owned(),
        boot_id: BOOT.to_owned(),
        process_id,
    }
}

struct Setup {
    root: PathBuf,
    medium: FakeFs,
    system: FakeFs,
}

impl Setup {
    fn new() -> Self {
        let medium = FakeFs::new();
        medium.add_file(root().join(".babawus/file"), b"", SystemTime::UNIX_EPOCH);
        let system = FakeFs::new();
        let name = |process| format!("/proc/{process}/comm");
        let line = format!("{PROGRAM}\n");
        system.add_file(name(OTHER_PROCESS), line.as_bytes(), SystemTime::UNIX_EPOCH);
        system.add_file(name(REUSED_PROCESS), b"firefox\n", SystemTime::UNIX_EPOCH);
        Self {
            root: root(),
            medium,
            system,
        }
    }

    fn take(&self) -> Result<MediumLock<'_>, LockError> {
        self.take_as(&new_run_id())
    }

    fn take_as(&self, run_id: &str) -> Result<MediumLock<'_>, LockError> {
        MediumLock::take(
            &self.medium,
            &self.root,
            Format::Ext4,
            &self.system,
            &this(),
            run_id,
            ACCOUNT,
        )
    }

    fn put_lock(&self, data: &[u8]) {
        self.medium
            .add_file(lock_path(), data, SystemTime::UNIX_EPOCH);
    }

    fn put_holder(&self, holder: &Holder) {
        self.put_lock(toml::to_string(holder).unwrap().as_bytes());
    }

    fn holder(&self) -> Option<Holder> {
        read_holder(&self.medium, &self.root).unwrap()
    }

    fn has_lock_file(&self) -> bool {
        self.medium.metadata(&lock_path()).is_ok()
    }

    fn live(&self) -> Option<Holder> {
        live_holder(&self.medium, &self.root, &self.system, &this()).unwrap()
    }
}

#[test]
fn taking_the_lock_records_who_holds_it() {
    let setup = Setup::new();

    let _lock = setup.take_as("0123456789abcdef").unwrap();

    let expected = Holder {
        run_id: "0123456789abcdef".to_owned(),
        account: ACCOUNT.to_owned(),
        computer_id: COMPUTER.to_owned(),
        boot_id: BOOT.to_owned(),
        process_id: OWN_PROCESS,
    };
    assert_eq!(setup.holder(), Some(expected));
}

#[test]
fn every_user_can_read_and_remove_the_lock_on_ext4() {
    let setup = Setup::new();

    let _lock = setup.take().unwrap();

    assert_eq!(setup.medium.metadata(&lock_path()).unwrap().mode, FILE_MODE);
}

#[test]
fn the_lock_works_on_exfat_which_has_no_permissions() {
    let setup = Setup::new();
    let medium = FakeFs::exfat();
    medium.add_file(root().join(".babawus/file"), b"", SystemTime::UNIX_EPOCH);

    let lock = MediumLock::take(
        &medium,
        &setup.root,
        Format::ExFat,
        &setup.system,
        &this(),
        &new_run_id(),
        ACCOUNT,
    );

    assert!(lock.is_ok());
}

#[test]
fn a_live_lock_of_another_process_stops_a_new_one() {
    let setup = Setup::new();
    let other = holder(OTHER_PROCESS);
    setup.put_holder(&other);

    let result = setup.take();

    assert!(matches!(result, Err(LockError::Held(Some(holder))) if holder == other));
    assert_eq!(setup.holder(), Some(other));
}

#[test]
fn a_lock_this_process_still_holds_stops_a_new_one() {
    let setup = Setup::new();
    let lock = setup.take().unwrap();

    assert!(matches!(setup.take(), Err(LockError::Held(_))));

    drop(lock);
    assert!(setup.take().is_ok());
}

#[test]
fn locks_that_are_not_live_are_replaced() {
    let other_computer = Holder {
        computer_id: OTHER_COMPUTER.to_owned(),
        ..holder(OTHER_PROCESS)
    };
    let other_boot = Holder {
        boot_id: OTHER_BOOT.to_owned(),
        ..holder(OTHER_PROCESS)
    };
    let leftovers = [
        ("another computer", other_computer),
        ("another boot", other_boot),
        ("a process that ended", holder(ENDED_PROCESS)),
        ("a reused process ID", holder(REUSED_PROCESS)),
        ("an earlier run of this process", holder(OWN_PROCESS)),
    ];
    for (name, leftover) in leftovers {
        let setup = Setup::new();
        setup.put_holder(&leftover);
        assert_eq!(setup.live(), None, "{name}");

        let lock = setup.take().unwrap_or_else(|_| panic!("{name}"));

        assert_eq!(setup.holder().unwrap().run_id, lock.run_id, "{name}");
    }
}

#[test]
fn a_lock_that_cannot_be_understood_is_replaced() {
    let garbled: [&[u8]; 4] = [b"", b"run_id = \"half", b"\xff\xfe", b"run_id = \"abc\"\n"];
    for data in garbled {
        let setup = Setup::new();
        setup.put_lock(data);
        assert_eq!(setup.live(), None);

        let lock = setup.take().unwrap();

        assert_eq!(setup.holder().unwrap().run_id, lock.run_id);
    }
}

#[test]
fn checking_a_lock_changes_nothing_on_the_medium() {
    let setup = Setup::new();
    setup.put_holder(&holder(ENDED_PROCESS));
    let steps = setup.medium.steps();

    assert_eq!(setup.live(), None);

    assert!(setup.has_lock_file());
    assert_eq!(setup.medium.steps(), steps);
}

#[test]
fn a_medium_without_a_lock_has_no_live_holder() {
    assert_eq!(Setup::new().live(), None);
}

#[test]
fn the_live_holder_is_reported() {
    let setup = Setup::new();
    let other = holder(OTHER_PROCESS);
    setup.put_holder(&other);

    assert_eq!(setup.live(), Some(other));
}

#[test]
fn dropping_the_lock_removes_it() {
    let setup = Setup::new();

    drop(setup.take().unwrap());

    assert!(!setup.has_lock_file());
}

#[test]
fn dropping_the_lock_leaves_a_lock_of_another_run_alone() {
    let setup = Setup::new();
    let lock = setup.take().unwrap();
    setup.medium.remove_file(&lock_path()).unwrap();
    let other = holder(OTHER_PROCESS);
    setup.put_holder(&other);

    drop(lock);

    assert_eq!(setup.holder(), Some(other));
}

#[test]
fn a_medium_pulled_during_a_run_is_not_locked_after_it() {
    let setup = Setup::new();
    let lock = setup.take().unwrap();
    setup.medium.pull_at_step(setup.medium.steps() + 1);
    assert!(setup.medium.create_dir(&root().join("x")).is_err());

    drop(lock);
    setup.medium.reinsert();

    assert!(setup.has_lock_file());
    assert_eq!(setup.live(), None);
    assert!(setup.take().is_ok());
}

#[test]
fn an_interrupted_take_never_leaves_the_medium_locked() {
    let steps = {
        let setup = Setup::new();
        setup.put_holder(&holder(ENDED_PROCESS));
        let before = setup.medium.steps();
        std::mem::forget(setup.take().unwrap());
        setup.medium.steps() - before
    };
    assert!(steps >= 3);

    for step in 1..=steps {
        for lose_unflushed in [false, true] {
            let setup = Setup::new();
            setup.put_holder(&holder(ENDED_PROCESS));
            setup.medium.pull_at_step(setup.medium.steps() + step);

            let lock = setup.take();
            assert!(lock.is_err(), "step {step}");

            if lose_unflushed {
                setup.medium.reinsert_losing_unflushed();
            } else {
                setup.medium.reinsert();
            }
            let lock = setup.take().unwrap_or_else(|_| panic!("step {step}"));
            assert_eq!(setup.live().unwrap().run_id, lock.run_id, "step {step}");
        }
    }
}

#[test]
fn a_lock_that_could_not_be_written_is_not_left_behind() {
    let setup = Setup::new();
    setup.medium.set_capacity(0);
    let run_id = new_run_id();

    assert!(matches!(setup.take_as(&run_id), Err(LockError::Io(_))));

    assert!(!setup.has_lock_file());
    assert!(!held_runs().contains(&run_id));
}

/// A medium where another Babawus acts right after this one has written its lock.
struct Rival {
    fake: FakeFs,
    /// What the rival leaves in the lock file. Without it, the rival only removes the file.
    replacement: Option<Vec<u8>>,
}

struct RivalFile {
    file: Box<dyn WriteFile>,
    fake: FakeFs,
    replacement: Option<Vec<u8>>,
}

impl WriteFile for RivalFile {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.file.write_all(data)?;
        self.fake.remove_file(&lock_path())?;
        if let Some(replacement) = &self.replacement {
            let mut file = self.fake.create_new(&lock_path())?;
            file.write_all(replacement)?;
        }
        Ok(())
    }

    fn set_modified(&mut self, time: SystemTime) -> io::Result<()> {
        self.file.set_modified(time)
    }

    fn sync(&mut self) -> io::Result<()> {
        self.file.sync()
    }
}

impl FileSystem for Rival {
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        self.fake.metadata(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        self.fake.read_dir(path)
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read + Send>> {
        self.fake.open(path)
    }

    fn create_new(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        Ok(Box::new(RivalFile {
            file: self.fake.create_new(path)?,
            fake: self.fake.clone(),
            replacement: self.replacement.clone(),
        }))
    }

    fn open_append(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        self.fake.open_append(path)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.fake.create_dir(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.fake.rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.fake.remove_file(path)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.fake.set_mode(path, mode)
    }

    fn free_space(&self, path: &Path) -> io::Result<u64> {
        self.fake.free_space(path)
    }

    fn sync_medium(&self, path: &Path) -> io::Result<()> {
        self.fake.sync_medium(path)
    }
}

fn take_against(rival: &Rival, setup: &Setup, run_id: &str) -> LockError {
    let result = MediumLock::take(
        rival,
        &setup.root,
        Format::Ext4,
        &setup.system,
        &this(),
        run_id,
        ACCOUNT,
    );
    match result {
        Ok(_) => panic!("the lock was taken"),
        Err(error) => error,
    }
}

#[test]
fn a_lock_replaced_right_after_writing_it_is_not_ours() {
    let setup = Setup::new();
    let other = holder(OTHER_PROCESS);
    let text = toml::to_string(&other).unwrap();
    let rival = Rival {
        fake: setup.medium.clone(),
        replacement: Some(text.clone().into_bytes()),
    };
    let run_id = new_run_id();

    let error = take_against(&rival, &setup, &run_id);

    assert!(matches!(error, LockError::Held(Some(holder)) if holder == other));
    assert!(!held_runs().contains(&run_id));
    let left = read_file(&setup.medium, &lock_path()).unwrap();
    assert_eq!(left, text.as_bytes());
}

#[test]
fn a_lock_removed_right_after_writing_it_is_not_ours() {
    let setup = Setup::new();
    let rival = Rival {
        fake: setup.medium.clone(),
        replacement: None,
    };
    let run_id = new_run_id();

    let error = take_against(&rival, &setup, &run_id);

    assert!(matches!(error, LockError::Held(None)));
    assert!(!held_runs().contains(&run_id));
}

#[test]
fn a_lock_half_written_by_a_rival_is_not_ours() {
    let setup = Setup::new();
    let rival = Rival {
        fake: setup.medium.clone(),
        replacement: Some(Vec::new()),
    };

    let error = take_against(&rival, &setup, &new_run_id());

    assert!(matches!(error, LockError::Held(None)));
    assert!(setup.has_lock_file());
}

#[test]
fn a_lock_that_cannot_be_read_is_reported() {
    let setup = Setup::new();
    setup.put_holder(&holder(ENDED_PROCESS));
    setup
        .medium
        .fail_path(lock_path(), ErrorKind::PermissionDenied);

    assert!(matches!(setup.take(), Err(LockError::Io(_))));
}

#[test]
fn this_process_is_read_without_line_endings() {
    let system = FakeFs::new();
    let line = format!("{BOOT}\n");
    system.add_file(BOOT_ID_FILE, line.as_bytes(), SystemTime::UNIX_EPOCH);
    system.add_file(OWN_PROGRAM_FILE, b"babawus\n", SystemTime::UNIX_EPOCH);

    let this = ThisProcess::read(&system, COMPUTER).unwrap();

    assert_eq!(this.computer_id, COMPUTER);
    assert_eq!(this.boot_id, BOOT);
    assert_eq!(this.process_id, std::process::id());
    assert_eq!(this.program, "babawus");
}

/// Keeps a copy of this test program running, for the test below.
#[test]
fn sleeps_when_started_by_another_test() {
    if std::env::var_os(SLEEP_IN_CHILD).is_some() {
        std::thread::sleep(Duration::from_secs(60));
    }
}

/// Starts a copy of this test program, and waits until it runs as this program. Until `exec`
/// has replaced it, the new process still has the name of the test thread that started it.
fn start_copy_of_this_program() -> Child {
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "lock::tests::sleeps_when_started_by_another_test",
            "--exact",
        ])
        .env(SLEEP_IN_CHILD, "1")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let own = std::fs::read_to_string("/proc/self/comm").unwrap();
    let name = format!("/proc/{}/comm", child.id());
    let start = std::time::Instant::now();
    while std::fs::read_to_string(&name).ok().as_ref() != Some(&own) {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "the copy did not start"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    child
}

#[test]
fn real_processes_are_told_apart() {
    let this = ThisProcess::read(&RealFs, COMPUTER).unwrap();
    assert_eq!(this.boot_id.len(), BOOT.len());
    let medium = FakeFs::new();
    medium.add_file(root().join(".babawus/file"), b"", SystemTime::UNIX_EPOCH);
    let check = |process: &Child| {
        let holder = Holder {
            boot_id: this.boot_id.clone(),
            ..holder(process.id())
        };
        medium.add_file(
            lock_path(),
            toml::to_string(&holder).unwrap().as_bytes(),
            SystemTime::UNIX_EPOCH,
        );
        live_holder(&medium, &root(), &RealFs, &this)
            .unwrap()
            .is_some()
    };

    let mut other_program = Command::new("sleep").arg("60").spawn().unwrap();
    assert!(!check(&other_program));
    other_program.kill().unwrap();
    other_program.wait().unwrap();

    let mut copy = start_copy_of_this_program();
    assert!(check(&copy));
    copy.kill().unwrap();
    copy.wait().unwrap();
    assert!(!check(&copy));
}
