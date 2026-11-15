//! The configuration editor's overview: the number of older versions, every computer and
//! account on a medium, and adding, linking and removing them. See "Configure USB media" in
//! Requirements.md and issues 83-85. Changes collect in an [`Editor`] until they are
//! saved; the UI only puts [`OverviewShown`] on the screen.

use std::ffi::OsString;
use std::io;
use std::path::Path;

use crate::choose_screen::Choice;
use crate::config::{self, Config, EntryError, Loaded, folders_on_medium};
use crate::fs::FileSystem;
use crate::language::Language;
use crate::lock::{LockError, MediumLock, ThisProcess};
use crate::log::{account_name, new_run_id};
use crate::media::user_of;
use crate::text::Text;
use crate::versions::Format;

/// Who is changing the setup: this computer and the account running Babawus.
pub struct Here<'a> {
    /// The computer's own file system, to check whether a lock is live.
    pub system: &'a dyn FileSystem,
    /// If this computer has no computer ID, `computer_id` is empty, and it can't be added.
    pub this: &'a ThisProcess,
    pub username: &'a str,
    pub hostname: &'a str,
}

/// Where the medium is.
pub struct Medium<'a> {
    pub files: &'a dyn FileSystem,
    pub root: &'a Path,
    pub format: Format,
}

/// What opening the editor found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opened {
    Editing(Editor),
    /// The configuration can't be trusted at all, so only a new one can be started.
    Unreadable,
    /// The configuration needs a newer Babawus, and must not be written.
    NewerFormat,
    /// There is no configuration any more.
    NotBabawus,
}

/// The configuration being changed, and what was last saved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editor {
    pub config: Config,
    /// What is on the medium now. `None` if that isn't a usable configuration, so that saving
    /// is always needed.
    saved: Option<Config>,
    /// The folders under `backup`, which new computers can't use.
    on_medium: Vec<OsString>,
    /// The configuration was damaged, so it should be checked and saved.
    pub damaged: bool,
}

/// Why saving failed.
#[derive(Debug)]
pub enum SaveError {
    /// Another Babawus holds the medium's lock, for the user named, if known.
    Busy(Option<String>),
    /// The change would make the configuration invalid, or clashes with an entry.
    Invalid,
    /// The folder of the account to link to and the folder of the new name both exist.
    BothFolders,
    Io(io::Error),
}

impl From<EntryError> for SaveError {
    fn from(error: EntryError) -> Self {
        match error {
            EntryError::Io(error) => Self::Io(error),
            EntryError::Clash | EntryError::NotFound | EntryError::Invalid => Self::Invalid,
        }
    }
}

impl From<LockError> for SaveError {
    fn from(error: LockError) -> Self {
        match error {
            LockError::Held(holder) => {
                Self::Busy(holder.map(|holder| user_of(&holder.account).to_owned()))
            }
            LockError::Io(error) => Self::Io(error),
        }
    }
}

impl Editor {
    /// Reads the configuration of `medium`, to change it.
    pub fn open(medium: &Medium) -> io::Result<Opened> {
        let (config, damaged) = match config::load(medium.files, medium.root)? {
            Loaded::Ready(config) => (config, false),
            Loaded::Damaged(Some(config)) => (config, true),
            Loaded::Damaged(None) => return Ok(Opened::Unreadable),
            Loaded::NewerFormat => return Ok(Opened::NewerFormat),
            Loaded::Missing => return Ok(Opened::NotBabawus),
        };
        Ok(Opened::Editing(Self {
            saved: (!damaged).then(|| config.clone()),
            config,
            on_medium: folders_on_medium(medium.files, medium.root)?,
            damaged,
        }))
    }

    /// A new configuration for `medium`, in place of one that can't be trusted. Nothing is
    /// written until it is saved, and the backups stay where they are. It needs saving, but
    /// there is nothing in it to check.
    pub fn start_new(medium: &Medium) -> io::Result<Self> {
        Ok(Self {
            config: Config::new(),
            saved: None,
            on_medium: folders_on_medium(medium.files, medium.root)?,
            damaged: false,
        })
    }

    /// There are changes that haven't been saved.
    pub fn unsaved(&self) -> bool {
        self.saved.as_ref() != Some(&self.config)
    }

    /// Sets N for the account `username` on the computer `id` (issue 136).
    pub fn set_older_versions(&mut self, id: &str, username: &str, older_versions: u32) {
        if let Some(account) = self.config.account_mut(id, username) {
            account.older_versions = older_versions;
        }
    }

    /// Sets `run_check_days` for the account `username` on the computer `id`.
    pub fn set_run_check_days(&mut self, id: &str, username: &str, days: u32) {
        if let Some(account) = self.config.account_mut(id, username) {
            account.run_check_days = days;
        }
    }

    /// The folder a new entry for this computer would get.
    pub fn suggested_folder(&self, here: &Here) -> String {
        self.config.suggest_folder(here.hostname, &self.on_medium)
    }

    /// Adds this computer, if it isn't there yet, and this account.
    pub fn add_here(&mut self, here: &Here) -> Result<(), EntryError> {
        let id = &here.this.computer_id;
        if id.is_empty() {
            return Err(EntryError::Invalid);
        }
        let mut changed = self.config.clone();
        if changed.computer(id).is_none() {
            let folder = self.suggested_folder(here);
            changed.add_computer(id, &folder, here.hostname, &self.on_medium)?;
        }
        changed.add_account(id, here.username)?;
        self.config = changed;
        Ok(())
    }

    /// Links this computer to the existing entry with the folder `folder`, for example after a
    /// reinstall.
    pub fn link_computer(&mut self, folder: &str, here: &Here) -> Result<(), EntryError> {
        if here.this.computer_id.is_empty() {
            return Err(EntryError::Invalid);
        }
        self.config
            .link_computer(folder, &here.this.computer_id, here.hostname)
    }

    /// What this account backs up, if it is set up.
    pub fn choice(&self, here: &Here) -> Option<Choice> {
        let (_, account) = self.config.account(&here.this.computer_id, here.username)?;
        Some(Choice::of(account))
    }

    /// Changes what this account backs up. It waits for [`save`](Self::save) like any other
    /// change.
    pub fn set_choice(&mut self, here: &Here, choice: &Choice) {
        let account = self
            .config
            .computers
            .iter_mut()
            .filter(|computer| computer.id == here.this.computer_id)
            .flat_map(|computer| computer.accounts.iter_mut())
            .find(|account| account.username == here.username);
        if let Some(account) = account {
            choice.apply(account);
        }
    }

    pub fn remove_computer(&mut self, id: &str) -> Result<(), EntryError> {
        self.config.remove_computer(id)
    }

    pub fn remove_account(&mut self, id: &str, username: &str) -> Result<(), EntryError> {
        self.config.remove_account(id, username)
    }

    /// Saves the configuration under the medium's lock, noting this computer's hostname.
    pub fn save(&mut self, medium: &Medium, here: &Here) -> Result<(), SaveError> {
        let mut config = self.config.clone();
        config.note_hostname(&here.this.computer_id, here.hostname);
        let _lock = self.lock(medium, here)?;
        config::save(medium.files, medium.root, medium.format, &config)?;
        self.config = config.clone();
        self.saved = Some(config);
        self.damaged = false;
        Ok(())
    }

    /// Links this account to its earlier entry `old_username`, after its user name changed.
    /// This renames its folder and its name in the log, and saves the configuration with every
    /// other change, under the medium's lock.
    pub fn link_account(
        &mut self,
        medium: &Medium,
        here: &Here,
        old_username: &str,
    ) -> Result<(), SaveError> {
        let mut config = self.config.clone();
        config.note_hostname(&here.this.computer_id, here.hostname);
        let _lock = self.lock(medium, here)?;
        let linked = config::link_account(
            medium.files,
            medium.root,
            medium.format,
            &mut config,
            &here.this.computer_id,
            old_username,
            here.username,
        );
        match linked {
            Err(EntryError::Clash) => return Err(SaveError::BothFolders),
            linked => linked?,
        }
        self.config = config.clone();
        self.saved = Some(config);
        self.damaged = false;
        Ok(())
    }

    fn lock<'a>(&self, medium: &Medium<'a>, here: &Here) -> Result<MediumLock<'a>, SaveError> {
        let folder = self
            .config
            .computer(&here.this.computer_id)
            .map_or("", |computer| computer.folder.as_str());
        let account = account_name(here.username, folder);
        let lock = MediumLock::take(
            medium.files,
            medium.root,
            medium.format,
            here.system,
            here.this,
            &new_run_id(),
            &account,
        )?;
        Ok(lock)
    }
}

/// What can be done about this computer and account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HereAction {
    /// Add this computer, if needed, and this account.
    Add,
    /// Link this computer to the entry with this folder.
    LinkComputer(String),
    /// Link this account to the entry with this user name.
    LinkAccount(String),
}

/// This computer and account, in the overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HereShown {
    /// For example "Your account, anna, is set up on this computer (laptop)."
    pub text: String,
    /// What can be done, with the label of its button.
    pub actions: Vec<(HereAction, String)>,
    /// This account is set up, so what it backs up can be chosen.
    pub can_choose: bool,
}

/// An account in the list of entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountShown {
    pub username: String,
    /// For example "anna (you) · 3 files or folders chosen".
    pub text: String,
    pub older_versions: u32,
    pub run_check_days: u32,
}

/// A computer in the list of entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComputerShown {
    pub id: String,
    pub folder: String,
    /// For example "laptop (this computer)" or "desktop · last seen as Desktop-PC".
    pub title: String,
    pub accounts: Vec<AccountShown>,
}

/// The overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewShown {
    /// Says that a damaged configuration must be checked and saved.
    pub banner: Option<String>,
    pub here: HereShown,
    pub computers: Vec<ComputerShown>,
    /// For example "No computers or accounts are set up on this stick yet."
    pub empty: Option<String>,
    pub can_save: bool,
}

/// The overview of `editor`, as seen from `here`.
pub fn overview(editor: &Editor, here: &Here, language: Language) -> OverviewShown {
    let config = &editor.config;
    let id = &here.this.computer_id;
    let username = here.username;
    let this_computer = config.computer(id);
    let here_shown = if id.is_empty() {
        HereShown {
            text: Text::SetupNoComputerId.get(language).to_owned(),
            actions: Vec::new(),
            can_choose: false,
        }
    } else {
        match this_computer {
            None => {
                let folder = editor.suggested_folder(here);
                let mut actions = vec![(
                    HereAction::Add,
                    Text::SetupAddComputer.fill(language, &[("folder", &folder)]),
                )];
                actions.extend(config.computers.iter().map(|computer| {
                    let label =
                        Text::SetupLinkComputer.fill(language, &[("folder", &computer.folder)]);
                    (HereAction::LinkComputer(computer.folder.clone()), label)
                }));
                let text = if config.computers.is_empty() {
                    Text::SetupComputerMissing
                } else {
                    Text::SetupComputerMissingLink
                };
                HereShown {
                    text: text.fill(
                        language,
                        &[("hostname", here.hostname), ("folder", &folder)],
                    ),
                    actions,
                    can_choose: false,
                }
            }
            Some(computer) if computer.account(username).is_none() => {
                let mut actions = vec![(
                    HereAction::Add,
                    Text::SetupAddAccount.fill(language, &[("user", username)]),
                )];
                actions.extend(computer.accounts.iter().map(|account| {
                    let label =
                        Text::SetupLinkAccount.fill(language, &[("old", &account.username)]);
                    (HereAction::LinkAccount(account.username.clone()), label)
                }));
                let text = if computer.accounts.is_empty() {
                    Text::SetupAccountMissing
                } else {
                    Text::SetupAccountMissingLink
                };
                HereShown {
                    text: text.fill(
                        language,
                        &[("user", username), ("folder", &computer.folder)],
                    ),
                    actions,
                    can_choose: false,
                }
            }
            Some(computer) => HereShown {
                text: Text::SetupHereReady.fill(
                    language,
                    &[("user", username), ("folder", &computer.folder)],
                ),
                actions: Vec::new(),
                can_choose: true,
            },
        }
    };
    let computers: Vec<ComputerShown> = config
        .computers
        .iter()
        .map(|computer| {
            let this = &computer.id == id;
            let title = if this {
                Text::SetupThisComputer.fill(language, &[("folder", &computer.folder)])
            } else {
                Text::SetupOtherComputer.fill(
                    language,
                    &[
                        ("folder", &computer.folder),
                        ("hostname", &computer.hostname),
                    ],
                )
            };
            let accounts = computer
                .accounts
                .iter()
                .map(|account| {
                    let chosen = match account.selected.len() {
                        0 => Text::SetupNothingChosen.get(language).to_owned(),
                        1 => Text::SetupOneChosen.get(language).to_owned(),
                        count => Text::SetupChosen.fill(language, &[("count", &count.to_string())]),
                    };
                    let name = if this && account.username == username {
                        Text::SetupYou.fill(language, &[("user", &account.username)])
                    } else {
                        account.username.clone()
                    };
                    AccountShown {
                        username: account.username.clone(),
                        text: format!("{name} · {chosen}"),
                        older_versions: account.older_versions,
                        run_check_days: account.run_check_days,
                    }
                })
                .collect();
            ComputerShown {
                id: computer.id.clone(),
                folder: computer.folder.clone(),
                title,
                accounts,
            }
        })
        .collect();
    OverviewShown {
        banner: editor
            .damaged
            .then(|| Text::SetupDamaged.get(language).to_owned()),
        here: here_shown,
        empty: computers
            .is_empty()
            .then(|| Text::SetupEmpty.get(language).to_owned()),
        computers,
        can_save: editor.unsaved(),
    }
}

/// What a failed save says, for the medium labelled `label`.
pub fn save_error_text(error: &SaveError, language: Language) -> String {
    match error {
        SaveError::Busy(Some(user)) => Text::SetupBusy.fill(language, &[("user", user)]),
        SaveError::Busy(None) => Text::VerdictBusy.get(language).to_owned(),
        SaveError::Invalid => Text::SetupInvalid.get(language).to_owned(),
        SaveError::BothFolders => Text::SetupBothFolders.get(language).to_owned(),
        SaveError::Io(error) => {
            Text::SetupSaveFailed.fill(language, &[("error", &error.to_string())])
        }
    }
}

#[cfg(test)]
mod tests;
