use std::path::PathBuf;
use std::time::SystemTime;

use super::*;
use crate::config::{Config, save};
use crate::fs::fake::FakeFs;
use crate::lock::ThisProcess;
use crate::medium::CONFIG_FILE;
use crate::versions::Format;

const EN: Language = Language::English;
const ID: &str = "0123456789abcdef0123456789abcdef";
const ROOT: &str = "/media/anna/BACKUP-A";
const HOME: &str = "/home/anna";

fn this() -> ThisProcess {
    ThisProcess {
        computer_id: ID.to_owned(),
        boot_id: String::new(),
        process_id: 1000,
        program: "babawus".to_owned(),
    }
}

/// A medium with an account for `anna` on this computer, which selects `selected`.
fn medium(selected: &[&str]) -> FakeFs {
    let mut config = Config::new();
    config.add_computer(ID, "laptop", "laptop", &[]).unwrap();
    config.add_account(ID, "anna").unwrap();
    config.computers[0].accounts[0].selected = selected.iter().map(|s| s.to_string()).collect();
    let files = FakeFs::exfat();
    files.add_dir_on_device(PathBuf::from(ROOT).join(".babawus"), 1);
    save(&files, &PathBuf::from(ROOT), Format::ExFat, &config).unwrap();
    files
}

/// This computer, with a few things in the home folder.
fn system() -> FakeFs {
    let system = FakeFs::new();
    let home = PathBuf::from(HOME);
    system.add_dir_on_device(home.join("Documents"), 1);
    system.add_file(home.join("notes.txt"), b"", SystemTime::UNIX_EPOCH);
    system.add_other(home.join("link"));
    system.add_dir_on_device(home.join(".cache"), 1);
    system.add_dir_on_device(home.join("Private"), 1);
    system.fail_path(home.join("Private"), std::io::ErrorKind::PermissionDenied);
    system
}

fn gather(files: &FakeFs, username: &str) -> Gathered {
    let system = system();
    let this = this();
    let viewer = Viewer {
        system: &system,
        this: &this,
        username,
    };
    viewer.coverage(files, &PathBuf::from(ROOT), &PathBuf::from(HOME))
}

fn kinds(gathered: &Gathered) -> Vec<Kind> {
    match gathered {
        Gathered::Covered(coverage) => coverage.selected.iter().map(|each| each.kind).collect(),
        other => panic!("not covered: {other:?}"),
    }
}

#[test]
fn each_selection_is_looked_up_on_this_computer() {
    let selected = [
        ".",
        "Documents",
        "notes.txt",
        "Gone",
        ".cache",
        "../bert",
        "link",
        "Private",
    ];

    let gathered = gather(&medium(&selected), "anna");

    assert_eq!(
        kinds(&gathered),
        [
            Kind::Home,
            Kind::Folder,
            Kind::File,
            Kind::Missing,
            Kind::Excluded,
            Kind::OutsideHome,
            Kind::Link,
            Kind::Unreadable,
        ]
    );
}

#[test]
fn another_account_is_not_set_up() {
    assert_eq!(gather(&medium(&["Documents"]), "bert"), Gathered::NotSetUp);
}

#[test]
fn a_damaged_configuration_is_unreadable() {
    let files = medium(&["Documents"]);
    files.add_file(
        PathBuf::from(ROOT).join(CONFIG_FILE),
        b"garbage",
        SystemTime::UNIX_EPOCH,
    );

    assert_eq!(gather(&files, "anna"), Gathered::Unreadable);
}

#[test]
fn the_screen_lists_the_selection_and_what_is_left_out() {
    let gathered = gather(&medium(&[".", "notes.txt", "Gone"]), "anna");

    let shown = coverage_screen(&gathered, "anna", EN);

    assert_eq!(
        shown,
        CoverageShown {
            intro: "These are backed up from this computer for anna:".to_owned(),
            items: vec![
                Item {
                    picture: Picture::Folder,
                    name: "Your home folder".to_owned(),
                    note: None,
                },
                Item {
                    picture: Picture::File,
                    name: "notes.txt".to_owned(),
                    note: None,
                },
                Item {
                    picture: Picture::NotBackedUp,
                    name: "Gone".to_owned(),
                    note: Some(
                        "Doesn't exist any more, so nothing is backed up from it".to_owned()
                    ),
                },
            ],
            left_out_heading: Some("Left out:".to_owned()),
            left_out: vec![
                ".local/share/Trash".to_owned(),
                "Anything named .cache, *.tmp or *~, in any folder".to_owned(),
            ],
        }
    );
}

#[test]
fn nothing_selected_says_who_to_ask() {
    let shown = coverage_screen(&gather(&medium(&[]), "anna"), "anna", EN);

    assert_eq!(
        shown.intro,
        "Nothing has been chosen for backup yet. Contact your poweruser."
    );
    assert!(shown.items.is_empty());
}

#[test]
fn nothing_left_out_has_no_heading() {
    let coverage = Coverage {
        selected: Vec::new(),
        exclude_patterns: Vec::new(),
        exclude_folders: Vec::new(),
    };

    let shown = coverage_screen(&Gathered::Covered(coverage), "anna", EN);

    assert_eq!(shown.left_out_heading, None);
    assert!(shown.left_out.is_empty());
}

#[test]
fn a_medium_that_no_longer_has_the_account_says_so() {
    let shown = coverage_screen(&Gathered::NotSetUp, "anna", EN);

    assert!(
        shown
            .intro
            .starts_with("This stick hasn't been set up for you")
    );
}
