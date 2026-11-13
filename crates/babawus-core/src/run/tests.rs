use std::ffi::OsStr;
use std::io::ErrorKind;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

use super::*;
use crate::config::save;
use crate::copy::CHUNK_SIZE;
use crate::fs::fake::FakeFs;
use crate::lock::Holder;
use crate::log::{Run, read_runs};
use crate::medium::{CONFIG_FILE, LOCK_FILE, LOG_FILE, TEMP_FOLDER, read_file};
use crate::versions::{older_version, version_folder};

const ID: &str = "0123456789abcdef0123456789abcdef";
const OTHER_ID: &str = "fedcba9876543210fedcba9876543210";
const BOOT: &str = "11111111-2222-3333-4444-555555555555";
const OTHER_PROCESS: u32 = 2000;
/// 2026-11-03 16:54:00 UTC.
const NOW: u64 = 1_793_724_840;

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

fn root() -> PathBuf {
    PathBuf::from("/medium")
}

fn config() -> Config {
    let mut config = Config::new();
    config.add_computer(ID, "pc", "mint", &[]).unwrap();
    config.add_account(ID, "usera").unwrap();
    config.computers[0].accounts[0].selected = vec!["Documents".to_owned()];
    config
}

struct Setup {
    medium: Arc<FakeFs>,
    system: Arc<FakeFs>,
    cancel: AtomicBool,
}

impl Setup {
    /// A medium set up for `usera`, with nothing in the home folder.
    fn empty() -> Self {
        let setup = Self {
            medium: Arc::new(FakeFs::new()),
            system: Arc::new(FakeFs::new()),
            cancel: AtomicBool::new(false),
        };
        let file = root().join(".babawus/file");
        setup.medium.add_file(file, b"", SystemTime::UNIX_EPOCH);
        setup.save(&config());
        setup
    }

    /// The same, with two files and an empty folder.
    fn new() -> Self {
        let setup = Self::empty();
        setup.source_file("Documents/a.txt", b"first", 1000);
        setup.source_file("Documents/b.txt", b"second", 1000);
        setup.source_file("Documents/empty/.tmp-marker~", b"", 1000);
        setup
    }

    fn save(&self, config: &Config) {
        save(&*self.medium, &root(), Format::Ext4, config).unwrap();
    }

    fn source_file(&self, relative: &str, data: &[u8], modified: u64) {
        let path = Path::new("/home/usera").join(relative);
        self.system.add_file(path, data, at(modified));
    }

    fn backup(&self) -> Backup {
        Backup {
            medium: Arc::clone(&self.medium) as Arc<dyn FileSystem>,
            root: root(),
            format: Format::Ext4,
            system: Arc::clone(&self.system) as Arc<dyn FileSystem>,
            this: ThisProcess {
                computer_id: ID.to_owned(),
                boot_id: BOOT.to_owned(),
                process_id: 1000,
                program: "babawus".to_owned(),
            },
            username: "usera".to_owned(),
            hostname: "mint".to_owned(),
            home: PathBuf::from("/home/usera"),
            now: Arc::new(|| at(NOW)),
        }
    }

    fn run_with(&self, backup: &Backup, on_progress: &mut dyn FnMut(Progress)) -> Ended {
        backup.run(&self.cancel, on_progress)
    }

    fn run(&self) -> Outcome {
        self.outcome_of(&self.backup())
    }

    fn outcome_of(&self, backup: &Backup) -> Outcome {
        match self.run_with(backup, &mut |_| {}) {
            Ended::Finished(outcome) => outcome,
            Ended::Busy(holder) => panic!("busy: {holder:?}"),
        }
    }

    /// Runs the backup, and sets Cancel at the first progress message that `when` picks.
    fn run_cancelling(&self, when: impl Fn(Progress) -> bool) -> Outcome {
        let cancel = &self.cancel;
        let ended = self.backup().run(cancel, &mut |progress| {
            if when(progress) {
                cancel.store(true, Ordering::Relaxed);
            }
        });
        match ended {
            Ended::Finished(outcome) => outcome,
            Ended::Busy(holder) => panic!("busy: {holder:?}"),
        }
    }

    fn progress(&self) -> Vec<Progress> {
        let mut messages = Vec::new();
        self.run_with(&self.backup(), &mut |progress| messages.push(progress));
        messages
    }

    fn copy(&self, relative: &str) -> Option<Vec<u8>> {
        let path = root().join("backup/pc/usera").join(relative);
        read_file(&*self.medium, &path).ok()
    }

    fn runs(&self) -> Vec<Run> {
        read_runs(&*self.medium, &root()).unwrap()
    }

    fn last_run(&self) -> Run {
        self.runs().pop().unwrap()
    }

    fn log_text(&self) -> String {
        let log = read_file(&*self.medium, &root().join(LOG_FILE)).unwrap();
        String::from_utf8(log).unwrap()
    }

    fn has(&self, relative: &str) -> bool {
        self.medium.metadata(&root().join(relative)).is_ok()
    }

    fn temp_files(&self) -> usize {
        self.medium
            .read_dir(&root().join(TEMP_FOLDER))
            .unwrap()
            .len()
    }

    fn put_live_lock(&self) -> Holder {
        let holder = Holder {
            run_id: "fedcba9876543210".to_owned(),
            account: "userb@pc".to_owned(),
            computer_id: ID.to_owned(),
            boot_id: BOOT.to_owned(),
            process_id: OTHER_PROCESS,
        };
        let name = format!("/proc/{OTHER_PROCESS}/comm");
        self.system
            .add_file(name, b"babawus\n", SystemTime::UNIX_EPOCH);
        let text = toml::to_string(&holder).unwrap();
        self.medium.add_file(
            root().join(LOCK_FILE),
            text.as_bytes(),
            SystemTime::UNIX_EPOCH,
        );
        holder
    }

    /// Leaves room on the medium for `room` more bytes.
    fn leave_room(&self, room: u64) {
        const CAPACITY: u64 = 1_000_000;
        self.medium.set_capacity(CAPACITY);
        let used = CAPACITY - self.medium.free_space(&root()).unwrap();
        self.medium.set_capacity(used + room);
    }
}

#[test]
fn a_backup_copies_the_selection_and_ends_all_good() {
    let setup = Setup::new();

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::None);
    assert_eq!(outcome.verdict(), Verdict::AllGood);
    assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"first");
    assert_eq!(setup.copy("Documents/b.txt").unwrap(), b"second");
    assert!(setup.has("backup/pc/usera/Documents/empty"));
    assert_eq!(outcome.stats.copied_bytes, 11);
    assert!(outcome.skipped.is_empty());
}

#[test]
fn the_log_has_the_whole_run() {
    let setup = Setup::new();
    setup.medium.set_capacity(1_000_000);

    let outcome = setup.run();

    let run = setup.last_run();
    assert_eq!(run.account, "usera@pc");
    let end = run.end.unwrap();
    assert_eq!(end.verdict, Verdict::AllGood);
    assert_eq!(end.reason, Reason::None);
    assert_eq!(end.stats, Some(outcome.stats));
    assert!(outcome.stats.free_bytes > 0);
    assert_eq!(outcome.stats.scanned_files, 2);
    assert_eq!(outcome.stats.copied_files, 2);
}

#[test]
fn the_lock_is_gone_after_a_run() {
    let setup = Setup::new();

    setup.run();

    assert!(!setup.has(LOCK_FILE));
}

#[test]
fn the_lock_is_held_until_the_end_entry_is_written() {
    let setup = Setup::new();
    // The steps of a run on an empty log, and with the lock removal as the last one.
    let steps = {
        let setup = Setup::new();
        let before = setup.medium.steps();
        setup.run();
        setup.medium.steps() - before
    };
    setup.medium.pull_at_step(setup.medium.steps() + steps);

    setup.run();
    setup.medium.reinsert();

    assert!(setup.has(LOCK_FILE));
    assert!(setup.last_run().end.is_some());
}

#[test]
fn a_second_run_copies_nothing_new() {
    let setup = Setup::new();
    setup.run();

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::None);
    assert_eq!(outcome.stats.scanned_files, 2);
    assert_eq!(outcome.stats.copied_files, 0);
    assert_eq!(setup.runs().len(), 2);
}

#[test]
fn a_changed_file_keeps_its_old_copy_under_a_utc_name() {
    let setup = Setup::new();
    setup.run();
    setup.source_file("Documents/a.txt", b"changed", 2000);

    setup.run();

    let name = OsStr::new("a.txt");
    let version = older_version(name, "2026-11-03_165400", 1);
    let folder = version_folder(name);
    let path = Path::new("Documents").join(folder).join(version);
    assert_eq!(setup.copy(path.to_str().unwrap()).unwrap(), b"first");
    assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"changed");
}

#[test]
fn the_older_versions_setting_is_used_and_counted() {
    let setup = Setup::new();
    let mut changed = config();
    changed.computers[0].accounts[0].older_versions = 1;
    setup.save(&changed);
    setup.run();
    let mut time = NOW;
    let mut removed = 0;
    for content in [&b"two"[..], b"three", b"four"] {
        time += 60;
        setup.source_file("Documents/a.txt", content, time);
        let mut backup = setup.backup();
        backup.now = Arc::new(move || at(time));
        removed += setup.outcome_of(&backup).stats.versions_removed;
    }

    assert_eq!(removed, 2);
}

#[test]
fn progress_starts_with_the_overview_and_counts_down_to_nothing() {
    let setup = Setup::new();

    let messages = setup.progress();

    assert_eq!(messages[0], Progress::Scanning);
    assert_eq!(
        messages[1],
        Progress::Overview {
            files: 2,
            bytes: 11
        }
    );
    assert_eq!(messages.last(), Some(&Progress::Finishing));
    let left = |files_left, bytes_left| Progress::Copying {
        files_left,
        bytes_left,
    };
    assert!(messages.contains(&left(1, 6)));
    assert_eq!(messages[messages.len() - 2], left(0, 0));
}

#[test]
fn the_byte_counter_moves_while_a_large_file_is_copied() {
    let setup = Setup::empty();
    let size = 2 * CHUNK_SIZE + 10;
    setup.source_file("Documents/large", &vec![1; size], 1000);

    let messages = setup.progress();

    let left = |bytes_left| Progress::Copying {
        files_left: 1,
        bytes_left,
    };
    assert!(messages.contains(&left((CHUNK_SIZE + 10) as u64)));
    assert!(messages.contains(&left(10)));
}

#[test]
fn an_empty_selection_is_all_good() {
    let setup = Setup::empty();
    let mut changed = config();
    changed.computers[0].accounts[0].selected = Vec::new();
    setup.save(&changed);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::None);
}

#[test]
fn a_skip_found_by_the_scan_gives_its_verdict_and_log_entry() {
    let setup = Setup::new();
    let mut changed = config();
    let selected = &mut changed.computers[0].accounts[0].selected;
    selected.push("Gone".to_owned());
    setup.save(&changed);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::FilesSkipped);
    assert_eq!(outcome.verdict(), Verdict::Skipped);
    assert_eq!(outcome.skipped[0].reason, Skip::SelectionMissing);
    assert_eq!(outcome.stats.errors, 1);
    assert_eq!(setup.last_run().errors[0].code, "SELECTION_MISSING");
    assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"first");
}

#[test]
fn a_skip_while_copying_gives_its_verdict_and_the_run_goes_on() {
    let setup = Setup::new();
    let folder = root().join("backup/pc/usera/Documents/a.txt/inside");
    setup.medium.add_file(folder, b"", at(1));

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::FilesSkipped);
    assert_eq!(outcome.skipped[0].reason, Skip::TypeConflict);
    assert_eq!(outcome.skipped[0].files, 1);
    assert_eq!(outcome.stats.skipped_files, 1);
    assert_eq!(outcome.stats.copied_files, 1);
    assert_eq!(setup.copy("Documents/b.txt").unwrap(), b"second");
    assert_eq!(setup.last_run().errors[0].code, "TYPE_CONFLICT");
}

#[test]
fn the_files_in_a_skipped_folder_are_counted() {
    let setup = Setup::new();
    setup.source_file("Documents/bad:name/one", b"1", 1000);
    setup.source_file("Documents/bad:name/two", b"2", 1000);
    let mut backup = setup.backup();
    backup.format = Format::ExFat;

    let outcome = setup.outcome_of(&backup);

    assert_eq!(outcome.stats.skipped_files, 2);
    assert_eq!(setup.last_run().errors[0].files, Some(2));
}

#[test]
fn a_failed_prune_is_logged_and_does_not_change_the_verdict() {
    let setup = Setup::new();
    let mut changed = config();
    changed.computers[0].accounts[0].older_versions = 1;
    setup.save(&changed);
    setup.run();
    let name = OsStr::new("a.txt");
    let folder = root()
        .join("backup/pc/usera/Documents")
        .join(version_folder(name));
    let oldest = folder.join(older_version(name, "2020-01-01_000000", 1));
    let newer = folder.join(older_version(name, "2021-01-01_000000", 1));
    setup.medium.add_file(&oldest, b"old", at(1));
    setup.medium.add_file(&newer, b"old", at(1));
    setup.medium.fail_path(&oldest, ErrorKind::PermissionDenied);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::None);
    assert_eq!(outcome.prune_failures, [PathBuf::from("Documents/a.txt")]);
    assert_eq!(outcome.stats.errors, 1);
    assert_eq!(setup.last_run().errors[0].code, PRUNE_FAILED);
}

#[test]
fn a_full_medium_stops_the_run_and_is_logged() {
    let setup = Setup::empty();
    setup.source_file("Documents/a.txt", &[1; 5000], 1000);
    setup.source_file("Documents/b.txt", b"small", 1000);
    setup.leave_room(1000);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::MediumFull);
    assert_eq!(outcome.verdict(), Verdict::Failed);
    assert_eq!(setup.copy("Documents/b.txt"), None);
    assert_eq!(setup.last_run().end.unwrap().reason, Reason::MediumFull);
    assert!(!setup.has(LOCK_FILE));
}

#[test]
fn the_worst_verdict_wins() {
    let setup = Setup::empty();
    setup.source_file("Documents/a.txt", &[1; 5000], 1000);
    let mut changed = config();
    let selected = &mut changed.computers[0].accounts[0].selected;
    selected.push("Gone".to_owned());
    setup.save(&changed);
    setup.leave_room(1000);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::MediumFull);
    assert_eq!(outcome.skipped.len(), 1);
}

#[test]
fn an_account_that_has_not_been_added_copies_nothing() {
    let setup = Setup::new();
    let mut backup = setup.backup();
    backup.username = "userb".to_owned();

    let outcome = setup.outcome_of(&backup);

    assert_eq!(outcome.reason, Reason::NotAdded);
    assert_eq!(outcome.verdict(), Verdict::Failed);
    assert!(!setup.has("backup"));
    let run = setup.last_run();
    assert_eq!(run.account, "userb@pc");
    assert_eq!(run.end.unwrap().reason, Reason::NotAdded);
}

#[test]
fn a_computer_that_has_not_been_added_is_logged_by_its_hostname() {
    let setup = Setup::new();
    let mut backup = setup.backup();
    backup.this.computer_id = OTHER_ID.to_owned();

    let outcome = setup.outcome_of(&backup);

    assert_eq!(outcome.reason, Reason::NotAdded);
    assert_eq!(setup.last_run().account, "usera@mint");
}

#[test]
fn a_computer_without_an_id_counts_as_not_added() {
    let setup = Setup::new();
    let mut backup = setup.backup();
    backup.this.computer_id = String::new();

    assert_eq!(setup.outcome_of(&backup).reason, Reason::NotAdded);
}

#[test]
fn a_damaged_configuration_has_its_own_reason_and_the_medium_is_not_touched() {
    let setup = Setup::new();
    let path = root().join(CONFIG_FILE);
    setup.medium.add_file(path, b"nonsense", at(1));
    let steps = setup.medium.steps();

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::ConfigDamaged);
    assert_eq!(outcome.verdict(), Verdict::Failed);
    assert_eq!(setup.medium.steps(), steps);
}

#[test]
fn a_configuration_from_a_newer_babawus_counts_as_damaged_and_is_not_touched() {
    let setup = Setup::new();
    let path = root().join(CONFIG_FILE);
    let newer = b"# Babawus configuration\nformat = 999\n";
    setup.medium.add_file(path, newer, at(1));
    let steps = setup.medium.steps();

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::ConfigDamaged);
    assert_eq!(setup.medium.steps(), steps);
}

#[test]
fn a_medium_without_a_babawus_folder_is_not_touched() {
    let setup = Setup::new();
    let medium = Arc::new(FakeFs::new());
    medium.add_file(root().join("holiday.jpg"), b"", at(1));
    let mut backup = setup.backup();
    backup.medium = Arc::clone(&medium) as Arc<dyn FileSystem>;
    let steps = medium.steps();

    let outcome = setup.outcome_of(&backup);

    assert_eq!(outcome.reason, Reason::NotAdded);
    assert_eq!(medium.steps(), steps);
}

#[test]
fn a_medium_whose_mount_folder_has_gone_is_reported_as_removed() {
    let setup = Setup::new();
    let mut backup = setup.backup();
    backup.medium = Arc::new(FakeFs::new());

    let outcome = setup.outcome_of(&backup);

    assert_eq!(outcome.reason, Reason::MediumRemoved);
    assert_eq!(outcome.verdict(), Verdict::NotFinished);
}

#[test]
fn a_medium_without_a_configuration_is_not_touched() {
    let setup = Setup::new();
    setup.medium.remove_file(&root().join(CONFIG_FILE)).unwrap();
    let steps = setup.medium.steps();

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::NotAdded);
    assert_eq!(setup.medium.steps(), steps);
}

#[test]
fn a_live_lock_means_busy_and_nothing_changes() {
    let setup = Setup::new();
    let holder = setup.put_live_lock();
    let steps = setup.medium.steps();

    let ended = setup.run_with(&setup.backup(), &mut |_| {});

    assert_eq!(ended, Ended::Busy(Some(holder)));
    assert_eq!(setup.medium.steps(), steps);
    assert!(setup.has(LOCK_FILE));
}

#[test]
fn a_run_starts_by_emptying_the_temporary_folder() {
    let setup = Setup::new();
    let left_over = root().join(TEMP_FOLDER).join("copy-1");
    setup.medium.add_file(left_over, b"half", at(1));

    setup.run();

    assert_eq!(setup.temp_files(), 0);
}

#[test]
fn a_run_closes_earlier_interrupted_runs() {
    let setup = Setup::new();
    let log = "2026-11-03 17:54:00 +0100\t0123456789abcdef\tSTART\taccount=usera@pc\n";
    setup
        .medium
        .add_file(root().join(LOG_FILE), log.as_bytes(), at(1));

    setup.run();

    let runs = setup.runs();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].end.unwrap().reason, Reason::Interrupted);
    assert_eq!(runs[1].end.unwrap().reason, Reason::None);
}

#[test]
fn the_interrupted_run_is_closed_before_the_new_one_starts() {
    let setup = Setup::new();
    let log = "2026-11-03 17:54:00 +0100\t0123456789abcdef\tSTART\taccount=usera@pc\n";
    setup
        .medium
        .add_file(root().join(LOG_FILE), log.as_bytes(), at(1));

    setup.run();

    let text = setup.log_text();
    let closed = text.find("reason=INTERRUPTED").unwrap();
    let started = text.rfind("START").unwrap();
    assert!(closed < started);
}

#[test]
fn cancelling_during_the_scan_copies_nothing() {
    let setup = Setup::new();
    setup.cancel.store(true, Ordering::Relaxed);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::Cancelled);
    assert_eq!(outcome.verdict(), Verdict::NotFinished);
    assert!(!setup.has("backup"));
    assert_eq!(setup.last_run().end.unwrap().reason, Reason::Cancelled);
    assert!(!setup.has(LOCK_FILE));
}

#[test]
fn cancelling_after_the_scan_stops_before_the_first_file() {
    let setup = Setup::new();

    let outcome = setup.run_cancelling(|progress| matches!(progress, Progress::Overview { .. }));

    assert_eq!(outcome.reason, Reason::Cancelled);
    assert_eq!(outcome.stats.copied_files, 0);
    assert_eq!(outcome.stats.scanned_files, 2);
}

#[test]
fn cancelling_stops_a_run_that_has_nothing_to_copy() {
    let setup = Setup::new();
    setup.run();

    let outcome = setup.run_cancelling(|progress| matches!(progress, Progress::Overview { .. }));

    assert_eq!(outcome.reason, Reason::Cancelled);
}

#[test]
fn cancelling_between_files_keeps_the_file_that_was_finished() {
    let setup = Setup::new();

    let outcome = setup.run_cancelling(|progress| {
        matches!(
            progress,
            Progress::Copying {
                files_left: 1,
                bytes_left: 6
            }
        )
    });

    assert_eq!(outcome.reason, Reason::Cancelled);
    assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"first");
    assert_eq!(setup.copy("Documents/b.txt"), None);
    assert_eq!(outcome.stats.copied_files, 1);
}

#[test]
fn cancelling_during_a_large_file_discards_the_temporary_file() {
    let setup = Setup::empty();
    setup.source_file("Documents/large", &vec![1; 2 * CHUNK_SIZE], 1000);

    let outcome = setup.run_cancelling(
        |progress| matches!(progress, Progress::Copying { bytes_left, .. } if bytes_left > 0),
    );

    assert_eq!(outcome.reason, Reason::Cancelled);
    assert_eq!(outcome.stats.copied_files, 0);
    assert_eq!(setup.copy("Documents/large"), None);
    assert_eq!(setup.temp_files(), 0);
}

#[test]
fn cancelling_after_the_last_file_changes_nothing() {
    let setup = Setup::new();

    let outcome = setup.run_cancelling(|progress| progress == Progress::Finishing);

    assert_eq!(outcome.reason, Reason::None);
}

#[test]
fn a_lock_that_cannot_be_written_means_the_medium_cannot_be_written() {
    let setup = Setup::new();
    setup
        .medium
        .fail_path(root().join(LOCK_FILE), ErrorKind::PermissionDenied);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::CannotWrite);
    assert_eq!(outcome.verdict(), Verdict::Failed);
    assert!(!setup.has("backup"));
}

#[test]
fn a_log_that_cannot_be_written_stops_the_run_before_any_copying() {
    let setup = Setup::new();
    setup
        .medium
        .fail_path(root().join(LOG_FILE), ErrorKind::PermissionDenied);

    let outcome = setup.run();

    assert_eq!(outcome.reason, Reason::CannotWrite);
    assert!(!setup.has("backup"));
    assert!(!setup.has(LOCK_FILE));
}

#[test]
fn a_medium_pulled_before_the_run_is_reported_as_removed() {
    let setup = Setup::new();
    setup.medium.pull_at_step(setup.medium.steps() + 1);
    assert!(setup.medium.create_dir(&root().join("x")).is_err());

    assert_eq!(setup.run().reason, Reason::MediumRemoved);
}

/// Pulls the medium at each step of a backup that adds one file and changes another. In every
/// case the verdict must be honest, and the next run must simply catch up.
fn check_pulling_at_every_step(losing_unflushed: bool) {
    let prepare = || {
        let setup = Setup::new();
        setup.run();
        setup.source_file("Documents/a.txt", b"changed", 2000);
        setup.source_file("Documents/new.txt", b"new", 2000);
        setup
    };
    let steps = {
        let setup = prepare();
        let before = setup.medium.steps();
        setup.run();
        setup.medium.steps() - before
    };
    assert!(steps >= 15);

    for step in 1..=steps {
        let setup = prepare();
        let runs_before = setup.runs().len();
        setup.medium.pull_at_step(setup.medium.steps() + step);

        let interrupted = setup.run();

        if losing_unflushed {
            setup.medium.reinsert_losing_unflushed();
        } else {
            setup.medium.reinsert();
        }
        if interrupted.verdict() == Verdict::AllGood {
            // Once the verdict appears, everything is on the medium.
            let end = setup.last_run().end.unwrap();
            assert_eq!(end.verdict, Verdict::AllGood, "step {step}");
            assert_eq!(setup.runs().len(), runs_before + 1, "step {step}");
            assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"changed");
            assert_eq!(setup.copy("Documents/new.txt").unwrap(), b"new");
        } else {
            assert_eq!(
                interrupted.verdict(),
                Verdict::NotFinished,
                "step {step}: {interrupted:?}"
            );
        }

        let next = setup.run();

        assert_eq!(next.reason, Reason::None, "step {step}");
        assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"changed");
        assert_eq!(setup.copy("Documents/b.txt").unwrap(), b"second");
        assert_eq!(setup.copy("Documents/new.txt").unwrap(), b"new");
        assert!(!setup.has(LOCK_FILE), "step {step}");
        assert!(setup.runs().iter().all(|run| run.end.is_some()));
    }
}

#[test]
fn a_medium_pulled_at_any_step_gives_an_honest_verdict_and_the_next_run_catches_up() {
    check_pulling_at_every_step(false);
}

#[test]
fn the_same_holds_when_what_was_not_flushed_is_lost() {
    check_pulling_at_every_step(true);
}

#[test]
fn a_medium_that_cannot_be_flushed_means_the_backup_is_not_finished() {
    let setup = Setup::new();

    let ended = setup.run_with(&setup.backup(), &mut |progress| {
        if progress == Progress::Finishing {
            setup.medium.fail_path(root(), ErrorKind::Other);
        }
    });

    let Ended::Finished(outcome) = ended else {
        panic!("busy");
    };
    assert_eq!(outcome.reason, Reason::FlushFailed);
    assert_eq!(outcome.verdict(), Verdict::NotFinished);
    assert_eq!(setup.last_run().end.unwrap().reason, Reason::FlushFailed);
}

fn saved_hostname(setup: &Setup) -> String {
    let Loaded::Ready(config) = load(&*setup.medium, &root()).unwrap() else {
        panic!("the configuration is not in order");
    };
    config.computer(ID).unwrap().hostname.clone()
}

fn renamed(setup: &Setup) -> Backup {
    let mut backup = setup.backup();
    backup.hostname = "new-name".to_owned();
    backup
}

#[test]
fn a_new_hostname_is_saved_and_the_backup_stays_in_its_folder() {
    let setup = Setup::new();

    let outcome = setup.outcome_of(&renamed(&setup));

    assert_eq!(outcome.reason, Reason::None);
    assert_eq!(saved_hostname(&setup), "new-name");
    assert_eq!(setup.copy("Documents/a.txt").unwrap(), b"first");
    assert_eq!(setup.last_run().account, "usera@pc");
}

#[test]
fn the_configuration_is_only_saved_when_the_hostname_has_changed() {
    let steps_of = |backup: &dyn Fn(&Setup) -> Backup| {
        let setup = Setup::new();
        setup.run();
        let before = setup.medium.steps();
        setup.outcome_of(&backup(&setup));
        setup.medium.steps() - before
    };

    let same = steps_of(&Setup::backup);
    let changed = steps_of(&renamed);

    assert!(same < changed);
}

#[test]
fn a_new_hostname_is_saved_even_if_the_account_has_not_been_added() {
    let setup = Setup::new();
    let mut backup = renamed(&setup);
    backup.username = "userb".to_owned();

    let outcome = setup.outcome_of(&backup);

    assert_eq!(outcome.reason, Reason::NotAdded);
    assert_eq!(saved_hostname(&setup), "new-name");
}

#[test]
fn a_computer_that_has_not_been_added_stays_out_of_the_configuration() {
    let setup = Setup::new();
    let mut backup = renamed(&setup);
    backup.this.computer_id = OTHER_ID.to_owned();
    let before = read_file(&*setup.medium, &root().join(CONFIG_FILE)).unwrap();

    setup.outcome_of(&backup);

    let after = read_file(&*setup.medium, &root().join(CONFIG_FILE)).unwrap();
    assert_eq!(before, after);
}

#[test]
fn a_hostname_that_cannot_be_saved_does_not_change_the_verdict() {
    let setup = Setup::new();
    let temp = root().join(TEMP_FOLDER).join("replacement");
    setup.medium.fail_path(temp, ErrorKind::PermissionDenied);

    let outcome = setup.outcome_of(&renamed(&setup));

    assert_eq!(outcome.reason, Reason::None);
    assert_eq!(outcome.stats.errors, 0);
    assert_eq!(saved_hostname(&setup), "mint");
}

#[test]
fn a_medium_pulled_while_the_hostname_is_saved_keeps_a_good_configuration() {
    let steps = {
        let setup = Setup::new();
        let before = setup.medium.steps();
        setup.outcome_of(&renamed(&setup));
        setup.medium.steps() - before
    };

    for step in 1..=steps {
        for lose_unflushed in [false, true] {
            let setup = Setup::new();
            setup.medium.pull_at_step(setup.medium.steps() + step);

            setup.outcome_of(&renamed(&setup));

            if lose_unflushed {
                setup.medium.reinsert_losing_unflushed();
            } else {
                setup.medium.reinsert();
            }
            let hostname = saved_hostname(&setup);
            assert!(
                ["mint", "new-name"].contains(&hostname.as_str()),
                "step {step}"
            );

            let next = setup.outcome_of(&renamed(&setup));

            assert_eq!(next.reason, Reason::None, "step {step}");
            assert_eq!(saved_hostname(&setup), "new-name", "step {step}");
        }
    }
}

fn collect(running: &RunningBackup) -> (Vec<Progress>, Ended) {
    let mut progress = Vec::new();
    for message in &running.messages {
        match message {
            Message::Progress(step) => progress.push(step),
            Message::Ended(ended) => return (progress, ended),
        }
    }
    panic!("the worker stopped without an ending");
}

#[test]
fn a_backup_on_a_worker_thread_reports_and_ends() {
    let setup = Setup::new();

    let running = setup.backup().start();
    let (progress, ended) = collect(&running);

    assert_eq!(progress[0], Progress::Scanning);
    assert_eq!(progress.last(), Some(&Progress::Finishing));
    assert!(matches!(ended, Ended::Finished(outcome) if outcome.reason == Reason::None));
    assert!(!setup.has(LOCK_FILE));
    assert!(running.messages.recv().is_err());
}

#[test]
fn a_backup_on_a_worker_thread_can_be_cancelled() {
    let setup = Setup::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let released = Arc::new(AtomicBool::new(false));
    let mut backup = setup.backup();
    let (seen, go) = (Arc::clone(&calls), Arc::clone(&released));
    // The clock holds the worker back at its start entry, until Cancel has been set.
    backup.now = Arc::new(move || {
        seen.fetch_add(1, Ordering::SeqCst);
        while !go.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        at(NOW)
    });

    let running = backup.start();
    while calls.load(Ordering::SeqCst) == 0 {
        std::thread::yield_now();
    }
    running.cancel();
    released.store(true, Ordering::SeqCst);
    let (_, ended) = collect(&running);

    assert!(matches!(ended, Ended::Finished(outcome) if outcome.reason == Reason::Cancelled));
}

#[test]
fn a_worker_finishes_properly_even_if_nobody_listens() {
    let setup = Setup::new();

    let RunningBackup { messages, .. } = setup.backup().start();
    drop(messages);

    let deadline = Instant::now() + Duration::from_secs(30);
    let finished =
        || setup.has(LOG_FILE) && !setup.has(LOCK_FILE) && setup.last_run().end.is_some();
    while !finished() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(setup.last_run().end.unwrap().reason, Reason::None);
}
