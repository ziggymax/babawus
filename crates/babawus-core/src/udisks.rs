//! udisks2, over D-Bus on the system bus. See "Technology" in Requirements.md.
//!
//! Babawus contains no code that runs as root: everything that needs administrator rights goes
//! through udisks2. This module only turns what udisks2 reports into a [`Snapshot`], so the rules
//! about it live in [`crate::media`], where they can be tested without udisks2.

use std::cell::Cell;
use std::ffi::CString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use gio::glib::{self, Variant, thread_guard::ThreadGuard};
use gio::prelude::*;
use gio::{BusType, DBusObjectManagerClient, DBusObjectManagerClientFlags, DBusProxy};

use crate::media::{Drive, Snapshot, Volume};

const SERVICE: &str = "org.freedesktop.UDisks2";
const MANAGER_PATH: &str = "/org/freedesktop/UDisks2";
const DRIVE: &str = "org.freedesktop.UDisks2.Drive";
const BLOCK: &str = "org.freedesktop.UDisks2.Block";
const FILESYSTEM: &str = "org.freedesktop.UDisks2.Filesystem";

/// How long to gather a burst of changes. This is short enough to look immediate.
const GATHER_CHANGES: Duration = Duration::from_millis(100);

/// A connection to udisks2. It keeps an up-to-date copy of what udisks2 reports, and hears
/// about changes on the main context that was the thread default when it was created.
pub struct Udisks {
    manager: DBusObjectManagerClient,
}

impl Udisks {
    /// Connects to udisks2. This waits for udisks2's first answer, which is quick, and starts
    /// udisks2 if it isn't running yet.
    pub fn connect() -> Result<Self, glib::Error> {
        let manager = DBusObjectManagerClient::for_bus_sync(
            BusType::System,
            DBusObjectManagerClientFlags::NONE,
            SERVICE,
            MANAGER_PATH,
            gio::Cancellable::NONE,
        )?;
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
            if let (Some(block), Some(file_system)) = (proxy(BLOCK), proxy(FILESYSTEM)) {
                snapshot
                    .volumes
                    .push(volume_from(&path, &block, &file_system));
            }
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

fn volume_from(path: &str, block: &DBusProxy, file_system: &DBusProxy) -> Volume {
    let mount_point = file_system
        .cached_property("MountPoints")
        .and_then(|points| points.get::<Vec<Vec<u8>>>())
        .and_then(|points| points.into_iter().next())
        .map(path_from_bytes);
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
    fn the_root_folder_is_not_mounted_read_only() {
        assert_eq!(mounted_read_only(Path::new("/")), Some(false));
        assert_eq!(mounted_read_only(Path::new("/no/such/folder")), None);
    }
}
