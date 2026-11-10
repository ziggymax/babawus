use std::path::PathBuf;
use std::time::SystemTime;

use super::*;
use crate::config::save;
use crate::fs::fake::FakeFs;
use crate::log::read_runs;
use crate::medium::{CONFIG_FILE, LOCK_FILE};

const EN: Language = Language::English;
const ID: &str = "0123456789abcdef0123456789abcdef";
const OTHER_ID: &str = "fedcba9876543210fedcba9876543210";
const BOOT: &str = "11111111-2222-3333-4444-555555555555";
const ROOT: &str = "/media/anna/BACKUP-A";

fn root() -> PathBuf {
    PathBuf::from(ROOT)
}

fn this(computer_id: &str) -> ThisProcess {
    ThisProcess {
        computer_id: computer_id.to_owned(),
        boot_id: BOOT.to_owned(),
        process_id: 1000,
        program: "babawus".to_owned(),
    }
}

/// A medium holding `config`, or a blank one.
fn medium_with(config: Option<&Config>) -> FakeFs {
    let files = FakeFs::exfat();
    files.add_dir_on_device(root().join(".babawus"), 1);
    if let Some(config) = config {
        save(&files, &root(), Format::ExFat, config).unwrap();
    }
    files
}

/// A configuration with `desktop` (another computer, with `bert`) and, if asked, this
/// computer as `laptop` with `accounts`.
fn config(this_computer: Option<&[&str]>) -> Config {
    let mut config = Config::new();
    config
        .add_computer(OTHER_ID, "desktop", "Desktop-PC", &[])
        .unwrap();
    config.add_account(OTHER_ID, "bert").unwrap();
    if let Some(accounts) = this_computer {
        config.add_computer(ID, "laptop", "laptop", &[]).unwrap();
        for account in accounts {
            config.add_account(ID, account).unwrap();
        }
    }
    config
}

struct Setup {
    files: FakeFs,
    system: FakeFs,
    this: ThisProcess,
}

impl Setup {
    fn new(config: Option<&Config>) -> Self {
        Self {
            files: medium_with(config),
            system: FakeFs::new(),
            this: this(ID),
        }
    }

    fn here(&self) -> Here<'_> {
        Here {
            system: &self.system,
            this: &self.this,
            username: "anna",
            hostname: "Laptop",
        }
    }

    fn medium(&self) -> Medium<'_> {
        Medium {
            files: &self.files,
            root: Path::new(ROOT),
            format: Format::ExFat,
        }
    }

    fn editor(&self) -> Editor {
        match Editor::open(&self.medium()).unwrap() {
            Opened::Editing(editor) => editor,
            other => panic!("not editing: {other:?}"),
        }
    }

    fn loaded(&self) -> Config {
        match config::load(&self.files, &root()).unwrap() {
            Loaded::Ready(config) => config,
            other => panic!("not ready: {other:?}"),
        }
    }
}

#[test]
fn a_ready_configuration_opens_with_nothing_to_save() {
    let setup = Setup::new(Some(&config(Some(&["anna"]))));

    let editor = setup.editor();

    assert!(!editor.unsaved());
    assert!(!editor.damaged);
}

#[test]
fn a_configuration_that_can_be_parsed_but_is_damaged_must_be_saved() {
    let setup = Setup::new(Some(&config(None)));
    let text = config(None).to_text();
    let changed = text.replace("older_versions = 10", "older_versions = 12");
    setup.files.add_file(
        root().join(CONFIG_FILE),
        changed.as_bytes(),
        SystemTime::UNIX_EPOCH,
    );

    let editor = setup.editor();

    assert!(editor.damaged);
    assert!(editor.unsaved());
    assert_eq!(editor.config.older_versions, 12);
    let shown = overview(&editor, &setup.here(), EN);
    assert!(shown.banner.unwrap().contains("Check it, then save it"));
    assert!(shown.can_save);
}

#[test]
fn a_configuration_that_can_not_be_parsed_can_only_be_started_again() {
    let setup = Setup::new(None);
    setup
        .files
        .add_file(root().join(CONFIG_FILE), b"garbage", SystemTime::UNIX_EPOCH);

    assert_eq!(Editor::open(&setup.medium()).unwrap(), Opened::Unreadable);
    let editor = Editor::start_new(&setup.medium()).unwrap();
    assert!(editor.config.computers.is_empty());
    assert!(editor.unsaved());
    let shown = overview(&editor, &setup.here(), EN);
    assert_eq!(shown.banner, None, "nothing in it to check");
    assert!(shown.can_save);
}

#[test]
fn a_new_configuration_keeps_folders_on_the_medium_in_use() {
    let setup = Setup::new(None);
    setup
        .files
        .add_dir_on_device(root().join("backup/laptop"), 1);
    let mut editor = Editor::start_new(&setup.medium()).unwrap();

    editor.add_here(&setup.here()).unwrap();

    assert_eq!(editor.config.computers[0].folder, "laptop-2");
}

#[test]
fn a_medium_without_a_configuration_is_not_babawus() {
    let setup = Setup::new(None);

    assert_eq!(Editor::open(&setup.medium()).unwrap(), Opened::NotBabawus);
}

#[test]
fn a_missing_computer_can_be_added_or_linked() {
    let setup = Setup::new(Some(&config(None)));
    let editor = setup.editor();

    let shown = overview(&editor, &setup.here(), EN);

    assert!(
        shown
            .here
            .text
            .starts_with("This computer (Laptop) isn't set up")
    );
    assert!(shown.here.text.contains("link it"));
    // The button doesn't name the folder, so the line above does.
    assert!(shown.here.text.contains("Add it as “laptop”"));
    assert_eq!(
        shown.here.actions,
        [
            (
                HereAction::Add,
                "Add this computer and your account".to_owned()
            ),
            (
                HereAction::LinkComputer("desktop".to_owned()),
                "Link to “desktop”…".to_owned()
            ),
        ]
    );
    assert!(!shown.here.can_choose);
}

#[test]
fn adding_here_adds_the_computer_and_the_account_until_saved() {
    let setup = Setup::new(Some(&config(None)));
    let mut editor = setup.editor();

    editor.add_here(&setup.here()).unwrap();

    assert!(editor.unsaved());
    assert!(editor.config.account(ID, "anna").is_some());
    assert!(
        setup.loaded().account(ID, "anna").is_none(),
        "not saved yet"
    );
    let shown = overview(&editor, &setup.here(), EN);
    assert_eq!(
        shown.here.text,
        "Your backup profile, “anna”, is set up on this computer (“laptop”)."
    );
    assert!(shown.here.can_choose);
}

#[test]
fn saving_writes_the_changes_and_releases_the_lock() {
    let setup = Setup::new(Some(&config(None)));
    let mut editor = setup.editor();
    editor.add_here(&setup.here()).unwrap();
    editor.set_older_versions(5);

    editor.save(&setup.medium(), &setup.here()).unwrap();

    assert!(!editor.unsaved());
    let loaded = setup.loaded();
    assert_eq!(loaded.older_versions, 5);
    assert_eq!(loaded.computer(ID).unwrap().hostname, "Laptop");
    assert!(setup.files.metadata(&root().join(LOCK_FILE)).is_err());
}

#[test]
fn saving_waits_for_a_live_lock() {
    let setup = Setup::new(Some(&config(None)));
    setup
        .system
        .add_file("/proc/2000/comm", b"babawus\n", SystemTime::UNIX_EPOCH);
    let holder = format!(
        "run_id = \"0123456789abcdef\"\naccount = \"bert@desktop\"\ncomputer_id = \"{ID}\"\n\
         boot_id = \"{BOOT}\"\nprocess_id = 2000\n"
    );
    setup.files.add_file(
        root().join(LOCK_FILE),
        holder.as_bytes(),
        SystemTime::UNIX_EPOCH,
    );
    let mut editor = setup.editor();
    editor.set_older_versions(5);

    let error = editor.save(&setup.medium(), &setup.here()).unwrap_err();

    assert!(matches!(&error, SaveError::Busy(Some(user)) if user == "bert"));
    assert!(save_error_text(&error, EN).contains("for bert"));
    assert!(editor.unsaved());
    assert_eq!(setup.loaded().older_versions, 10);
}

#[test]
fn linking_a_computer_takes_over_its_entry() {
    let setup = Setup::new(Some(&config(None)));
    let mut editor = setup.editor();

    editor.link_computer("desktop", &setup.here()).unwrap();

    let computer = editor.config.computer(ID).unwrap();
    assert_eq!(computer.folder, "desktop");
    assert_eq!(computer.accounts[0].username, "bert");
}

#[test]
fn a_missing_account_can_be_added_or_linked_to_an_earlier_name() {
    let setup = Setup::new(Some(&config(Some(&["anne"]))));
    let editor = setup.editor();

    let shown = overview(&editor, &setup.here(), EN);

    assert_eq!(
        shown.here.actions,
        [
            (HereAction::Add, "Add “anna”".to_owned()),
            (
                HereAction::LinkAccount("anne".to_owned()),
                "Link to “anne”…".to_owned()
            ),
        ]
    );
}

#[test]
fn linking_an_account_renames_it_and_saves_at_once() {
    let setup = Setup::new(Some(&config(Some(&["anne"]))));
    setup
        .files
        .add_dir_on_device(root().join("backup/laptop/anne"), 1);
    let mut editor = setup.editor();
    editor.set_older_versions(7);

    editor
        .link_account(&setup.medium(), &setup.here(), "anne")
        .unwrap();

    assert!(!editor.unsaved());
    let loaded = setup.loaded();
    assert!(loaded.account(ID, "anna").is_some());
    assert_eq!(loaded.older_versions, 7);
    assert!(
        setup
            .files
            .metadata(&root().join("backup/laptop/anna"))
            .is_ok()
    );
    assert!(read_runs(&setup.files, &root()).unwrap().is_empty());
}

#[test]
fn linking_an_account_with_both_folders_changes_nothing() {
    let setup = Setup::new(Some(&config(Some(&["anne"]))));
    setup
        .files
        .add_dir_on_device(root().join("backup/laptop/anne"), 1);
    setup
        .files
        .add_dir_on_device(root().join("backup/laptop/anna"), 1);
    let mut editor = setup.editor();

    let error = editor
        .link_account(&setup.medium(), &setup.here(), "anne")
        .unwrap_err();

    assert!(matches!(error, SaveError::BothFolders));
    assert!(setup.loaded().account(ID, "anne").is_some());
}

#[test]
fn removing_an_entry_waits_for_save() {
    let setup = Setup::new(Some(&config(Some(&["anna"]))));
    let mut editor = setup.editor();

    editor.remove_account(OTHER_ID, "bert").unwrap();
    editor.remove_computer(OTHER_ID).unwrap();

    assert!(editor.unsaved());
    assert!(editor.config.computer(OTHER_ID).is_none());
    assert!(setup.loaded().computer(OTHER_ID).is_some());
}

#[test]
fn the_entries_say_which_is_this_computer_and_you() {
    let mut config = config(Some(&["anna", "carl"]));
    config.computers[1].accounts[0].selected = vec!["Documents".to_owned()];
    let setup = Setup::new(Some(&config));
    let editor = setup.editor();

    let shown = overview(&editor, &setup.here(), EN);

    assert_eq!(
        shown.computers[0].title,
        "desktop · last seen as Desktop-PC"
    );
    assert_eq!(
        shown.computers[0].accounts[0].text,
        "bert · nothing chosen yet"
    );
    assert_eq!(shown.computers[1].title, "laptop (this computer)");
    assert_eq!(
        shown.computers[1].accounts[0].text,
        "anna (you) · 1 file or folder chosen"
    );
    assert_eq!(
        shown.computers[1].accounts[1].text,
        "carl · nothing chosen yet"
    );
    assert_eq!(shown.empty, None);
}

#[test]
fn a_new_medium_says_nothing_is_set_up() {
    let setup = Setup::new(Some(&Config::new()));
    let editor = setup.editor();

    let shown = overview(&editor, &setup.here(), EN);

    assert!(shown.computers.is_empty());
    assert!(shown.empty.is_some());
    assert_eq!(shown.here.actions.len(), 1);
    assert!(!shown.here.text.contains("link"), "{}", shown.here.text);
}

#[test]
fn a_computer_without_an_id_can_not_be_added() {
    let mut setup = Setup::new(Some(&Config::new()));
    setup.this = this("");
    let mut editor = setup.editor();

    let shown = overview(&editor, &setup.here(), EN);

    assert!(shown.here.actions.is_empty());
    assert!(editor.add_here(&setup.here()).is_err());
}
