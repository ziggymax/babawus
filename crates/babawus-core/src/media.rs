//! The media list: which USB media are attached, and what each one is ready for.
//!
//! See "Start Babawus and see status" and "Formatting a medium" in Requirements.md. This module
//! only works on what udisks2 has reported, so it can be tested without it. [`crate::udisks`]
//! fills in a [`Snapshot`] and reports changes.

use std::io::ErrorKind;
use std::path::PathBuf;

use crate::config::{self, Loaded};
use crate::fs::FileSystem;
use crate::lock::{ThisProcess, live_holder};
use crate::log::{LastBackup, account_name, last_backup, read_runs};
use crate::versions::Format;

/// The `ConnectionBus` of a USB drive.
const USB: &str = "usb";

/// A drive, as udisks2 reports it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Drive {
    /// The udisks2 object path, which is what calls to udisks2 need.
    pub object_path: String,
    /// The udisks2 drive ID, which identifies the drive. See "Formatting a medium".
    pub id: String,
    pub vendor: String,
    pub model: String,
    /// In bytes.
    pub size: u64,
    pub removable: bool,
    pub connection_bus: String,
    pub can_power_off: bool,
}

impl Drive {
    /// Whether this is a USB medium. `MediaRemovable` is not used, because it is false for a USB
    /// stick, whose medium cannot be taken out of the drive.
    pub fn is_usb_medium(&self) -> bool {
        self.removable && self.connection_bus == USB
    }
}

/// A block device with a file system, as udisks2 reports it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Volume {
    /// The udisks2 object path of the block device.
    pub object_path: String,
    /// The object path of its drive, or `/` if it has none, like a loop device.
    pub drive: String,
    pub label: String,
    /// For example `exfat`, `ext4` or `vfat`.
    pub file_system: String,
    /// Where it is mounted, if it is. udisks2 can report several places, and the first is used.
    pub mount_point: Option<PathBuf>,
    /// The device, or the mount, only allows reading.
    pub read_only: bool,
    /// In bytes.
    pub size: u64,
}

impl Volume {
    /// The file system as a format Babawus can use.
    pub fn format(&self) -> Option<Format> {
        match self.file_system.as_str() {
            "exfat" => Some(Format::ExFat),
            "ext4" => Some(Format::Ext4),
            _ => None,
        }
    }
}

/// A block device, with or without a file system, as udisks2 reports it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block {
    pub object_path: String,
    /// The object path of its drive, or `/` if it has none.
    pub drive: String,
    /// The object path of the whole disk, if this is a partition on it.
    pub partition_of: Option<String>,
    /// In bytes.
    pub size: u64,
    /// Where its file system is mounted.
    pub mount_points: Vec<PathBuf>,
    /// udisks2 thinks it belongs to the system, rather than to a user.
    pub hint_system: bool,
    /// It is in use by something other than a mounted file system: it is active swap, or
    /// another device sits on it, such as encryption, LVM or RAID.
    pub held: bool,
}

/// Everything udisks2 reported at one moment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub drives: Vec<Drive>,
    /// The block devices that have a file system.
    pub volumes: Vec<Volume>,
    /// All block devices.
    pub blocks: Vec<Block>,
}

/// A row of the media list: a mounted file system on a USB medium.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Medium {
    pub drive: Drive,
    pub volume: Volume,
    /// The root folder of the file system.
    pub root: PathBuf,
}

impl Snapshot {
    /// The rows of the media list, sorted by label. Media that are not mounted are left out,
    /// because Babawus never mounts media.
    pub fn media(&self) -> Vec<Medium> {
        let mut media: Vec<_> = self
            .volumes
            .iter()
            .filter_map(|volume| {
                let root = volume.mount_point.clone()?;
                let drive = self.drive(&volume.drive)?;
                drive.is_usb_medium().then(|| Medium {
                    drive: drive.clone(),
                    volume: volume.clone(),
                    root,
                })
            })
            .collect();
        media.sort_by(|a, b| {
            (&a.volume.label, &a.volume.object_path).cmp(&(&b.volume.label, &b.volume.object_path))
        });
        media
    }

    /// The drive with the udisks2 object path `object_path`.
    pub fn drive(&self, object_path: &str) -> Option<&Drive> {
        self.drives
            .iter()
            .find(|drive| drive.object_path == object_path)
    }
}

/// What ejecting a drive involves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EjectPlan {
    pub drive: Drive,
    /// The object paths of the drive's mounted file systems, which are unmounted first.
    pub mounted: Vec<String>,
}

impl Snapshot {
    /// What ejecting the drive with the udisks2 drive ID `id` involves, or nothing if it has
    /// gone or isn't a USB medium. Only USB media are ever ejected, because powering off a
    /// drive takes it away until it is plugged in again.
    pub fn eject_plan(&self, id: &str) -> Option<EjectPlan> {
        let drive = self
            .drives
            .iter()
            .find(|drive| !id.is_empty() && drive.id == id && drive.is_usb_medium())?;
        let mounted = self
            .blocks
            .iter()
            .filter(|block| block.drive == drive.object_path && !block.mount_points.is_empty())
            .map(|block| block.object_path.clone())
            .collect();
        Some(EjectPlan {
            drive: drive.clone(),
            mounted,
        })
    }
}

/// What a medium in the list is ready for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// This computer and account have been added, so a backup can start.
    Ready {
        last_backup: Option<LastBackup>,
    },
    /// A Babawus medium, but this computer or account has not been added.
    NotSetUp,
    /// Another Babawus holds the medium's lock, for the user `user`.
    InProgress {
        user: String,
    },
    /// The configuration is damaged, or needs a newer Babawus.
    NeedsAttention,
    /// A Babawus medium that is mounted read-only.
    WriteProtected,
    NotBackupMedium,
}

/// Who is looking at the list: this computer, and the account running Babawus.
pub struct Viewer<'a> {
    /// The computer's own file system, to check whether a lock is live.
    pub system: &'a dyn FileSystem,
    /// If this computer has no computer ID, `computer_id` is empty, and it counts as not added.
    pub this: &'a ThisProcess,
    pub username: &'a str,
}

impl Viewer<'_> {
    /// Works out the status of `medium`, whose file system is reached through `files`. This only
    /// reads, so it is safe to do for every medium in the list, as often as needed.
    pub fn status(&self, files: &dyn FileSystem, medium: &Medium) -> Status {
        if medium.volume.format().is_none() {
            return Status::NotBackupMedium;
        }
        let root = &medium.root;
        let config = match config::load(files, root) {
            Ok(Loaded::Ready(config)) => config,
            Ok(Loaded::Missing) => return Status::NotBackupMedium,
            Ok(Loaded::Damaged(_) | Loaded::NewerFormat) => return Status::NeedsAttention,
            // Everything on a Babawus medium can be read by everyone.
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                return Status::NotBackupMedium;
            }
            Err(_) => return Status::NeedsAttention,
        };
        if medium.volume.read_only {
            return Status::WriteProtected;
        }
        // A lock that can't be read stops nobody yet. A backup would find out what is wrong.
        if let Ok(Some(holder)) = live_holder(files, root, self.system, self.this) {
            return Status::InProgress {
                user: user_of(&holder.account).to_owned(),
            };
        }
        let Some((computer, _)) = config.account(&self.this.computer_id, self.username) else {
            return Status::NotSetUp;
        };
        let account = account_name(self.username, &computer.folder);
        // The log only adds "Last backup", so a log that can't be read doesn't stop a backup.
        let last_backup = read_runs(files, root)
            .ok()
            .and_then(|runs| last_backup(&runs, &account));
        Status::Ready { last_backup }
    }
}

/// The user name in `username@computer folder`.
pub(crate) fn user_of(account: &str) -> &str {
    account.split_once('@').map_or(account, |(user, _)| user)
}

#[cfg(test)]
mod tests;
