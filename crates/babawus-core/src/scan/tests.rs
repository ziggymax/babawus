use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::time::{Duration, SystemTime};

use super::*;
use crate::fs::fake::FakeFs;

const HOME: &str = "/home/user";

fn time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + seconds)
}

struct Setup {
    source: FakeFs,
    medium: FakeFs,
    format: Format,
    selected: Vec<&'static str>,
    exclusions: Exclusions,
    cancel: AtomicBool,
}

impl Setup {
    fn new(format: Format, selected: &[&'static str]) -> Self {
        let medium = match format {
            Format::ExFat => FakeFs::exfat(),
            Format::Ext4 => FakeFs::new(),
        };
        Self {
            source: FakeFs::new(),
            medium,
            format,
            selected: selected.to_vec(),
            exclusions: Exclusions::default(),
            cancel: AtomicBool::new(false),
        }
    }

    fn add(&self, relative: &str, data: &[u8]) {
        self.source
            .add_file(format!("{HOME}/{relative}"), data, time(0));
    }

    fn add_copy(&self, relative: &str, data: &[u8], modified: SystemTime) {
        let path = format!("/medium/backup/mint/user/{relative}");
        self.medium.add_file(path, data, modified);
    }

    fn run(&self) -> Scan {
        let selection = Selection {
            home: PathBuf::from(HOME),
            selected: self.selected.iter().map(PathBuf::from).collect(),
            exclusions: self.exclusions.clone(),
        };
        let destination = Destination {
            medium: &self.medium,
            root: PathBuf::from("/medium"),
            account: account_folder("mint", "user"),
            format: self.format,
        };
        scan(&self.source, &selection, &destination, &self.cancel).expect("not cancelled")
    }
}

fn files(scan: &Scan) -> Vec<String> {
    let files = scan.entries.iter().filter_map(|entry| match entry {
        Entry::File { relative, .. } => Some(relative.to_string_lossy().into_owned()),
        Entry::Folder { .. } => None,
    });
    files.collect()
}

fn folders(scan: &Scan) -> Vec<String> {
    let folders = scan.entries.iter().filter_map(|entry| match entry {
        Entry::Folder { relative } => Some(relative.to_string_lossy().into_owned()),
        Entry::File { .. } => None,
    });
    folders.collect()
}

fn skipped(relative: &str, reason: Skip, files: u64) -> Skipped {
    Skipped {
        relative: PathBuf::from(relative),
        reason,
        files,
    }
}

#[test]
fn the_user_name_is_lower_case_in_the_account_folder() {
    assert_eq!(
        account_folder("mint", "UserA"),
        Path::new("backup/mint/usera")
    );
}

#[test]
fn a_selected_folder_is_scanned_with_everything_in_it() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/a.txt", b"12");
    setup.add("Documents/sub/b.txt", b"345");
    setup.add("Pictures/c.jpg", b"6");

    let scan = setup.run();

    assert_eq!(files(&scan), ["Documents/a.txt", "Documents/sub/b.txt"]);
    assert_eq!(folders(&scan), ["Documents", "Documents/sub"]);
    assert_eq!(scan.files_scanned, 2);
    assert_eq!(scan.files_to_copy, 2);
    assert_eq!(scan.bytes_to_copy, 5);
}

#[test]
fn a_selected_file_is_scanned_on_its_own() {
    let setup = Setup::new(Format::Ext4, &["Documents/a.txt"]);
    setup.add("Documents/a.txt", b"12");
    setup.add("Documents/b.txt", b"3");

    assert_eq!(files(&setup.run()), ["Documents/a.txt"]);
}

#[test]
fn an_empty_folder_is_backed_up_too() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup
        .source
        .add_dir_on_device(format!("{HOME}/Documents/empty"), 1);

    assert_eq!(folders(&setup.run()), ["Documents", "Documents/empty"]);
}

#[test]
fn the_whole_home_folder_can_be_selected() {
    let setup = Setup::new(Format::Ext4, &[""]);
    setup.add("a.txt", b"1");

    assert_eq!(files(&setup.run()), ["a.txt"]);
}

#[test]
fn overlapping_selections_count_once() {
    let setup = Setup::new(Format::ExFat, &["Documents/a.txt", "Documents"]);
    setup.add("Documents/a.txt", b"12");

    let scan = setup.run();

    assert_eq!(files(&scan), ["Documents/a.txt"]);
    assert_eq!(scan.bytes_to_copy, 2);
    assert!(scan.skipped.is_empty());
}

#[test]
fn an_unchanged_file_is_listed_but_not_counted_for_copying() {
    let setup = Setup::new(Format::Ext4, &["a.txt"]);
    setup.add("a.txt", b"same");
    setup.add_copy("a.txt", b"same", time(0));

    let scan = setup.run();

    let unchanged = Entry::File {
        relative: PathBuf::from("a.txt"),
        len: 4,
        needs_copy: false,
    };
    assert_eq!(scan.entries, [unchanged]);
    assert_eq!(scan.files_scanned, 1);
    assert_eq!(scan.files_to_copy, 0);
    assert_eq!(scan.bytes_to_copy, 0);
}

#[test]
fn a_changed_file_is_counted_for_copying() {
    let setup = Setup::new(Format::Ext4, &["a.txt"]);
    setup.add("a.txt", b"longer");
    setup.add_copy("a.txt", b"old", time(0));

    assert_eq!(setup.run().bytes_to_copy, 6);
}

#[test]
fn symbolic_links_and_special_files_are_ignored_without_a_report() {
    let setup = Setup::new(Format::Ext4, &["Documents", "link"]);
    setup.add("Documents/a.txt", b"1");
    setup.source.add_other(format!("{HOME}/Documents/link"));
    setup.source.add_other(format!("{HOME}/link"));

    let scan = setup.run();

    assert_eq!(files(&scan), ["Documents/a.txt"]);
    assert!(scan.skipped.is_empty());
}

#[test]
fn a_symbolic_link_with_a_name_exfat_cannot_store_is_still_ignored() {
    let setup = Setup::new(Format::ExFat, &["bad:link", "Documents"]);
    setup.add("Documents/a.txt", b"1");
    setup.source.add_other(format!("{HOME}/bad:link"));
    setup.source.add_other(format!("{HOME}/Documents/bad:link"));

    assert!(setup.run().skipped.is_empty());
}

#[test]
fn folders_on_another_file_system_are_not_entered() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/a.txt", b"1");
    setup
        .source
        .add_dir_on_device(format!("{HOME}/Documents/cloud"), 2);

    let scan = setup.run();

    assert_eq!(folders(&scan), ["Documents"]);
    assert!(scan.skipped.is_empty());
}

#[test]
fn a_selected_folder_may_be_on_another_file_system() {
    let setup = Setup::new(Format::Ext4, &["cloud"]);
    setup.source.add_dir_on_device(format!("{HOME}/cloud"), 2);

    assert_eq!(folders(&setup.run()), ["cloud"]);
}

#[test]
fn excluded_names_are_left_out_without_a_report() {
    let mut setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.exclusions.patterns = vec!["*.tmp".to_owned(), "node_modules".to_owned()];
    setup.add("Documents/a.txt", b"1");
    setup.add("Documents/b.tmp", b"2");
    setup.add("Documents/node_modules/c.js", b"3");

    let scan = setup.run();

    assert_eq!(files(&scan), ["Documents/a.txt"]);
    assert_eq!(folders(&scan), ["Documents"]);
    assert!(scan.skipped.is_empty());
}

#[test]
fn an_excluded_folder_is_left_out_with_everything_in_it() {
    let mut setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.exclusions.folders = vec![PathBuf::from("Documents/old")];
    setup.add("Documents/old/sub/a.txt", b"1");

    assert_eq!(files(&setup.run()), Vec::<String>::new());
}

#[test]
fn a_selection_inside_an_excluded_folder_stays_excluded() {
    let mut setup = Setup::new(Format::Ext4, &[".cache/keep.txt"]);
    setup.exclusions.patterns = vec![".cache".to_owned()];
    setup.add(".cache/keep.txt", b"1");

    let scan = setup.run();

    assert!(scan.entries.is_empty());
    assert!(scan.skipped.is_empty());
}

#[test]
fn a_missing_selection_is_reported() {
    let setup = Setup::new(Format::Ext4, &["gone"]);

    assert_eq!(
        setup.run().skipped,
        [skipped("gone", Skip::SelectionMissing, 0)]
    );
}

#[test]
fn a_selection_outside_the_home_folder_is_refused() {
    for path in ["../other/file", "a/../../b", "/etc/passwd"] {
        let setup = Setup::new(Format::Ext4, &[path]);
        setup.source.add_file("/etc/passwd", b"secret", time(0));

        let scan = setup.run();

        assert_eq!(scan.skipped, [skipped(path, Skip::OutsideHome, 0)]);
        assert!(scan.entries.is_empty());
    }
}

#[test]
fn a_selection_that_cannot_be_read_is_reported() {
    let setup = Setup::new(Format::Ext4, &["secret"]);
    setup.add("secret", b"1");
    setup
        .source
        .fail_path(format!("{HOME}/secret"), ErrorKind::PermissionDenied);

    assert_eq!(
        setup.run().skipped,
        [skipped("secret", Skip::Unreadable, 0)]
    );
}

#[test]
fn a_folder_that_cannot_be_read_is_reported() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/secret/a.txt", b"1");
    setup.add("Documents/b.txt", b"1");
    setup.source.fail_path(
        format!("{HOME}/Documents/secret"),
        ErrorKind::PermissionDenied,
    );

    let scan = setup.run();

    assert_eq!(files(&scan), ["Documents/b.txt"]);
    assert_eq!(scan.skipped.len(), 1);
    assert_eq!(scan.skipped[0].reason, Skip::Unreadable);
}

#[test]
fn a_file_gone_between_listing_and_looking_has_disappeared() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/a.txt", b"1");
    setup.add("Documents/b.txt", b"1");
    setup
        .source
        .fail_path(format!("{HOME}/Documents/a.txt"), ErrorKind::NotFound);

    let scan = setup.run();

    assert_eq!(
        scan.skipped,
        [skipped("Documents/a.txt", Skip::Disappeared, 0)]
    );
    assert_eq!(files(&scan), ["Documents/b.txt"]);
}

#[test]
fn a_folder_gone_before_it_is_read_has_disappeared() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/a.txt", b"1");
    let flag = AtomicBool::new(false);
    let source = CancellingSource(&setup.source, &flag, true);
    let selection = Selection {
        home: PathBuf::from(HOME),
        selected: vec![PathBuf::from("Documents")],
        exclusions: Exclusions::default(),
    };
    let destination = Destination {
        medium: &setup.medium,
        root: PathBuf::from("/medium"),
        account: account_folder("mint", "user"),
        format: setup.format,
    };

    let scan = scan(&source, &selection, &destination, &setup.cancel).unwrap();

    assert_eq!(scan.skipped, [skipped("Documents", Skip::Disappeared, 0)]);
}

#[test]
fn a_name_that_exfat_cannot_store_is_skipped_only_on_exfat() {
    let name = "Documents/what?.txt";
    for (format, expected) in [
        (
            Format::ExFat,
            vec![skipped(name, Skip::NameInvalidChars, 1)],
        ),
        (Format::Ext4, vec![]),
    ] {
        let setup = Setup::new(format, &["Documents"]);
        setup.add(name, b"1");

        assert_eq!(setup.run().skipped, expected);
    }
}

#[test]
fn a_name_that_is_not_utf8_is_skipped_on_exfat() {
    let setup = Setup::new(Format::ExFat, &["Documents"]);
    let name = OsString::from_vec(b"caf\xe9".to_vec());
    let path = Path::new(HOME).join("Documents").join(&name);
    setup.source.add_file(&path, b"1", time(0));

    let scan = setup.run();

    assert_eq!(scan.skipped[0].reason, Skip::NameNotUtf8);
    assert_eq!(scan.entries.len(), 1);
}

#[test]
fn a_skipped_folder_is_one_entry_with_its_number_of_files() {
    let mut setup = Setup::new(Format::ExFat, &["Documents"]);
    setup.exclusions.patterns = vec!["*.tmp".to_owned()];
    setup.add("Documents/bad:folder/a.txt", b"1");
    setup.add("Documents/bad:folder/b.tmp", b"1");
    setup.add("Documents/bad:folder/sub/c.txt", b"1");

    let scan = setup.run();

    assert_eq!(
        scan.skipped,
        [skipped("Documents/bad:folder", Skip::NameInvalidChars, 2)]
    );
    assert!(files(&scan).is_empty());
}

#[test]
fn a_selection_below_a_folder_with_a_bad_name_is_skipped() {
    let setup = Setup::new(Format::ExFat, &["bad:folder/a.txt"]);
    setup.add("bad:folder/a.txt", b"1");

    let scan = setup.run();

    assert_eq!(
        scan.skipped,
        [skipped("bad:folder/a.txt", Skip::NameInvalidChars, 1)]
    );
}

#[test]
fn a_name_in_the_form_of_a_version_folder_is_skipped() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/report[]pdf", b"1");

    assert_eq!(
        setup.run().skipped,
        [skipped("Documents/report[]pdf", Skip::VersionFolderForm, 1)]
    );
}

#[test]
fn on_exfat_the_first_of_two_names_in_another_case_wins() {
    let setup = Setup::new(Format::ExFat, &["Documents"]);
    setup.add("Documents/report.pdf", b"1");
    setup.add("Documents/Report.pdf", b"2");

    let scan = setup.run();

    assert_eq!(files(&scan), ["Documents/Report.pdf"]);
    assert_eq!(
        scan.skipped,
        [skipped("Documents/report.pdf", Skip::CaseClash, 1)]
    );
}

#[test]
fn on_ext4_names_in_another_case_are_all_backed_up() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/report.pdf", b"1");
    setup.add("Documents/Report.pdf", b"2");

    let scan = setup.run();

    assert_eq!(scan.files_scanned, 2);
    assert!(scan.skipped.is_empty());
}

#[test]
fn on_exfat_a_clashing_folder_is_skipped_with_its_files() {
    let setup = Setup::new(Format::ExFat, &["Documents"]);
    setup.add("Documents/Photos/a.jpg", b"1");
    setup.add("Documents/photos/b.jpg", b"1");
    setup.add("Documents/photos/c.jpg", b"1");

    assert_eq!(
        setup.run().skipped,
        [skipped("Documents/photos", Skip::CaseClash, 2)]
    );
}

#[test]
fn on_exfat_separate_selections_can_clash() {
    let setup = Setup::new(Format::ExFat, &["report", "Report"]);
    setup.add("report", b"1");
    setup.add("Report", b"2");

    let scan = setup.run();

    assert_eq!(files(&scan), ["Report"]);
    assert_eq!(scan.skipped, [skipped("report", Skip::CaseClash, 1)]);
}

#[test]
fn on_exfat_a_file_renamed_only_in_case_is_the_same_file() {
    let setup = Setup::new(Format::ExFat, &["Report.pdf"]);
    setup.add("Report.pdf", b"same");
    setup.add_copy("report.pdf", b"same", time(0));

    assert_eq!(setup.run().files_to_copy, 0);
}

#[test]
fn on_ext4_a_file_renamed_only_in_case_is_a_new_file() {
    let setup = Setup::new(Format::Ext4, &["Report.pdf"]);
    setup.add("Report.pdf", b"same");
    setup.add_copy("report.pdf", b"same", time(0));

    assert_eq!(setup.run().files_to_copy, 1);
}

#[test]
fn a_path_too_long_for_the_medium_is_skipped() {
    let long = "a".repeat(200);
    let deep = vec![long; 21].join("/");
    let setup = Setup::new(Format::Ext4, &[""]);
    setup.add(&deep, b"1");

    let scan = setup.run();

    assert_eq!(scan.skipped.len(), 1);
    assert_eq!(scan.skipped[0].reason, Skip::PathTooLong);
    assert_eq!(scan.skipped[0].files, 1);
}

/// A source that sets a flag when a folder is read, and can lose the folder at that moment.
struct CancellingSource<'a>(&'a FakeFs, &'a AtomicBool, bool);

impl FileSystem for CancellingSource<'_> {
    fn metadata(&self, path: &Path) -> std::io::Result<Metadata> {
        self.0.metadata(path)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<OsString>> {
        self.1.store(true, Ordering::Relaxed);
        if self.2 {
            return Err(ErrorKind::NotFound.into());
        }
        self.0.read_dir(path)
    }

    fn open(&self, path: &Path) -> std::io::Result<Box<dyn std::io::Read + Send>> {
        self.0.open(path)
    }

    fn create_new(&self, path: &Path) -> std::io::Result<Box<dyn crate::fs::WriteFile>> {
        self.0.create_new(path)
    }

    fn open_append(&self, path: &Path) -> std::io::Result<Box<dyn crate::fs::WriteFile>> {
        self.0.open_append(path)
    }

    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        self.0.create_dir(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        self.0.rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        self.0.remove_file(path)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> std::io::Result<()> {
        self.0.set_mode(path, mode)
    }

    fn free_space(&self, path: &Path) -> std::io::Result<u64> {
        self.0.free_space(path)
    }

    fn sync_medium(&self, path: &Path) -> std::io::Result<()> {
        self.0.sync_medium(path)
    }
}

#[test]
fn cancelling_stops_the_scan_part_way_through_a_folder() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/a.txt", b"1");
    let source = CancellingSource(&setup.source, &setup.cancel, false);
    let selection = Selection {
        home: PathBuf::from(HOME),
        selected: vec![PathBuf::from("Documents")],
        exclusions: Exclusions::default(),
    };
    let destination = Destination {
        medium: &setup.medium,
        root: PathBuf::from("/medium"),
        account: account_folder("mint", "user"),
        format: setup.format,
    };

    assert!(scan(&source, &selection, &destination, &setup.cancel).is_none());
}

#[test]
fn cancelling_stops_the_scan() {
    let setup = Setup::new(Format::Ext4, &["Documents"]);
    setup.add("Documents/a.txt", b"1");
    setup.cancel.store(true, Ordering::Relaxed);
    let selection = Selection {
        home: PathBuf::from(HOME),
        selected: vec![PathBuf::from("Documents")],
        exclusions: Exclusions::default(),
    };
    let destination = Destination {
        medium: &setup.medium,
        root: PathBuf::from("/medium"),
        account: account_folder("mint", "user"),
        format: setup.format,
    };

    assert!(scan(&setup.source, &selection, &destination, &setup.cancel).is_none());
}
