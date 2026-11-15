use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;
use crate::lock::Holder;
use crate::log::Stats;
use crate::scan::Skipped;
use crate::time::utc_clock;

fn en() -> Language {
    crate::language::testing::english()
}
const DAY: Duration = Duration::from_secs(86_400);

fn now() -> SystemTime {
    // 2024-10-03, around midday.
    SystemTime::UNIX_EPOCH + 20_000 * DAY + Duration::from_secs(12 * 3600)
}

fn context(previous_days_ago: Option<u32>) -> Context<'static> {
    Context {
        label: "BACKUP-A",
        previous: previous_days_ago.map(|days| LastBackup {
            time: now() - days * DAY,
            skipped_files: false,
        }),
        now: now(),
        clock: utc_clock,
    }
}

fn outcome(reason: Reason) -> Outcome {
    Outcome {
        reason,
        stats: Stats {
            scanned_files: 201,
            copied_files: 12,
            copied_bytes: 3_400_000,
            skipped_files: 0,
            errors: 0,
            versions_removed: 2,
            free_bytes: 14_890_860_544,
        },
        skipped: Vec::new(),
        prune_failures: Vec::new(),
    }
}

fn skipped(path: &str, reason: Skip, files: u64) -> Skipped {
    Skipped {
        relative: PathBuf::from(path),
        reason,
        files,
    }
}

fn shown(outcome: Outcome, previous: Option<u32>) -> VerdictShown {
    verdict_screen(Some(&Ended::Finished(outcome)), &context(previous), en())
}

#[test]
fn all_good_needs_nothing_more() {
    let shown = shown(outcome(Reason::None), Some(47));
    assert_eq!(shown.mark, VerdictMark::Good);
    assert_eq!(shown.title, "All good");
    assert_eq!(shown.reason, None);
    assert_eq!(shown.next_step, None);
    assert_eq!(shown.copied.as_deref(), Some("Copied: 12 files (3.4 MB)"));
    // This run is the last backup now.
    assert_eq!(shown.last_backup, "Last backup: today at 12:00");
    assert!(shown.offer_eject);
}

#[test]
fn a_run_with_nothing_new_says_so() {
    let mut nothing = outcome(Reason::None);
    nothing.stats.copied_files = 0;
    nothing.stats.copied_bytes = 0;
    let shown = shown(nothing, None);
    assert_eq!(
        shown.copied.as_deref(),
        Some("Everything was already up to date.")
    );
}

#[test]
fn skipped_files_are_grouped_by_reason() {
    let mut with_skips = outcome(Reason::FilesSkipped);
    with_skips.skipped = vec![
        skipped("Documents/a.txt", Skip::Unreadable, 1),
        skipped("Documents/b:c.txt", Skip::NameInvalidChars, 1),
        skipped("Private", Skip::Unreadable, 4),
        skipped("Documents/d?.txt", Skip::NameInvalidChars, 1),
    ];
    let shown = shown(with_skips, Some(3));
    assert_eq!(shown.mark, VerdictMark::Attention);
    assert_eq!(shown.title, "Some files were skipped");
    // The title says it, so no reason repeats it.
    assert_eq!(shown.reason, None);
    assert_eq!(
        shown.next_step.as_deref(),
        Some("Look up its number in Help to see what to do.")
    );
    let groups = [
        "Couldn't be read: 5 files #19",
        "Name has characters this stick can't store: 2 files #25",
    ];
    assert_eq!(shown.skipped, groups);
    // A run that skipped files still counts as a backup.
    assert_eq!(shown.last_backup, "Last backup: today at 12:00");
    assert!(shown.offer_eject);
}

#[test]
fn an_unfinished_run_says_to_run_it_again() {
    let shown = shown(outcome(Reason::MediumRemoved), Some(47));
    assert_eq!(shown.mark, VerdictMark::Attention);
    assert_eq!(shown.title, "Backup not finished");
    assert_eq!(shown.reason.as_deref(), Some("The stick was removed. #10"));
    let next = "Run the backup again. If it happens again, look up its number in Help.";
    assert_eq!(shown.next_step.as_deref(), Some(next));
    // The last backup is still the one before.
    assert_eq!(
        shown.last_backup,
        "Last backup: 47 days ago (2024-08-18 at 12:00)"
    );
    assert!(!shown.offer_eject);
}

#[test]
fn a_run_the_user_stopped_needs_nothing_looked_up() {
    let shown = shown(outcome(Reason::Cancelled), Some(47));
    assert_eq!(shown.title, "Backup not finished");
    assert_eq!(shown.reason.as_deref(), Some("You stopped the backup."));
    assert_eq!(
        shown.next_step.as_deref(),
        Some("Run the backup again when it suits you.")
    );
}

#[test]
fn a_failed_run_says_why() {
    let shown = shown(outcome(Reason::MediumFull), None);
    assert_eq!(shown.mark, VerdictMark::Failed);
    assert_eq!(shown.title, "Backup failed");
    assert_eq!(shown.reason.as_deref(), Some("The stick is full. #14"));
    assert_eq!(
        shown.next_step.as_deref(),
        Some("Look up its number in Help to see what to do.")
    );
    assert_eq!(shown.last_backup, "No backup yet");
}

#[test]
fn every_reason_has_plain_words_but_none() {
    for code in [
        "FILES_SKIPPED",
        "CANCELLED",
        "MEDIUM_REMOVED",
        "INTERRUPTED",
        "FLUSH_FAILED",
        "MEDIUM_FULL",
        "NOT_ADDED",
        "CONFIG_DAMAGED",
        "CANNOT_WRITE",
    ] {
        let reason = Reason::from_code(code).unwrap();
        assert!(reason_text(reason).is_some(), "{code}");
    }
    assert_eq!(reason_text(Reason::None), None);
}

#[test]
fn a_busy_medium_is_not_a_verdict() {
    let holder = Holder {
        run_id: "0123456789abcdef".to_owned(),
        account: "anna@mint-1".to_owned(),
        computer_id: String::new(),
        boot_id: String::new(),
        process_id: 1,
    };
    let busy = Ended::Busy(Some(holder));
    let shown = verdict_screen(Some(&busy), &context(Some(2)), en());
    assert_eq!(shown.title, "Backup in progress (by anna)");
    assert_eq!(
        shown.next_step.as_deref(),
        Some("Wait until it has finished, then try again.")
    );
    assert!(!shown.offer_eject);
    let starting = verdict_screen(Some(&Ended::Busy(None)), &context(None), en());
    assert_eq!(
        starting.title,
        "Another backup to this stick is just starting #18"
    );
}

#[test]
fn a_crash_leaves_the_backup_unfinished() {
    let shown = verdict_screen(None, &context(Some(5)), en());
    assert_eq!(shown.title, "Backup not finished");
    assert_eq!(
        shown.reason.as_deref(),
        Some("Babawus stopped unexpectedly. #12")
    );
    assert_eq!(
        shown.last_backup,
        "Last backup: 5 days ago (2024-09-29 at 12:00)"
    );
}

#[test]
fn the_last_backup_shows_when_it_finished_by_calendar_day() {
    let midnight = SystemTime::UNIX_EPOCH + 20_000 * DAY;
    let last = |time| {
        Some(LastBackup {
            time,
            skipped_files: false,
        })
    };
    let text = |time, now| last_backup_text(last(time), now, utc_clock, en());

    // 23:50, seen ten minutes after midnight, was yesterday, though it is not a day ago.
    let late = midnight - Duration::from_secs(600);
    let soon_after = midnight + Duration::from_secs(600);
    assert_eq!(text(late, soon_after), "Last backup: yesterday at 23:50");
    // Two backups on one day: the time tells them apart.
    let morning = midnight + Duration::from_secs(9 * 3600 + 5 * 60);
    let evening = midnight + Duration::from_secs(18 * 3600);
    assert_eq!(text(morning, evening), "Last backup: today at 09:05");
    // A clock that was wrong can put the backup in the future.
    assert_eq!(text(evening, morning), "Last backup: today at 18:00");
    assert_eq!(
        last_backup_text(None, morning, utc_clock, en()),
        "No backup yet"
    );
}

#[test]
fn the_last_backup_reads_well_in_german() {
    let now = SystemTime::UNIX_EPOCH + 20_000 * DAY + Duration::from_secs(12 * 3600);
    let last = Some(LastBackup {
        time: now - 3 * DAY,
        skipped_files: false,
    });

    let text = last_backup_text(last, now, utc_clock, crate::language::testing::german());

    assert_eq!(text, "Letzte Sicherung: vor 3 Tagen (2024-10-01 um 12:00)");
}
