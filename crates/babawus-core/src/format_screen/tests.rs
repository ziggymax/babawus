use std::path::PathBuf;

use super::*;
use crate::media::{Block, Drive};

fn en() -> Language {
    crate::language::testing::english()
}
const STICK: &str = "/org/freedesktop/UDisks2/drives/Kingston";
const STICK_ID: &str = "Kingston-DataTraveler-3.0-08606E694BE4E320B7282A6C";
const OTHER: &str = "/org/freedesktop/UDisks2/drives/SanDisk";
const OTHER_ID: &str = "SanDisk-Cruzer-1234";
const SDB: &str = "/org/freedesktop/UDisks2/block_devices/sdb";
const SDB1: &str = "/org/freedesktop/UDisks2/block_devices/sdb1";
const SDB2: &str = "/org/freedesktop/UDisks2/block_devices/sdb2";
const SDC: &str = "/org/freedesktop/UDisks2/block_devices/sdc";
const SDC1: &str = "/org/freedesktop/UDisks2/block_devices/sdc1";

fn drive(object_path: &str, id: &str, vendor: &str, model: &str) -> Drive {
    Drive {
        object_path: object_path.to_owned(),
        id: id.to_owned(),
        vendor: vendor.to_owned(),
        model: model.to_owned(),
        size: 15_502_147_584,
        removable: true,
        connection_bus: "usb".to_owned(),
        can_power_off: true,
    }
}

fn block(path: &str, drive: &str, partition_of: Option<&str>) -> Block {
    Block {
        object_path: path.to_owned(),
        drive: drive.to_owned(),
        partition_of: partition_of.map(str::to_owned),
        ..Block::default()
    }
}

fn volume(path: &str, drive: &str, label: &str, file_system: &str, mounted: bool) -> Volume {
    Volume {
        object_path: path.to_owned(),
        drive: drive.to_owned(),
        label: label.to_owned(),
        file_system: file_system.to_owned(),
        mount_point: mounted.then(|| PathBuf::from(format!("/media/anna/{label}"))),
        ..Volume::default()
    }
}

/// A Kingston stick with a Babawus medium and an unmounted NTFS partition, and a SanDisk stick
/// with nothing on it.
fn snapshot() -> Snapshot {
    Snapshot {
        drives: vec![
            drive(STICK, STICK_ID, "Kingston", "DataTraveler 3.0"),
            drive(OTHER, OTHER_ID, "", " "),
        ],
        volumes: vec![
            volume(SDB1, STICK, "BACKUP-A", "exfat", true),
            volume(SDB2, STICK, "DATA", "ntfs", false),
        ],
        blocks: vec![
            block(SDB, STICK, None),
            block(SDB1, STICK, Some(SDB)),
            block(SDB2, STICK, Some(SDB)),
            block(SDC, OTHER, None),
        ],
    }
}

fn babawus() -> HashSet<String> {
    HashSet::from([SDB1.to_owned()])
}

#[test]
fn each_drive_says_what_is_on_it() {
    let choices = drive_choices(&snapshot(), &babawus(), en());

    assert_eq!(
        choices,
        [
            DriveChoice {
                id: STICK_ID.to_owned(),
                name: "Kingston DataTraveler 3.0 · 15.5 GB".to_owned(),
                contents: "BACKUP-A (exFAT), with Babawus backups; DATA (NTFS), not mounted"
                    .to_owned(),
            },
            DriveChoice {
                id: OTHER_ID.to_owned(),
                name: "15.5 GB".to_owned(),
                contents: "No file system".to_owned(),
            },
        ]
    );
}

#[test]
fn a_volume_with_no_label_says_so() {
    let mut snapshot = snapshot();
    snapshot.volumes.push(volume(SDC1, OTHER, "", "vfat", true));
    snapshot.blocks.push(block(SDC1, OTHER, Some(SDC)));

    let choices = drive_choices(&snapshot, &babawus(), en());

    assert_eq!(choices[1].contents, "No name (FAT)");
}

#[test]
fn the_warning_says_what_is_erased_and_that_it_cannot_be_stopped() {
    let choices = drive_choices(&snapshot(), &babawus(), en());

    let text = warning(&choices[0], en());

    assert!(
        text.contains("Kingston DataTraveler 3.0 · 15.5 GB"),
        "{text}"
    );
    assert!(text.contains("with Babawus backups"), "{text}");
    assert!(text.contains("every partition"), "{text}");
    assert!(text.contains("can't be stopped"), "{text}");
}

#[test]
fn the_suggested_label_ignores_the_drives_own_labels() {
    assert_eq!(suggested_label(&snapshot(), STICK_ID), "BACKUP-A");
    assert_eq!(suggested_label(&snapshot(), OTHER_ID), "BACKUP-B");
}

#[test]
fn a_label_problem_names_the_rule() {
    assert_eq!(label_problem("BACKUP-A", en()), None);
    assert!(label_problem("", en()).is_some());
    assert!(label_problem("MY BACKUP", en()).is_some());
    assert!(label_problem("BACKUP-12345", en()).is_some());
}

#[test]
fn formatting_needs_a_drive_a_valid_label_and_the_checkbox() {
    assert!(can_format(true, "BACKUP-A", true));
    assert!(!can_format(false, "BACKUP-A", true));
    assert!(!can_format(true, "", true));
    assert!(!can_format(true, "BACKUP-A", false));
}

#[test]
fn success_says_to_set_the_stick_up() {
    let shown = result_screen("BACKUP-B", None, en());

    assert!(shown.succeeded);
    assert_eq!(shown.title, "BACKUP-B is ready");
    assert!(shown.detail.contains("each computer and account"));
}

#[test]
fn a_failure_before_erasing_says_nothing_was_changed() {
    let failure = Failure::of(&FormatError::Failed {
        step: Step::Unmount,
        message: "Device is busy".to_owned(),
    });

    let shown = result_screen("BACKUP-B", Some(&failure), en());

    assert!(!shown.succeeded);
    assert!(shown.detail.ends_with("Nothing on the stick was changed."));
    assert_eq!(
        shown.message.as_deref(),
        Some("The system said: Device is busy")
    );
}

#[test]
fn a_failure_after_erasing_says_to_format_again() {
    let failure = Failure::of(&FormatError::Failed {
        step: Step::Format,
        message: "mkfs failed".to_owned(),
    });

    let shown = result_screen("BACKUP-B", Some(&failure), en());

    assert!(shown.detail.contains("Format it again"), "{}", shown.detail);
}

#[test]
fn a_removed_stick_was_not_changed() {
    let failure = Failure::of(&FormatError::Refused(Refusal::Gone));

    let shown = result_screen("BACKUP-B", Some(&failure), en());

    assert_eq!(
        shown.detail,
        "The stick was removed. Nothing on the stick was changed."
    );
    assert_eq!(shown.message, None);
}

#[test]
fn a_failure_while_setting_up_says_the_stick_is_not_ready() {
    let failure = Failure::of(&FormatError::Failed {
        step: Step::Prepare,
        message: "Permission denied".to_owned(),
    });

    let shown = result_screen("BACKUP-B", Some(&failure), en());

    assert!(shown.detail.starts_with("The stick was formatted, but"));
}

#[test]
fn only_an_administrator_can_choose_ext4() {
    assert!(ext4_allowed(true));
    assert!(!ext4_allowed(false));
}

#[test]
fn anyone_else_is_told_why_ext4_cannot_be_chosen() {
    let note = ext4_note(false, false, en()).unwrap();
    assert!(note.starts_with("Only an administrator can choose Ext4"));
    assert_eq!(ext4_note(false, true, en()), Some(note));
}

#[test]
fn an_administrator_who_chooses_ext4_is_told_about_the_password() {
    assert_eq!(ext4_note(true, false, en()), None);
    assert_eq!(
        ext4_note(true, true, en()),
        Some("Ext4 asks for your password while formatting.")
    );
}
