//! The medium's lock on real exFAT and Ext4.
//!
//! These use sudo and loop devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use babawus_core::fs::RealFs;
use babawus_core::lock::{LockError, MediumLock, ThisProcess, live_holder};
use babawus_core::log::new_run_id;
use babawus_core::medium::LOCK_FILE;
use babawus_core::versions::Format;
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;
const ACCOUNT: &str = "usera@mint";
const ID: &str = "0123456789abcdef0123456789abcdef";
const OTHER_ID: &str = "fedcba9876543210fedcba9876543210";

struct Test {
    medium: LoopMedium,
    format: Format,
    this: ThisProcess,
}

impl Test {
    fn new(format: Format) -> Self {
        let medium_format = match format {
            Format::ExFat => MediumFormat::ExFat,
            Format::Ext4 => MediumFormat::Ext4,
        };
        let medium = LoopMedium::new(medium_format, 32 * MIB);
        fs::create_dir(medium.path().join(".babawus")).unwrap();
        let this = ThisProcess::read(&RealFs, ID).unwrap();
        Self {
            medium,
            format,
            this,
        }
    }

    fn root(&self) -> &Path {
        self.medium.path()
    }

    fn take(&self, this: &ThisProcess) -> Result<MediumLock<'_>, LockError> {
        let run_id = new_run_id();
        MediumLock::take(
            &RealFs,
            self.root(),
            self.format,
            &RealFs,
            this,
            &run_id,
            ACCOUNT,
        )
    }
}

fn the_lock_holds_and_is_released(format: Format) {
    let test = Test::new(format);
    let path = test.root().join(LOCK_FILE);

    let lock = test.take(&test.this).unwrap();

    let holder = live_holder(&RealFs, test.root(), &RealFs, &test.this).unwrap();
    assert_eq!(holder.unwrap().account, ACCOUNT);
    assert!(matches!(test.take(&test.this), Err(LockError::Held(_))));
    if format == Format::Ext4 {
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o666);
    }

    drop(lock);

    assert!(!path.exists());
    assert!(test.take(&test.this).is_ok());
}

fn a_lock_from_another_computer_is_replaced(format: Format) {
    let test = Test::new(format);
    let other_computer = ThisProcess {
        computer_id: OTHER_ID.to_owned(),
        ..test.this.clone()
    };
    let left_over = test.take(&other_computer).unwrap();
    std::mem::forget(left_over);

    assert!(test.take(&test.this).is_ok());
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn the_lock_holds_and_is_released_on_exfat() {
    the_lock_holds_and_is_released(Format::ExFat);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn the_lock_holds_and_is_released_on_ext4() {
    the_lock_holds_and_is_released(Format::Ext4);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn a_lock_from_another_computer_is_replaced_on_exfat() {
    a_lock_from_another_computer_is_replaced(Format::ExFat);
}

#[test]
#[ignore = "uses sudo and a loop device"]
fn a_lock_from_another_computer_is_replaced_on_ext4() {
    a_lock_from_another_computer_is_replaced(Format::Ext4);
}
