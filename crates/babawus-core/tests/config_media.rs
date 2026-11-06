//! The configuration on real exFAT and Ext4.
//!
//! These use sudo and loop devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use babawus_core::config::{
    Config, EntryError, Loaded, folders_on_medium, link_account, load, save,
};
use babawus_core::fs::RealFs;
use babawus_core::identity::this_computer_id;
use babawus_core::medium::CONFIG_FILE;
use babawus_core::versions::Format;
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;
const ID: &str = "0123456789abcdef0123456789abcdef";

fn medium(format: Format) -> LoopMedium {
    let medium_format = match format {
        Format::ExFat => MediumFormat::ExFat,
        Format::Ext4 => MediumFormat::Ext4,
    };
    let medium = LoopMedium::new(medium_format, 32 * MIB);
    fs::create_dir(medium.path().join(".babawus")).unwrap();
    medium
}

fn config() -> Config {
    let mut config = Config::new();
    config.add_computer(ID, "mint", "mint", &[]).unwrap();
    config.add_account(ID, "usera").unwrap();
    config
}

fn saving_and_loading(format: Format) {
    let medium = medium(format);
    let root = medium.path();
    assert_eq!(load(&RealFs, root).unwrap(), Loaded::Missing);

    let mut config = config();
    save(&RealFs, root, format, &config).unwrap();
    assert_eq!(load(&RealFs, root).unwrap(), Loaded::Ready(config.clone()));

    config.older_versions = 3;
    save(&RealFs, root, format, &config).unwrap();
    assert_eq!(load(&RealFs, root).unwrap(), Loaded::Ready(config));
    let temp = fs::read_dir(root.join(".babawus/tmp")).unwrap();
    assert_eq!(temp.count(), 0);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn saving_and_loading_on_exfat() {
    saving_and_loading(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn saving_and_loading_on_ext4() {
    saving_and_loading(Format::Ext4);
}

fn an_edit_is_noticed(format: Format) {
    let medium = medium(format);
    let root = medium.path();
    let config = config();
    save(&RealFs, root, format, &config).unwrap();

    let path = root.join(CONFIG_FILE);
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replace("usera", "userb")).unwrap();

    let Loaded::Damaged(Some(found)) = load(&RealFs, root).unwrap() else {
        panic!("the edit went unnoticed");
    };
    save(&RealFs, root, format, &found).unwrap();
    assert_eq!(load(&RealFs, root).unwrap(), Loaded::Ready(found));
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn an_edit_is_noticed_on_exfat() {
    an_edit_is_noticed(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn an_edit_is_noticed_on_ext4() {
    an_edit_is_noticed(Format::Ext4);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn on_ext4_everyone_can_read_and_write_the_configuration() {
    let medium = medium(Format::Ext4);
    let root = medium.path();

    save(&RealFs, root, Format::Ext4, &config()).unwrap();

    let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode(&root.join(CONFIG_FILE)), 0o666);
    assert_eq!(mode(&root.join(".babawus/tmp")), 0o777);
}

fn a_full_medium_keeps_the_old_configuration(format: Format, size: u64) {
    let medium_format = match format {
        Format::ExFat => MediumFormat::ExFat,
        Format::Ext4 => MediumFormat::Ext4,
    };
    let medium = LoopMedium::new(medium_format, size);
    let root = medium.path();
    fs::create_dir(root.join(".babawus")).unwrap();
    let mut config = config();
    save(&RealFs, root, format, &config).unwrap();

    // Fill the medium completely.
    let mut number = 0;
    let filler = vec![0; 64 * 1024];
    while fs::write(root.join(format!("filler-{number}")), &filler).is_ok() {
        number += 1;
    }
    let tail = vec![0; 512];
    while fs::write(root.join(format!("tail-{number}")), &tail).is_ok() {
        number += 1;
    }

    config.older_versions = 5;
    let result = save(&RealFs, root, format, &config);

    assert!(matches!(result, Err(EntryError::Io(_))), "{result:?}");
    config.older_versions = 10;
    assert_eq!(load(&RealFs, root).unwrap(), Loaded::Ready(config));
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn a_full_exfat_medium_keeps_the_old_configuration() {
    a_full_medium_keeps_the_old_configuration(Format::ExFat, 4 * MIB);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn a_full_ext4_medium_keeps_the_old_configuration() {
    a_full_medium_keeps_the_old_configuration(Format::Ext4, 4 * MIB);
}

fn linking_an_account(format: Format) {
    let medium = medium(format);
    let root = medium.path();
    let mut config = config();
    save(&RealFs, root, format, &config).unwrap();
    let old = root.join("backup/mint/usera");
    fs::create_dir_all(old.join("Documents")).unwrap();
    fs::write(old.join("Documents/report.pdf"), b"data").unwrap();

    link_account(&RealFs, root, format, &mut config, ID, "usera", "Anna").unwrap();

    assert!(!old.exists());
    let new = root.join("backup/mint/anna/Documents/report.pdf");
    assert_eq!(fs::read(new).unwrap(), b"data");
    assert_eq!(load(&RealFs, root).unwrap(), Loaded::Ready(config));
    assert_eq!(folders_on_medium(&RealFs, root).unwrap(), ["mint"]);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn linking_an_account_on_exfat() {
    linking_an_account(Format::ExFat);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn linking_an_account_on_ext4() {
    linking_an_account(Format::Ext4);
}

#[test]
fn this_computer_has_a_stable_id() {
    let id = this_computer_id(&RealFs).unwrap().unwrap();

    assert_eq!(id.len(), 32);
    assert_eq!(this_computer_id(&RealFs).unwrap().unwrap(), id);
}
