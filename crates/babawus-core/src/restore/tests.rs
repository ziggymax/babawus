use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::fs::fake::FakeFs;

const ROOT: &str = "/media/anna/BACKUP-A/backup/laptop/anna";
const HOME: &str = "/home/anna";

fn time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_794_000_000 + seconds)
}

/// The time in an older version's name, as `versions` reads it.
fn replaced(timestamp: &str) -> SystemTime {
    let (date, clock) = timestamp.split_once('_').unwrap();
    let clock = format!("{}:{}:{}", &clock[..2], &clock[2..4], &clock[4..]);
    crate::time::parse_log_timestamp(&format!("{date} {clock} +0000")).unwrap()
}

fn clock(time: SystemTime) -> ClockTime {
    crate::time::utc_clock(time)
}

fn path(relative: &str) -> PathBuf {
    PathBuf::from(relative)
}

/// A backup of Documents, with older versions of report.pdf, a file deleted from the computer,
/// and a version folder whose file is gone; and a home folder that has only some of it.
struct Setup {
    medium: FakeFs,
    system: FakeFs,
    format: Format,
}

impl Setup {
    fn new(format: Format) -> Self {
        let medium = match format {
            Format::ExFat => FakeFs::exfat(),
            Format::Ext4 => FakeFs::new(),
        };
        let setup = Self {
            medium,
            system: FakeFs::new(),
            format,
        };
        let stick = |relative: &str| PathBuf::from(ROOT).join(relative);
        setup.medium.add_dir_on_device(ROOT, 1);
        setup.medium.add_dir_on_device(stick("Documents"), 1);
        setup
            .medium
            .add_file(stick("Documents/report.pdf"), b"newest", time(300));
        setup
            .medium
            .add_dir_on_device(stick("Documents/report[]pdf"), 1);
        for (name, data, modified) in [
            ("report[2026-11-01_080000].pdf", b"oldest".as_slice(), 100),
            ("report[2026-11-02_080000].pdf", b"older", 200),
        ] {
            setup.medium.add_file(
                stick("Documents/report[]pdf").join(name),
                data,
                time(modified),
            );
        }
        setup
            .medium
            .add_file(stick("Documents/deleted.txt"), b"gone", time(10));
        setup.medium.add_dir_on_device(stick("Documents/old"), 1);
        setup
            .medium
            .add_file(stick("Documents/old/draft.txt"), b"draft", time(20));
        setup
            .medium
            .add_dir_on_device(stick("Documents/lost[]txt"), 1);
        setup.medium.add_file(
            stick("Documents/lost[]txt/lost[2026-11-03_120000].txt"),
            b"lost",
            time(30),
        );
        setup
            .medium
            .add_file(stick("notes.txt"), b"notes", time(40));
        setup.medium.add_other(stick("link"));

        let home = |relative: &str| PathBuf::from(HOME).join(relative);
        setup.system.add_dir_on_device(HOME, 1);
        setup.system.add_dir_on_device(home("Documents"), 1);
        setup
            .system
            .add_file(home("Documents/report.pdf"), b"changed", time(400));
        setup
    }

    fn backup(&self) -> Backup<'_> {
        Backup {
            medium: &self.medium,
            root: PathBuf::from(ROOT),
            format: self.format,
            system: &self.system,
            home: PathBuf::from(HOME),
        }
    }

    fn plan(&self, choice: &Choice) -> Plan {
        self.backup()
            .plan(choice, Path::new(HOME), &AtomicBool::new(false))
            .unwrap()
    }

    fn restore(&self, choice: &Choice, resolution: Resolution) -> Outcome {
        let plan = self.plan(choice);
        self.backup().restore(
            &plan,
            resolution,
            &clock,
            &Progress::default(),
            &AtomicBool::new(false),
        )
    }

    fn home_file(&self, relative: &str) -> Option<(Vec<u8>, SystemTime)> {
        let path = PathBuf::from(HOME).join(relative);
        let metadata = self.system.metadata(&path).ok()?;
        let mut data = Vec::new();
        self.system.open(&path).ok()?.read_to_end(&mut data).ok()?;
        Some((data, metadata.modified))
    }
}

fn names(items: &[Item]) -> Vec<&str> {
    items.iter().map(|item| item.name.as_str()).collect()
}

fn ticked(paths: &[&str]) -> Choice {
    Choice {
        ticked: paths.iter().map(|each| path(each)).collect(),
        ..Choice::default()
    }
}

#[test]
fn the_tree_shows_folders_first_and_version_folders_as_their_files() {
    let setup = Setup::new(Format::Ext4);
    let backup = setup.backup();

    let top = backup.children(Path::new("")).unwrap();
    let documents = backup.children(Path::new("Documents")).unwrap();

    assert_eq!(names(&top), ["Documents", "notes.txt"], "no link");
    assert_eq!(
        names(&documents),
        ["old", "deleted.txt", "lost.txt", "report.pdf"]
    );
    let item = |name: &str| documents.iter().find(|item| item.name == name).unwrap();
    assert!(item("report.pdf").on_computer);
    assert!(!item("deleted.txt").on_computer);
    assert!(!item("report.pdf").only_older);
    assert!(item("lost.txt").only_older);
    assert_eq!(item("old").kind, FileKind::Dir);
    assert_eq!(item("report.pdf").relative, path("Documents/report.pdf"));
}

#[test]
fn a_file_shows_its_newest_copy_and_then_its_older_versions_newest_first() {
    let setup = Setup::new(Format::Ext4);

    let versions = setup
        .backup()
        .versions(Path::new("Documents/report.pdf"))
        .unwrap();

    assert_eq!(
        versions,
        [
            Version {
                copy: Copy::Newest,
                time: time(300),
                len: 6,
            },
            Version {
                copy: Copy::Older("report[2026-11-02_080000].pdf".into()),
                time: replaced("2026-11-02_080000"),
                len: 5,
            },
            Version {
                copy: Copy::Older("report[2026-11-01_080000].pdf".into()),
                time: replaced("2026-11-01_080000"),
                len: 6,
            },
        ]
    );
    let lost = setup
        .backup()
        .versions(Path::new("Documents/lost.txt"))
        .unwrap();
    assert_eq!(lost.len(), 1);
    assert_eq!(
        lost[0].copy,
        Copy::Older("lost[2026-11-03_120000].txt".into())
    );
}

#[test]
fn on_exfat_versions_are_found_whatever_their_case() {
    let setup = Setup::new(Format::ExFat);
    let stick = PathBuf::from(ROOT).join("Documents/report[]pdf");
    setup
        .medium
        .add_file(stick.join("REPORT[2026-11-03_080000].PDF"), b"x", time(250));

    let versions = setup
        .backup()
        .versions(Path::new("Documents/report.pdf"))
        .unwrap();

    assert_eq!(versions.len(), 4);
    assert_eq!(
        versions[1].copy,
        Copy::Older("REPORT[2026-11-03_080000].PDF".into())
    );
}

#[test]
fn ticking_works_as_in_choose_what_to_back_up() {
    let mut choice = Choice::default();

    assert!(choice.toggle(Path::new("Documents"), true));
    assert!(choice.row(Path::new("Documents/report.pdf"), false).ticked);

    assert!(choice.toggle(Path::new("Documents/report.pdf"), false));
    let left_out = choice.row(Path::new("Documents/report.pdf"), false);
    assert!(!left_out.ticked);
    assert!(left_out.crossed);
    assert!(choice.row(Path::new("Documents"), true).partly);
    assert!(
        choice.row(Path::new(""), true).partly,
        "the home folder too"
    );

    assert!(choice.toggle(Path::new("Documents/old"), true));
    let below = choice.row(Path::new("Documents/old/draft.txt"), false);
    assert!(below.left_out_above);
    assert!(!below.can_toggle);
    assert!(!choice.toggle(Path::new("Documents/old/draft.txt"), false));

    assert!(choice.toggle(Path::new("Documents/report.pdf"), false));
    assert!(choice.row(Path::new("Documents/report.pdf"), false).ticked);
    assert_eq!(choice.left_out, [path("Documents/old")]);
}

#[test]
fn unticking_a_ticked_file_inside_a_ticked_folder_leaves_it_out() {
    let mut choice = ticked(&["Documents", "Documents/report.pdf"]);

    choice.toggle(Path::new("Documents/report.pdf"), false);

    assert_eq!(choice.ticked, [path("Documents")]);
    assert_eq!(choice.left_out, [path("Documents/report.pdf")]);
}

#[test]
fn an_unticked_folder_with_something_ticked_inside_is_partly_ticked() {
    let choice = ticked(&["Documents/report.pdf"]);

    assert!(choice.row(Path::new("Documents"), true).partly);
    assert!(!choice.row(Path::new("Documents"), true).ticked);
}

#[test]
fn picking_an_older_version_ticks_the_file_and_newest_takes_it_back() {
    let mut choice = Choice::default();
    let report = Path::new("Documents/report.pdf");
    let older = Copy::Older("report[2026-11-02_080000].pdf".into());

    assert!(choice.pick(report, older.clone()));

    assert!(choice.row(report, false).ticked);
    assert_eq!(choice.copy_of(report), older);
    choice.pick(report, Copy::Newest);
    assert_eq!(choice.copy_of(report), Copy::Newest);
    assert!(choice.row(report, false).ticked);
}

#[test]
fn picking_a_version_inside_a_left_out_folder_does_not_tick_it() {
    let mut choice = ticked(&["Documents"]);
    choice.left_out = vec![path("Documents/old")];

    let ticked = choice.pick(Path::new("Documents/old/draft.txt"), Copy::Newest);

    assert!(!ticked);
}

#[test]
fn the_plan_follows_the_ticks_and_the_picked_versions() {
    let setup = Setup::new(Format::Ext4);
    let mut choice = ticked(&["Documents"]);
    choice.left_out = vec![path("Documents/old")];
    choice.pick(
        Path::new("Documents/report.pdf"),
        Copy::Older("report[2026-11-01_080000].pdf".into()),
    );

    let plan = setup.plan(&choice);

    assert_eq!(plan.folders, [path("Documents")]);
    let files: Vec<_> = plan
        .files
        .iter()
        .map(|file| (file.relative.clone(), file.copy.clone(), file.existing))
        .collect();
    assert_eq!(
        files,
        [
            (path("Documents/deleted.txt"), Copy::Newest, None),
            (
                path("Documents/lost.txt"),
                Copy::Older("lost[2026-11-03_120000].txt".into()),
                None
            ),
            (
                path("Documents/report.pdf"),
                Copy::Older("report[2026-11-01_080000].pdf".into()),
                Some(Existing::File)
            ),
        ]
    );
    assert_eq!(plan.already_there(), 1);
    assert_eq!(plan.bytes(), 4 + 4 + 6);
}

#[test]
fn the_whole_home_folder_can_be_restored() {
    let setup = Setup::new(Format::Ext4);

    let plan = setup.plan(&ticked(&[""]));

    assert_eq!(plan.files.len(), 5);
    assert_eq!(plan.folders, [path("Documents"), path("Documents/old")]);
}

#[test]
fn a_ticked_path_inside_a_ticked_folder_is_planned_once() {
    let setup = Setup::new(Format::Ext4);

    let plan = setup.plan(&ticked(&["Documents", "Documents/deleted.txt"]));

    let deleted = plan
        .files
        .iter()
        .filter(|file| file.relative == path("Documents/deleted.txt"))
        .count();
    assert_eq!(deleted, 1);
}

#[test]
fn restored_files_get_their_content_and_modification_time_and_the_stick_is_not_changed() {
    let setup = Setup::new(Format::Ext4);
    let steps = setup.medium.steps();
    let mut choice = ticked(&["Documents"]);
    choice.left_out = vec![path("Documents/report.pdf")];

    let outcome = setup.restore(&choice, Resolution::Skip);

    assert_eq!(outcome.restored, 3);
    assert_eq!(outcome.bytes, 4 + 4 + 5);
    assert!(outcome.failed.is_empty());
    assert_eq!(
        setup.home_file("Documents/deleted.txt"),
        Some((b"gone".to_vec(), time(10)))
    );
    assert_eq!(
        setup.home_file("Documents/old/draft.txt"),
        Some((b"draft".to_vec(), time(20)))
    );
    assert_eq!(setup.home_file("Documents/lost.txt").unwrap().0, b"lost");
    assert_eq!(setup.medium.steps(), steps, "nothing written to the stick");
    let leftovers: Vec<_> = setup
        .system
        .read_dir(Path::new("/home/anna/Documents"))
        .unwrap()
        .into_iter()
        .filter(|name| name.to_string_lossy().starts_with(".babawus"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn a_file_already_in_place_is_replaced_skipped_or_kept_next_to_it() {
    let report = "Documents/report.pdf";

    let setup = Setup::new(Format::Ext4);
    let outcome = setup.restore(&ticked(&[report]), Resolution::Skip);
    assert_eq!((outcome.restored, outcome.skipped), (0, 1));
    assert_eq!(setup.home_file(report).unwrap().0, b"changed");

    let setup = Setup::new(Format::Ext4);
    let outcome = setup.restore(&ticked(&[report]), Resolution::Replace);
    assert_eq!(outcome.restored, 1);
    assert_eq!(
        setup.home_file(report),
        Some((b"newest".to_vec(), time(300)))
    );

    let setup = Setup::new(Format::Ext4);
    let kept = kept_name(OsStr::new("report.pdf"), clock(time(300)), 1);
    let kept = format!("Documents/{}", kept.to_string_lossy());
    setup.restore(&ticked(&[report]), Resolution::KeepBoth);
    assert_eq!(setup.home_file(report).unwrap().0, b"changed");
    assert_eq!(setup.home_file(&kept).unwrap().0, b"newest");
    setup.restore(&ticked(&[report]), Resolution::KeepBoth);
    let second = kept_name(OsStr::new("report.pdf"), clock(time(300)), 2);
    let second = format!("Documents/{}", second.to_string_lossy());
    assert_eq!(setup.home_file(&second).unwrap().0, b"newest");
}

#[test]
fn an_older_version_kept_next_to_the_file_has_the_time_it_was_replaced() {
    let setup = Setup::new(Format::Ext4);
    let mut choice = Choice::default();
    choice.pick(
        Path::new("Documents/report.pdf"),
        Copy::Older("report[2026-11-02_080000].pdf".into()),
    );

    setup.restore(&choice, Resolution::KeepBoth);

    assert_eq!(
        setup.home_file("Documents/report (2026-11-02 08.00).pdf"),
        Some((b"older".to_vec(), time(200)))
    );
}

#[test]
fn something_in_the_way_is_left_alone_and_reported() {
    let setup = Setup::new(Format::Ext4);
    setup.system.add_dir_on_device("/home/anna/notes.txt", 1);
    setup
        .system
        .add_file("/home/anna/Documents/old", b"a file", time(1));

    let outcome = setup.restore(
        &ticked(&["notes.txt", "Documents/old"]),
        Resolution::Replace,
    );

    assert_eq!(outcome.restored, 0);
    assert_eq!(
        outcome.failed,
        [
            (path("Documents/old/draft.txt"), Failure::InTheWay),
            (path("notes.txt"), Failure::InTheWay),
        ]
    );
}

#[test]
fn a_full_home_folder_stops_the_restore() {
    let setup = Setup::new(Format::Ext4);
    setup.system.set_capacity(0);

    let outcome = setup.restore(&ticked(&["Documents"]), Resolution::Skip);

    assert!(outcome.full);
    assert_eq!(outcome.restored, 0);
    assert_eq!(outcome.failed.len(), 1);
}

#[test]
fn an_unreadable_copy_is_reported_and_the_rest_goes_on() {
    let setup = Setup::new(Format::Ext4);
    setup.medium.fail_path(
        PathBuf::from(ROOT).join("Documents/deleted.txt"),
        ErrorKind::PermissionDenied,
    );

    let outcome = setup.restore(
        &ticked(&["Documents/deleted.txt", "notes.txt"]),
        Resolution::Skip,
    );

    assert_eq!(outcome.restored, 1);
    assert_eq!(
        outcome.failed,
        [(path("Documents/deleted.txt"), Failure::Unreadable)]
    );
}

#[test]
fn a_cancelled_restore_stops_before_the_next_file() {
    let setup = Setup::new(Format::Ext4);
    let plan = setup.plan(&ticked(&["Documents"]));
    let cancel = AtomicBool::new(true);

    let outcome = setup.backup().restore(
        &plan,
        Resolution::Skip,
        &clock,
        &Progress::default(),
        &cancel,
    );

    assert!(outcome.cancelled);
    assert_eq!(outcome.restored, 0);
}

#[test]
fn cancel_pressed_while_a_file_is_flushed_leaves_the_file_there_alone() {
    let setup = Setup::new(Format::Ext4);
    let plan = setup.plan(&ticked(&["Documents/report.pdf"]));
    let cancel = Arc::new(AtomicBool::new(false));
    setup.system.raise_on_sync(cancel.clone());

    let outcome = setup.backup().restore(
        &plan,
        Resolution::Replace,
        &clock,
        &Progress::default(),
        &cancel,
    );

    assert!(outcome.cancelled);
    assert_eq!(outcome.restored, 0);
    let folder = PathBuf::from(HOME).join("Documents");
    // The file there is untouched, and no temporary file is left.
    assert_eq!(setup.system.read_dir(&folder).unwrap(), ["report.pdf"]);
    let mut data = Vec::new();
    setup
        .system
        .open(&folder.join("report.pdf"))
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    assert_eq!(data, b"changed");
}

#[test]
fn a_cancelled_plan_gives_nothing() {
    let setup = Setup::new(Format::Ext4);

    let plan = setup.backup().plan(
        &ticked(&["Documents"]),
        Path::new(HOME),
        &AtomicBool::new(true),
    );

    assert!(plan.is_none());
}

#[test]
fn a_kept_name_puts_the_date_before_the_extension() {
    let at = ClockTime {
        year: 2026,
        month: 11,
        day: 3,
        hour: 17,
        minute: 54,
    };
    let cases = [
        ("report.pdf", 1, "report (2026-11-03 17.54).pdf"),
        ("report.pdf", 2, "report (2026-11-03 17.54, 2).pdf"),
        ("archive.tar.gz", 1, "archive.tar (2026-11-03 17.54).gz"),
        ("Makefile", 1, "Makefile (2026-11-03 17.54)"),
        (".bashrc", 1, ".bashrc (2026-11-03 17.54)"),
    ];
    for (name, number, kept) in cases {
        assert_eq!(
            kept_name(OsStr::new(name), at, number),
            OsString::from(kept)
        );
    }
}

#[test]
fn files_can_go_into_another_folder_below_which_they_keep_their_folders() {
    let setup = Setup::new(Format::Ext4);
    let target = Path::new("/home/anna/Restored/2026");
    let plan = setup
        .backup()
        .plan(
            &ticked(&["Documents/report.pdf", "notes.txt"]),
            target,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(plan.already_there(), 0, "asked about the other folder");

    let outcome = setup.backup().restore(
        &plan,
        Resolution::Skip,
        &clock,
        &Progress::default(),
        &AtomicBool::new(false),
    );

    assert_eq!(outcome.restored, 2);
    assert_eq!(
        setup.home_file("Restored/2026/Documents/report.pdf"),
        Some((b"newest".to_vec(), time(300)))
    );
    assert_eq!(
        setup.home_file("Restored/2026/notes.txt").unwrap().0,
        b"notes"
    );
    assert_eq!(
        setup.home_file("Documents/report.pdf").unwrap().0,
        b"changed"
    );
}

#[test]
fn files_already_in_another_folder_are_asked_about_there() {
    let setup = Setup::new(Format::Ext4);
    setup.system.add_dir_on_device("/home/anna/Restored", 1);
    setup
        .system
        .add_file("/home/anna/Restored/notes.txt", b"mine", time(1));

    let plan = setup
        .backup()
        .plan(
            &ticked(&["notes.txt", "Documents/report.pdf"]),
            Path::new("/home/anna/Restored"),
            &AtomicBool::new(false),
        )
        .unwrap();

    assert_eq!(plan.already_there(), 1);
}

#[test]
fn nothing_is_ever_restored_onto_the_stick() {
    let setup = Setup::new(Format::Ext4);
    let backup = setup.backup();
    let on_stick = Path::new("/media/anna/BACKUP-A/restored");

    assert!(!backup.may_restore_to(on_stick));
    assert!(!backup.may_restore_to(Path::new("/media/anna/BACKUP-A")));
    assert!(!backup.may_restore_to(Path::new("relative")));
    assert!(backup.may_restore_to(Path::new("/media/anna/BACKUP-AB")));
    assert!(backup.may_restore_to(Path::new(HOME)));

    let plan = backup
        .plan(&ticked(&["notes.txt"]), on_stick, &AtomicBool::new(false))
        .unwrap();
    let steps = setup.medium.steps();
    let outcome = backup.restore(
        &plan,
        Resolution::Skip,
        &clock,
        &Progress::default(),
        &AtomicBool::new(false),
    );

    assert_eq!(outcome.failed, [(path("notes.txt"), Failure::OnTheStick)]);
    assert_eq!(setup.medium.steps(), steps);
}

#[test]
fn the_suggested_folder_is_new_and_in_the_home_folder() {
    let setup = Setup::new(Format::Ext4);
    let now = ClockTime {
        year: 2026,
        month: 11,
        day: 17,
        hour: 15,
        minute: 2,
    };
    let backup = setup.backup();

    let first = backup.suggested_folder("Restored from BACKUP-A", now);
    assert_eq!(
        first,
        Path::new("/home/anna/Restored from BACKUP-A 2026-11-17 15.02")
    );
    setup.system.add_dir_on_device(&first, 1);

    assert_eq!(
        backup.suggested_folder("Restored from BACKUP-A", now),
        Path::new("/home/anna/Restored from BACKUP-A 2026-11-17 15.02, 2")
    );
}

#[test]
fn a_file_in_the_way_of_the_target_folder_stops_every_file() {
    let setup = Setup::new(Format::Ext4);
    setup
        .system
        .add_file("/home/anna/Restored", b"a file", time(1));
    let plan = setup
        .backup()
        .plan(
            &ticked(&["notes.txt"]),
            Path::new("/home/anna/Restored/2026"),
            &AtomicBool::new(false),
        )
        .unwrap();

    let outcome = setup.backup().restore(
        &plan,
        Resolution::Skip,
        &clock,
        &Progress::default(),
        &AtomicBool::new(false),
    );

    assert_eq!(outcome.failed, [(path("notes.txt"), Failure::InTheWay)]);
}

#[test]
fn a_left_over_is_the_temporary_name_of_another_process() {
    let own = 4321;
    let cases = [
        (".babawus-restore-1234-3", true),
        (".babawus-restore-4321-3", false),
        (".babawus-restore-1234-", false),
        (".babawus-restore-1234", false),
        (".babawus-restore-x-3", false),
        (".babawus-restore-1234-3.txt", false),
        ("babawus-restore-1234-3", false),
        ("notes.txt", false),
    ];
    for (name, left_over) in cases {
        assert_eq!(is_left_over(OsStr::new(name), own), left_over, "{name}");
    }
}

#[test]
fn files_left_over_by_a_restore_that_crashed_are_removed_where_it_writes() {
    let setup = Setup::new(Format::Ext4);
    let other = std::process::id().wrapping_add(1);
    let own = std::process::id();
    let documents = PathBuf::from(HOME).join("Documents");
    for name in [
        format!(".babawus-restore-{other}-1"),
        format!(".babawus-restore-{own}-9"),
        ".babawus-restore-notes".to_owned(),
    ] {
        setup
            .system
            .add_file(documents.join(name), b"half", time(1));
    }
    setup.system.add_dir_on_device("/home/anna/Elsewhere", 1);
    let elsewhere =
        PathBuf::from("/home/anna/Elsewhere").join(format!(".babawus-restore-{other}-2"));
    setup.system.add_file(&elsewhere, b"half", time(1));

    setup.restore(&ticked(&["Documents/deleted.txt"]), Resolution::Skip);

    let names: Vec<String> = setup
        .system
        .read_dir(&documents)
        .unwrap()
        .iter()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".babawus"))
        .collect();
    assert_eq!(
        names,
        [
            format!(".babawus-restore-{own}-9"),
            ".babawus-restore-notes".to_owned()
        ]
    );
    assert!(
        setup.system.metadata(&elsewhere).is_ok(),
        "not a folder it writes to"
    );
}
