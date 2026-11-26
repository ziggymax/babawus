//! Where Babawus keeps its files on Linux: the user's settings and the note of their last
//! backup, by the XDG Base Directory Specification; the computer lock; and the installed language
//! files. See "Platform layer" in Requirements.md.

use std::env;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

const SETTINGS_FILE: &str = "babawus/settings.toml";
const LAST_BACKUP_FILE: &str = "babawus/last_backup";
const COMPUTER_LOCK_FILE: &str = "/run/lock/babawus.lock";
const INSTALLED_LANGUAGES_FOLDER: &str = "/usr/share/babawus/languages";

/// The user's home folder: `$HOME`, or else the one in the user database.
pub fn home_folder() -> PathBuf {
    env::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// The user's settings file, in `$XDG_CONFIG_HOME`.
pub fn settings_file() -> PathBuf {
    let home = home_folder();
    xdg_folder(env::var_os("XDG_CONFIG_HOME").as_deref(), &home, ".config").join(SETTINGS_FILE)
}

/// The folder of the programs started at the user's login, `autostart` in `$XDG_CONFIG_HOME`,
/// as the XDG Desktop Application Autostart Specification says, which GNOME and Cinnamon follow
/// (T-136).
pub fn autostart_folder() -> PathBuf {
    let home = home_folder();
    xdg_folder(env::var_os("XDG_CONFIG_HOME").as_deref(), &home, ".config").join("autostart")
}

/// The note of the user's last backup, in `$XDG_STATE_HOME`.
pub fn last_backup_file() -> PathBuf {
    let home = home_folder();
    xdg_folder(
        env::var_os("XDG_STATE_HOME").as_deref(),
        &home,
        ".local/state",
    )
    .join(LAST_BACKUP_FILE)
}

/// The lock file that lets only one Babawus run on this computer, in the shared `/run/lock/`.
pub fn computer_lock_file() -> PathBuf {
    PathBuf::from(COMPUTER_LOCK_FILE)
}

/// Where an installed Babawus finds its language files.
pub fn installed_languages_folder() -> PathBuf {
    PathBuf::from(INSTALLED_LANGUAGES_FOLDER)
}

/// The file name made of `bytes`. On Linux a name is any bytes.
pub fn name_from_bytes(bytes: Vec<u8>) -> OsString {
    OsString::from_vec(bytes)
}

/// An XDG base folder: `set`, the value of its variable, if that is an absolute path, and
/// otherwise `default` in `home`, as the XDG Base Directory Specification says.
fn xdg_folder(set: Option<&OsStr>, home: &Path, default: &str) -> PathBuf {
    set.map(Path::new)
        .filter(|folder| folder.is_absolute())
        .map_or_else(|| home.join(default), Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_xdg_folder_follows_its_variable_if_that_is_absolute() {
        let home = Path::new("/home/anna");
        let default = PathBuf::from("/home/anna/.local/state");

        assert_eq!(xdg_folder(None, home, ".local/state"), default);
        assert_eq!(
            xdg_folder(Some(OsStr::new("")), home, ".local/state"),
            default
        );
        assert_eq!(
            xdg_folder(Some(OsStr::new("relative")), home, ".local/state"),
            default
        );
        assert_eq!(
            xdg_folder(Some(OsStr::new("/var/state/anna")), home, ".local/state"),
            PathBuf::from("/var/state/anna")
        );
    }

    #[test]
    fn the_files_are_where_earlier_versions_kept_them() {
        let home = home_folder();
        assert!(home.is_absolute());
        assert!(settings_file().ends_with("babawus/settings.toml"));
        assert!(last_backup_file().ends_with("babawus/last_backup"));
        assert_eq!(computer_lock_file(), Path::new("/run/lock/babawus.lock"));
        assert_eq!(
            installed_languages_folder(),
            Path::new("/usr/share/babawus/languages")
        );
    }
}
