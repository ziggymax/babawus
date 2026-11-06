//! The log on real exFAT and Ext4.
//!
//! These use sudo and loop devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use babawus_core::config::{Config, link_account, save};
use babawus_core::fs::RealFs;
use babawus_core::log::{RunLog, Stats, last_backup, new_run_id, read_runs};
use babawus_core::medium::LOG_FILE;
use babawus_core::time::{days_since, log_timestamp};
use babawus_core::verdict::{Reason, Verdict};
use babawus_core::versions::Format;
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;
const ACCOUNT: &str = "usera@mint";
const ID: &str = "0123456789abcdef0123456789abcdef";

struct Test {
    medium: LoopMedium,
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
        fs::create_dir(medium.path().join(".babawus/tmp")).unwrap();
        Self { medium, format }
    }

    fn root(&self) -> &Path {
        self.medium.path()
    }

    fn log_path(&self) -> PathBuf {
        self.root().join(LOG_FILE)
    }

    fn start(&self, now: &'static dyn Fn() -> String) -> RunLog<'_> {
        RunLog::start(
            &RealFs,
            self.root(),
            self.format,
            new_run_id(),
            ACCOUNT,
            now,
            None,
        )
        .unwrap()
    }
}

fn now() -> String {
    log_timestamp(SystemTime::now())
}

fn a_whole_run(format: Format) {
    let test = Test::new(format);

    let mut log = test.start(&now);
    log.error("UNREADABLE", Path::new("a\tb.txt"), 1).unwrap();
    let stats = Stats {
        scanned_files: 3,
        free_bytes: 100,
        ..Stats::default()
    };
    log.end(Reason::FilesSkipped, stats).unwrap();

    let runs = read_runs(&RealFs, test.root()).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].verdict(), Verdict::Skipped);
    assert_eq!(runs[0].errors[0].path, Some(PathBuf::from("a\tb.txt")));
    let last = last_backup(&runs, ACCOUNT).unwrap();
    assert!(last.skipped_files);
    assert_eq!(days_since(last.time, SystemTime::now()), 0);
    let tomorrow = SystemTime::now() + Duration::from_secs(25 * 3600);
    assert_eq!(days_since(last.time, tomorrow), 1);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn a_whole_run_on_exfat() {
    a_whole_run(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn a_whole_run_on_ext4() {
    a_whole_run(Format::Ext4);
}

fn an_interruption_is_caught_up_with(format: Format) {
    let test = Test::new(format);

    // A run that never ended, and whose last entry was only half written.
    drop(test.start(&now));
    let mut contents = fs::read(test.log_path()).unwrap();
    contents.extend_from_slice(b"2026-11-03 17:5");
    fs::write(test.log_path(), contents).unwrap();

    test.start(&now)
        .end(Reason::None, Stats::default())
        .unwrap();

    let runs = read_runs(&RealFs, test.root()).unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].end.unwrap().reason, Reason::Interrupted);
    assert_eq!(runs[1].verdict(), Verdict::AllGood);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn an_interruption_is_caught_up_with_on_exfat() {
    an_interruption_is_caught_up_with(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn an_interruption_is_caught_up_with_on_ext4() {
    an_interruption_is_caught_up_with(Format::Ext4);
}

fn a_long_log_is_trimmed(format: Format) {
    let test = Test::new(format);
    let path = "x".repeat(4000);
    let mut log = String::new();
    for run in 1..=300 {
        let id = format!("{run:016x}");
        let time = "2026-11-03 17:54:00 +0100";
        log.push_str(&format!(
            "{time}\t{id}\tSTART\taccount=userb@mint\n\
             {time}\t{id}\tERROR\tcode=X\tpath={path}\n\
             {time}\t{id}\tEND\tverdict=FAILED\treason=MEDIUM_FULL\n"
        ));
    }
    assert!(log.len() > 1_000_000);
    fs::write(test.log_path(), log).unwrap();

    test.start(&now)
        .end(Reason::None, Stats::default())
        .unwrap();

    assert!(fs::metadata(test.log_path()).unwrap().len() < 1_000_000);
    let runs = read_runs(&RealFs, test.root()).unwrap();
    assert!(runs.len() < 300);
    assert_eq!(runs.last().unwrap().verdict(), Verdict::AllGood);
    assert_eq!(
        fs::read_dir(test.root().join(".babawus/tmp"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn a_long_log_is_trimmed_on_exfat() {
    a_long_log_is_trimmed(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn a_long_log_is_trimmed_on_ext4() {
    a_long_log_is_trimmed(Format::Ext4);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn on_ext4_everyone_can_read_and_write_the_log() {
    let test = Test::new(Format::Ext4);

    test.start(&now)
        .end(Reason::None, Stats::default())
        .unwrap();

    let mode = fs::metadata(test.log_path()).unwrap().permissions().mode();
    assert_eq!(mode & 0o7777, 0o666);
}

fn linking_keeps_the_last_backup(format: Format) {
    let test = Test::new(format);
    let mut config = Config::new();
    config.add_computer(ID, "mint", "mint", &[]).unwrap();
    config.add_account(ID, "usera").unwrap();
    save(&RealFs, test.root(), format, &config).unwrap();
    test.start(&now)
        .end(Reason::None, Stats::default())
        .unwrap();

    link_account(
        &RealFs,
        test.root(),
        format,
        &mut config,
        ID,
        "usera",
        "anna",
    )
    .unwrap();

    let runs = read_runs(&RealFs, test.root()).unwrap();
    assert!(last_backup(&runs, "anna@mint").is_some());
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn linking_keeps_the_last_backup_on_exfat() {
    linking_keeps_the_last_backup(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn linking_keeps_the_last_backup_on_ext4() {
    linking_keeps_the_last_backup(Format::Ext4);
}
