//! Formatting a medium: which drives may be formatted, the label, the partition table, and
//! preparing the new file system.
//!
//! See "Format USB media" and "Formatting a medium" in Requirements.md. The calls that format are in
//! [`crate::platform::Sticks`], which follows the plan worked out here.

use std::io;
use std::path::{Path, PathBuf};

use crate::config::{self, Config, EntryError};
use crate::fs::FileSystem;
use crate::media::{Drive, Snapshot};
use crate::medium::{BACKUP_FOLDER, DIR_MODE};
use crate::versions::Format;

/// The longest label: the exFAT limit.
pub const MAX_LABEL: usize = 11;

const LABEL_STEM: &str = "BACKUP-";

/// A device larger than this gets a GPT partition table instead of MBR.
const MBR_LIMIT: u64 = 2 * 1024 * 1024 * 1024 * 1024;

/// Where a desktop mounts removable media. A file system mounted anywhere else is in use by
/// the system or by someone who set it up by hand, so its drive is not offered.
const MEDIA_FOLDERS: [&str; 2] = ["/media/", "/run/media/"];

/// The `.babawus` folder.
const BABAWUS_FOLDER: &str = ".babawus";

/// The partition table that formatting writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    Mbr,
    Gpt,
}

impl Table {
    /// MBR, or GPT if the device is larger than 2 TiB.
    pub fn for_size(size: u64) -> Self {
        if size > MBR_LIMIT {
            Self::Gpt
        } else {
            Self::Mbr
        }
    }

    /// The partition type for a file system in `format`.
    pub fn partition_type(self, format: Format) -> &'static str {
        match (self, format) {
            (Self::Mbr, Format::ExFat) => "0x07",
            (Self::Mbr, Format::Ext4) => "0x83",
            // Microsoft basic data.
            (Self::Gpt, Format::ExFat) => "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7",
            // Linux file system.
            (Self::Gpt, Format::Ext4) => "0fc63daf-8483-4772-8e79-3d69d8477de4",
        }
    }
}

/// Whether `label` can be given to a medium: 1 to 11 characters, using letters, digits and `-`.
pub fn label_is_valid(label: &str) -> bool {
    (1..=MAX_LABEL).contains(&label.len())
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// A label for a new medium that no medium in `in_use` has: `BACKUP-A`, then `BACKUP-B` and so
/// on. Labels are compared without regard to case, as exFAT does.
pub fn suggest_label<'a>(in_use: impl IntoIterator<Item = &'a str>) -> String {
    let in_use: Vec<String> = in_use.into_iter().map(str::to_ascii_uppercase).collect();
    let free = |label: &String| !in_use.contains(label);
    let letters = ('A'..='Z').map(|letter| format!("{LABEL_STEM}{letter}"));
    let numbers = (1..).map(|number| format!("{LABEL_STEM}{number}"));
    letters
        .chain(numbers)
        .find(free)
        .expect("the numbers never run out")
}

/// A drive that may be formatted, and what has to happen to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub drive: Drive,
    /// The object path of the whole disk.
    pub disk: String,
    pub table: Table,
    /// The object paths of the file systems on the drive that are mounted, and are unmounted
    /// first.
    pub mounted: Vec<String>,
    /// The object paths of the partitions on the disk. Their signatures are wiped before the
    /// new partition table is written: otherwise the new partition, which starts where an old
    /// one did, shows the old file system, and the desktop may mount it while it is formatted.
    pub partitions: Vec<String>,
}

/// Why a drive may not be formatted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No drive has the ID, so it has been removed.
    Gone,
    NotUsbMedium,
    /// udisks2 hasn't reported the drive's whole disk.
    NoDisk,
    /// udisks2 counts the drive as part of the system.
    System,
    /// A partition on the drive is in use other than as a medium.
    InUse,
}

impl Snapshot {
    /// The drives that may be formatted, mounted or not.
    pub fn formattable(&self) -> Vec<Target> {
        let mut targets: Vec<_> = self
            .drives
            .iter()
            .filter_map(|drive| self.target(&drive.id).ok())
            .collect();
        targets.sort_by(|a, b| a.drive.object_path.cmp(&b.drive.object_path));
        targets
    }

    /// Checks that the drive with the udisks2 drive ID `id` may be formatted, and works out
    /// what has to happen to it. This is done again right before formatting.
    pub fn target(&self, id: &str) -> Result<Target, Refusal> {
        let drive = self
            .drives
            .iter()
            .find(|drive| !id.is_empty() && drive.id == id)
            .ok_or(Refusal::Gone)?;
        if !drive.is_usb_medium() {
            return Err(Refusal::NotUsbMedium);
        }
        let blocks: Vec<_> = self
            .blocks
            .iter()
            .filter(|block| block.drive == drive.object_path)
            .collect();
        let disk = blocks
            .iter()
            .find(|block| block.partition_of.is_none())
            .ok_or(Refusal::NoDisk)?;
        if blocks.iter().any(|block| block.hint_system) {
            return Err(Refusal::System);
        }
        let mounted_elsewhere = |point: &PathBuf| {
            let point = point.as_os_str().as_encoded_bytes();
            !MEDIA_FOLDERS
                .iter()
                .any(|folder| point.starts_with(folder.as_bytes()))
        };
        if blocks
            .iter()
            .any(|block| block.held || block.mount_points.iter().any(mounted_elsewhere))
        {
            return Err(Refusal::InUse);
        }
        Ok(Target {
            drive: drive.clone(),
            disk: disk.object_path.clone(),
            table: Table::for_size(disk.size),
            mounted: blocks
                .iter()
                .filter(|block| !block.mount_points.is_empty())
                .map(|block| block.object_path.clone())
                .collect(),
            partitions: blocks
                .iter()
                .filter(|block| block.partition_of.as_ref() == Some(&disk.object_path))
                .map(|block| block.object_path.clone())
                .collect(),
        })
    }
}

/// Prepares a newly formatted, mounted file system at `root` for Babawus: on Ext4, everyone may
/// read and write it, and then the folders and a new configuration are created.
///
/// The default ACL is set separately, by the platform layer, because only a real file system
/// has one.
pub fn prepare(medium: &dyn FileSystem, root: &Path, format: Format) -> io::Result<()> {
    if format == Format::Ext4 {
        medium.set_mode(root, DIR_MODE)?;
    }
    for folder in [BABAWUS_FOLDER, BACKUP_FOLDER] {
        let path = root.join(folder);
        medium.create_dir(&path)?;
        if format == Format::Ext4 {
            medium.set_mode(&path, DIR_MODE)?;
        }
    }
    config::save(medium, root, format, &Config::new()).map_err(|error| match error {
        EntryError::Io(error) => error,
        other => io::Error::other(format!("the new configuration is not valid: {other:?}")),
    })
}

#[cfg(test)]
mod tests;
