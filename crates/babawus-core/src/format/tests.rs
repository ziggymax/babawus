use std::io::ErrorKind;
use std::time::SystemTime;

use super::*;
use crate::config::Loaded;
use crate::fs::fake::FakeFs;
use crate::media::Block;
use crate::medium::CONFIG_FILE;

const STICK: &str = "/org/freedesktop/UDisks2/drives/Kingston";
const STICK_ID: &str = "Kingston-DataTraveler-3.0-08606E694BE4E320B7282A6C";
const DISK: &str = "/org/freedesktop/UDisks2/drives/VBOX_HARDDISK";
const DISK_ID: &str = "VBOX-HARDDISK-VBb559a512";
const SDB: &str = "/org/freedesktop/UDisks2/block_devices/sdb";
const SDB1: &str = "/org/freedesktop/UDisks2/block_devices/sdb1";
const SDB2: &str = "/org/freedesktop/UDisks2/block_devices/sdb2";
const GIB: u64 = 1024 * 1024 * 1024;
const TIB: u64 = 1024 * GIB;

fn stick() -> Drive {
    Drive {
        object_path: STICK.to_owned(),
        id: STICK_ID.to_owned(),
        size: 16 * GIB,
        removable: true,
        connection_bus: "usb".to_owned(),
        ..Drive::default()
    }
}

fn block(path: &str, drive: &str, partition_of: Option<&str>) -> Block {
    Block {
        object_path: path.to_owned(),
        drive: drive.to_owned(),
        partition_of: partition_of.map(str::to_owned),
        size: 16 * GIB,
        ..Block::default()
    }
}

/// The stick, with two partitions, of which the first is mounted where the desktop mounts it.
fn snapshot() -> Snapshot {
    let mut mounted = block(SDB1, STICK, Some(SDB));
    mounted.mount_points = vec![PathBuf::from("/media/usera/BACKUP-A")];
    Snapshot {
        drives: vec![
            stick(),
            Drive {
                object_path: DISK.to_owned(),
                id: DISK_ID.to_owned(),
                ..Drive::default()
            },
        ],
        volumes: Vec::new(),
        blocks: vec![
            block("/org/freedesktop/UDisks2/block_devices/sda", DISK, None),
            block(SDB, STICK, None),
            mounted,
            block(SDB2, STICK, Some(SDB)),
        ],
    }
}

fn changed(change: impl FnOnce(&mut Snapshot)) -> Snapshot {
    let mut snapshot = snapshot();
    change(&mut snapshot);
    snapshot
}

fn block_mut<'a>(snapshot: &'a mut Snapshot, path: &str) -> &'a mut Block {
    snapshot
        .blocks
        .iter_mut()
        .find(|block| block.object_path == path)
        .unwrap()
}

#[test]
fn a_usb_stick_can_be_formatted_and_its_mounted_file_systems_are_unmounted_first() {
    let target = snapshot().target(STICK_ID).unwrap();

    assert_eq!(target.drive, stick());
    assert_eq!(target.disk, SDB);
    assert_eq!(target.table, Table::Mbr);
    assert_eq!(target.mounted, [SDB1]);
    assert_eq!(target.partitions, [SDB1, SDB2]);
}

#[test]
fn only_usb_media_are_offered() {
    let snapshot = snapshot();

    let offered: Vec<_> = snapshot
        .formattable()
        .into_iter()
        .map(|target| target.drive.id)
        .collect();

    assert_eq!(offered, [STICK_ID]);
    assert_eq!(snapshot.target(DISK_ID), Err(Refusal::NotUsbMedium));
}

#[test]
fn a_drive_that_has_gone_is_refused() {
    assert_eq!(snapshot().target("Some-Other-Stick"), Err(Refusal::Gone));
    let gone = changed(|snapshot| snapshot.drives.retain(|drive| drive.id != STICK_ID));
    assert_eq!(gone.target(STICK_ID), Err(Refusal::Gone));
    // A drive without an ID can't be checked again, so it is never offered.
    let no_id = changed(|snapshot| snapshot.drives[0].id.clear());
    assert_eq!(no_id.target(""), Err(Refusal::Gone));
    assert!(no_id.formattable().is_empty());
}

#[test]
fn a_drive_that_is_no_longer_a_usb_medium_is_refused() {
    let snapshot = changed(|snapshot| snapshot.drives[0].removable = false);
    assert_eq!(snapshot.target(STICK_ID), Err(Refusal::NotUsbMedium));
}

#[test]
fn a_drive_whose_disk_is_not_reported_is_refused() {
    let snapshot = changed(|snapshot| snapshot.blocks.retain(|block| block.object_path != SDB));
    assert_eq!(snapshot.target(STICK_ID), Err(Refusal::NoDisk));
}

#[test]
fn a_drive_that_belongs_to_the_system_is_refused() {
    let snapshot = changed(|snapshot| block_mut(snapshot, SDB2).hint_system = true);
    assert_eq!(snapshot.target(STICK_ID), Err(Refusal::System));
    assert!(snapshot.formattable().is_empty());
}

#[test]
fn a_drive_with_a_partition_in_use_is_refused() {
    for place in ["/", "/home", "/mnt/stick", "/media", "/srv/media/usera/X"] {
        let snapshot = changed(|snapshot| {
            block_mut(snapshot, SDB2).mount_points = vec![PathBuf::from(place)];
        });
        assert_eq!(snapshot.target(STICK_ID), Err(Refusal::InUse), "{place}");
    }
    let snapshot = changed(|snapshot| block_mut(snapshot, SDB2).held = true);
    assert_eq!(snapshot.target(STICK_ID), Err(Refusal::InUse));
    let snapshot = changed(|snapshot| block_mut(snapshot, SDB).held = true);
    assert_eq!(snapshot.target(STICK_ID), Err(Refusal::InUse));
}

#[test]
fn file_systems_mounted_by_the_desktop_do_not_count_as_in_use() {
    let snapshot = changed(|snapshot| {
        block_mut(snapshot, SDB2).mount_points = vec![
            PathBuf::from("/run/media/usera/SPARE"),
            PathBuf::from("/media/userb/SPARE"),
        ];
    });

    let target = snapshot.target(STICK_ID).unwrap();

    assert_eq!(target.mounted, [SDB1, SDB2]);
}

#[test]
fn a_device_over_2_tib_gets_gpt() {
    assert_eq!(Table::for_size(16 * GIB), Table::Mbr);
    assert_eq!(Table::for_size(2 * TIB), Table::Mbr);
    assert_eq!(Table::for_size(2 * TIB + 1), Table::Gpt);
    let large = changed(|snapshot| block_mut(snapshot, SDB).size = 4 * TIB);
    assert_eq!(large.target(STICK_ID).unwrap().table, Table::Gpt);
}

#[test]
fn partition_types_match_the_file_system() {
    assert_eq!(Table::Mbr.partition_type(Format::ExFat), "0x07");
    assert_eq!(Table::Mbr.partition_type(Format::Ext4), "0x83");
    assert_eq!(
        Table::Gpt.partition_type(Format::ExFat),
        "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7"
    );
    assert_eq!(
        Table::Gpt.partition_type(Format::Ext4),
        "0fc63daf-8483-4772-8e79-3d69d8477de4"
    );
}

#[test]
fn labels_have_up_to_11_letters_digits_and_dashes() {
    for good in ["BACKUP-A", "a", "Backup-2026", "12345678901", "-"] {
        assert!(label_is_valid(good), "{good}");
    }
    for bad in [
        "",
        "123456789012",
        "BACKUP A",
        "BACKUP_A",
        "BÄCKUP",
        "BACKUP/A",
        "BACKUP.A",
    ] {
        assert!(!label_is_valid(bad), "{bad}");
    }
}

#[test]
fn the_suggested_label_is_the_first_one_not_in_use() {
    assert_eq!(suggest_label([]), "BACKUP-A");
    assert_eq!(suggest_label(["BACKUP-A", "Photos"]), "BACKUP-B");
    assert_eq!(suggest_label(["backup-a", "BACKUP-C"]), "BACKUP-B");
    let all: Vec<String> = ('A'..='Z')
        .map(|letter| format!("BACKUP-{letter}"))
        .collect();
    let mut in_use: Vec<&str> = all.iter().map(String::as_str).collect();
    assert_eq!(suggest_label(in_use.iter().copied()), "BACKUP-1");
    in_use.push("BACKUP-1");
    assert_eq!(suggest_label(in_use), "BACKUP-2");
    assert!(label_is_valid(&suggest_label([])));
}

fn root() -> PathBuf {
    PathBuf::from("/media/usera/BACKUP-A")
}

fn new_medium() -> FakeFs {
    let medium = FakeFs::new();
    medium.add_dir_on_device(root(), 0);
    medium.set_mode(&root(), 0o700).unwrap();
    medium
}

#[test]
fn a_prepared_ext4_medium_is_open_to_everyone_and_has_a_new_configuration() {
    let medium = new_medium();

    prepare(&medium, &root(), Format::Ext4).unwrap();

    for path in [root(), root().join(".babawus"), root().join("backup")] {
        assert_eq!(medium.metadata(&path).unwrap().mode, DIR_MODE, "{path:?}");
    }
    assert_eq!(
        config::load(&medium, &root()).unwrap(),
        Loaded::Ready(Config::new())
    );
}

#[test]
fn a_prepared_exfat_medium_has_the_folders_and_a_new_configuration() {
    let medium = FakeFs::exfat();
    medium.add_dir_on_device(root(), 0);

    prepare(&medium, &root(), Format::ExFat).unwrap();

    assert!(medium.metadata(&root().join("backup")).is_ok());
    assert_eq!(
        config::load(&medium, &root()).unwrap(),
        Loaded::Ready(Config::new())
    );
}

#[test]
fn preparing_reports_a_failure() {
    let medium = new_medium();
    medium.add_file(root().join("backup"), b"", SystemTime::UNIX_EPOCH);

    let error = prepare(&medium, &root(), Format::Ext4).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert!(medium.metadata(&root().join(CONFIG_FILE)).is_err());
}
