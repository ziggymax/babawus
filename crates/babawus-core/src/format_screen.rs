//! What the format view shows: the drives that may be formatted, the warning, and how
//! formatting went. See "Format USB media" and "Formatting a medium" in Requirements.md, and
//! Issues.md items 78-81. The UI only puts these texts on the screen.

use std::collections::HashSet;

use crate::format::{Refusal, label_is_valid, suggest_label};
use crate::media::{Snapshot, Volume};
use crate::settings::Language;
use crate::text::{Text, size};
use crate::udisks::{FormatError, Step};

/// A drive the poweruser can choose to format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveChoice {
    /// The udisks2 drive ID.
    pub id: String,
    /// Maker, model and size, for example "Kingston DataTraveler 3.0 · 15.5 GB".
    pub name: String,
    /// What is on it now, for example "BACKUP-A (exFAT), with Babawus backups".
    pub contents: String,
}

/// The drives that may be formatted, mounted or not. `babawus` holds the udisks2 object paths
/// of the file systems known to be Babawus media.
pub fn drive_choices(
    snapshot: &Snapshot,
    babawus: &HashSet<String>,
    language: Language,
) -> Vec<DriveChoice> {
    snapshot
        .formattable()
        .into_iter()
        .map(|target| {
            let drive = &target.drive;
            let maker = format!("{} {}", drive.vendor.trim(), drive.model.trim());
            let size = size(drive.size, language);
            let name = match maker.trim() {
                "" => size,
                maker => format!("{maker} · {size}"),
            };
            let volumes: Vec<&Volume> = snapshot
                .volumes
                .iter()
                .filter(|volume| volume.drive == drive.object_path)
                .collect();
            let contents = if volumes.is_empty() {
                Text::FormatNoFileSystem.get(language).to_owned()
            } else {
                volumes
                    .iter()
                    .map(|volume| volume_text(volume, babawus, language))
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            DriveChoice {
                id: drive.id.clone(),
                name,
                contents,
            }
        })
        .collect()
}

/// "BACKUP-A (exFAT), with Babawus backups", or "DATA (NTFS), not mounted".
fn volume_text(volume: &Volume, babawus: &HashSet<String>, language: Language) -> String {
    let label = match volume.label.trim() {
        "" => Text::FormatNoLabel.get(language),
        label => label,
    };
    let file_system = match volume.file_system.as_str() {
        "exfat" => "exFAT",
        "ext4" => "Ext4",
        "vfat" => "FAT",
        "ntfs" => "NTFS",
        other => other,
    };
    let text = format!("{label} ({file_system})");
    if babawus.contains(&volume.object_path) {
        Text::FormatWithBackups.fill(language, &[("volume", &text)])
    } else if volume.mount_point.is_none() {
        Text::FormatNotMounted.fill(language, &[("volume", &text)])
    } else {
        text
    }
}

/// The label to suggest for the drive with the udisks2 drive ID `id`: the first one that no
/// other attached medium has. The drive's own labels don't count, because they are erased.
pub fn suggested_label(snapshot: &Snapshot, id: &str) -> String {
    let own = snapshot
        .drives
        .iter()
        .find(|drive| drive.id == id)
        .map(|drive| drive.object_path.as_str());
    let others = snapshot
        .volumes
        .iter()
        .filter(|volume| Some(volume.drive.as_str()) != own)
        .map(|volume| volume.label.as_str());
    suggest_label(others)
}

/// What is wrong with `label`, if anything.
pub fn label_problem(label: &str, language: Language) -> Option<String> {
    (!label_is_valid(label)).then(|| Text::FormatLabelRule.get(language).to_owned())
}

/// The warning for formatting `choice`.
pub fn warning(choice: &DriveChoice, language: Language) -> String {
    Text::FormatWarning.fill(
        language,
        &[("drive", &choice.name), ("contents", &choice.contents)],
    )
}

/// Whether "Erase and format" can be pressed.
pub fn can_format(drive_chosen: bool, label: &str, understood: bool) -> bool {
    drive_chosen && label_is_valid(label) && understood
}

/// Why formatting failed, kept so that it can be shown again in another language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    /// The drive still holds what it held before.
    pub nothing_erased: bool,
    reason: FailureReason,
    /// What udisks2 or the system said, for the poweruser.
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailureReason {
    Gone,
    NotAllowed,
    /// It was formatted, but the folders or configuration couldn't be created.
    NotPrepared,
    Failed,
}

impl Failure {
    pub fn of(error: &FormatError) -> Self {
        let (reason, message) = match error {
            FormatError::Refused(Refusal::Gone) => (FailureReason::Gone, None),
            FormatError::InvalidLabel | FormatError::Refused(_) => {
                (FailureReason::NotAllowed, None)
            }
            FormatError::Failed { step, message } => {
                let reason = if *step == Step::Prepare {
                    FailureReason::NotPrepared
                } else {
                    FailureReason::Failed
                };
                (reason, Some(message.clone()))
            }
        };
        Self {
            nothing_erased: error.nothing_erased(),
            reason,
            message,
        }
    }
}

/// How formatting went, as shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultShown {
    pub succeeded: bool,
    pub title: String,
    /// What it means, and what to do next.
    pub detail: String,
    /// What udisks2 or the system said, for the poweruser.
    pub message: Option<String>,
}

/// The screen after formatting the medium labelled `label`. `failure` is `None` if it
/// succeeded.
pub fn result_screen(label: &str, failure: Option<&Failure>, language: Language) -> ResultShown {
    let Some(failure) = failure else {
        return ResultShown {
            succeeded: true,
            title: Text::FormatDone.fill(language, &[("label", label)]),
            detail: Text::FormatDoneDetail.get(language).to_owned(),
            message: None,
        };
    };
    let reason = match failure.reason {
        FailureReason::Gone => Text::FormatGone,
        FailureReason::NotAllowed => Text::FormatNotAllowed,
        FailureReason::NotPrepared => Text::FormatNotPrepared,
        FailureReason::Failed => Text::FormatFailedReason,
    };
    let state = if failure.nothing_erased {
        Text::FormatNothingErased
    } else {
        Text::FormatPartlyErased
    };
    ResultShown {
        succeeded: false,
        title: Text::FormatFailed.get(language).to_owned(),
        detail: format!("{} {}", reason.get(language), state.get(language)),
        message: failure
            .message
            .as_ref()
            .map(|message| Text::FormatMessage.fill(language, &[("message", message)])),
    }
}

#[cfg(test)]
mod tests;
