//! What the start screen shows for each medium. See "Start Babawus and see status" in
//! Requirements.md. The UI only puts these texts and marks on the screen.

use std::ffi::CString;
use std::path::Path;
use std::time::SystemTime;

use crate::fs::FileSystem;
use crate::log::read_runs;
use crate::media::{Medium, Snapshot, Status, Viewer};
use crate::settings::Language;
use crate::text::{Text, size};
use crate::time::ClockTime;
use crate::verdict_screen::last_backup_text;

/// What reading a medium found out. Reading may take a while, so it is done off the main thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checked {
    pub status: Status,
    /// In bytes, if it could be found out.
    pub free: Option<u64>,
    /// The log has at least one run, so there is a history to show.
    pub has_history: bool,
}

impl Viewer<'_> {
    /// Reads `medium`, whose file system is reached through `files`, for the start screen.
    pub fn check(&self, files: &dyn FileSystem, medium: &Medium) -> Checked {
        let status = self.status(files, medium);
        let babawus = status.is_babawus();
        let has_history =
            babawus && read_runs(files, &medium.root).is_ok_and(|runs| !runs.is_empty());
        Checked {
            status,
            free: free_space(&medium.root),
            has_history,
        }
    }
}

/// The space left for files on the file system at `root`, in bytes.
fn free_space(root: &Path) -> Option<u64> {
    let path = CString::new(root.as_os_str().as_encoded_bytes()).ok()?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is a valid C string, and `stats` is valid for writes of a `statvfs`.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: `statvfs` succeeded, so it filled in `stats`.
    let stats = unsafe { stats.assume_init() };
    Some(stats.f_bavail.saturating_mul(stats.f_frsize))
}

/// The file system that udisks2 calls `id`, named as people know it, the same in every language
/// (Issues.md item 133). Nothing if udisks2 reports none.
fn file_system_name(id: &str) -> Option<&str> {
    Some(match id {
        "" => return None,
        "exfat" => "exFAT",
        "ext4" => "Ext4",
        "ext3" => "Ext3",
        "ext2" => "Ext2",
        "vfat" => "FAT",
        "ntfs" => "NTFS",
        other => other,
    })
}

/// Whether a backup can be started, at a glance. The shapes differ as well as the colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// A green tick.
    Ready,
    /// An amber "!": usable once someone acts, or once another backup has finished.
    Attention,
    /// A grey cross.
    Unusable,
}

impl Mark {
    pub fn of(status: &Status) -> Self {
        match status {
            Status::Ready { .. } => Self::Ready,
            Status::NotSetUp
            | Status::InProgress { .. }
            | Status::NeedsAttention
            | Status::OtherAccount => Self::Attention,
            Status::WriteProtected | Status::NotBackupMedium => Self::Unusable,
        }
    }
}

/// A row of the media list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub label: String,
    /// A Babawus medium, so its stick is in colour. Grey until it has been checked.
    pub babawus: bool,
    /// Nothing while the medium is being checked: a spinner is shown instead.
    pub mark: Option<Mark>,
    /// The status in plain words.
    pub tooltip: String,
}

/// The status in plain words.
pub fn status_text(status: Option<&Status>, language: Language) -> String {
    let Some(status) = status else {
        return Text::Checking.get(language).to_owned();
    };
    match status {
        Status::Ready { .. } => Text::Ready.get(language).to_owned(),
        Status::NotSetUp => Text::NotSetUp.get(language).to_owned(),
        Status::InProgress { user } => Text::InProgress.fill(language, &[("user", user)]),
        Status::NeedsAttention => Text::NeedsAttention.get(language).to_owned(),
        Status::WriteProtected => Text::WriteProtected.get(language).to_owned(),
        Status::OtherAccount => Text::OtherAccount.get(language).to_owned(),
        Status::NotBackupMedium => Text::NotBackupMedium.get(language).to_owned(),
    }
}

/// What the status means and what to do about it, for any status but Ready.
fn explanation(status: &Status, language: Language) -> Option<String> {
    Some(match status {
        Status::Ready { .. } => return None,
        Status::NotSetUp => Text::ExplainNotSetUp.get(language).to_owned(),
        Status::InProgress { user } => Text::ExplainInProgress.fill(language, &[("user", user)]),
        Status::NeedsAttention => Text::ExplainNeedsAttention.get(language).to_owned(),
        Status::WriteProtected => Text::ExplainWriteProtected.get(language).to_owned(),
        Status::OtherAccount => Text::ExplainOtherAccount.get(language).to_owned(),
        Status::NotBackupMedium => Text::ExplainNotBackupMedium.get(language).to_owned(),
    })
}

/// The row for `medium`, whose check is `checked` once it is done.
pub fn row(medium: &Medium, checked: Option<&Checked>, language: Language) -> Row {
    let status = checked.map(|checked| &checked.status);
    Row {
        label: medium.volume.label.clone(),
        babawus: status.is_some_and(Status::is_babawus),
        mark: status.map(Mark::of),
        tooltip: status_text(status, language),
    }
}

/// An action for the selected medium.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    StartBackup,
    ShowHistory,
    WhatsBackedUp,
    Eject,
    Prepare,
    ChangeSetup,
}

impl Action {
    /// The order on the screen.
    pub const ALL: [Self; 6] = [
        Self::StartBackup,
        Self::ShowHistory,
        Self::WhatsBackedUp,
        Self::Eject,
        Self::Prepare,
        Self::ChangeSetup,
    ];

    /// Only shown while Advanced is on.
    pub fn for_poweruser(self) -> bool {
        matches!(self, Self::Prepare | Self::ChangeSetup)
    }

    pub fn label(self, language: Language) -> &'static str {
        match self {
            Self::StartBackup => Text::StartBackup,
            Self::ShowHistory => Text::ShowHistory,
            Self::WhatsBackedUp => Text::WhatsBackedUp,
            Self::Eject => Text::Eject,
            Self::Prepare => Text::Prepare,
            Self::ChangeSetup => Text::ChangeSetup,
        }
        .get(language)
    }

    fn tooltip(self) -> Text {
        match self {
            Self::StartBackup => Text::StartBackupTooltip,
            Self::ShowHistory => Text::ShowHistoryTooltip,
            Self::WhatsBackedUp => Text::WhatsBackedUpTooltip,
            Self::Eject => Text::EjectTooltip,
            Self::Prepare => Text::PrepareTooltip,
            Self::ChangeSetup => Text::ChangeSetupTooltip,
        }
    }
}

/// A button for an action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Button {
    pub action: Action,
    pub enabled: bool,
    pub tooltip: String,
}

/// The details of the selected medium.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Details {
    pub label: String,
    pub mark: Option<Mark>,
    pub status: String,
    /// Maker, model, size, free space and file system, for example
    /// "Kingston DataTraveler 3.0 · 15.5 GB · 9.8 GB free · exFAT".
    pub about: String,
    /// For a Ready medium: when the last backup was, and whether it skipped files.
    pub last_backup: Option<(String, bool)>,
    /// For any other status: what it means and what to do.
    pub explanation: Option<String>,
    /// The actions shown, in order.
    pub buttons: Vec<Button>,
}

/// What the start screen shows about one medium.
pub struct Shown<'a> {
    pub medium: &'a Medium,
    /// Nothing while the medium is being checked.
    pub checked: Option<&'a Checked>,
    /// Babawus may format at least one drive, so "Prepare…" can open the format view, which
    /// lists them all. See "Formatting a medium" and Issues.md item 78.
    pub formattable: bool,
}

/// The details of `shown`, at the time `now`, which `clock` turns into a date and time to
/// show. `advanced` shows the poweruser's actions too.
pub fn details(
    shown: &Shown,
    now: SystemTime,
    clock: fn(SystemTime) -> ClockTime,
    language: Language,
    advanced: bool,
) -> Details {
    let Shown {
        medium,
        checked,
        formattable,
    } = *shown;
    let status = checked.map(|checked| &checked.status);
    let drive = &medium.drive;
    let mut about = vec![];
    let maker = format!("{} {}", drive.vendor.trim(), drive.model.trim());
    if !maker.trim().is_empty() {
        about.push(maker.trim().to_owned());
    }
    about.push(size(drive.size, language));
    if let Some(free) = checked.and_then(|checked| checked.free) {
        about.push(Text::Free.fill(language, &[("size", &size(free, language))]));
    }
    if let Some(name) = file_system_name(&medium.volume.file_system) {
        about.push(name.to_owned());
    }
    let last_backup = match status {
        Some(Status::Ready { last_backup }) => Some(match last_backup {
            None => (Text::NoBackupYet.get(language).to_owned(), false),
            Some(last) => {
                let text = last_backup_text(Some(*last), now, clock, language);
                (text, last.skipped_files)
            }
        }),
        _ => None,
    };
    let ready = matches!(status, Some(Status::Ready { .. }));
    let babawus = status.is_some_and(Status::is_babawus);
    let buttons = Action::ALL
        .into_iter()
        .filter(|action| advanced || !action.for_poweruser())
        .filter(|action| match action {
            Action::StartBackup | Action::Eject => true,
            Action::ShowHistory => checked.is_some_and(|checked| checked.has_history),
            Action::WhatsBackedUp => ready,
            // What is on a stick open for another account is unknown, so it isn't offered
            // for erasing (Issues.md item 110).
            Action::Prepare => formattable && !matches!(status, Some(Status::OtherAccount)),
            Action::ChangeSetup => babawus,
        })
        .map(|action| {
            let can_start = action != Action::StartBackup || ready;
            let tooltip = if !can_start {
                let reason = status_text(status, language);
                Text::CantStart.fill(language, &[("reason", &reason)])
            } else {
                action.tooltip().get(language).to_owned()
            };
            Button {
                action,
                enabled: can_start,
                tooltip,
            }
        })
        .collect();
    Details {
        label: medium.volume.label.clone(),
        mark: status.map(Mark::of),
        status: status_text(status, language),
        about: about.join(" · "),
        last_backup,
        explanation: status.and_then(|status| explanation(status, language)),
        buttons,
    }
}

/// A USB stick that is plugged in, but isn't in the list. See Issues.md item 121.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlisted {
    /// It has a file system that isn't opened, which plugging it back in opens.
    NotOpened,
    /// It has no file system Babawus can open, so it has to be prepared first.
    CantBeUsed,
}

/// What kind of stick is plugged in but not listed, if any. A stick that isn't opened comes
/// first, because the user can do something about it.
pub fn unlisted(snapshot: &Snapshot) -> Option<Unlisted> {
    let listed: Vec<String> = snapshot
        .media()
        .into_iter()
        .map(|medium| medium.drive.object_path)
        .collect();
    let mut found = None;
    for drive in &snapshot.drives {
        if !drive.is_usb_medium() || listed.contains(&drive.object_path) {
            continue;
        }
        let on_drive = |path: &String| *path == drive.object_path;
        if snapshot
            .volumes
            .iter()
            .any(|volume| on_drive(&volume.drive))
        {
            return Some(Unlisted::NotOpened);
        }
        // A card reader without a card has a drive, but nothing with a size on it.
        if snapshot
            .blocks
            .iter()
            .any(|block| on_drive(&block.drive) && block.size > 0)
        {
            found = Some(Unlisted::CantBeUsed);
        }
    }
    found
}

/// What the page says when no medium is selected: a short heading, and what to do, if
/// anything more.
pub fn empty_text(
    udisks_running: bool,
    any_listed: bool,
    unlisted: Option<Unlisted>,
) -> (Text, Option<Text>) {
    if !udisks_running {
        (Text::NoUdisks, None)
    } else if any_listed {
        (Text::ChooseMedium, None)
    } else {
        match unlisted {
            Some(Unlisted::NotOpened) => (Text::StickNotOpened, Some(Text::StickNotOpenedAdvice)),
            Some(Unlisted::CantBeUsed) => {
                (Text::StickCantBeUsed, Some(Text::StickCantBeUsedAdvice))
            }
            None => (Text::PlugIn, None),
        }
    }
}

/// The medium to select when the list changes. `selected` is the udisks2 object path of the
/// selected medium's file system, if any. The selection follows the medium while it is in the
/// list. If nothing is selected and exactly one medium is Ready, that one is.
pub fn selection<'a>(
    media: &'a [(Medium, Option<Checked>)],
    selected: Option<&str>,
) -> Option<&'a Medium> {
    if let Some(selected) = selected
        && let Some((medium, _)) = media
            .iter()
            .find(|(medium, _)| medium.volume.object_path == selected)
    {
        return Some(medium);
    }
    let mut ready = media.iter().filter(|(_, checked)| {
        checked
            .as_ref()
            .is_some_and(|checked| matches!(checked.status, Status::Ready { .. }))
    });
    match (ready.next(), ready.next()) {
        (Some((medium, _)), None) => Some(medium),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
