//! The configuration of a medium: `.babawus/babawus_config`.
//!
//! See "Format of the configuration file" and the sections after it in Requirements.md.
//!
//! Every function that changes the medium expects the caller to hold the medium's lock.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::exclude::Exclusions;
use crate::fs::FileSystem;
use crate::identity::{hex, is_computer_id};
use crate::log::{account_name, rename_account};
use crate::medium::{BACKUP_FOLDER, CONFIG_FILE, read_file, replace_file};
use crate::names::{fold, name_problem};
use crate::scan::{Selection, account_folder};
use crate::time::{log_timestamp, parse_log_timestamp, parse_utc_timestamp, utc_timestamp};
use crate::versions::{Format, fits_on};

/// The newest format this Babawus understands. A file in the layout of 0.1.0, which had no
/// `[babawus]` table, is damaged: there were no real users yet (issue 136).
pub const FORMAT: u32 = 1;

/// The table that says which Babawus wrote the file, and when.
const HEADER_TABLE: &str = "[babawus]";

pub const DEFAULT_OLDER_VERSIONS: u32 = 10;
pub const OLDER_VERSIONS: std::ops::RangeInclusive<u32> = 1..=99;
pub const DEFAULT_RUN_CHECK_DAYS: u32 = 7;
/// 0 means never remind (issue 136).
pub const RUN_CHECK_DAYS: std::ops::RangeInclusive<u32> = 0..=365;

const MARKER: &str = "# Babawus configuration\n";
const HEADER: &str = "\
# This file is written by Babawus. Change it with the Advanced functions in Babawus.
# If it is edited by hand, the medium needs attention until it has been saved in Babawus.
";
const CHECKSUM_PREFIX: &str = "# checksum: ";
const CHECKSUM_BYTES: usize = 8;

/// The folder name to use if a hostname would not make a valid one.
const FALLBACK_FOLDER: &str = "computer";

const SUGGESTED_PATTERNS: [&str; 3] = [".cache", "*.tmp", "*~"];
const SUGGESTED_FOLDER: &str = ".local/share/Trash";

/// What a medium's configuration file turned out to hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loaded {
    /// There is no configuration, so this is not a Babawus medium.
    Missing,
    Ready(Config),
    /// The file needs a newer Babawus, and must never be written.
    NewerFormat,
    /// The medium needs attention. With a configuration, the poweruser can check it and save it.
    /// Without one, nothing in the file can be trusted, and only a new configuration can be
    /// started.
    Damaged(Option<Config>),
}

/// Why an entry could not be changed.
#[derive(Debug)]
pub enum EntryError {
    /// The computer or account is not in the configuration.
    NotFound,
    /// The computer ID, name or folder is already in use.
    Clash,
    /// The result would not be a valid configuration.
    Invalid,
    Io(io::Error),
}

impl From<io::Error> for EntryError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// The configuration: the computers and their accounts. Which Babawus wrote the file, and when,
/// is only in the file, so that writing it again doesn't count as a change.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub computers: Vec<Computer>,
}

/// The file as it is written.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    babawus: Header,
    #[serde(rename = "computer", default, skip_serializing_if = "Vec::is_empty")]
    computers: Vec<Computer>,
}

/// The `[babawus]` table (issue 136).
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    config_format: u32,
    /// The version of Babawus that last wrote the file.
    config_app_version: String,
    /// When the file was last written, in local time with the UTC offset.
    config_update_local: String,
    /// The same time in UTC.
    config_update_utc: String,
}

impl Header {
    fn written_at(time: SystemTime) -> Self {
        Self {
            config_format: FORMAT,
            config_app_version: crate::VERSION.to_owned(),
            config_update_local: log_timestamp(time),
            config_update_utc: utc_timestamp(time),
        }
    }

    fn is_valid(&self) -> bool {
        self.config_format == FORMAT
            && !self.config_app_version.trim().is_empty()
            && parse_log_timestamp(&self.config_update_local).is_some()
            && parse_utc_timestamp(&self.config_update_utc).is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Computer {
    pub id: String,
    /// The computer's folder under `backup`.
    pub folder: String,
    /// The hostname last seen, for display only.
    pub hostname: String,
    #[serde(rename = "account", default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<Account>,
}

/// The configuration file is UTF-8, so these are strings: a path that isn't valid UTF-8 can't be
/// selected at all.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    pub username: String,
    /// N: the number of older versions to keep of each file.
    pub older_versions: u32,
    /// The most days that may pass since the last backup before Babawus reminds the user at
    /// login. 0 means never (T-059).
    pub run_check_days: u32,
    /// Selected files and folders, relative to the home folder.
    pub selected: Vec<String>,
    pub exclude_patterns: Vec<String>,
    /// Excluded folders, relative to the home folder.
    pub exclude_folders: Vec<String>,
}

impl Account {
    /// A new account, with nothing selected and the suggested exclusions.
    fn new(username: &str) -> Self {
        Self {
            username: username.to_owned(),
            older_versions: DEFAULT_OLDER_VERSIONS,
            run_check_days: DEFAULT_RUN_CHECK_DAYS,
            selected: Vec::new(),
            exclude_patterns: SUGGESTED_PATTERNS.map(str::to_owned).to_vec(),
            exclude_folders: vec![SUGGESTED_FOLDER.to_owned()],
        }
    }

    /// What this account wants backed up from the home folder `home`.
    pub fn selection(&self, home: &Path) -> Selection {
        Selection {
            home: home.to_owned(),
            selected: self.selected.iter().map(PathBuf::from).collect(),
            exclusions: Exclusions {
                patterns: self.exclude_patterns.clone(),
                folders: self.exclude_folders.iter().map(PathBuf::from).collect(),
            },
        }
    }

    fn is_valid(&self) -> bool {
        is_storable_name(&self.username)
            && OLDER_VERSIONS.contains(&self.older_versions)
            && RUN_CHECK_DAYS.contains(&self.run_check_days)
            && self
                .exclude_patterns
                .iter()
                .all(|pattern| is_pattern(pattern))
            && self
                .exclude_folders
                .iter()
                .all(|folder| is_plain_path(folder))
    }
}

impl Config {
    /// A configuration with no computers.
    pub fn new() -> Self {
        Self::default()
    }

    /// The file's contents as written at the time `written`, with the checksum as the last line.
    pub fn to_text(&self, written: SystemTime) -> String {
        let file = File {
            babawus: Header::written_at(written),
            computers: self.computers.clone(),
        };
        let fields = toml::to_string(&file).expect("a configuration can always be written");
        let body = format!("{MARKER}{HEADER}{fields}\n");
        let checksum = checksum(&body);
        format!("{body}{CHECKSUM_PREFIX}{checksum}\n")
    }

    /// The computer with the ID `id`.
    pub fn computer(&self, id: &str) -> Option<&Computer> {
        self.computers.iter().find(|computer| computer.id == id)
    }

    /// The entry for `username` on the computer with the ID `id`, and that computer. Without one,
    /// this computer or account has not been added.
    pub fn account(&self, id: &str, username: &str) -> Option<(&Computer, &Account)> {
        let computer = self.computer(id)?;
        let account = computer.account(username)?;
        Some((computer, account))
    }

    /// Records that the computer `id` now has the name `hostname`, and returns whether that
    /// changed anything. A computer that isn't in the configuration stays out of it, and an
    /// empty name tells nothing.
    pub fn note_hostname(&mut self, id: &str, hostname: &str) -> bool {
        let computer = self.computers.iter_mut().find(|computer| computer.id == id);
        match computer {
            Some(computer) if !hostname.is_empty() && computer.hostname != hostname => {
                computer.hostname = hostname.to_owned();
                true
            }
            _ => false,
        }
    }

    /// The folder name to suggest for a computer with the name `hostname`. `on_medium` is what
    /// [`folders_on_medium`] returned: a name is used if it is in the configuration or on the
    /// medium, so that a new computer never ends up in an old one's folder.
    pub fn suggest_folder(&self, hostname: &str, on_medium: &[OsString]) -> String {
        let hostname = hostname.to_lowercase();
        let base = if is_folder_name(&hostname) {
            hostname.as_str()
        } else {
            FALLBACK_FOLDER
        };
        let used = self.used_folders(on_medium);
        let free = |name: &String| !used.contains(&fold(name));
        std::iter::once(base.to_owned())
            .chain((2..).map(|number| format!("{base}-{number}")))
            .find(free)
            .expect("the numbers go on until one is free")
    }

    fn used_folders(&self, on_medium: &[OsString]) -> HashSet<String> {
        let configured = self.computers.iter().map(|computer| fold(&computer.folder));
        let existing = on_medium.iter().map(|name| fold(&name.to_string_lossy()));
        configured.chain(existing).collect()
    }

    /// Adds a computer with no accounts.
    pub fn add_computer(
        &mut self,
        id: &str,
        folder: &str,
        hostname: &str,
        on_medium: &[OsString],
    ) -> Result<(), EntryError> {
        if self.used_folders(on_medium).contains(&fold(folder)) {
            return Err(EntryError::Clash);
        }
        self.change(|config| {
            config.computers.push(Computer {
                id: id.to_owned(),
                folder: folder.to_owned(),
                hostname: hostname.to_owned(),
                accounts: Vec::new(),
            });
            Ok(())
        })
    }

    /// Adds an account to a computer, with nothing selected and the suggested exclusions.
    pub fn add_account(&mut self, id: &str, username: &str) -> Result<(), EntryError> {
        self.change(|config| {
            let computer = config.computer_mut(id)?;
            computer.accounts.push(Account::new(username));
            Ok(())
        })
    }

    /// Links this computer to the existing entry with the folder `folder`, so that backups go on
    /// in that folder, for example after a reinstall.
    pub fn link_computer(
        &mut self,
        folder: &str,
        id: &str,
        hostname: &str,
    ) -> Result<(), EntryError> {
        self.change(|config| {
            let computer = config
                .computers
                .iter_mut()
                .find(|computer| computer.folder == folder)
                .ok_or(EntryError::NotFound)?;
            computer.id = id.to_owned();
            computer.hostname = hostname.to_owned();
            Ok(())
        })
    }

    /// Removes a computer and its accounts from the configuration. Its backups stay on the
    /// medium.
    pub fn remove_computer(&mut self, id: &str) -> Result<(), EntryError> {
        self.computer_mut(id)?;
        self.computers.retain(|computer| computer.id != id);
        Ok(())
    }

    /// Removes an account from the configuration. Its backups stay on the medium.
    pub fn remove_account(&mut self, id: &str, username: &str) -> Result<(), EntryError> {
        let computer = self.computer_mut(id)?;
        computer.account(username).ok_or(EntryError::NotFound)?;
        computer
            .accounts
            .retain(|account| account.username != username);
        Ok(())
    }

    /// The entry for `username` on the computer with the ID `id`, to change it.
    pub fn account_mut(&mut self, id: &str, username: &str) -> Option<&mut Account> {
        self.computers
            .iter_mut()
            .find(|computer| computer.id == id)?
            .accounts
            .iter_mut()
            .find(|account| account.username == username)
    }

    fn computer_mut(&mut self, id: &str) -> Result<&mut Computer, EntryError> {
        self.computers
            .iter_mut()
            .find(|computer| computer.id == id)
            .ok_or(EntryError::NotFound)
    }

    /// Makes a change, and leaves the configuration alone if the result would not be valid.
    fn change(
        &mut self,
        change: impl FnOnce(&mut Self) -> Result<(), EntryError>,
    ) -> Result<(), EntryError> {
        let mut changed = self.clone();
        change(&mut changed)?;
        changed.validate()?;
        *self = changed;
        Ok(())
    }

    /// Checks the rules in "Reading the configuration file" in Requirements.md.
    fn validate(&self) -> Result<(), EntryError> {
        let (mut ids, mut folders) = (HashSet::new(), HashSet::new());
        for computer in &self.computers {
            if !is_computer_id(&computer.id) || !is_folder_name(&computer.folder) {
                return Err(EntryError::Invalid);
            }
            if !ids.insert(&computer.id) || !folders.insert(fold(&computer.folder)) {
                return Err(EntryError::Clash);
            }
            let mut usernames = HashSet::new();
            for account in &computer.accounts {
                if !account.is_valid() {
                    return Err(EntryError::Invalid);
                }
                if !usernames.insert(fold(&account.username)) {
                    return Err(EntryError::Clash);
                }
            }
        }
        Ok(())
    }
}

impl Computer {
    /// The entry for `username`. The name must match exactly: a changed name, even in case, is
    /// a different account until it has been linked.
    pub fn account(&self, username: &str) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|account| account.username == username)
    }
}

/// Whether `name` is a single name that can be stored on both file systems.
fn is_storable_name(name: &str) -> bool {
    let name = OsStr::new(name);
    let is_single = !name.is_empty() && !name.as_encoded_bytes().contains(&b'/');
    is_single && name_problem(name, Format::ExFat).is_none() && fits_on(name, Format::ExFat)
}

fn is_folder_name(name: &str) -> bool {
    is_storable_name(name) && name == name.to_lowercase()
}

fn is_pattern(pattern: &str) -> bool {
    !pattern.is_empty() && !pattern.contains('/')
}

/// Whether `path` is relative, with no `.` or `..` in it.
fn is_plain_path(path: &str) -> bool {
    let is_part = |part: &str| !part.is_empty() && part != "." && part != "..";
    Path::new(path)
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
        && path.split('/').all(is_part)
}

fn checksum(body: &str) -> String {
    hex(&Sha256::digest(body), CHECKSUM_BYTES)
}

/// Whether the last line holds the checksum of all lines above it.
fn checksum_matches(text: &str) -> bool {
    let lines = text.strip_suffix('\n').unwrap_or_default();
    let Some(end) = lines.rfind('\n') else {
        return false;
    };
    let (body, last) = text.split_at(end + 1);
    let stored = last.strip_prefix(CHECKSUM_PREFIX).unwrap_or_default();
    let stored = stored.strip_suffix('\n').unwrap_or_default();
    stored.eq_ignore_ascii_case(&checksum(body))
}

/// The value of `config_format`, from the `[babawus]` table, which comes first. This doesn't
/// parse the file, so it also works if a newer format can't be parsed.
fn declared_format(text: &str) -> Option<u32> {
    text.lines()
        .skip_while(|line| line.trim() != HEADER_TABLE)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "config_format").then(|| value.trim().parse().ok())?
        })
}

/// Works out what the contents of a configuration file hold. See "Reading the configuration file"
/// in Requirements.md.
pub fn parse(contents: &[u8]) -> Loaded {
    let Ok(text) = str::from_utf8(contents) else {
        return Loaded::Damaged(None);
    };
    let marked = text.starts_with(MARKER);
    if marked && declared_format(text).is_some_and(|format| format > FORMAT) {
        return Loaded::NewerFormat;
    }
    let config = toml::from_str::<File>(text)
        .ok()
        .filter(|file| file.babawus.is_valid())
        .map(|file| Config {
            computers: file.computers,
        })
        .filter(|config| config.validate().is_ok());
    match config {
        Some(config) if marked && checksum_matches(text) => Loaded::Ready(config),
        config => Loaded::Damaged(config),
    }
}

/// Reads the configuration of the medium at `root`.
pub fn load(medium: &dyn FileSystem, root: &Path) -> io::Result<Loaded> {
    match read_file(medium, &root.join(CONFIG_FILE)) {
        Ok(contents) => Ok(parse(&contents)),
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            Ok(Loaded::Missing)
        }
        Err(error) => Err(error),
    }
}

/// Writes the configuration safely, noting this Babawus and the time, and flushes it to the
/// medium.
pub fn save(
    medium: &dyn FileSystem,
    root: &Path,
    format: Format,
    config: &Config,
) -> Result<(), EntryError> {
    config.validate()?;
    let text = config.to_text(SystemTime::now());
    replace_file(medium, root, format, CONFIG_FILE, text.as_bytes())?;
    Ok(())
}

/// The folders under `backup`, some of which may be from removed entries.
pub fn folders_on_medium(medium: &dyn FileSystem, root: &Path) -> io::Result<Vec<OsString>> {
    match medium.read_dir(&root.join(BACKUP_FOLDER)) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        names => names,
    }
}

/// Links this account to the existing entry `old_username` after its user name has changed, so
/// that backups go on in the same place. Renames its folder and its name in the log, and then
/// saves the configuration.
///
/// The configuration comes last, so that an interrupted link can simply be repeated. If both
/// folders exist, nothing is changed, because the poweruser has to resolve that by hand.
pub fn link_account(
    medium: &dyn FileSystem,
    root: &Path,
    format: Format,
    config: &mut Config,
    id: &str,
    old_username: &str,
    new_username: &str,
) -> Result<(), EntryError> {
    let mut changed = config.clone();
    changed.change(|config| {
        let computer = config.computer_mut(id)?;
        let account = computer
            .accounts
            .iter_mut()
            .find(|account| account.username == old_username)
            .ok_or(EntryError::NotFound)?;
        account.username = new_username.to_owned();
        Ok(())
    })?;
    let folder = &config.computer(id).ok_or(EntryError::NotFound)?.folder;
    let old_folder = root.join(account_folder(folder, old_username));
    let new_folder = root.join(account_folder(folder, new_username));
    if old_folder != new_folder && exists(medium, &old_folder)? {
        if exists(medium, &new_folder)? {
            return Err(EntryError::Clash);
        }
        medium.rename(&old_folder, &new_folder)?;
    }
    let old_account = account_name(old_username, folder);
    let new_account = account_name(new_username, folder);
    rename_account(medium, root, format, &old_account, &new_account)?;
    save(medium, root, format, &changed)?;
    *config = changed;
    Ok(())
}

fn exists(medium: &dyn FileSystem, path: &Path) -> io::Result<bool> {
    match medium.metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests;
