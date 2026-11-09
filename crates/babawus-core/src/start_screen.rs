//! What the start screen shows for each medium. See "Start Babawus and see status" in
//! Requirements.md. The UI only puts these texts and marks on the screen.

use std::ffi::CString;
use std::path::Path;
use std::time::SystemTime;

use crate::fs::FileSystem;
use crate::log::read_runs;
use crate::media::{Medium, Status, Viewer};
use crate::settings::Language;
use crate::text::{Text, size};
use crate::time::days_since;

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
        let babawus = !matches!(status, Status::NotBackupMedium);
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
            Status::NotSetUp | Status::InProgress { .. } | Status::NeedsAttention => {
                Self::Attention
            }
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
        Status::NotBackupMedium => Text::ExplainNotBackupMedium.get(language).to_owned(),
    })
}

/// The row for `medium`, whose check is `checked` once it is done.
pub fn row(medium: &Medium, checked: Option<&Checked>, language: Language) -> Row {
    let status = checked.map(|checked| &checked.status);
    Row {
        label: medium.volume.label.clone(),
        babawus: status.is_some_and(|status| !matches!(status, Status::NotBackupMedium)),
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

    /// Whether the screen this action leads to has been built yet.
    fn available(self) -> bool {
        // TODO(T-018): Change the setup.
        self != Self::ChangeSetup
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
    /// Maker, model, size and free space, for example
    /// "Kingston DataTraveler 3.0 · 15.5 GB · 9.8 GB free".
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

/// The details of `shown`, at the time `now`. `advanced` shows the poweruser's actions too.
pub fn details(shown: &Shown, now: SystemTime, language: Language, advanced: bool) -> Details {
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
    let last_backup = match status {
        Some(Status::Ready { last_backup }) => Some(match last_backup {
            None => (Text::NoBackupYet.get(language).to_owned(), false),
            Some(last) => {
                let text = match days_since(last.time, now) {
                    0 => Text::LastBackupToday.get(language).to_owned(),
                    1 => Text::LastBackupYesterday.get(language).to_owned(),
                    days => Text::LastBackupDaysAgo.fill(language, &[("days", &days.to_string())]),
                };
                (text, last.skipped_files)
            }
        }),
        _ => None,
    };
    let ready = matches!(status, Some(Status::Ready { .. }));
    let babawus = status.is_some_and(|status| !matches!(status, Status::NotBackupMedium));
    let buttons = Action::ALL
        .into_iter()
        .filter(|action| advanced || !action.for_poweruser())
        .filter(|action| match action {
            Action::StartBackup | Action::Eject => true,
            Action::ShowHistory => checked.is_some_and(|checked| checked.has_history),
            Action::WhatsBackedUp => ready,
            Action::Prepare => formattable,
            Action::ChangeSetup => babawus,
        })
        .map(|action| {
            let can_start = action != Action::StartBackup || ready;
            let tooltip = if !can_start {
                let reason = status_text(status, language);
                Text::CantStart.fill(language, &[("reason", &reason)])
            } else if !action.available() {
                Text::NotYetAvailable.get(language).to_owned()
            } else {
                action.tooltip().get(language).to_owned()
            };
            Button {
                action,
                enabled: can_start && action.available(),
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
