//! The scan and the copy together, on real exFAT and Ext4.
//!
//! These use sudo and loop devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use babawus_core::copy::{FileCopier, Skip, Visit};
use babawus_core::exclude::Exclusions;
use babawus_core::fs::{FileSystem, RealFs};
use babawus_core::scan::{Destination, Entry, Scan, Selection, account_folder, scan};
use babawus_core::versions::Format;
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;

struct Test {
    medium: LoopMedium,
    format: Format,
    home: PathBuf,
    clock: Mutex<String>,
}

impl Test {
    fn new(format: Format) -> Self {
        let medium_format = match format {
            Format::ExFat => MediumFormat::ExFat,
            Format::Ext4 => MediumFormat::Ext4,
        };
        let medium = LoopMedium::new(medium_format, 32 * MIB);
        fs::create_dir(medium.path().join(".babawus")).unwrap();
        let home = medium.path().with_file_name("home");
        fs::create_dir(&home).unwrap();
        Self {
            medium,
            format,
            home,
            clock: Mutex::new("2026-11-03_175400".to_owned()),
        }
    }

    fn add(&self, relative: &str, data: &[u8]) {
        let path = self.home.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
    }

    /// Scans and backs up everything, as a run does.
    fn back_up(&self) -> (Scan, Vec<Visit>) {
        let account = account_folder("mint", "user");
        let selection = Selection {
            home: self.home.clone(),
            selected: vec![PathBuf::new()],
            exclusions: Exclusions::default(),
        };
        let destination = Destination {
            medium: &RealFs,
            root: self.medium.path().to_owned(),
            account: account.clone(),
            format: self.format,
        };
        let cancel = AtomicBool::new(false);
        let found = scan(&RealFs, &selection, &destination, &cancel).unwrap();

        let now = || self.clock.lock().unwrap().clone();
        let root = self.medium.path().to_owned();
        let mut copier = FileCopier::new(&RealFs, &RealFs, root, self.format, 1, &now, &cancel);
        copier.empty_temp_folder().unwrap();
        let mut visits = Vec::new();
        for entry in &found.entries {
            match entry {
                Entry::Folder { relative } => {
                    assert_eq!(copier.visit_folder(&account.join(relative)), Ok(None));
                }
                Entry::File { relative, .. } => {
                    let source = self.home.join(relative);
                    let target = account.join(relative);
                    visits.push(copier.visit(&source, &target, &mut |_| {}).unwrap());
                }
            }
        }
        copier.flush_medium().unwrap();
        (found, visits)
    }

    fn on_medium(&self, relative: &str) -> PathBuf {
        self.medium.path().join("backup/mint/user").join(relative)
    }

    fn names_on_medium(&self, relative: &str) -> Vec<String> {
        let entries = RealFs
            .read_dir(&self.on_medium(relative))
            .unwrap_or_default();
        entries
            .iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect()
    }
}

fn copied(visits: &[Visit]) -> usize {
    visits
        .iter()
        .filter(|visit| matches!(visit, Visit::Copied(_)))
        .count()
}

fn backs_up_a_tree_and_finds_nothing_to_do_the_next_time(format: Format) {
    let test = Test::new(format);
    test.add("Documents/a.txt", b"a");
    test.add("Documents/sub/b.txt", b"b");
    fs::create_dir(test.home.join("Documents/empty")).unwrap();

    let (first, visits) = test.back_up();

    assert_eq!(first.files_to_copy, 2);
    assert_eq!(copied(&visits), 2);
    assert_eq!(
        fs::read(test.on_medium("Documents/sub/b.txt")).unwrap(),
        b"b"
    );
    assert!(test.on_medium("Documents/empty").is_dir());

    let (second, visits) = test.back_up();

    assert_eq!(second.files_to_copy, 0);
    assert_eq!(copied(&visits), 0);
}

fn does_not_follow_symbolic_links(format: Format) {
    let test = Test::new(format);
    test.add("Documents/a.txt", b"a");
    std::os::unix::fs::symlink(&test.home, test.home.join("Documents/loop")).unwrap();

    let (found, _) = test.back_up();

    assert_eq!(found.files_scanned, 1);
    assert!(found.skipped.is_empty());
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_backs_up_a_tree() {
    backs_up_a_tree_and_finds_nothing_to_do_the_next_time(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_backs_up_a_tree() {
    backs_up_a_tree_and_finds_nothing_to_do_the_next_time(Format::Ext4);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_ignores_symbolic_links() {
    does_not_follow_symbolic_links(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_ignores_symbolic_links() {
    does_not_follow_symbolic_links(Format::Ext4);
}

/// Every name the rules let through must arrive on exFAT exactly as it is, or a backup could
/// silently go to another name.
#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_stores_every_name_the_rules_accept_exactly() {
    let test = Test::new(Format::ExFat);
    let names = [
        "plain",
        "a b",
        " lead",
        "trail ",
        "trail  ",
        "..x",
        "dot. ",
        "[x]",
        "a]b[",
        "del\u{7f}x",
        "c1\u{85}x",
        "nbsp\u{a0}",
        "😀",
        "e\u{301}",
        "\u{e9}",
        "#%&'()+,;=@^`{}~!$",
    ];
    for name in names {
        test.add(&format!("Documents/{name}"), b"x");
    }

    let (found, _) = test.back_up();

    assert!(found.skipped.is_empty(), "{:?}", found.skipped);
    let mut expected: Vec<&str> = names.to_vec();
    expected.sort();
    assert_eq!(test.names_on_medium("Documents"), expected);
}

/// Every name refused on exFAT really can't be stored as it is.
#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_really_cannot_store_the_names_the_rules_refuse() {
    let test = Test::new(Format::ExFat);
    let folder = test.medium.path().join("refused");
    fs::create_dir(&folder).unwrap();
    for name in [
        "a\"b", "a*b", "a:b", "a<b", "a>b", "a?b", "a\\b", "a|b", "a\tb", "name.", "x .",
    ] {
        let path = folder.join(name);
        let stored = fs::write(&path, b"x").is_ok() && RealFs.read_dir(&folder).unwrap() == [name];
        assert!(!stored, "{name:?}");
        for entry in RealFs.read_dir(&folder).unwrap() {
            fs::remove_file(folder.join(entry)).unwrap();
        }
    }
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_skips_the_second_of_two_names_in_another_case() {
    let test = Test::new(Format::ExFat);
    test.add("Documents/Report.pdf", b"first");
    test.add("Documents/report.pdf", b"second");

    let (found, _) = test.back_up();

    assert_eq!(found.skipped.len(), 1);
    assert_eq!(found.skipped[0].reason, Skip::CaseClash);
    assert_eq!(test.names_on_medium("Documents"), ["Report.pdf"]);
    assert_eq!(
        fs::read(test.on_medium("Documents/Report.pdf")).unwrap(),
        b"first"
    );
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_treats_a_file_renamed_only_in_case_as_the_same_file() {
    let test = Test::new(Format::ExFat);
    test.add("report.pdf", b"one");
    test.back_up();

    fs::rename(test.home.join("report.pdf"), test.home.join("Report.pdf")).unwrap();
    let source = test.home.join("Report.pdf");
    let original = RealFs
        .metadata(&test.on_medium("report.pdf"))
        .unwrap()
        .modified;
    fs::File::options()
        .write(true)
        .open(&source)
        .unwrap()
        .set_modified(original)
        .unwrap();
    let (found, visits) = test.back_up();

    assert_eq!(found.files_to_copy, 0);
    assert_eq!(copied(&visits), 0);
    assert_eq!(test.names_on_medium(""), ["report.pdf"]);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_takes_the_new_case_with_the_next_change_and_keeps_pruning() {
    let test = Test::new(Format::ExFat);
    test.add("report.pdf", b"one");
    test.back_up();
    test.add("report.pdf", b"two!");
    *test.clock.lock().unwrap() = "2026-11-04_080000".to_owned();
    test.back_up();
    assert_eq!(
        test.names_on_medium("report[]pdf"),
        ["report[2026-11-04_080000].pdf"]
    );

    fs::remove_file(test.home.join("report.pdf")).unwrap();
    test.add("Report.pdf", b"three");
    *test.clock.lock().unwrap() = "2026-11-05_080000".to_owned();
    test.back_up();

    // The newest copy takes the new case. The version folder keeps the old one, and with a limit
    // of 1, the version in the old case goes.
    assert_eq!(test.names_on_medium(""), ["Report.pdf", "report[]pdf"]);
    assert_eq!(
        test.names_on_medium("report[]pdf"),
        ["Report[2026-11-05_080000].pdf"]
    );
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_backs_up_both_names_in_another_case() {
    let test = Test::new(Format::Ext4);
    test.add("Documents/Report.pdf", b"first");
    test.add("Documents/report.pdf", b"second");

    let (found, _) = test.back_up();

    assert!(found.skipped.is_empty());
    assert_eq!(
        test.names_on_medium("Documents"),
        ["Report.pdf", "report.pdf"]
    );
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_keeps_a_name_that_is_not_utf8() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let test = Test::new(Format::Ext4);
    let name = OsStr::from_bytes(b"caf\xe9.txt");
    fs::write(test.home.join(name), b"x").unwrap();

    let (found, _) = test.back_up();

    assert!(found.skipped.is_empty());
    assert!(Path::new(&test.on_medium("")).join(name).is_file());
}
