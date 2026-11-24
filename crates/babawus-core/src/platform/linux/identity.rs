//! Identity on Linux: the machine ID, the computer's name, the boot ID, and whether the user is
//! an administrator. See "Identifying a computer" and "Platform layer" in Requirements.md.

use std::io;
use std::path::Path;

use crate::fs::FileSystem;
use crate::medium::read_file;

pub(crate) const MACHINE_ID_FILE: &str = "/etc/machine-id";
const HOSTNAME_FILE: &str = "/proc/sys/kernel/hostname";
pub(crate) const BOOT_ID_FILE: &str = "/proc/sys/kernel/random/boot_id";

/// The groups whose members polkit asks for an administrator's password: `sudo` and `admin` on
/// Mint and Ubuntu (`/usr/share/polkit-1/rules.d/49-ubuntu-admin.rules`), `sudo` on Debian and
/// `wheel` on Fedora (`50-default.rules`). Each exists only where it means administrator, so the
/// list is the same everywhere (issue 108).
const ADMINISTRATOR_GROUPS: [&str; 3] = ["sudo", "admin", "wheel"];

/// The contents of `/etc/machine-id`, read from `system`, the computer's own file system.
/// [`crate::identity::computer_id`] checks them and turns them into the computer ID.
pub fn machine_id(system: &dyn FileSystem) -> io::Result<String> {
    let contents = read_file(system, Path::new(MACHINE_ID_FILE))?;
    Ok(String::from_utf8_lossy(&contents).into_owned())
}

/// The hostname of this computer, read from `system`, the computer's own file system.
pub fn hostname(system: &dyn FileSystem) -> io::Result<String> {
    let contents = read_file(system, Path::new(HOSTNAME_FILE))?;
    Ok(String::from_utf8_lossy(&contents).trim_end().to_owned())
}

/// The ID of the current boot, which changes each time the computer starts, read from `system`,
/// the computer's own file system.
pub fn boot_id(system: &dyn FileSystem) -> io::Result<String> {
    let contents = read_file(system, Path::new(BOOT_ID_FILE))?;
    Ok(String::from_utf8_lossy(&contents).trim_end().to_owned())
}

/// Whether the user is an administrator, whose own password polkit accepts. See issue 108.
pub fn is_administrator() -> bool {
    this_users_groups()
        .is_ok_and(|groups| has_administrator_group(groups.iter().map(String::as_str)))
}

/// Whether an account in the groups named `groups` is an administrator.
fn has_administrator_group<'a>(groups: impl IntoIterator<Item = &'a str>) -> bool {
    groups
        .into_iter()
        .any(|group| ADMINISTRATOR_GROUPS.contains(&group))
}

/// The names of the groups this process belongs to. A group without a name is left out.
fn this_users_groups() -> io::Result<Vec<String>> {
    // SAFETY: a size of 0 only asks for the number of groups.
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    let count = usize::try_from(count).map_err(|_| io::Error::last_os_error())?;
    let mut ids: Vec<libc::gid_t> = vec![0; count];
    // SAFETY: `ids` has room for `count` group IDs.
    let found = unsafe { libc::getgroups(count as libc::c_int, ids.as_mut_ptr()) };
    let found = usize::try_from(found).map_err(|_| io::Error::last_os_error())?;
    ids.truncate(found);
    // SAFETY: getegid can't fail.
    ids.push(unsafe { libc::getegid() });
    Ok(ids.into_iter().filter_map(group_name).collect())
}

fn group_name(id: libc::gid_t) -> Option<String> {
    let mut buffer = vec![0 as libc::c_char; 4096];
    loop {
        // SAFETY: an all-zero `group` is valid, and is only read after getgrgid_r fills it.
        let mut group: libc::group = unsafe { std::mem::zeroed() };
        let mut result = std::ptr::null_mut();
        // SAFETY: the pointers are valid, and `buffer` is as long as it says.
        let error = unsafe {
            libc::getgrgid_r(
                id,
                &mut group,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut result,
            )
        };
        if error == libc::ERANGE && buffer.len() < 1 << 20 {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        if error != 0 || result.is_null() {
            return None;
        }
        // SAFETY: getgrgid_r succeeded, so the name is a valid C string inside `buffer`.
        let name = unsafe { std::ffi::CStr::from_ptr(group.gr_name) };
        return Some(name.to_string_lossy().into_owned());
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::fs::fake::FakeFs;

    #[test]
    fn the_hostname_is_read_without_its_line_ending() {
        let fake = FakeFs::new();
        fake.add_file(HOSTNAME_FILE, b"mint\n", SystemTime::UNIX_EPOCH);

        assert_eq!(hostname(&fake).unwrap(), "mint");
    }

    #[test]
    fn this_computer_has_a_hostname() {
        assert!(!hostname(&crate::fs::RealFs).unwrap().is_empty());
    }

    #[test]
    fn the_boot_id_is_read_without_its_line_ending() {
        let fake = FakeFs::new();
        fake.add_file(
            BOOT_ID_FILE,
            b"0b5c4a2e-1d1f-4e0a-9a51-2f8f0c9d7e61\n",
            SystemTime::UNIX_EPOCH,
        );

        assert_eq!(
            boot_id(&fake).unwrap(),
            "0b5c4a2e-1d1f-4e0a-9a51-2f8f0c9d7e61"
        );
    }

    #[test]
    fn members_of_sudo_admin_or_wheel_are_administrators() {
        assert!(has_administrator_group(["steen", "adm", "sudo", "plugdev"]));
        assert!(has_administrator_group(["admin"]));
        assert!(has_administrator_group(["steen", "wheel"]));
    }

    #[test]
    fn a_standard_account_is_not_an_administrator() {
        assert!(!has_administrator_group(["testuser"]));
        assert!(!has_administrator_group([
            "adm", "plugdev", "users", "sudoers", "empower"
        ]));
        assert!(!has_administrator_group([]));
    }

    #[test]
    fn this_process_has_named_groups() {
        let groups = this_users_groups().unwrap();
        assert!(!groups.is_empty());
        assert!(groups.iter().all(|group| !group.is_empty()));
    }
}
