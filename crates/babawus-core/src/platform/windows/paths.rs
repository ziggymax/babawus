//! Where Babawus keeps its files on Windows: a placeholder that gives empty paths, which can't be
//! read or written. See `platform/windows.rs`.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn home_folder() -> PathBuf {
    env::home_dir().unwrap_or_default()
}

pub fn settings_file() -> PathBuf {
    PathBuf::new()
}

pub fn last_backup_file() -> PathBuf {
    PathBuf::new()
}

pub fn computer_lock_file() -> PathBuf {
    PathBuf::new()
}

pub fn installed_languages_folder() -> PathBuf {
    PathBuf::new()
}

/// The file name made of `bytes`, which on Windows must be text.
pub fn name_from_bytes(bytes: Vec<u8>) -> OsString {
    String::from_utf8_lossy(&bytes).into_owned().into()
}
