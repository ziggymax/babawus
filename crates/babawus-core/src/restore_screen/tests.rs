use std::time::Duration;

use super::*;
use crate::config::{Config, save};
use crate::fs::FileKind;
use crate::fs::fake::FakeFs;
use crate::lock::ThisProcess;
use crate::time::utc_clock;
use crate::versions::Format;

const ID: &str = "0123456789abcdef0123456789abcdef";
const STICK: &str = "/media/anna/BACKUP-A";

fn en() -> Language {
    crate::language::testing::english()
}

fn this() -> ThisProcess {
    ThisProcess {
        computer_id: ID.to_owned(),
        boot_id: String::new(),
        process_id: 1000,
        program: "babawus".to_owned(),
    }
}

/// 2026-11-03 17:54 UTC.
fn at() -> SystemTime {
    crate::time::parse_log_timestamp("2026-11-03 17:54:00 +0000").unwrap()
}

fn item(on_computer: bool, only_older: bool) -> Item {
    Item {
        name: "report.pdf".to_owned(),
        relative: PathBuf::from("Documents/report.pdf"),
        kind: FileKind::File,
        on_computer,
        only_older,
    }
}

fn row(ticked: bool, crossed: bool, left_out_above: bool) -> Row {
    Row {
        ticked,
        can_toggle: !left_out_above,
        partly: false,
        crossed,
        left_out_above,
    }
}

#[test]
fn the_backup_folder_is_the_users_profile_on_this_computer() {
    let mut config = Config::new();
    config.add_computer(ID, "laptop", "laptop", &[]).unwrap();
    config.add_account(ID, "Anna").unwrap();
    let files = FakeFs::new();
    files.add_dir_on_device(PathBuf::from(STICK).join(".babawus"), 1);
    save(&files, Path::new(STICK), Format::Ext4, &config).unwrap();
    let this = this();
    let viewer = |username| Viewer {
        system: &files,
        this: &this,
        username,
    };

    assert_eq!(
        viewer("Anna").backup_folder(&files, Path::new(STICK)),
        Some(PathBuf::from("/media/anna/BACKUP-A/backup/laptop/anna"))
    );
    assert_eq!(viewer("bob").backup_folder(&files, Path::new(STICK)), None);
    assert_eq!(
        viewer("Anna").backup_folder(&files, Path::new("/media/anna/OTHER")),
        None
    );
}

#[test]
fn a_rows_note_says_why_it_is_left_out_which_version_and_whether_it_is_here() {
    let picked = Some(utc_clock(at()));
    let cases = [
        (item(true, false), row(true, false, false), None, None),
        (
            item(false, false),
            row(true, false, false),
            None,
            Some("Not on this computer"),
        ),
        (
            item(true, false),
            row(true, false, false),
            picked,
            Some("Version of 2026-11-03 17:54"),
        ),
        (
            item(false, true),
            row(false, true, false),
            picked,
            Some("Left out · Only older versions"),
        ),
        (
            item(true, false),
            row(false, true, true),
            None,
            Some("Left out with a folder above it"),
        ),
        (item(true, false), row(false, false, false), picked, None),
    ];
    for (item, row, picked, note) in cases {
        assert_eq!(
            row_note(&item, &row, picked, en()).as_deref(),
            note,
            "{row:?} {picked:?}"
        );
    }
}

#[test]
fn versions_show_what_they_are_when_and_how_big() {
    let newest = Version {
        copy: Copy::Newest,
        time: at(),
        len: 1200,
    };
    let older = Version {
        copy: Copy::Older("report[2026-11-02_080000].pdf".into()),
        time: at() - Duration::from_secs(86_400),
        len: 999,
    };

    assert_eq!(
        version_text(&newest, utc_clock, en()),
        "Newest copy, changed 2026-11-03 17:54 · 1.2 kB"
    );
    assert_eq!(
        version_text(&older, utc_clock, en()),
        "Replaced 2026-11-02 17:54 · 999 bytes"
    );
}

#[test]
fn the_question_and_the_progress_count_files() {
    assert_eq!(question(1, en()).0, "One of the files is already there");
    assert_eq!(question(12, en()).0, "12 of the files are already there");
    assert!(question(2, en()).1.contains("“Keep both”"));
    assert_eq!(progress_text(3, 1234, en()), "Restoring… 3 of 1,234 files");
}

#[test]
fn the_result_says_how_many_were_restored_and_why_others_werent() {
    let all = Outcome {
        restored: 12,
        ..Outcome::default()
    };
    assert_eq!(result_text(&all, 12, en()).summary, "12 files restored.");
    let one = Outcome {
        restored: 1,
        ..Outcome::default()
    };
    assert_eq!(result_text(&one, 1, en()).summary, "1 file restored.");

    let mixed = Outcome {
        restored: 9,
        skipped: 2,
        failed: vec![(PathBuf::from("Documents/a.txt"), Failure::InTheWay)],
        ..Outcome::default()
    };
    let shown = result_text(&mixed, 12, en());
    assert_eq!(shown.summary, "9 of 12 files restored.");
    assert_eq!(
        shown.skipped.as_deref(),
        Some("Skipped, because they were already there: 2 files")
    );
    assert_eq!(
        shown.failed_heading.as_deref(),
        Some("These weren't restored:")
    );
    assert_eq!(
        shown.failed,
        ["Documents/a.txt: Something else of that name is in the way #59"]
    );

    let stopped = Outcome {
        restored: 3,
        cancelled: true,
        ..Outcome::default()
    };
    assert_eq!(
        result_text(&stopped, 12, en()).summary,
        "The restore was stopped. 3 of 12 files restored."
    );
    let full = Outcome {
        full: true,
        failed: vec![(PathBuf::from("big.bin"), Failure::NotWritable)],
        ..Outcome::default()
    };
    let shown = result_text(&full, 2, en());
    assert_eq!(shown.summary, "Nothing was restored.");
    assert_eq!(
        shown.full.as_deref(),
        Some("The disk is full, so the restore stopped. #60")
    );
}
