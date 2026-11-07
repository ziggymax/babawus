//! Formatting through udisks2, on loop devices that the tests create themselves.
//!
//! These use sudo, loop devices and udisks2, so a plain `cargo test` skips them. A loop device
//! has no drive, so it can't pass the check for USB media: these tests format it with
//! [`Udisks::format_target`], and the check itself is tested in the unit tests.

#[allow(dead_code)]
mod support;

use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use babawus_core::config::{self, Config, Loaded};
use babawus_core::format::{Table, Target};
use babawus_core::fs::RealFs;
use babawus_core::media::{Drive, Snapshot};
use babawus_core::udisks::{FormatError, Step, Udisks};
use babawus_core::versions::Format;
use gio::glib;
use support::BlankLoop;

const MIB: u64 = 1024 * 1024;
const WAIT: Duration = Duration::from_secs(20);

fn with_context(test: impl FnOnce(&glib::MainContext)) {
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| test(&context))
        .expect("use the main context");
}

/// Waits until udisks2 reports the loop device, and returns what it reports.
fn wait_for(context: &glib::MainContext, udisks: &Udisks, blank: &BlankLoop) -> Snapshot {
    let start = Instant::now();
    loop {
        let snapshot = udisks.snapshot();
        let path = blank.object_path();
        if snapshot
            .blocks
            .iter()
            .any(|block| block.object_path == path)
        {
            return snapshot;
        }
        assert!(start.elapsed() < WAIT, "udisks2 did not report {path}");
        if !context.iteration(false) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

/// The target for the loop device, worked out like [`Snapshot::target`] does for a drive.
fn target(snapshot: &Snapshot, blank: &BlankLoop, table: Table) -> Target {
    let disk = blank.object_path();
    let on_disk: Vec<_> = snapshot
        .blocks
        .iter()
        .filter(|block| block.object_path == disk || block.partition_of.as_ref() == Some(&disk))
        .collect();
    Target {
        drive: Drive::default(),
        table,
        mounted: on_disk
            .iter()
            .filter(|block| !block.mount_points.is_empty())
            .map(|block| block.object_path.clone())
            .collect(),
        partitions: on_disk
            .iter()
            .filter(|block| block.partition_of.is_some())
            .map(|block| block.object_path.clone())
            .collect(),
        disk,
    }
}

fn format(
    blank: &BlankLoop,
    table: Table,
    format: Format,
    label: &str,
) -> Result<std::path::PathBuf, FormatError> {
    let mut result = None;
    with_context(|context| {
        let udisks = Udisks::connect().expect("connect to udisks2");
        let snapshot = wait_for(context, &udisks, blank);
        let target = target(&snapshot, blank, table);
        result = Some(context.block_on(udisks.format_target(&target, format, label)));
    });
    result.unwrap()
}

/// `lsblk`'s value of `column` for `device`.
fn lsblk(column: &str, device: &str) -> String {
    let output = Command::new("lsblk")
        .args(["-dno", column, device])
        .output()
        .expect("run lsblk");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn sudo_output(args: &[&str]) -> String {
    let output = Command::new("sudo")
        .arg("-n")
        .args(args)
        .output()
        .expect("run sudo");
    assert!(output.status.success(), "sudo {args:?} failed");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Checks what every new medium has.
fn check_medium(blank: &BlankLoop, root: &Path, table: Table, format: Format, label: &str) {
    let partition = format!("{}p1", blank.device());
    let (file_system, partition_table, partition_type) = match (format, table) {
        (Format::ExFat, Table::Mbr) => ("exfat", "dos", "0x7"),
        (Format::Ext4, Table::Mbr) => ("ext4", "dos", "0x83"),
        (Format::ExFat, Table::Gpt) => ("exfat", "gpt", "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7"),
        (Format::Ext4, Table::Gpt) => ("ext4", "gpt", "0fc63daf-8483-4772-8e79-3d69d8477de4"),
    };
    assert_eq!(lsblk("FSTYPE", &partition), file_system);
    assert_eq!(lsblk("LABEL", &partition), label);
    assert_eq!(lsblk("PTTYPE", blank.device()), partition_table);
    assert_eq!(lsblk("PARTTYPE", &partition), partition_type);
    assert_eq!(lsblk("MOUNTPOINTS", &partition), root.to_str().unwrap());
    // One partition, over the whole device apart from the alignment and, for GPT, the table.
    let disk_size: u64 = lsblk("SIZE --bytes", blank.device()).parse().unwrap_or(0);
    let size: u64 = Command::new("lsblk")
        .args(["-dnbo", "SIZE", &partition])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap()
        })
        .unwrap();
    assert!(size + 2 * MIB >= disk_size, "{size} of {disk_size}");
    assert!(
        !Path::new(&format!("{}p2", blank.device())).exists(),
        "only one partition"
    );

    assert_eq!(
        config::load(&RealFs, root).unwrap(),
        Loaded::Ready(Config::new())
    );
    assert!(root.join("backup").is_dir());
}

fn formats(table: Table, format: Format, label: &str) {
    let blank = BlankLoop::new(64 * MIB);

    let root = self::format(&blank, table, format, label).expect("formatting works");

    check_medium(&blank, &root, table, format, label);
}

fn check_ext4_permissions(blank: &BlankLoop, root: &Path) {
    let me = own_uid();
    for path in [root.to_owned(), root.join(".babawus"), root.join("backup")] {
        let metadata = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata.mode() & 0o7777, 0o777, "{path:?}");
        assert_eq!(metadata.uid(), me, "{path:?}");
    }
    let acl = Command::new("getfacl")
        .args(["--default", "--omit-header"])
        .arg(root)
        .output()
        .expect("run getfacl");
    let acl = String::from_utf8_lossy(&acl.stdout);
    let entries: Vec<_> = acl.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(entries, ["user::rwx", "group::rwx", "other::rwx"]);
    // A file that another program creates gets the default ACL.
    let file = root.join("from-another-program");
    std::fs::write(&file, b"").unwrap();
    let mode = std::fs::metadata(&file).unwrap().mode() & 0o777;
    std::fs::remove_file(&file).unwrap();
    // The umask can't take rights away when a default ACL is set.
    assert_eq!(mode, 0o666);

    let partition = format!("{}p1", blank.device());
    let details = sudo_output(&["dumpe2fs", "-h", &partition]);
    let value = |name: &str| {
        details
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .map(|value| value.trim().to_owned())
            .unwrap_or_default()
    };
    assert_eq!(value("Reserved block count:"), "0");
    assert!(value("Default mount options:").contains("acl"));
    let groups = sudo_output(&["dumpe2fs", &partition]);
    let lazy = groups
        .lines()
        .filter(|line| line.starts_with("Group ") && line.contains('['))
        .filter(|line| !line.contains("ITABLE_ZEROED"))
        .count();
    assert_eq!(lazy, 0, "every block group is initialised");
}

fn own_uid() -> u32 {
    // SAFETY: getuid has no preconditions.
    unsafe { libc::getuid() }
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn formats_exfat() {
    formats(Table::Mbr, Format::ExFat, "TEST-EXFAT");
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn formats_ext4_open_to_everyone() {
    let blank = BlankLoop::new(64 * MIB);

    let root = format(&blank, Table::Mbr, Format::Ext4, "TEST-EXT4").expect("formatting works");

    check_medium(&blank, &root, Table::Mbr, Format::Ext4, "TEST-EXT4");
    check_ext4_permissions(&blank, &root);
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn formats_with_gpt() {
    formats(Table::Gpt, Format::ExFat, "TEST-GPT-X");
    formats(Table::Gpt, Format::Ext4, "TEST-GPT-4");
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn formats_a_medium_again_after_unmounting_it() {
    let blank = BlankLoop::new(64 * MIB);
    let first = format(&blank, Table::Mbr, Format::ExFat, "TEST-FIRST").unwrap();
    std::fs::write(first.join("holiday.jpg"), b"old").unwrap();
    blank.keep_attached();

    let root = format(&blank, Table::Mbr, Format::Ext4, "TEST-AGAIN").unwrap();

    check_medium(&blank, &root, Table::Mbr, Format::Ext4, "TEST-AGAIN");
    assert!(!root.join("holiday.jpg").exists());
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn an_invalid_label_changes_nothing() {
    let blank = BlankLoop::new(64 * MIB);

    let error = format(&blank, Table::Mbr, Format::ExFat, "TOO-LONG-LABEL").unwrap_err();

    assert!(matches!(error, FormatError::InvalidLabel));
    assert!(error.nothing_erased());
    assert_eq!(lsblk("PTTYPE", blank.device()), "");
}

#[test]
#[ignore = "needs udisks2"]
fn a_drive_that_is_not_attached_is_refused() {
    with_context(|context| {
        let udisks = Udisks::connect().expect("connect to udisks2");
        let error = context
            .block_on(udisks.format("No-Such-Drive", Format::ExFat, "BACKUP-A"))
            .unwrap_err();
        assert!(matches!(
            error,
            FormatError::Refused(babawus_core::format::Refusal::Gone)
        ));
        assert!(error.nothing_erased());
    });
}

#[test]
fn only_a_failed_unmount_leaves_the_data() {
    let failed = |step| FormatError::Failed {
        step,
        message: String::new(),
    };
    assert!(failed(Step::Unmount).nothing_erased());
    for step in [Step::Partition, Step::Format, Step::Mount, Step::Prepare] {
        assert!(!failed(step).nothing_erased(), "{step:?}");
    }
}
