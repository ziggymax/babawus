//! Ejecting through udisks2, on loop devices that the tests create themselves.
//!
//! A loop device has no drive, so it can't be powered off: these tests check unmounting with
//! [`Sticks::unmount_all`], and powering off is on the manual checklist.

#[allow(dead_code)]
mod support;

use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use babawus_core::format::{Table, Target};
use babawus_core::media::{Drive, Snapshot};
use babawus_core::platform::{EjectError, Sticks};
use babawus_core::versions::Format;
use gio::glib;
use support::BlankLoop;

const MIB: u64 = 1024 * 1024;
const WAIT: Duration = Duration::from_secs(20);

/// A loop device with a file system that udisks2 mounted for this user, and the udisks2
/// object path of that file system.
fn mounted_medium(
    context: &glib::MainContext,
    udisks: &Sticks,
    label: &str,
) -> (BlankLoop, String) {
    let blank = BlankLoop::new(32 * MIB);
    let start = Instant::now();
    while !udisks
        .snapshot()
        .blocks
        .iter()
        .any(|block| block.object_path == blank.object_path())
    {
        assert!(start.elapsed() < WAIT, "udisks2 did not report the device");
        context.iteration(false);
    }
    let target = Target {
        drive: Drive::default(),
        disk: blank.object_path(),
        table: Table::Mbr,
        mounted: Vec::new(),
        partitions: Vec::new(),
    };
    context
        .block_on(udisks.format_target(&target, Format::ExFat, label))
        .expect("formatting works");
    // So that unmounting doesn't detach the loop device before the test is done with it.
    blank.keep_attached();
    let partition = format!("{}p1", blank.object_path());
    (blank, partition)
}

fn mount_points(udisks: &Sticks, block: &str) -> Vec<std::path::PathBuf> {
    let snapshot: Snapshot = udisks.snapshot();
    snapshot
        .blocks
        .into_iter()
        .find(|each| each.object_path == block)
        .map(|block| block.mount_points)
        .unwrap_or_default()
}

fn with_udisks(test: impl FnOnce(&glib::MainContext, &Sticks)) {
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| {
            let udisks = Sticks::connect().expect("connect to udisks2");
            test(&context, &udisks);
        })
        .expect("use the main context");
}

/// Waits until `done` holds, while udisks2's reports come in.
fn wait_until(context: &glib::MainContext, mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < WAIT, "udisks2 did not report the change");
        if !context.iteration(false) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn unmounts_the_medium() {
    with_udisks(|context, udisks| {
        let (_blank, partition) = mounted_medium(context, udisks, "TEST-EJECT");
        wait_until(context, || !mount_points(udisks, &partition).is_empty());
        let root = mount_points(udisks, &partition).remove(0);

        context
            .block_on(udisks.unmount_all(std::slice::from_ref(&partition)))
            .expect("unmounting works");

        wait_until(context, || mount_points(udisks, &partition).is_empty());
        assert!(!is_mounted(&root));
    });
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn a_medium_in_use_is_busy_and_stays_mounted() {
    with_udisks(|context, udisks| {
        let (_blank, partition) = mounted_medium(context, udisks, "TEST-BUSY");
        wait_until(context, || !mount_points(udisks, &partition).is_empty());
        let root = mount_points(udisks, &partition).remove(0);
        let open_file = File::create(root.join("open.txt")).unwrap();

        let result = context.block_on(udisks.unmount_all(std::slice::from_ref(&partition)));

        assert_eq!(result, Err(EjectError::Busy));
        assert!(is_mounted(&root));
        drop(open_file);
        context
            .block_on(udisks.unmount_all(std::slice::from_ref(&partition)))
            .expect("unmounting works once the file is closed");
    });
}

#[test]
#[ignore = "needs udisks2"]
fn a_drive_that_is_not_attached_has_gone() {
    with_udisks(|context, udisks| {
        let result = context.block_on(udisks.eject("No-Such-Drive"));
        assert_eq!(result, Err(EjectError::Gone));
    });
}

fn is_mounted(root: &Path) -> bool {
    let mounts = std::fs::read_to_string("/proc/self/mounts").unwrap();
    let root = root.to_str().unwrap();
    mounts
        .lines()
        .any(|line| line.split(' ').nth(1) == Some(root))
}
