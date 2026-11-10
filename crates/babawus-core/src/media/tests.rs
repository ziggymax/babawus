use std::path::Path;
use std::time::{Duration, SystemTime};

use super::*;
use crate::config::{Config, save};
use crate::fs::fake::FakeFs;
use crate::lock::Holder;
use crate::log::{RunLog, Stats, new_run_id};
use crate::medium::{CONFIG_FILE, LOCK_FILE, LOG_FILE};
use crate::time::log_timestamp;
use crate::verdict::Reason;

const ID: &str = "0123456789abcdef0123456789abcdef";
const BOOT: &str = "11111111-2222-3333-4444-555555555555";
const OWN_PROCESS: u32 = 1000;
const OTHER_PROCESS: u32 = 2000;
const STICK: &str = "/org/freedesktop/UDisks2/drives/Kingston_DataTraveler";
const DISK: &str = "/org/freedesktop/UDisks2/drives/VBOX_HARDDISK";
const ROOT: &str = "/media/usera/BACKUP-A";

fn stick() -> Drive {
    Drive {
        object_path: STICK.to_owned(),
        id: "Kingston-DataTraveler-3.0-08606E694BE4E320B7282A6C".to_owned(),
        vendor: "Kingston".to_owned(),
        model: "DataTraveler 3.0".to_owned(),
        size: 15_502_147_584,
        removable: true,
        connection_bus: "usb".to_owned(),
        can_power_off: true,
    }
}

fn disk() -> Drive {
    Drive {
        object_path: DISK.to_owned(),
        id: "VBOX-HARDDISK-VBb559a512-81cb64cb".to_owned(),
        model: "VBOX HARDDISK".to_owned(),
        size: 26_843_545_600,
        ..Drive::default()
    }
}

fn volume(name: &str, drive: &str, label: &str, mount_point: Option<&str>) -> Volume {
    Volume {
        object_path: format!("/org/freedesktop/UDisks2/block_devices/{name}"),
        drive: drive.to_owned(),
        label: label.to_owned(),
        file_system: "exfat".to_owned(),
        mount_point: mount_point.map(PathBuf::from),
        read_only: false,
        size: 15_500_000_000,
    }
}

fn labels(snapshot: &Snapshot) -> Vec<String> {
    let media = snapshot.media();
    media
        .into_iter()
        .map(|medium| medium.volume.label)
        .collect()
}

#[test]
fn a_usb_medium_is_removable_and_on_usb() {
    assert!(stick().is_usb_medium());
    let not_removable = Drive {
        removable: false,
        ..stick()
    };
    assert!(!not_removable.is_usb_medium());
    // A card reader inside the computer, or a CD drive.
    let not_usb = Drive {
        connection_bus: "sdio".to_owned(),
        ..stick()
    };
    assert!(!not_usb.is_usb_medium());
}

#[test]
fn the_list_has_the_mounted_file_systems_on_usb_media() {
    let snapshot = Snapshot {
        drives: vec![disk(), stick()],
        volumes: vec![
            volume("sda3", DISK, "", Some("/")),
            volume("sdb1", STICK, "BACKUP-A", Some(ROOT)),
            volume("sdb2", STICK, "SPARE", None),
            volume("loop0", "/", "IMAGE", Some("/media/usera/IMAGE")),
        ],
        blocks: Vec::new(),
    };

    let media = snapshot.media();

    assert_eq!(media.len(), 1);
    assert_eq!(media[0].volume.label, "BACKUP-A");
    assert_eq!(media[0].drive, stick());
    assert_eq!(media[0].root, Path::new(ROOT));
}

#[test]
fn each_file_system_on_a_medium_has_a_row_of_its_own_sorted_by_label() {
    let snapshot = Snapshot {
        drives: vec![stick()],
        volumes: vec![
            volume("sdb2", STICK, "BACKUP-B", Some("/media/usera/BACKUP-B")),
            volume("sdb1", STICK, "BACKUP-A", Some(ROOT)),
        ],
        blocks: Vec::new(),
    };

    assert_eq!(labels(&snapshot), ["BACKUP-A", "BACKUP-B"]);
}

#[test]
fn a_volume_whose_drive_is_not_reported_yet_is_left_out() {
    let snapshot = Snapshot {
        drives: Vec::new(),
        volumes: vec![volume("sdb1", STICK, "BACKUP-A", Some(ROOT))],
        blocks: Vec::new(),
    };

    assert!(snapshot.media().is_empty());
}

fn block(name: &str, drive: &str, mount_point: Option<&str>) -> Block {
    Block {
        object_path: format!("/org/freedesktop/UDisks2/block_devices/{name}"),
        drive: drive.to_owned(),
        mount_points: mount_point.map(PathBuf::from).into_iter().collect(),
        ..Block::default()
    }
}

fn eject_snapshot() -> Snapshot {
    Snapshot {
        drives: vec![disk(), stick()],
        volumes: Vec::new(),
        blocks: vec![
            block("sda3", DISK, Some("/")),
            block("sdb", STICK, None),
            block("sdb1", STICK, Some(ROOT)),
            block("sdb2", STICK, None),
            block("sdb3", STICK, Some("/media/usera/SPARE")),
        ],
    }
}

#[test]
fn ejecting_unmounts_every_mounted_file_system_on_the_drive() {
    let plan = eject_snapshot().eject_plan(&stick().id).unwrap();

    assert_eq!(plan.drive, stick());
    assert_eq!(
        plan.mounted,
        [
            "/org/freedesktop/UDisks2/block_devices/sdb1",
            "/org/freedesktop/UDisks2/block_devices/sdb3"
        ]
    );
}

#[test]
fn only_a_usb_medium_that_is_still_there_is_ejected() {
    let snapshot = eject_snapshot();
    assert_eq!(snapshot.eject_plan(&disk().id), None);
    assert_eq!(snapshot.eject_plan("Some-Other-Stick"), None);
    assert_eq!(snapshot.eject_plan(""), None);
}

#[test]
fn a_medium_that_is_not_mounted_can_still_be_powered_off() {
    let mut snapshot = eject_snapshot();
    for block in &mut snapshot.blocks {
        block.mount_points.clear();
    }

    let plan = snapshot.eject_plan(&stick().id).unwrap();

    assert!(plan.mounted.is_empty());
}

#[test]
fn only_exfat_and_ext4_are_formats_babawus_uses() {
    let with = |file_system: &str| Volume {
        file_system: file_system.to_owned(),
        ..Volume::default()
    };
    assert_eq!(with("exfat").format(), Some(Format::ExFat));
    assert_eq!(with("ext4").format(), Some(Format::Ext4));
    for other in ["vfat", "ntfs", "ext3", "btrfs", ""] {
        assert_eq!(with(other).format(), None, "{other}");
    }
}

fn this() -> ThisProcess {
    ThisProcess {
        computer_id: ID.to_owned(),
        boot_id: BOOT.to_owned(),
        process_id: OWN_PROCESS,
        program: "babawus".to_owned(),
    }
}

fn root() -> PathBuf {
    PathBuf::from(ROOT)
}

fn medium(file_system: &str) -> Medium {
    Medium {
        drive: stick(),
        volume: Volume {
            file_system: file_system.to_owned(),
            ..volume("sdb1", STICK, "BACKUP-A", Some(ROOT))
        },
        root: root(),
    }
}

fn config() -> Config {
    let mut config = Config::new();
    config.add_computer(ID, "pc", "mint", &[]).unwrap();
    config.add_account(ID, "usera").unwrap();
    config
}

struct Setup {
    files: FakeFs,
    system: FakeFs,
    this: ThisProcess,
    medium: Medium,
}

impl Setup {
    /// A Babawus medium with this computer and `usera` added.
    fn new() -> Self {
        let setup = Self::blank();
        setup.files.create_dir(&root().join(".babawus")).unwrap();
        save(&setup.files, &root(), Format::ExFat, &config()).unwrap();
        setup
    }

    fn blank() -> Self {
        let files = FakeFs::exfat();
        files.add_file(root().join("holiday.jpg"), b"", SystemTime::UNIX_EPOCH);
        let system = FakeFs::new();
        let comm = format!("/proc/{OTHER_PROCESS}/comm");
        system.add_file(comm, b"babawus\n", SystemTime::UNIX_EPOCH);
        Self {
            files,
            system,
            this: this(),
            medium: medium("exfat"),
        }
    }

    fn status_for(&self, username: &str) -> Status {
        let viewer = Viewer {
            system: &self.system,
            this: &self.this,
            username,
        };
        viewer.status(&self.files, &self.medium)
    }

    fn status(&self) -> Status {
        self.status_for("usera")
    }

    fn write(&self, relative: &str, data: &[u8]) {
        self.files
            .add_file(root().join(relative), data, SystemTime::UNIX_EPOCH);
    }

    fn lock(&self, process_id: u32) {
        let holder = Holder {
            run_id: new_run_id(),
            account: "userb@pc".to_owned(),
            computer_id: ID.to_owned(),
            boot_id: BOOT.to_owned(),
            process_id,
        };
        self.write(LOCK_FILE, toml::to_string(&holder).unwrap().as_bytes());
    }

    fn log_run(&self, account: &str, reason: Reason, end: SystemTime) {
        let now = move || log_timestamp(end);
        let root = root();
        let log = RunLog::start(
            &self.files,
            &root,
            Format::ExFat,
            new_run_id(),
            account,
            &now,
            None,
        )
        .unwrap();
        log.end(reason, Stats::default()).unwrap();
    }
}

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

#[test]
fn an_added_account_is_ready_and_has_had_no_backup_yet() {
    let setup = Setup::new();

    assert_eq!(setup.status(), Status::Ready { last_backup: None });
}

#[test]
fn ready_shows_the_last_backup_of_this_account() {
    let setup = Setup::new();
    setup.log_run("usera@pc", Reason::FilesSkipped, at(1_793_000_000));
    setup.log_run("usera@pc", Reason::Cancelled, at(1_793_100_000));
    setup.log_run("userb@pc", Reason::None, at(1_793_200_000));

    let expected = LastBackup {
        time: at(1_793_000_000),
        skipped_files: true,
    };
    assert_eq!(
        setup.status(),
        Status::Ready {
            last_backup: Some(expected)
        }
    );
}

#[test]
fn a_log_that_cannot_be_read_does_not_stop_a_backup() {
    let setup = Setup::new();
    setup
        .files
        .fail_path(root().join(LOG_FILE), ErrorKind::Other);

    assert_eq!(setup.status(), Status::Ready { last_backup: None });
}

#[test]
fn another_account_or_computer_is_not_set_up() {
    let setup = Setup::new();
    assert_eq!(setup.status_for("userb"), Status::NotSetUp);

    let mut setup = Setup::new();
    setup.this.computer_id = "fedcba9876543210fedcba9876543210".to_owned();
    assert_eq!(setup.status(), Status::NotSetUp);

    // Without a computer ID, a computer counts as not added.
    let mut setup = Setup::new();
    setup.this.computer_id = String::new();
    assert_eq!(setup.status(), Status::NotSetUp);
}

#[test]
fn a_live_lock_means_a_backup_is_in_progress() {
    let setup = Setup::new();
    setup.lock(OTHER_PROCESS);

    let expected = Status::InProgress {
        user: "userb".to_owned(),
    };
    assert_eq!(setup.status(), expected);
    assert_eq!(setup.status_for("userb"), expected);
}

#[test]
fn a_lock_that_is_left_over_stops_nobody() {
    let setup = Setup::new();
    setup.lock(3000);

    assert_eq!(setup.status(), Status::Ready { last_backup: None });
}

#[test]
fn a_lock_that_cannot_be_understood_stops_nobody() {
    let setup = Setup::new();
    setup.write(LOCK_FILE, b"");

    assert_eq!(setup.status(), Status::Ready { last_backup: None });
}

#[test]
fn a_damaged_configuration_needs_attention() {
    let setup = Setup::new();
    setup.write(
        CONFIG_FILE,
        b"# Babawus configuration\nolder_versions = 10\n",
    );

    assert_eq!(setup.status(), Status::NeedsAttention);
}

#[test]
fn a_configuration_for_a_newer_babawus_needs_attention() {
    let setup = Setup::blank();
    let text = "# Babawus configuration\n# checksum: 0\nformat = 2\n";
    setup.write(CONFIG_FILE, text.as_bytes());

    assert_eq!(setup.status(), Status::NeedsAttention);
}

#[test]
fn a_configuration_that_cannot_be_read_needs_attention() {
    let setup = Setup::new();
    setup
        .files
        .fail_path(root().join(CONFIG_FILE), ErrorKind::Other);

    assert_eq!(setup.status(), Status::NeedsAttention);
}

#[test]
fn a_damaged_configuration_needs_attention_even_while_locked() {
    let setup = Setup::new();
    setup.lock(OTHER_PROCESS);
    setup.write(CONFIG_FILE, b"garbage");

    assert_eq!(setup.status(), Status::NeedsAttention);
}

#[test]
fn a_babawus_medium_mounted_read_only_is_write_protected() {
    let mut setup = Setup::new();
    setup.medium.volume.read_only = true;

    assert_eq!(setup.status(), Status::WriteProtected);
}

#[test]
fn a_medium_without_a_configuration_is_not_a_backup_medium() {
    let setup = Setup::blank();
    assert_eq!(setup.status(), Status::NotBackupMedium);

    let mut setup = Setup::blank();
    setup.medium.volume.read_only = true;
    assert_eq!(setup.status(), Status::NotBackupMedium);
}

#[test]
fn a_medium_that_this_account_may_not_read_is_not_a_backup_medium() {
    let setup = Setup::new();
    setup
        .files
        .fail_path(root().join(CONFIG_FILE), ErrorKind::PermissionDenied);

    assert_eq!(setup.status(), Status::NotBackupMedium);
}

#[test]
fn other_file_systems_are_not_backup_media_and_are_not_read() {
    let mut setup = Setup::new();
    setup.medium = medium("vfat");
    setup
        .files
        .fail_path(root().join(CONFIG_FILE), ErrorKind::Other);

    assert_eq!(setup.status(), Status::NotBackupMedium);
}

#[test]
fn ext4_media_work_too() {
    let mut setup = Setup::new();
    setup.medium = medium("ext4");

    assert_eq!(setup.status(), Status::Ready { last_backup: None });
}

#[test]
fn the_user_is_the_part_of_the_account_before_the_at_sign() {
    assert_eq!(user_of("usera@pc"), "usera");
    assert_eq!(user_of("usera"), "usera");
}

#[test]
fn a_medium_this_account_cannot_reach_is_open_for_another_account() {
    let setup = Setup::new();
    setup.files.fail_path(root(), ErrorKind::PermissionDenied);

    let status = setup.status();

    assert_eq!(status, Status::OtherAccount);
    assert!(!status.is_babawus());
}

#[test]
fn only_known_babawus_media_count_as_babawus_media() {
    assert!(Status::NotSetUp.is_babawus());
    assert!(Status::WriteProtected.is_babawus());
    assert!(!Status::NotBackupMedium.is_babawus());
    assert!(!Status::OtherAccount.is_babawus());
}
