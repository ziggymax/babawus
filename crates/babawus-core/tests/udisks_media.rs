//! What udisks2 reports about real media, and the status of a real Babawus medium.
//!
//! These use sudo and loop devices, and the udisks2 service, so a plain `cargo test` skips them.
//! A loop device has no drive, so it is never a USB medium: the tests check what udisks2 reports
//! about its file system, and treat it as if it were on a USB stick for the status.

#[allow(dead_code)]
mod support;

use std::path::Path;
use std::time::{Duration, Instant};

use babawus_core::config::{Config, save};
use babawus_core::fs::RealFs;
use babawus_core::identity::this_computer_id;
use babawus_core::lock::ThisProcess;
use babawus_core::media::{Drive, Medium, Snapshot, Status, Viewer, Volume};
use babawus_core::udisks::Udisks;
use babawus_core::versions::Format;
use gio::glib;
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;
const WAIT: Duration = Duration::from_secs(20);

fn volume_at<'a>(snapshot: &'a Snapshot, root: &Path) -> Option<&'a Volume> {
    snapshot
        .volumes
        .iter()
        .find(|volume| volume.mount_point.as_deref() == Some(root))
}

/// Runs `test` with its own main context, so that tests running side by side don't share one.
fn with_context(test: impl FnOnce(&glib::MainContext)) {
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| test(&context))
        .expect("use the main context");
}

/// Runs the main context until `done` holds, or fails after a while.
fn wait_until(context: &glib::MainContext, mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < WAIT, "udisks2 did not report the change");
        if !context.iteration(false) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn reports_the_file_system(format: MediumFormat, name: &str) {
    with_context(|context| {
        let udisks = Udisks::connect().expect("connect to udisks2");
        let medium = LoopMedium::new(format, 32 * MIB);

        // udisks2 reads the mount table on its own schedule, and can list the mount before it
        // has probed the label and file system, so wait for those too.
        let mut snapshot = Snapshot::default();
        wait_until(context, || {
            snapshot = udisks.snapshot();
            volume_at(&snapshot, medium.path())
                .is_some_and(|volume| !volume.label.is_empty() && !volume.file_system.is_empty())
        });

        let volume = volume_at(&snapshot, medium.path()).unwrap();
        assert_eq!(volume.label, "BACKUP-A");
        assert_eq!(volume.file_system, name);
        assert!(!volume.read_only);
        assert!(volume.size > 0);
        assert_eq!(volume.drive, "/", "a loop device has no drive");
        assert!(
            snapshot
                .media()
                .iter()
                .all(|listed| listed.root != medium.path()),
            "a loop device is not a USB medium"
        );
    });
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn exfat_is_reported() {
    reports_the_file_system(MediumFormat::ExFat, "exfat");
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn ext4_is_reported() {
    reports_the_file_system(MediumFormat::Ext4, "ext4");
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn changes_are_reported() {
    with_context(|context| {
        let udisks = Udisks::connect().expect("connect to udisks2");
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = calls.clone();
        udisks.connect_changed(move || counter.set(counter.get() + 1));

        let medium = LoopMedium::new(MediumFormat::ExFat, 32 * MIB);
        wait_until(context, || {
            calls.get() > 0 && volume_at(&udisks.snapshot(), medium.path()).is_some()
        });

        let root = medium.path().to_owned();
        let before = calls.get();
        drop(medium);
        wait_until(context, || {
            calls.get() > before && volume_at(&udisks.snapshot(), &root).is_none()
        });
    });
}

#[test]
#[ignore = "needs sudo, loop devices and udisks2"]
fn a_real_babawus_medium_is_ready() {
    for (format, real_format) in [
        (MediumFormat::ExFat, Format::ExFat),
        (MediumFormat::Ext4, Format::Ext4),
    ] {
        with_context(|context| {
            let udisks = Udisks::connect().expect("connect to udisks2");
            let loop_medium = LoopMedium::new(format, 32 * MIB);
            let root = loop_medium.path();
            let computer_id = this_computer_id(&RealFs).unwrap().expect("a computer ID");
            let this = ThisProcess::read(&RealFs, &computer_id).unwrap();
            let username = glib::user_name().to_string_lossy().into_owned();

            std::fs::create_dir(root.join(".babawus")).unwrap();
            let mut config = Config::new();
            config
                .add_computer(&computer_id, "pc", "mint", &[])
                .unwrap();
            config.add_account(&computer_id, &username).unwrap();
            save(&RealFs, root, real_format, &config).unwrap();

            // The status depends on the file system, which udisks2 can report after the mount.
            let mut snapshot = Snapshot::default();
            wait_until(context, || {
                snapshot = udisks.snapshot();
                volume_at(&snapshot, root).is_some_and(|volume| !volume.file_system.is_empty())
            });
            let medium = Medium {
                drive: Drive::default(),
                volume: volume_at(&snapshot, root).unwrap().clone(),
                root: root.to_owned(),
            };
            let viewer = Viewer {
                system: &RealFs,
                this: &this,
                username: &username,
            };

            assert_eq!(
                viewer.status(&RealFs, &medium),
                Status::Ready { last_backup: None },
                "{format:?}"
            );

            loop_medium.remount_read_only();
            wait_until(context, || {
                volume_at(&udisks.snapshot(), root).is_some_and(|volume| volume.read_only)
            });
            let medium = Medium {
                volume: volume_at(&udisks.snapshot(), root).unwrap().clone(),
                ..medium
            };
            assert_eq!(
                viewer.status(&RealFs, &medium),
                Status::WriteProtected,
                "{format:?}"
            );
        });
    }
}
