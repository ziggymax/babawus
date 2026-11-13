//! The reminder at login: `babawus --checklastbackup` reminds the user to back up when their last
//! backup is too long ago. Each backup that counts notes when it finished, with the account's
//! `run_check_days` from the stick, in a state file of the user's, because the stick is usually
//! not plugged in at login. See "Remind me at login when my last backup is too old" in
//! Requirements.md and Issues.md item 137.

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::fs::FileSystem;
use crate::medium::read_file;
use crate::settings::save_user_file;
use crate::time::{ClockTime, log_timestamp, parse_log_timestamp};

/// The state file, in the user's state folder (`$XDG_STATE_HOME`).
pub const STATE_FILE: &str = "babawus/last_backup";

/// The state file of the user whose home folder is `home`, in `$XDG_STATE_HOME`, which is
/// `xdg_state_home` if it is set to an absolute path, and `~/.local/state` otherwise, as the XDG
/// Base Directory Specification says.
pub fn state_file(xdg_state_home: Option<&OsStr>, home: &Path) -> PathBuf {
    let folder = xdg_state_home
        .map(Path::new)
        .filter(|folder| folder.is_absolute())
        .map_or_else(|| home.join(".local/state"), Path::to_path_buf);
    folder.join(STATE_FILE)
}

/// The command line option that asks for the reminder.
pub const CHECK_OPTION: &str = "--checklastbackup";

/// The last backup that counts, as noted in the state file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Noted {
    pub finished: SystemTime,
    /// The most days that may pass before a reminder. 0 means never.
    pub run_check_days: u32,
}

/// The state file as it is written.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    /// When the backup finished, in the log's form.
    last_backup: String,
    run_check_days: u32,
}

/// Notes in the state file at `path` that a backup that counts finished at `finished`.
pub fn note(fs: &dyn FileSystem, path: &Path, noted: Noted) -> io::Result<()> {
    let file = File {
        last_backup: log_timestamp(noted.finished),
        run_check_days: noted.run_check_days,
    };
    let text = toml::to_string(&file).map_err(io::Error::other)?;
    save_user_file(fs, path, text.as_bytes())
}

/// What the state file at `path` says. A file that is missing, or can't be understood, says
/// nothing, so there is no reminder.
pub fn read(fs: &dyn FileSystem, path: &Path) -> Option<Noted> {
    let contents = read_file(fs, path).ok()?;
    let file: File = toml::from_str(str::from_utf8(&contents).ok()?).ok()?;
    Some(Noted {
        finished: parse_log_timestamp(&file.last_backup)?,
        run_check_days: file.run_check_days,
    })
}

/// How many calendar days ago the last backup was, if that is more than `run_check_days`, so
/// that a reminder is due at the time `now`. `clock` turns a time into a date to count with.
pub fn due(
    noted: Option<Noted>,
    now: SystemTime,
    clock: fn(SystemTime) -> ClockTime,
) -> Option<i64> {
    let noted = noted.filter(|noted| noted.run_check_days > 0)?;
    let days = clock(noted.finished).days_until(clock(now));
    (days > i64::from(noted.run_check_days)).then_some(days)
}

#[cfg(test)]
mod tests;
