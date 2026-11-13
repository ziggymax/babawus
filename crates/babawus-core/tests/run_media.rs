//! Whole backups on real exFAT and Ext4.
//!
//! These use sudo and loop devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, SystemTime};

use babawus_core::config::{Config, Loaded, load, save};
use babawus_core::fs::RealFs;
use babawus_core::lock::ThisProcess;
use babawus_core::log::{new_run_id, read_runs};
use babawus_core::medium::{LOCK_FILE, TEMP_FOLDER};
use babawus_core::run::{Backup, Ended, Message, Outcome, Progress};
use babawus_core::verdict::Reason;
use babawus_core::versions::{Format, version_folder};
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;
const ID: &str = "0123456789abcdef0123456789abcdef";

/// A home folder that is removed afterwards.
struct Home(PathBuf);

impl Home {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("babawus-home-{}", new_run_id()));
        fs::create_dir_all(path.join("Documents/empty")).unwrap();
        Self(path)
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Test {
    medium: LoopMedium,
    home: Home,
    format: Format,
}

impl Test {
    fn new(format: Format) -> Self {
        let medium_format = match format {
            Format::ExFat => MediumFormat::ExFat,
            Format::Ext4 => MediumFormat::Ext4,
        };
        let medium = LoopMedium::new(medium_format, 32 * MIB);
        fs::create_dir(medium.path().join(".babawus")).unwrap();
        let mut config = Config::new();
        config.add_computer(ID, "pc", "mint", &[]).unwrap();
        config.add_account(ID, "usera").unwrap();
        config.computers[0].accounts[0].selected = vec!["Documents".to_owned()];
        save(&RealFs, medium.path(), format, &config).unwrap();
        Self {
            medium,
            home: Home::new(),
            format,
        }
    }

    fn root(&self) -> &Path {
        self.medium.path()
    }

    fn write(&self, relative: &str, data: &[u8]) {
        fs::write(self.home.0.join(relative), data).unwrap();
    }

    fn backup(&self) -> Backup {
        Backup {
            medium: Arc::new(RealFs),
            root: self.root().to_owned(),
            format: self.format,
            system: Arc::new(RealFs),
            this: ThisProcess::read(&RealFs, ID).unwrap(),
            username: "usera".to_owned(),
            hostname: "mint".to_owned(),
            home: self.home.0.clone(),
            now: Arc::new(SystemTime::now),
            state_file: None,
        }
    }

    fn run(&self) -> Outcome {
        match self.backup().run(&AtomicBool::new(false), &mut |_| {}) {
            Ended::Finished(outcome) => outcome,
            Ended::Busy(holder) => panic!("busy: {holder:?}"),
        }
    }

    fn copy_path(&self, relative: &str) -> PathBuf {
        self.root().join("backup/pc/usera").join(relative)
    }
}

fn runs_change_and_catch_up(format: Format) {
    let test = Test::new(format);
    test.write("Documents/a.txt", b"first");
    test.write("Documents/b.txt", &vec![7; 5 * MIB as usize]);

    let first = test.run();

    assert_eq!(first.reason, Reason::None);
    assert_eq!(first.stats.copied_files, 2);
    assert_eq!(first.stats.copied_bytes, 5 * MIB + 5);
    assert!(first.stats.free_bytes > 0);
    let a = test.copy_path("Documents/a.txt");
    assert_eq!(fs::read(&a).unwrap(), b"first");
    assert!(test.copy_path("Documents/empty").is_dir());

    let second = test.run();

    assert_eq!(second.reason, Reason::None);
    assert_eq!(second.stats.copied_files, 0);

    test.write("Documents/a.txt", b"changed");
    let later = SystemTime::now() + Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(test.home.0.join("Documents/a.txt"))
        .unwrap()
        .set_modified(later)
        .unwrap();

    let third = test.run();

    assert_eq!(third.stats.copied_files, 1);
    assert_eq!(fs::read(&a).unwrap(), b"changed");
    let folder = version_folder(OsStr::new("a.txt"));
    let versions = test.copy_path("Documents").join(folder);
    let older = fs::read_dir(versions).unwrap().next().unwrap().unwrap();
    assert_eq!(fs::read(older.path()).unwrap(), b"first");

    assert!(!test.root().join(LOCK_FILE).exists());
    let temp = fs::read_dir(test.root().join(TEMP_FOLDER)).unwrap();
    assert_eq!(temp.count(), 0);
    let runs = read_runs(&RealFs, test.root()).unwrap();
    assert_eq!(runs.len(), 3);
    assert!(runs.iter().all(|run| run.counts_as_backup()));
}

fn a_worker_thread_runs_a_backup(format: Format) {
    let test = Test::new(format);
    test.write("Documents/a.txt", b"first");

    let running = test.backup().start();
    let messages: Vec<Message> = running.messages.iter().collect();

    assert_eq!(messages[0], Message::Progress(Progress::Scanning));
    let last = messages.last().unwrap();
    assert!(matches!(
        last,
        Message::Ended(Ended::Finished(outcome)) if outcome.reason == Reason::None
    ));
    assert!(test.copy_path("Documents/a.txt").exists());
}

fn a_new_hostname_is_saved(format: Format) {
    let test = Test::new(format);
    test.write("Documents/a.txt", b"first");
    let mut backup = test.backup();
    backup.hostname = "new-name".to_owned();

    let ended = backup.run(&AtomicBool::new(false), &mut |_| {});

    assert!(matches!(ended, Ended::Finished(outcome) if outcome.reason == Reason::None));
    let Loaded::Ready(config) = load(&RealFs, test.root()).unwrap() else {
        panic!("the configuration is not in order");
    };
    assert_eq!(config.computer(ID).unwrap().hostname, "new-name");
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn a_new_hostname_is_saved_on_exfat() {
    a_new_hostname_is_saved(Format::ExFat);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn a_new_hostname_is_saved_on_ext4() {
    a_new_hostname_is_saved(Format::Ext4);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn runs_change_and_catch_up_on_exfat() {
    runs_change_and_catch_up(Format::ExFat);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn runs_change_and_catch_up_on_ext4() {
    runs_change_and_catch_up(Format::Ext4);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn a_worker_thread_runs_a_backup_on_exfat() {
    a_worker_thread_runs_a_backup(Format::ExFat);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn a_worker_thread_runs_a_backup_on_ext4() {
    a_worker_thread_runs_a_backup(Format::Ext4);
}
