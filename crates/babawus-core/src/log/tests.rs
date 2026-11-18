use std::os::unix::ffi::OsStrExt;
use std::sync::Mutex;
use std::time::Duration;

use super::*;
use crate::fs::fake::FakeFs;
use crate::time::log_timestamp;

const TIME: &str = "2026-11-03 17:54:00 +0100";
const LATER: &str = "2026-11-04 09:00:00 +0100";
const RUN: &str = "0123456789abcdef";
const OTHER_RUN: &str = "fedcba9876543210";
const ACCOUNT: &str = "usera@mint";
const RUNNER: Runner = Runner {
    account: ACCOUNT,
    hostname: "Mint-PC",
};
const END_FIELDS: &str = "verdict=ALL_GOOD\treason=NONE\tscanned_files=5\tcopied_files=2\t\
                          copied_bytes=300\tskipped_files=0\terrors=0\tversions_removed=1\t\
                          free_bytes=4000";

fn root() -> PathBuf {
    PathBuf::from("/medium")
}

fn log_path() -> PathBuf {
    root().join(LOG_FILE)
}

fn start_line(run: &str, account: &str) -> String {
    format!("{TIME}\t{run}\tSTART\taccount={account}\tBackup started\n")
}

fn end_line(run: &str, verdict: &str, reason: &str) -> String {
    format!(
        "{TIME}\t{run}\tEND\tverdict={verdict}\treason={reason}\tscanned_files=0\t\
         copied_files=0\tcopied_bytes=0\tskipped_files=0\terrors=0\tversions_removed=0\t\
         free_bytes=0\n"
    )
}

/// A whole run, with the given number as its ID.
fn run_lines(number: u64, account: &str, verdict: &str, reason: &str) -> String {
    let run = format!("{number:016x}");
    start_line(&run, account) + &end_line(&run, verdict, reason)
}

struct Setup {
    medium: FakeFs,
    clock: Mutex<String>,
}

impl Setup {
    fn new() -> Self {
        let medium = FakeFs::new();
        medium.add_dir_on_device(root().join(".babawus/tmp"), 0);
        Self {
            medium,
            clock: Mutex::new(LATER.to_owned()),
        }
    }

    fn with_log(contents: &str) -> Self {
        Self::with_log_bytes(contents.as_bytes())
    }

    fn with_log_bytes(contents: &[u8]) -> Self {
        let setup = Self::new();
        setup
            .medium
            .add_file(log_path(), contents, SystemTime::UNIX_EPOCH);
        setup
    }

    fn contents(&self) -> Vec<u8> {
        read_file(&self.medium, &log_path()).unwrap()
    }

    fn text(&self) -> String {
        String::from_utf8(self.contents()).unwrap()
    }

    fn runs(&self) -> Vec<Run> {
        read_runs(&self.medium, &root()).unwrap()
    }

    fn with_run<T>(&self, format: Format, action: impl FnOnce(RunLog) -> io::Result<T>) -> T {
        let now = || self.clock.lock().unwrap().clone();
        let root = root();
        let log = RunLog::start(
            &self.medium,
            &root,
            format,
            RUN.to_owned(),
            &RUNNER,
            &now,
            None,
        )
        .unwrap();
        action(log).unwrap()
    }

    fn start_only(&self) {
        self.with_run(Format::ExFat, |_| Ok(()));
    }

    fn whole_run(&self) {
        self.with_run(Format::ExFat, |log| log.end(Reason::None, Stats::default()));
    }
}

fn os_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(platform::name_from_bytes(bytes.to_vec()))
}

#[test]
fn a_run_writes_the_entries_in_the_requirements() {
    let setup = Setup::new();

    setup.with_run(Format::ExFat, |mut log| {
        log.error("UNREADABLE", Path::new("Documents/a.txt"), 1)?;
        log.error("CASE_CLASH", Path::new("Bilder"), 12)?;
        let stats = Stats {
            scanned_files: 5,
            copied_files: 2,
            copied_bytes: 300,
            skipped_files: 13,
            errors: 0,
            versions_removed: 1,
            free_bytes: 4000,
        };
        log.end(Reason::FilesSkipped, stats)
    });

    let version = crate::VERSION;
    let expected = format!(
        "{LATER}\t{RUN}\tSTART\taccount=usera@mint\tversion={version}\thostname=Mint-PC\t\
         Backup started\n\
         {LATER}\t{RUN}\tERROR\tcode=UNREADABLE\tpath=Documents/a.txt\n\
         {LATER}\t{RUN}\tERROR\tcode=CASE_CLASH\tpath=Bilder\tfiles=12\n\
         {LATER}\t{RUN}\tEND\tverdict=SKIPPED\treason=FILES_SKIPPED\tscanned_files=5\t\
         copied_files=2\tcopied_bytes=300\tskipped_files=13\terrors=2\tversions_removed=1\t\
         free_bytes=4000\n"
    );
    assert_eq!(setup.text(), expected);
}

#[test]
fn what_is_written_reads_back() {
    let setup = Setup::new();

    setup.with_run(Format::ExFat, |mut log| {
        log.error("UNREADABLE", Path::new("a.txt"), 1)?;
        log.error("CASE_CLASH", Path::new("Bilder"), 12)?;
        let stats = Stats {
            copied_files: 7,
            ..Stats::default()
        };
        log.end(Reason::MediumFull, stats)
    });

    let runs = setup.runs();
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.id, RUN);
    assert_eq!(run.account, ACCOUNT);
    assert_eq!(Some(run.started), parse_log_timestamp(LATER));
    assert_eq!(
        run.errors,
        [
            LoggedError {
                code: "UNREADABLE".to_owned(),
                path: Some(PathBuf::from("a.txt")),
                files: None,
            },
            LoggedError {
                code: "CASE_CLASH".to_owned(),
                path: Some(PathBuf::from("Bilder")),
                files: Some(12),
            },
        ]
    );
    let end = run.end.unwrap();
    assert_eq!(end.verdict, Verdict::Failed);
    assert_eq!(end.reason, Reason::MediumFull);
    let stats = end.stats.unwrap();
    assert_eq!(stats.copied_files, 7);
    assert_eq!(stats.errors, 2);
}

#[test]
fn the_start_entry_reaches_the_medium_at_once() {
    let setup = Setup::new();

    setup.start_only();

    assert!(setup.medium.content_is_flushed(log_path()));
}

#[test]
fn every_entry_reaches_the_medium_at_once() {
    let setup = Setup::new();

    setup.with_run(Format::ExFat, |mut log| {
        log.error("UNREADABLE", Path::new("a"), 1)?;
        assert!(setup.medium.content_is_flushed(log_path()));
        log.end(Reason::None, Stats::default())
    });

    assert!(setup.medium.content_is_flushed(log_path()));
}

#[test]
fn on_ext4_everyone_can_read_and_write_a_new_log() {
    let setup = Setup::new();

    setup.with_run(Format::Ext4, |_| Ok(()));

    assert_eq!(setup.medium.metadata(&log_path()).unwrap().mode, FILE_MODE);
}

#[test]
fn an_existing_log_keeps_its_mode() {
    let setup = Setup::with_log(&run_lines(1, ACCOUNT, "ALL_GOOD", "NONE"));
    let mode = setup.medium.metadata(&log_path()).unwrap().mode;

    setup.with_run(Format::Ext4, |_| Ok(()));

    assert_eq!(setup.medium.metadata(&log_path()).unwrap().mode, mode);
}

#[test]
fn a_medium_with_no_log_has_no_runs() {
    assert!(Setup::new().runs().is_empty());
}

#[test]
fn the_first_100_errors_are_logged_and_the_rest_only_counted() {
    let setup = Setup::new();

    setup.with_run(Format::ExFat, |mut log| {
        for number in 0..105 {
            log.error("UNREADABLE", Path::new(&format!("file-{number}")), 1)?;
        }
        log.end(Reason::FilesSkipped, Stats::default())
    });

    let run = &setup.runs()[0];
    assert_eq!(run.errors.len(), 101);
    assert_eq!(run.errors[99].path, Some(PathBuf::from("file-99")));
    let last = &run.errors[100];
    assert_eq!(last.code, "MULTIPLE_ERRORS");
    assert_eq!(last.path, None);
    assert_eq!(run.end.unwrap().stats.unwrap().errors, 105);
    assert_eq!(
        setup
            .text()
            .matches("Multiple errors, further errors are only counted")
            .count(),
        1
    );
}

#[test]
fn exactly_100_errors_need_no_note_about_further_errors() {
    let setup = Setup::new();

    setup.with_run(Format::ExFat, |mut log| {
        for _ in 0..100 {
            log.error("UNREADABLE", Path::new("file"), 1)?;
        }
        Ok(())
    });

    assert!(!setup.text().contains("MULTIPLE_ERRORS"));
}

#[test]
fn the_start_entry_says_which_babawus_ran_where() {
    let setup = Setup::new();

    setup.start_only();

    let start = format!(
        "{LATER}\t{RUN}\tSTART\taccount={ACCOUNT}\tversion={}\thostname=Mint-PC\tBackup started\n",
        crate::VERSION
    );
    assert!(setup.text().ends_with(&start), "{}", setup.text());
    assert_eq!(setup.runs()[0].account, ACCOUNT);
}

#[test]
fn values_are_escaped_as_the_requirements_describe() {
    assert_eq!(escape(b"a\tb"), "a\\tb");
    assert_eq!(escape(b"a\nb"), "a\\nb");
    assert_eq!(escape(b"a\\b"), "a\\\\b");
    assert_eq!(escape(b"a\x01b\x7f"), "a\\x01b\\x7f");
    assert_eq!(escape(b"caf\xe9"), "caf\\xe9");
    assert_eq!(escape("é=😀".as_bytes()), "é=😀");
}

#[test]
fn every_byte_survives_escaping() {
    let all: Vec<u8> = (0..=255).collect();
    let values: [&[u8]; 6] = [
        &all,
        b"",
        b"\\x41",
        b"\\\\",
        b"a\xc3",
        "\u{85}\u{2028}".as_bytes(),
    ];
    for value in values {
        let escaped = escape(value);

        assert!(!escaped.contains(['\t', '\n']), "{escaped}");
        assert_eq!(unescape(&escaped).unwrap(), value, "{escaped}");
    }
}

#[test]
fn a_bad_escape_is_not_a_value() {
    for text in ["\\", "\\q", "\\x", "\\x4", "\\xzz", "\\xé0"] {
        assert_eq!(unescape(text), None, "{text}");
    }
}

#[test]
fn any_file_name_gets_through_the_log() {
    let setup = Setup::new();
    let name = b"Docs/a\tb\nc=d\\e\xff.txt";

    setup.with_run(Format::ExFat, |mut log| {
        log.error("UNREADABLE", &os_path(name), 1)
    });

    assert!(str::from_utf8(&setup.contents()).is_ok());
    assert_eq!(setup.text().lines().count(), 2);
    let path = setup.runs()[0].errors[0].path.clone().unwrap();
    assert_eq!(path.as_os_str().as_bytes(), name);
}

#[test]
fn garbled_lines_are_ignored() {
    let good = run_lines(1, ACCOUNT, "ALL_GOOD", "NONE");
    let bad_lines = [
        "\n".to_owned(),
        "garbage\n".to_owned(),
        format!("{TIME}\n"),
        format!("{TIME}\t{RUN}\n"),
        format!("{TIME}\tnot-a-run\tSTART\taccount=a@b\n"),
        format!("{TIME}\tABCDEF0123456789\tSTART\taccount=a@b\n"),
        format!("yesterday\t{RUN}\tSTART\taccount=a@b\n"),
        format!("{TIME}\t{RUN}\tBEGIN\taccount=a@b\n"),
        format!("{TIME}\t{RUN}\tSTART\n"),
        format!("{TIME}\t{RUN}\tSTART\taccount=a\\q\n"),
    ];
    for bad in bad_lines {
        let log = format!("{bad}{good}{bad}");

        let runs = parse_runs(log.as_bytes());

        assert_eq!(runs.len(), 1, "{bad}");
        assert_eq!(runs[0].verdict(), Verdict::AllGood, "{bad}");
    }
}

#[test]
fn garbled_error_and_end_entries_are_ignored() {
    let start = start_line(RUN, ACCOUNT);
    for bad in [
        format!("{TIME}\t{RUN}\tERROR\tpath=a\n"),
        format!("{TIME}\t{RUN}\tERROR\tcode=X\tpath=a\\q\n"),
        format!("{TIME}\t{RUN}\tERROR\tcode=X\tfiles=many\n"),
        format!("{TIME}\t{RUN}\tEND\treason=NONE\n"),
        format!("{TIME}\t{RUN}\tEND\tverdict=ALL_GOOD\n"),
        format!("{TIME}\t{RUN}\tEND\tverdict=GREAT\treason=NONE\n"),
        format!("{TIME}\t{RUN}\tEND\tverdict=ALL_GOOD\treason=WHO_KNOWS\n"),
    ] {
        let runs = parse_runs(format!("{start}{bad}").as_bytes());

        assert!(runs[0].errors.is_empty(), "{bad}");
        assert_eq!(runs[0].end, None, "{bad}");
    }
}

#[test]
fn a_line_that_is_not_utf8_is_ignored() {
    let mut log = b"\xff\xfe\n".to_vec();
    log.extend_from_slice(run_lines(1, ACCOUNT, "ALL_GOOD", "NONE").as_bytes());

    assert_eq!(parse_runs(&log).len(), 1);
}

#[test]
fn a_last_line_with_no_line_ending_never_counts() {
    let whole = end_line(RUN, "ALL_GOOD", "NONE");
    let log = format!("{}{}", start_line(RUN, ACCOUNT), whole.trim_end());

    let runs = parse_runs(log.as_bytes());

    assert_eq!(runs[0].end, None);
}

#[test]
fn entries_of_a_run_with_no_start_are_ignored() {
    let log = end_line(RUN, "ALL_GOOD", "NONE");

    assert!(parse_runs(log.as_bytes()).is_empty());
}

#[test]
fn the_text_of_an_entry_is_not_taken_for_a_pair() {
    let log = format!(
        "{}{TIME}\t{RUN}\tERROR\tcode=X\tSome text\n",
        start_line(RUN, ACCOUNT)
    );

    let runs = parse_runs(log.as_bytes());

    assert_eq!(runs[0].errors[0].code, "X");
    assert_eq!(runs[0].errors[0].path, None);
}

#[test]
fn a_path_with_an_equals_sign_is_read_whole() {
    let log = format!(
        "{}{TIME}\t{RUN}\tERROR\tcode=X\tpath=a=b\n",
        start_line(RUN, ACCOUNT)
    );

    let runs = parse_runs(log.as_bytes());

    assert_eq!(runs[0].errors[0].path, Some(PathBuf::from("a=b")));
}

#[test]
fn an_end_entry_without_numbers_is_still_an_end_entry() {
    let log = format!(
        "{}{TIME}\t{RUN}\tEND\tverdict=NOT_FINISHED\treason=INTERRUPTED\n",
        start_line(RUN, ACCOUNT)
    );

    let end = parse_runs(log.as_bytes())[0].end.unwrap();

    assert_eq!(end.verdict, Verdict::NotFinished);
    assert_eq!(end.stats, None);
}

#[test]
fn a_complete_end_entry_has_all_its_numbers() {
    let log = format!(
        "{}{TIME}\t{RUN}\tEND\t{END_FIELDS}\n",
        start_line(RUN, ACCOUNT)
    );

    let stats = parse_runs(log.as_bytes())[0].end.unwrap().stats.unwrap();

    assert_eq!(
        stats,
        Stats {
            scanned_files: 5,
            copied_files: 2,
            copied_bytes: 300,
            skipped_files: 0,
            errors: 0,
            versions_removed: 1,
            free_bytes: 4000,
        }
    );
}

#[test]
fn a_run_with_no_end_entry_is_not_finished_and_not_a_backup() {
    let runs = parse_runs(start_line(RUN, ACCOUNT).as_bytes());

    assert_eq!(runs[0].verdict(), Verdict::NotFinished);
    assert!(!runs[0].counts_as_backup());
}

#[test]
fn an_interrupted_run_is_closed_by_the_next_one() {
    let setup = Setup::with_log(&start_line(OTHER_RUN, "userb@mint"));

    setup.start_only();

    let runs = setup.runs();
    let end = runs[0].end.unwrap();
    assert_eq!(end.verdict, Verdict::NotFinished);
    assert_eq!(end.reason, Reason::Interrupted);
    assert_eq!(Some(end.time), parse_log_timestamp(LATER));
    assert_eq!(end.stats, None);
    assert_eq!(runs[1].id, RUN);
    assert_eq!(runs[1].end, None);
}

#[test]
fn every_interrupted_run_is_closed_and_finished_runs_are_left_alone() {
    let finished = run_lines(1, ACCOUNT, "ALL_GOOD", "NONE");
    let log = format!(
        "{}{finished}{}",
        start_line("00000000000000aa", ACCOUNT),
        start_line("00000000000000bb", ACCOUNT)
    );
    let setup = Setup::with_log(&log);

    setup.start_only();

    let runs = setup.runs();
    assert_eq!(runs[0].end.unwrap().reason, Reason::Interrupted);
    assert_eq!(runs[1].end.unwrap().reason, Reason::None);
    assert_eq!(runs[2].end.unwrap().reason, Reason::Interrupted);
    assert_eq!(setup.text().matches("INTERRUPTED").count(), 2);
}

#[test]
fn a_run_with_a_live_lock_is_never_closed() {
    let setup = Setup::with_log(&start_line(OTHER_RUN, "userb@mint"));
    let now = || LATER.to_owned();
    let root = root();

    let log = RunLog::start(
        &setup.medium,
        &root,
        Format::ExFat,
        RUN.to_owned(),
        &RUNNER,
        &now,
        Some(OTHER_RUN),
    );

    assert!(log.is_ok());
    assert_eq!(setup.runs()[0].end, None);
}

#[test]
fn a_half_written_last_line_does_not_swallow_the_next_entry() {
    let log = format!("{}{TIME}\t{RUN}\tEND\tverd", start_line(OTHER_RUN, ACCOUNT));
    let setup = Setup::with_log(&log);

    setup.start_only();

    let runs = setup.runs();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[1].id, RUN);
}

#[test]
fn a_half_written_start_entry_does_not_swallow_the_next_entry() {
    let setup = Setup::with_log(&format!("{TIME}\t{OTHER_RUN}\tSTA"));

    setup.start_only();

    let runs = setup.runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, RUN);
}

#[test]
fn run_ids_have_the_right_form_and_differ() {
    let (first, second) = (new_run_id(), new_run_id());

    assert!(is_run_id(&first), "{first}");
    assert_ne!(first, second);
}

fn account_runs(account: &str, lines: &[(&str, &str)]) -> Vec<Run> {
    let log: String = lines
        .iter()
        .zip(1..)
        .map(|((verdict, reason), number)| run_lines(number, account, verdict, reason))
        .collect();
    parse_runs(log.as_bytes())
}

#[test]
fn there_is_no_last_backup_before_the_first_one() {
    assert_eq!(last_backup(&[], ACCOUNT), None);
}

#[test]
fn a_finished_run_counts_as_a_backup() {
    let runs = account_runs(ACCOUNT, &[("ALL_GOOD", "NONE")]);

    let last = last_backup(&runs, ACCOUNT).unwrap();

    assert_eq!(Some(last.time), parse_log_timestamp(TIME));
    assert!(!last.skipped_files);
}

#[test]
fn skipped_files_count_as_a_backup_with_a_warning() {
    let runs = account_runs(ACCOUNT, &[("SKIPPED", "FILES_SKIPPED")]);

    assert!(last_backup(&runs, ACCOUNT).unwrap().skipped_files);
}

#[test]
fn runs_that_did_not_finish_do_not_count() {
    let runs = account_runs(
        ACCOUNT,
        &[("NOT_FINISHED", "CANCELLED"), ("FAILED", "MEDIUM_FULL")],
    );

    assert_eq!(last_backup(&runs, ACCOUNT), None);
}

#[test]
fn the_most_recent_backup_counts_past_later_failures() {
    let runs = account_runs(
        ACCOUNT,
        &[
            ("SKIPPED", "FILES_SKIPPED"),
            ("ALL_GOOD", "NONE"),
            ("FAILED", "MEDIUM_FULL"),
        ],
    );

    assert!(!last_backup(&runs, ACCOUNT).unwrap().skipped_files);
}

#[test]
fn only_the_account_itself_counts() {
    let runs = account_runs("userb@mint", &[("ALL_GOOD", "NONE")]);

    assert_eq!(last_backup(&runs, ACCOUNT), None);
    assert_eq!(last_backup(&runs, "userb@mint-2"), None);
}

#[test]
fn the_account_name_is_user_at_folder() {
    assert_eq!(account_name("usera", "mint-2"), "usera@mint-2");
}

/// A log of `count` old runs for `account`, large enough to need trimming with `padding`.
fn long_log(count: u64, account: &str, padding: usize) -> String {
    let text = "x".repeat(padding);
    (1..=count)
        .map(|number| {
            let run = format!("{number:016x}");
            format!(
                "{}{TIME}\t{run}\tERROR\tcode=X\tpath={text}\n{}",
                start_line(&run, account),
                end_line(&run, "FAILED", "MEDIUM_FULL")
            )
        })
        .collect()
}

#[test]
fn a_log_within_the_limit_is_not_touched() {
    let log = long_log(10, ACCOUNT, 1000);

    assert_eq!(trimmed(log.as_bytes(), None), None);
}

#[test]
fn a_log_of_exactly_the_limit_is_not_touched() {
    let log = format!("{}\n", "x".repeat(TRIM_LIMIT - 1));

    assert_eq!(trimmed(log.as_bytes(), None), None);
}

#[test]
fn trimming_removes_the_oldest_whole_runs_until_it_fits() {
    let log = long_log(200, ACCOUNT, 10_000);
    assert!(log.len() > 2 * TRIM_LIMIT - 1);

    let result = trimmed(log.as_bytes(), None).unwrap();

    assert!(result.len() <= TRIM_LIMIT);
    assert!(result.len() > TRIM_LIMIT - 11_000, "only as much as needed");
    let runs = parse_runs(&result);
    assert_eq!(runs.last().unwrap().id, format!("{:016x}", 200));
    let oldest: u64 = u64::from_str_radix(&runs[0].id, 16).unwrap();
    assert_eq!(runs.len() as u64, 200 - oldest + 1, "no gaps");
    for run in runs {
        assert_eq!(run.errors.len(), 1, "whole runs");
        assert!(run.end.is_some(), "whole runs");
    }
}

#[test]
fn trimming_keeps_each_accounts_latest_backup() {
    let old_backup = run_lines(0x1000, "usera@mint", "ALL_GOOD", "NONE");
    let older_backup = run_lines(0x1001, "userb@mint", "ALL_GOOD", "NONE");
    let newer_backup = run_lines(0x1002, "userb@mint", "SKIPPED", "FILES_SKIPPED");
    let log = format!(
        "{old_backup}{older_backup}{newer_backup}{}",
        long_log(200, "userc@mint", 10_000)
    );

    let result = trimmed(log.as_bytes(), None).unwrap();

    let runs = parse_runs(&result);
    assert!(result.len() <= TRIM_LIMIT);
    assert_eq!(runs[0].id, format!("{:016x}", 0x1000));
    assert_eq!(runs[1].id, format!("{:016x}", 0x1002), "only the latest");
    assert!(last_backup(&runs, "usera@mint").is_some());
    assert!(last_backup(&runs, "userb@mint").unwrap().skipped_files);
}

#[test]
fn trimming_keeps_the_latest_backup_past_a_later_failure() {
    let backup = run_lines(0x1000, ACCOUNT, "ALL_GOOD", "NONE");
    let failure = run_lines(0x1001, ACCOUNT, "FAILED", "MEDIUM_FULL");
    let log = format!("{backup}{failure}{}", long_log(200, "userc@mint", 10_000));

    let result = trimmed(log.as_bytes(), None).unwrap();

    assert!(last_backup(&parse_runs(&result), ACCOUNT).is_some());
}

#[test]
fn trimming_keeps_a_live_run() {
    let start = start_line(OTHER_RUN, ACCOUNT);
    let log = format!("{start}{}", long_log(200, ACCOUNT, 10_000));

    let result = trimmed(log.as_bytes(), Some(OTHER_RUN)).unwrap();

    assert_eq!(parse_runs(&result)[0].id, OTHER_RUN);
}

#[test]
fn trimming_drops_garbled_lines_and_keeps_the_order_of_lines() {
    let first = format!("{:016x}", 0x1000);
    let second = format!("{:016x}", 0x1001);
    let log = format!(
        "{}garbage\n{}{}{}{}",
        long_log(100, ACCOUNT, 10_000),
        start_line(&first, ACCOUNT),
        start_line(&second, ACCOUNT),
        end_line(&first, "NOT_FINISHED", "INTERRUPTED"),
        end_line(&second, "ALL_GOOD", "NONE"),
    );

    let result = trimmed(log.as_bytes(), None).unwrap();

    let text = String::from_utf8(result).unwrap();
    assert!(!text.contains("garbage"));
    assert!(text.ends_with(&format!(
        "{}{}{}{}",
        start_line(&first, ACCOUNT),
        start_line(&second, ACCOUNT),
        end_line(&first, "NOT_FINISHED", "INTERRUPTED"),
        end_line(&second, "ALL_GOOD", "NONE"),
    )));
}

#[test]
fn if_only_protected_runs_remain_they_all_stay() {
    let text = "x".repeat(100_000);
    let log: String = (0..10)
        .map(|number| {
            let run = format!("{number:016x}");
            format!(
                "{}{TIME}\t{run}\tERROR\tcode=X\tpath={text}\n{}",
                start_line(&run, &format!("user{number}@mint")),
                end_line(&run, "ALL_GOOD", "NONE")
            )
        })
        .collect();

    let result = trimmed(log.as_bytes(), None).unwrap();

    assert_eq!(parse_runs(&result).len(), 10);
}

#[test]
fn a_run_trims_a_long_log_before_its_start_entry() {
    let setup = Setup::with_log(&long_log(200, ACCOUNT, 10_000));

    setup.start_only();

    let contents = setup.contents();
    assert!(contents.len() < 1_000_000);
    let runs = setup.runs();
    assert_eq!(runs.last().unwrap().id, RUN);
    assert!(runs.len() < 200);
}

#[test]
fn an_interrupted_run_is_closed_before_it_could_be_trimmed() {
    let start = start_line(OTHER_RUN, ACCOUNT);
    let setup = Setup::with_log(&format!("{}{start}", long_log(200, ACCOUNT, 10_000)));

    setup.start_only();

    let text = setup.text();
    let closed = setup.runs().iter().any(|run| run.id == OTHER_RUN);
    assert!(closed);
    assert!(text.contains("reason=INTERRUPTED"));
}

#[test]
fn a_full_medium_does_not_stop_a_run_from_starting_if_the_log_still_fits() {
    let log = long_log(200, ACCOUNT, 10_000);
    let setup = Setup::with_log(&log);
    setup.medium.set_capacity(log.len() as u64 + 1_000);

    setup.start_only();

    assert_eq!(setup.runs().last().unwrap().id, RUN);
}

#[test]
fn pulling_the_medium_at_any_step_of_a_run_never_loses_earlier_runs() {
    let earlier = format!(
        "{}{}",
        long_log(10, "userb@mint", 91_000),
        run_lines(0x1000, ACCOUNT, "ALL_GOOD", "NONE"),
    );
    let run = |setup: &Setup| -> io::Result<()> {
        let now = || LATER.to_owned();
        let root = root();
        let mut log = RunLog::start(
            &setup.medium,
            &root,
            Format::Ext4,
            RUN.to_owned(),
            &RUNNER,
            &now,
            None,
        )?;
        log.error("UNREADABLE", Path::new("a"), 1)?;
        log.end(Reason::None, Stats::default())
    };
    let steps = {
        let setup = Setup::with_log(&earlier);
        run(&setup).unwrap();
        setup.medium.steps()
    };
    for step in 1..=steps {
        for lose_unflushed in [false, true] {
            let setup = Setup::with_log(&earlier);
            setup.medium.pull_at_step(step);

            let result = run(&setup);

            if lose_unflushed {
                setup.medium.reinsert_losing_unflushed();
            } else {
                setup.medium.reinsert();
            }
            let account = |runs: &[Run]| last_backup(runs, ACCOUNT).is_some();
            assert!(result.is_err(), "step {step}");
            assert!(account(&setup.runs()), "step {step}");

            // The next run catches up.
            setup.whole_run();
            let runs = setup.runs();
            let unfinished = runs.iter().filter(|run| run.end.is_none()).count();
            assert_eq!(unfinished, 0, "step {step}");
        }
    }
}

#[test]
fn an_error_while_writing_is_reported() {
    let setup = Setup::new();
    let now = || LATER.to_owned();
    let root = root();
    setup
        .medium
        .fail_path(log_path(), ErrorKind::PermissionDenied);

    let result = RunLog::start(
        &setup.medium,
        &root,
        Format::ExFat,
        RUN.to_owned(),
        &RUNNER,
        &now,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn renaming_an_account_changes_only_its_start_entries() {
    let log = format!(
        "{}{}{}",
        run_lines(1, "usera@mint", "ALL_GOOD", "NONE"),
        run_lines(2, "userb@mint", "ALL_GOOD", "NONE"),
        run_lines(3, "usera@mint-2", "ALL_GOOD", "NONE"),
    );
    let setup = Setup::with_log(&log);

    rename_account(
        &setup.medium,
        &root(),
        Format::Ext4,
        "usera@mint",
        "anna@mint",
    )
    .unwrap();

    let expected = log.replacen("usera@mint\t", "anna@mint\t", 1);
    assert_eq!(setup.text(), expected);
    assert_eq!(setup.medium.metadata(&log_path()).unwrap().mode, FILE_MODE);
}

#[test]
fn renaming_keeps_the_accounts_last_backup() {
    let setup = Setup::with_log(&run_lines(1, ACCOUNT, "ALL_GOOD", "NONE"));

    rename_account(&setup.medium, &root(), Format::ExFat, ACCOUNT, "anna@mint").unwrap();

    assert!(last_backup(&setup.runs(), "anna@mint").is_some());
    assert!(last_backup(&setup.runs(), ACCOUNT).is_none());
}

#[test]
fn renaming_keeps_lines_it_does_not_understand() {
    let log = format!(
        "garbage account=usera@mint\n{}half line",
        run_lines(1, ACCOUNT, "ALL_GOOD", "NONE")
    );
    let setup = Setup::with_log(&log);

    rename_account(&setup.medium, &root(), Format::ExFat, ACCOUNT, "anna@mint").unwrap();

    let text = setup.text();
    assert!(text.starts_with("garbage account=usera@mint\n"));
    assert!(text.ends_with("half line"));
    assert!(text.contains("account=anna@mint"));
}

#[test]
fn renaming_keeps_lines_that_are_not_utf8() {
    let mut log = b"\xff\xfe\n".to_vec();
    log.extend_from_slice(run_lines(1, ACCOUNT, "ALL_GOOD", "NONE").as_bytes());
    let setup = Setup::with_log_bytes(&log);

    rename_account(&setup.medium, &root(), Format::ExFat, ACCOUNT, "anna@mint").unwrap();

    assert!(setup.contents().starts_with(b"\xff\xfe\n"));
    assert!(last_backup(&setup.runs(), "anna@mint").is_some());
}

#[test]
fn a_path_that_looks_like_an_account_is_not_renamed() {
    let run = start_line(RUN, "userb@mint");
    let log = format!("{run}{TIME}\t{RUN}\tERROR\tcode=X\tpath=account=usera@mint\n");
    let setup = Setup::with_log(&log);
    let steps = setup.medium.steps();

    rename_account(&setup.medium, &root(), Format::ExFat, ACCOUNT, "anna@mint").unwrap();

    assert_eq!(setup.text(), log);
    assert_eq!(setup.medium.steps(), steps, "nothing to write");
}

#[test]
fn renaming_on_a_medium_with_no_log_does_nothing() {
    let setup = Setup::new();

    rename_account(&setup.medium, &root(), Format::ExFat, ACCOUNT, "anna@mint").unwrap();

    assert!(setup.medium.metadata(&log_path()).is_err());
}

#[test]
fn a_timestamp_from_the_real_clock_can_be_read_back() {
    let setup = Setup::new();
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_793_000_000);
    *setup.clock.lock().unwrap() = log_timestamp(time);

    setup.whole_run();

    assert_eq!(setup.runs()[0].started, time);
}

#[test]
fn every_skip_code_is_a_valid_error_code() {
    use crate::copy::Skip;

    let setup = Setup::new();
    setup.with_run(Format::ExFat, |mut log| {
        log.error(Skip::VersionNameTooLong.code(), Path::new("a"), 1)?;
        log.error(PRUNE_FAILED, Path::new("b"), 1)
    });

    let codes: Vec<_> = setup.runs()[0]
        .errors
        .iter()
        .map(|error| error.code.clone())
        .collect();
    assert_eq!(codes, ["VERSION_NAME_TOO_LONG", "PRUNE_FAILED"]);
}
