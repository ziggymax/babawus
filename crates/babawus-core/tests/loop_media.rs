//! Integration tests on real exFAT and Ext4 file systems.
//!
//! The tests that need a medium use sudo and loop devices, so they are ignored by a plain
//! `cargo test`. Run them with `cargo test -- --include-ignored`.

#[allow(dead_code)]
mod support;

use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::Path;

use support::{LoopMedium, MediumFormat, check_device};

const MIB: u64 = 1024 * 1024;

fn writes_and_reads_back(format: MediumFormat) {
    let medium = LoopMedium::new(format, 32 * MIB);
    let file = medium.path().join("file.txt");

    fs::write(&file, "content").unwrap();

    assert_eq!(fs::read_to_string(&file).unwrap(), "content");
}

fn reports_a_full_medium(format: MediumFormat) {
    let medium = LoopMedium::new(format, 32 * MIB);
    let mut file = File::create(medium.path().join("large.bin")).unwrap();
    let chunk = vec![0; MIB as usize];

    let error = (0..64)
        .find_map(|_| file.write_all(&chunk).err())
        .expect("a 32 MiB medium cannot hold 64 MiB");

    assert_eq!(error.kind(), ErrorKind::StorageFull);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_writes_and_reads_back() {
    writes_and_reads_back(MediumFormat::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_writes_and_reads_back() {
    writes_and_reads_back(MediumFormat::Ext4);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat_reports_a_full_medium() {
    reports_a_full_medium(MediumFormat::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_reports_a_full_medium() {
    reports_a_full_medium(MediumFormat::Ext4);
}

#[test]
fn accepts_a_loop_device_backed_by_the_image() {
    let image = Path::new("/tmp/medium.img");

    assert!(check_device("/dev/loop7", "/tmp/medium.img", image).is_ok());
}

#[test]
fn refuses_anything_but_a_loop_device() {
    let image = Path::new("/tmp/medium.img");

    for device in ["/dev/sda", "/dev/sda1", "/dev/loop", "/dev/loop0p1", ""] {
        assert!(
            check_device(device, "/tmp/medium.img", image).is_err(),
            "{device}"
        );
    }
}

#[test]
fn refuses_a_loop_device_backed_by_another_file() {
    let image = Path::new("/tmp/medium.img");

    assert!(check_device("/dev/loop7", "/tmp/other.img", image).is_err());
}
