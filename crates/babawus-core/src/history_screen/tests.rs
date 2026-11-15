use std::time::{Duration, SystemTime};

use super::*;
use crate::log::{End, Stats};
use crate::time::utc_clock;
use crate::verdict::Verdict;

fn en() -> Language {
    crate::language::testing::english()
}
const HOUR: Duration = Duration::from_secs(3600);
const DAY: Duration = Duration::from_secs(86_400);

fn now() -> SystemTime {
    // 2024-10-04 12:00 UTC.
    SystemTime::UNIX_EPOCH + 20_000 * DAY + 12 * HOUR
}

fn stats(copied_files: u64) -> Stats {
    Stats {
        scanned_files: 201,
        copied_files,
        copied_bytes: copied_files * 300_000,
        ..Default::default()
    }
}

fn run(account: &str, started: SystemTime, end: Option<(Verdict, Reason, Stats)>) -> Run {
    Run {
        id: "0123456789abcdef".to_owned(),
        account: account.to_owned(),
        started,
        errors: Vec::new(),
        end: end.map(|(verdict, reason, stats)| End {
            time: started + HOUR / 4,
            verdict,
            reason,
            stats: Some(stats),
        }),
    }
}

fn good(account: &str, started: SystemTime) -> Run {
    run(
        account,
        started,
        Some((Verdict::AllGood, Reason::None, stats(12))),
    )
}

fn rows(runs: &[Run], in_progress: Option<&str>) -> Vec<HistoryRow> {
    history(runs, in_progress, now(), utc_clock, en())
}

#[test]
fn the_newest_run_comes_first() {
    let runs = [
        good("anna@laptop", now() - 3 * DAY),
        good("anna@laptop", now() - HOUR),
    ];

    let rows = rows(&runs, None);

    assert_eq!(rows[0].when, "Today, 11:00");
    assert_eq!(rows[1].when, "2024-10-01, 12:00");
}

#[test]
fn a_good_run_shows_what_was_copied_and_no_reason() {
    let rows = rows(&[good("anna@laptop", now() - HOUR)], None);

    assert_eq!(
        rows[0],
        HistoryRow {
            run_id: "0123456789abcdef".to_owned(),
            mark: VerdictMark::Good,
            verdict: "All good".to_owned(),
            when: "Today, 11:00".to_owned(),
            who: "By anna on laptop".to_owned(),
            reason: None,
            copied: Some("Copied: 12 files (3.6 MB)".to_owned()),
            skipped: None,
        }
    );
}

#[test]
fn days_are_calendar_days() {
    // 23:50 the day before is yesterday, though it is only just over 12 hours ago.
    let late = now() - 12 * HOUR - Duration::from_secs(600);

    let rows = rows(&[good("anna@laptop", late)], None);

    assert_eq!(rows[0].when, "Yesterday, 23:50");
}

#[test]
fn a_run_with_no_end_was_interrupted() {
    let rows = rows(&[run("anna@laptop", now() - DAY, None)], None);

    assert_eq!(rows[0].mark, VerdictMark::Attention);
    assert_eq!(rows[0].verdict, "Backup not finished");
    assert_eq!(
        rows[0].reason.as_deref(),
        Some("The backup was interrupted.")
    );
    assert_eq!(rows[0].copied, None);
}

#[test]
fn the_newest_run_of_the_user_holding_the_lock_is_in_progress() {
    let runs = [
        run("anna@laptop", now() - DAY, None),
        run("anna@laptop", now() - HOUR, None),
    ];

    let rows = rows(&runs, Some("anna"));

    assert_eq!(rows[0].verdict, "Backup in progress");
    assert_eq!(rows[0].reason, None);
    assert_eq!(rows[1].verdict, "Backup not finished");
}

#[test]
fn a_lock_held_by_someone_else_leaves_the_runs_as_they_are() {
    let rows = rows(&[run("anna@laptop", now() - HOUR, None)], Some("bert"));

    assert_eq!(rows[0].verdict, "Backup not finished");
}

#[test]
fn a_failed_run_gives_its_reason() {
    let runs = [run(
        "anna@laptop",
        now() - HOUR,
        Some((Verdict::Failed, Reason::MediumFull, stats(3))),
    )];

    let rows = rows(&runs, None);

    assert_eq!(rows[0].mark, VerdictMark::Failed);
    assert_eq!(rows[0].verdict, "Backup failed");
    assert_eq!(rows[0].reason.as_deref(), Some("The stick is full."));
    assert_eq!(
        rows[0].copied.as_deref(),
        Some("Copied: 3 files (900.0 kB)")
    );
}

#[test]
fn nothing_to_copy_says_so() {
    let runs = [run(
        "anna@laptop",
        now() - HOUR,
        Some((Verdict::AllGood, Reason::None, stats(0))),
    )];

    let rows = rows(&runs, None);

    assert_eq!(
        rows[0].copied.as_deref(),
        Some("Everything was already up to date.")
    );
}

#[test]
fn every_run_says_whose_it_was() {
    let shared = rows(
        &[
            good("anna@laptop", now() - 2 * HOUR),
            good("bert@desktop", now() - HOUR),
        ],
        None,
    );

    assert_eq!(shared[0].who, "By bert on desktop");
    assert_eq!(shared[1].who, "By anna on laptop");
}

#[test]
fn skipped_files_are_counted() {
    let skipped = |files| Stats {
        skipped_files: files,
        ..stats(9)
    };
    let runs = [
        run(
            "anna@laptop",
            now() - 2 * HOUR,
            Some((Verdict::Skipped, Reason::FilesSkipped, skipped(1))),
        ),
        run(
            "anna@laptop",
            now() - HOUR,
            Some((Verdict::Skipped, Reason::FilesSkipped, skipped(1_234))),
        ),
    ];

    let rows = rows(&runs, None);

    assert_eq!(rows[0].skipped.as_deref(), Some("Skipped: 1,234 files"));
    assert_eq!(rows[1].skipped.as_deref(), Some("Skipped: 1 file"));
}

#[test]
fn no_runs_no_rows() {
    assert!(rows(&[], None).is_empty());
}
