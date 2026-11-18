//! Sticks on Linux: listing, mounting, ejecting and formatting them through udisks2, over D-Bus
//! on the system bus, and setting their permissions. See "Technology" and "Platform layer" in
//! Requirements.md.
//!
//! Babawus contains no code that runs as root: everything that needs administrator rights goes
//! through udisks2. This module only turns what udisks2 reports into a [`Snapshot`], so the rules
//! about it live in [`crate::media`] and [`crate::format`], where they can be tested without
//! udisks2.

use std::cell::Cell;
use std::ffi::CString;
use std::io::{self, ErrorKind};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use gio::glib::{self, Variant, thread_guard::ThreadGuard};
use gio::prelude::*;
use gio::{
    BusType, DBusCallFlags, DBusConnection, DBusObjectManagerClient, DBusObjectManagerClientFlags,
    DBusProxy,
};

use crate::format::{Table, Target, label_is_valid, prepare};
use crate::fs::RealFs;
use crate::media::{Block, Drive, Snapshot, Volume};
use crate::platform::{EjectError, Ejected, FormatError, Step};
use crate::versions::Format;

const SERVICE: &str = "org.freedesktop.UDisks2";
const MANAGER_PATH: &str = "/org/freedesktop/UDisks2";
const DRIVE: &str = "org.freedesktop.UDisks2.Drive";
const BLOCK: &str = "org.freedesktop.UDisks2.Block";
const FILESYSTEM: &str = "org.freedesktop.UDisks2.Filesystem";
const PARTITION: &str = "org.freedesktop.UDisks2.Partition";
const PARTITION_TABLE: &str = "org.freedesktop.UDisks2.PartitionTable";
const SWAPSPACE: &str = "org.freedesktop.UDisks2.Swapspace";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// The extended attribute that holds a folder's default ACL.
const DEFAULT_ACL: &str = "system.posix_acl_default";

/// No timeout: formatting Ext4 took up to a minute on a 16 GB stick, and the default is 25
/// seconds. Unmounting can take long too, while it writes out what is still in memory.
const NO_TIMEOUT: i32 = i32::MAX;

/// How long to gather a burst of changes. This is short enough to look immediate.
const GATHER_CHANGES: Duration = Duration::from_millis(100);

/// The sticks attached to this computer, through a connection to udisks2. It keeps an
/// up-to-date copy of what udisks2 reports, and hears about changes on the main context that was
/// the thread default when it was created.
pub struct Sticks {
    manager: DBusObjectManagerClient,
}

impl Sticks {
    /// Connects to udisks2. This waits for udisks2's first answer, which is quick, and starts
    /// udisks2 if it isn't running yet.
    pub fn connect() -> io::Result<Self> {
        let manager = DBusObjectManagerClient::for_bus_sync(
            BusType::System,
            DBusObjectManagerClientFlags::NONE,
            SERVICE,
            MANAGER_PATH,
            gio::Cancellable::NONE,
        )
        .map_err(|error| io::Error::other(format!("udisks2: {}", error.message())))?;
        Ok(Self { manager })
    }

    /// What udisks2 reports right now.
    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for object in self.manager.objects() {
            let path = object.object_path();
            let proxy = |name| {
                DBusObjectExt::interface(&object, name)
                    .and_then(|interface| interface.downcast::<DBusProxy>().ok())
            };
            if let Some(drive) = proxy(DRIVE) {
                snapshot.drives.push(drive_from(&path, &drive));
            }
            let Some(block) = proxy(BLOCK) else {
                continue;
            };
            let file_system = proxy(FILESYSTEM);
            if let Some(file_system) = &file_system {
                snapshot
                    .volumes
                    .push(volume_from(&path, &block, file_system));
            }
            snapshot.blocks.push(block_from(
                &path,
                &block,
                file_system.as_ref(),
                proxy(PARTITION).as_ref(),
                proxy(SWAPSPACE).as_ref(),
            ));
        }
        snapshot
    }

    /// Calls `changed` whenever something udisks2 reports may have changed: a drive or file
    /// system came or went, or was mounted or unmounted. udisks2 reports a change as a burst of
    /// signals, which can arrive over several messages, so `changed` is called once, a moment
    /// after the first of them. It is called until the connection is dropped.
    pub fn connect_changed(&self, changed: impl Fn() + 'static) {
        let changed: Rc<dyn Fn()> = Rc::new(changed);
        let pending = Rc::new(Cell::new(false));
        let context = glib::MainContext::ref_thread_default();
        let call = |changed: &Rc<dyn Fn()>| {
            let changed = changed.clone();
            let pending = pending.clone();
            let context = context.clone();
            move || {
                if !pending.replace(true) {
                    let changed = changed.clone();
                    let pending = pending.clone();
                    context.spawn_local(async move {
                        glib::timeout_future(GATHER_CHANGES).await;
                        pending.set(false);
                        changed();
                    });
                }
            }
        };
        let on_object = call(&changed);
        self.manager.connect_object_added(move |_, _| on_object());
        let on_object = call(&changed);
        self.manager.connect_object_removed(move |_, _| on_object());
        let on_interface = call(&changed);
        self.manager
            .connect_interface_added(move |_, _, _| on_interface());
        let on_interface = call(&changed);
        self.manager
            .connect_interface_removed(move |_, _, _| on_interface());
        // gio needs these two handlers to be Send, although it calls them on the main context
        // that `connect` ran on, like the others. The guard checks that.
        // udisks2 was restarted, or stopped.
        let on_owner = ThreadGuard::new(call(&changed));
        self.manager
            .connect_name_owner_notify(move |_| on_owner.get_ref()());
        let on_property = ThreadGuard::new(call(&changed));
        self.manager
            .connect_interface_proxy_properties_changed(move |_, _, _, _, _| {
                on_property.get_ref()()
            });
    }
}

fn failed(step: Step) -> impl FnOnce(glib::Error) -> FormatError {
    move |error| FormatError::Failed {
        step,
        message: error.message().to_owned(),
    }
}

impl Sticks {
    /// Formats the drive with the udisks2 drive ID `id` as a Babawus medium, and returns where
    /// the new file system is mounted. See "Formatting a medium" in Requirements.md.
    ///
    /// The drive is checked again first, and nothing is changed if it may not be formatted.
    /// This runs on the main context, and can't be cancelled once it has started.
    pub async fn format(
        &self,
        id: &str,
        format: Format,
        label: &str,
    ) -> Result<PathBuf, FormatError> {
        if !label_is_valid(label) {
            return Err(FormatError::InvalidLabel);
        }
        let target = self.snapshot().target(id).map_err(FormatError::Refused)?;
        self.format_target(&target, format, label).await
    }

    /// Formats `target`, which must have come from [`Snapshot::target`] just now. Only the tests
    /// use this directly, on a loop device they created themselves, which has no drive.
    pub async fn format_target(
        &self,
        target: &Target,
        format: Format,
        label: &str,
    ) -> Result<PathBuf, FormatError> {
        if !label_is_valid(label) {
            return Err(FormatError::InvalidLabel);
        }
        for file_system in &target.mounted {
            self.call(file_system, FILESYSTEM, "Unmount", &[options(&[])])
                .await
                .map_err(failed(Step::Unmount))?;
        }
        for partition in &target.partitions {
            self.call(
                partition,
                BLOCK,
                "Format",
                &["empty".to_variant(), options(&[])],
            )
            .await
            .map_err(failed(Step::Partition))?;
        }
        let table = target.table;
        self.call(
            &target.disk,
            BLOCK,
            "Format",
            &[table_type(table).to_variant(), options(&[])],
        )
        .await
        .map_err(failed(Step::Partition))?;
        let created = self
            .call(
                &target.disk,
                PARTITION_TABLE,
                "CreatePartition",
                &[
                    0u64.to_variant(),
                    // The largest size that fits.
                    0u64.to_variant(),
                    table.partition_type(format).to_variant(),
                    "".to_variant(),
                    options(&[]),
                ],
            )
            .await
            .map_err(failed(Step::Partition))?;
        let partition = created
            .child_value(0)
            .str()
            .map(str::to_owned)
            .ok_or_else(|| FormatError::Failed {
                step: Step::Partition,
                message: "udisks2 did not name the new partition".to_owned(),
            })?;
        self.format_partition(&partition, format, label).await?;
        let root = self.mount(&partition).await?;
        let prepared = if format == Format::Ext4 {
            set_default_acl(&root)
        } else {
            Ok(())
        }
        .and_then(|()| prepare(&RealFs, &root, format));
        prepared.map_err(|error| FormatError::Failed {
            step: Step::Prepare,
            message: error.to_string(),
        })?;
        Ok(root)
    }

    /// Creates the file system on `partition`. The formatting user is made the owner of an
    /// Ext4 root folder, so that Babawus can set its permissions without administrator rights.
    async fn format_partition(
        &self,
        partition: &str,
        format: Format,
        label: &str,
    ) -> Result<(), FormatError> {
        let args: Vec<String> = mkfs_args(format)
            .iter()
            .map(|&arg| arg.to_owned())
            .collect();
        let mut format_options = vec![("label", label.to_variant())];
        if format == Format::Ext4 {
            format_options.push(("take-ownership", true.to_variant()));
            format_options.push(("mkfs-args", args.to_variant()));
        }
        let parameters = [
            file_system_type(format).to_variant(),
            options(&format_options),
        ];
        self.call(partition, BLOCK, "Format", &parameters)
            .await
            .map_err(failed(Step::Format))?;
        Ok(())
    }

    /// Mounts the file system on `block`, and returns where. If the desktop got there first,
    /// its mount is used.
    async fn mount(&self, block: &str) -> Result<PathBuf, FormatError> {
        let mounted = self.call(block, FILESYSTEM, "Mount", &[options(&[])]).await;
        let error = match mounted {
            Ok(reply) => {
                if let Some(path) = reply.child_value(0).str() {
                    return Ok(PathBuf::from(path));
                }
                glib::Error::new(gio::IOErrorEnum::Failed, "udisks2 did not say where")
            }
            Err(error) => error,
        };
        let points = self
            .call(
                block,
                PROPERTIES,
                "Get",
                &[FILESYSTEM.to_variant(), "MountPoints".to_variant()],
            )
            .await
            .ok()
            .and_then(|reply| reply.child_value(0).as_variant())
            .and_then(|points| points.get::<Vec<Vec<u8>>>());
        match points.and_then(|points| points.into_iter().next()) {
            Some(point) => Ok(path_from_bytes(point)),
            None => Err(failed(Step::Mount)(error)),
        }
    }

    /// Calls `method` on udisks2, allowing it to ask for the administrator password, and with
    /// no timeout.
    async fn call(
        &self,
        object_path: &str,
        interface: &str,
        method: &str,
        parameters: &[Variant],
    ) -> Result<Variant, glib::Error> {
        let connection: DBusConnection = self.manager.connection();
        connection
            .call_future(
                Some(SERVICE),
                object_path,
                interface,
                method,
                Some(&Variant::tuple_from_iter(parameters.iter().cloned())),
                None,
                DBusCallFlags::ALLOW_INTERACTIVE_AUTHORIZATION,
                NO_TIMEOUT,
            )
            .await
    }
}

/// udisks2's error for a file system that is in use.
const DEVICE_BUSY: &str = "org.freedesktop.UDisks2.Error.DeviceBusy";

impl Sticks {
    /// Ejects the drive with the udisks2 drive ID `id`, like "Safely remove" in the file
    /// manager: every file system on it is unmounted, and then it is powered off, if possible.
    /// See "Offer to eject" in Requirements.md.
    pub async fn eject(&self, id: &str) -> Result<Ejected, EjectError> {
        let plan = self.snapshot().eject_plan(id).ok_or(EjectError::Gone)?;
        self.unmount_all(&plan.mounted).await?;
        if !plan.drive.can_power_off {
            return Ok(Ejected::UnmountedOnly { reason: None });
        }
        let powered_off = self
            .call(&plan.drive.object_path, DRIVE, "PowerOff", &[options(&[])])
            .await;
        Ok(match powered_off {
            Ok(_) => Ejected::PoweredOff,
            Err(error) => Ejected::UnmountedOnly {
                reason: Some(error.message().to_owned()),
            },
        })
    }

    /// Unmounts the file systems `mounted`, given by their object paths. Only the tests use
    /// this directly, on loop devices, which have no drive to eject.
    pub async fn unmount_all(&self, mounted: &[String]) -> Result<(), EjectError> {
        for file_system in mounted {
            self.call(file_system, FILESYSTEM, "Unmount", &[options(&[])])
                .await
                .map_err(|error| {
                    if gio::DBusError::remote_error(&error).as_deref() == Some(DEVICE_BUSY) {
                        EjectError::Busy
                    } else {
                        EjectError::Failed(error.message().to_owned())
                    }
                })?;
        }
        Ok(())
    }
}

/// The type of partition table that udisks2's `Format` takes.
fn table_type(table: Table) -> &'static str {
    match table {
        Table::Mbr => "dos",
        Table::Gpt => "gpt",
    }
}

/// The name that udisks2 and `mkfs` use for `format`.
fn file_system_type(format: Format) -> &'static str {
    match format {
        Format::ExFat => "exfat",
        Format::Ext4 => "ext4",
    }
}

/// The extra arguments for `mkfs`: on Ext4, no reserved blocks, and full initialisation, so
/// that the medium is finished when formatting is.
fn mkfs_args(format: Format) -> &'static [&'static str] {
    match format {
        Format::ExFat => &[],
        Format::Ext4 => &["-m", "0", "-E", "lazy_itable_init=0,lazy_journal_init=0"],
    }
}

/// Sets a default ACL on the folder `path`, the equivalent of
/// `setfacl -m d:u::rwx,d:g::rwx,d:o::rwx`, so that what other programs create in it can be
/// read and written by everyone too.
fn set_default_acl(path: &Path) -> io::Result<()> {
    let path = CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|error| io::Error::new(ErrorKind::InvalidInput, error))?;
    let name = CString::new(DEFAULT_ACL).expect("the name has no zero byte");
    let value = default_acl();
    // SAFETY: both strings are valid C strings, and `value` is valid for reads of its length.
    let result = unsafe {
        libc::setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr().cast(),
            value.len(),
            0,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// A POSIX ACL as the kernel stores it: a version, then one entry each for the owner, the group
/// and everyone else, in that order, each with read, write and execute.
fn default_acl() -> Vec<u8> {
    const VERSION: u32 = 2;
    const USER_OBJ: u16 = 0x01;
    const GROUP_OBJ: u16 = 0x04;
    const OTHER: u16 = 0x20;
    const RWX: u16 = 0o7;
    const NO_ID: u32 = u32::MAX;
    let mut value = VERSION.to_le_bytes().to_vec();
    for tag in [USER_OBJ, GROUP_OBJ, OTHER] {
        value.extend_from_slice(&tag.to_le_bytes());
        value.extend_from_slice(&RWX.to_le_bytes());
        value.extend_from_slice(&NO_ID.to_le_bytes());
    }
    value
}

/// udisks2's options for a call, an `a{sv}`.
fn options(entries: &[(&str, Variant)]) -> Variant {
    let dict = glib::VariantDict::new(None);
    for (key, value) in entries {
        dict.insert_value(key, value);
    }
    dict.end()
}

fn drive_from(path: &str, drive: &DBusProxy) -> Drive {
    Drive {
        object_path: path.to_owned(),
        id: string(drive, "Id"),
        vendor: string(drive, "Vendor"),
        model: string(drive, "Model"),
        size: number(drive, "Size"),
        removable: flag(drive, "Removable"),
        connection_bus: string(drive, "ConnectionBus"),
        can_power_off: flag(drive, "CanPowerOff"),
    }
}

fn block_from(
    path: &str,
    block: &DBusProxy,
    file_system: Option<&DBusProxy>,
    partition: Option<&DBusProxy>,
    swap: Option<&DBusProxy>,
) -> Block {
    let device = property::<Vec<u8>>(block, "Device").map(path_from_bytes);
    let swap_active = swap.is_some_and(|swap| flag(swap, "Active"));
    Block {
        object_path: path.to_owned(),
        drive: string(block, "Drive"),
        partition_of: partition.map(|partition| string(partition, "Table")),
        size: number(block, "Size"),
        mount_points: file_system.map(mount_points).unwrap_or_default(),
        hint_system: flag(block, "HintSystem"),
        held: swap_active || device.as_deref().is_some_and(has_holders),
    }
}

/// Whether another device, such as encryption, LVM or RAID, sits on the device `device`, for
/// example `/dev/sdb1`. The kernel lists those in the device's `holders` folder.
fn has_holders(device: &Path) -> bool {
    let Some(name) = device.file_name() else {
        return false;
    };
    let holders = Path::new("/sys/class/block").join(name).join("holders");
    std::fs::read_dir(holders).is_ok_and(|mut entries| entries.next().is_some())
}

fn mount_points(file_system: &DBusProxy) -> Vec<PathBuf> {
    property::<Vec<Vec<u8>>>(file_system, "MountPoints")
        .unwrap_or_default()
        .into_iter()
        .map(path_from_bytes)
        .collect()
}

fn volume_from(path: &str, block: &DBusProxy, file_system: &DBusProxy) -> Volume {
    let mount_point = mount_points(file_system).into_iter().next();
    let read_only = flag(block, "ReadOnly")
        || mount_point
            .as_deref()
            .is_some_and(|point| mounted_read_only(point).unwrap_or(false));
    Volume {
        object_path: path.to_owned(),
        drive: string(block, "Drive"),
        label: string(block, "IdLabel"),
        file_system: string(block, "IdType"),
        mount_point,
        read_only,
        size: number(block, "Size"),
    }
}

/// A string or object path property. A missing property, or one of another type, is empty.
fn string(proxy: &DBusProxy, name: &str) -> String {
    proxy
        .cached_property(name)
        .and_then(|value| value.str().map(str::to_owned))
        .unwrap_or_default()
}

fn number(proxy: &DBusProxy, name: &str) -> u64 {
    property(proxy, name).unwrap_or_default()
}

fn flag(proxy: &DBusProxy, name: &str) -> bool {
    property(proxy, name).unwrap_or_default()
}

fn property<T: glib::variant::FromVariant>(proxy: &DBusProxy, name: &str) -> Option<T> {
    proxy.cached_property(name).as_ref().and_then(Variant::get)
}

/// udisks2 sends paths as bytes that end with a zero byte.
fn path_from_bytes(mut bytes: Vec<u8>) -> PathBuf {
    if bytes.last() == Some(&0) {
        bytes.pop();
    }
    PathBuf::from(std::ffi::OsString::from_vec(bytes))
}

/// Whether the file system mounted at `mount_point` was mounted read-only, which udisks2 doesn't
/// report: its `ReadOnly` is about the device.
fn mounted_read_only(mount_point: &Path) -> Option<bool> {
    let path = CString::new(mount_point.as_os_str().as_encoded_bytes()).ok()?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is a valid C string, and `stats` is valid for writes of a `statvfs`.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: `statvfs` succeeded, so it filled in `stats`.
    let stats = unsafe { stats.assume_init() };
    Some(stats.f_flag & libc::ST_RDONLY != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_from_udisks2_loses_its_zero_byte() {
        let path = path_from_bytes(b"/media/usera/BACKUP-A\0".to_vec());
        assert_eq!(path, Path::new("/media/usera/BACKUP-A"));
        assert_eq!(path_from_bytes(b"/mnt".to_vec()), Path::new("/mnt"));
    }

    #[test]
    fn the_table_types_are_udisks2_s_names() {
        assert_eq!(table_type(Table::Mbr), "dos");
        assert_eq!(table_type(Table::Gpt), "gpt");
    }

    #[test]
    fn only_ext4_gets_extra_mkfs_arguments() {
        assert!(mkfs_args(Format::ExFat).is_empty());
        assert_eq!(
            mkfs_args(Format::Ext4),
            ["-m", "0", "-E", "lazy_itable_init=0,lazy_journal_init=0"]
        );
    }

    #[test]
    fn the_default_acl_gives_everyone_everything() {
        let acl = default_acl();

        assert_eq!(acl.len(), 4 + 3 * 8);
        assert_eq!(&acl[..4], &[2, 0, 0, 0]);
        let entries: Vec<_> = acl[4..].chunks(8).map(|entry| entry.to_vec()).collect();
        assert_eq!(entries[0], [0x01, 0, 7, 0, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(entries[1], [0x04, 0, 7, 0, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(entries[2], [0x20, 0, 7, 0, 0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn the_root_folder_is_not_mounted_read_only() {
        assert_eq!(mounted_read_only(Path::new("/")), Some(false));
        assert_eq!(mounted_read_only(Path::new("/no/such/folder")), None);
    }
}
