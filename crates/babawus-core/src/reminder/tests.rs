use std::path::PathBuf;
use std::time::Duration;

use super::*;
use crate::fs::fake::FakeFs;
use crate::time::utc_clock;

const DAY: Duration = Duration::from_secs(86_400);
const HOUR: Duration = Duration::from_secs(3600);

fn path() -> PathBuf {
    PathBuf::from("/home/anna/.local/state").join(STATE_FILE)
}

/// 2024-10-04 12:00 UTC.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + 20_000 * DAY + 12 * HOUR
}

fn noted(days: u32) -> Option<Noted> {
    Some(Noted {
        finished: noon(),
        run_check_days: days,
    })
}

#[test]
fn a_note_reads_back_and_creates_its_folders() {
    let fs = FakeFs::new();
    let noted = noted(7).unwrap();

    note(&fs, &path(), noted).unwrap();

    assert_eq!(read(&fs, &path()), Some(noted));
}

#[test]
fn a_later_note_replaces_the_earlier_one() {
    let fs = FakeFs::new();
    note(&fs, &path(), noted(7).unwrap()).unwrap();
    let later = Noted {
        finished: noon() + DAY,
        run_check_days: 3,
    };

    note(&fs, &path(), later).unwrap();

    assert_eq!(read(&fs, &path()), Some(later));
}

#[test]
fn a_missing_or_unreadable_file_says_nothing() {
    let fs = FakeFs::new();
    assert_eq!(read(&fs, &path()), None);

    for contents in [
        "",
        "not toml",
        "last_backup = \"yesterday\"\nrun_check_days = 7\n",
    ] {
        fs.add_file(path(), contents.as_bytes(), SystemTime::UNIX_EPOCH);
        assert_eq!(read(&fs, &path()), None, "{contents}");
    }
}

#[test]
fn a_reminder_is_due_only_after_more_calendar_days_than_allowed() {
    let at = |days: u32, hours: u64| noon() + DAY * days + Duration::from_secs(hours * 3600);

    assert_eq!(due(noted(7), at(7, 11), utc_clock), None);
    // Just after midnight, it is the 8th day.
    assert_eq!(due(noted(7), at(7, 12), utc_clock), Some(8));
    assert_eq!(due(noted(7), at(30, 0), utc_clock), Some(30));
    assert_eq!(due(noted(1), at(1, 0), utc_clock), None);
    assert_eq!(due(noted(1), at(2, 0), utc_clock), Some(2));
}

#[test]
fn no_reminder_without_a_note_or_with_0_days() {
    assert_eq!(due(None, noon() + 100 * DAY, utc_clock), None);
    assert_eq!(due(noted(0), noon() + 100 * DAY, utc_clock), None);
}

#[test]
fn a_clock_that_went_back_gives_no_reminder() {
    assert_eq!(due(noted(7), noon() - 30 * DAY, utc_clock), None);
}

#[test]
fn the_state_file_follows_xdg_state_home_if_it_is_absolute() {
    let home = Path::new("/home/anna");
    let default = PathBuf::from("/home/anna/.local/state/babawus/last_backup");

    assert_eq!(state_file(None, home), default);
    assert_eq!(state_file(Some(OsStr::new("")), home), default);
    assert_eq!(state_file(Some(OsStr::new("relative")), home), default);
    assert_eq!(
        state_file(Some(OsStr::new("/var/state/anna")), home),
        PathBuf::from("/var/state/anna/babawus/last_backup")
    );
}
