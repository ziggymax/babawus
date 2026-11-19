//! What the Restore window shows: where the user's backup is on a stick, and the texts of its
//! rows, versions, progress and result. See "Restore files" in Requirements.md and issue 161.
//! The UI only puts these texts on the screen; the restoring itself is [`crate::restore`].

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::backup_screen::number;
use crate::choose_screen::{self, Totals};
use crate::config::{self, Loaded};
use crate::fs::FileSystem;
use crate::language::Language;
use crate::media::Viewer;
use crate::restore::{Copy, Failure, Item, Outcome, Row, Version};
use crate::scan::account_folder;
use crate::text::{Text, size};
use crate::time::ClockTime;

impl Viewer<'_> {
    /// The folder of the user's backup profile on the stick whose top folder is `stick`, reached
    /// through `files`, or `None` if the setup can't be read or this account isn't on it.
    pub fn backup_folder(&self, files: &dyn FileSystem, stick: &Path) -> Option<PathBuf> {
        let Ok(Loaded::Ready(config)) = config::load(files, stick) else {
            return None;
        };
        let (computer, _) = config.account(&self.this.computer_id, self.username)?;
        Some(stick.join(account_folder(&computer.folder, self.username)))
    }
}

/// A date and time as the Restore window shows it: `2026-11-03 17:54`.
pub fn date_text(time: ClockTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        time.year, time.month, time.day, time.hour, time.minute
    )
}

/// The note after a row's name, if any: why it is left out, which version is picked, and
/// whether it is still on the computer. `picked` is the time of the older version picked for a
/// ticked file.
pub fn row_note(
    item: &Item,
    row: &Row,
    picked: Option<ClockTime>,
    language: Language,
) -> Option<String> {
    let mut parts = Vec::new();
    if row.left_out_above {
        parts.push(Text::ChooseNoteLeftOutAbove.get(language).to_owned());
    } else if row.crossed {
        parts.push(Text::ChooseNoteLeftOut.get(language).to_owned());
    } else if let Some(picked) = picked.filter(|_| row.ticked) {
        parts.push(Text::RestoreNoteVersion.fill(language, &[("when", &date_text(picked))]));
    }
    if item.only_older {
        parts.push(Text::RestoreNoteOnlyOlder.get(language).to_owned());
    } else if !item.on_computer {
        parts.push(Text::RestoreNoteNotHere.get(language).to_owned());
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// A version in the side pane: "Newest copy, changed 2026-11-03 17:54 · 1.2 kB", or
/// "Replaced 2026-11-02 08:00 · 1.1 kB".
pub fn version_text(
    version: &Version,
    clock: fn(SystemTime) -> ClockTime,
    language: Language,
) -> String {
    let when = date_text(clock(version.time));
    let what = match version.copy {
        Copy::Newest => Text::RestoreVersionNewest,
        Copy::Older(_) => Text::RestoreVersionOlder,
    };
    format!(
        "{} · {}",
        what.fill(language, &[("when", &when)]),
        size(version.len, language)
    )
}

/// The count below the tree: "Chosen: 12 files (3.4 MB)", the text "Choose what to back up"
/// shows without a stick, or "Counting…" while `totals` is `None` (issue 167).
pub fn totals_text(totals: Option<Totals>, language: Language) -> String {
    choose_screen::totals_text(totals, None, language).0
}

/// "Restoring… 3 of 12 files".
pub fn progress_text(done: u64, total: usize, language: Language) -> String {
    Text::RestoreProgress.fill(
        language,
        &[
            ("done", &number(done, language)),
            ("total", &number(total as u64, language)),
        ],
    )
}

/// The question asked once, before anything is copied, when `already` files are there: the
/// heading and the detail.
pub fn question(already: usize, language: Language) -> (String, String) {
    let heading = if already == 1 {
        Text::RestoreQuestionOne.get(language).to_owned()
    } else {
        Text::RestoreQuestion.fill(language, &[("count", &number(already as u64, language))])
    };
    (
        heading,
        Text::RestoreQuestionDetail.get(language).to_owned(),
    )
}

/// "1 file" or "12 files".
fn files_text(files: u64, language: Language) -> String {
    if files == 1 {
        Text::OneFile.get(language).to_owned()
    } else {
        Text::Files.fill(language, &[("files", &number(files, language))])
    }
}

/// Why a file wasn't restored.
pub fn failure_text(failure: Failure, language: Language) -> &'static str {
    match failure {
        Failure::InTheWay => Text::RestoreInTheWay,
        Failure::Unreadable => Text::RestoreUnreadable,
        Failure::NotWritable => Text::RestoreNotWritable,
        Failure::OnTheStick => Text::RestoreOnStick,
    }
    .get(language)
}

/// How a restore went, as shown below the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultShown {
    /// "12 files restored.", "10 of 12 files restored.", or "Nothing was restored.", after
    /// "The restore was stopped." if it was.
    pub summary: String,
    /// The files skipped because they were already there, if any.
    pub skipped: Option<String>,
    /// "The disk is full, so the restore stopped. #60", if it is.
    pub full: Option<String>,
    /// "These weren't restored:", if any weren't.
    pub failed_heading: Option<String>,
    /// One line for each file that wasn't restored, with the reason.
    pub failed: Vec<String>,
}

/// The result of restoring `total` files.
pub fn result_text(outcome: &Outcome, total: usize, language: Language) -> ResultShown {
    let total = total as u64;
    let mut summary = if outcome.restored == 0 {
        Text::RestoreNone.get(language).to_owned()
    } else if outcome.restored + outcome.skipped == total {
        Text::RestoreDone.fill(
            language,
            &[("files", &files_text(outcome.restored, language))],
        )
    } else {
        Text::RestoreSome.fill(
            language,
            &[
                ("done", &number(outcome.restored, language)),
                ("total", &number(total, language)),
            ],
        )
    };
    if outcome.cancelled {
        summary = format!("{} {summary}", Text::RestoreStopped.get(language));
    }
    let failed: Vec<String> = outcome
        .failed
        .iter()
        .map(|(path, failure)| {
            Text::RestoreFailedLine.fill(
                language,
                &[
                    ("path", &path.to_string_lossy()),
                    ("reason", failure_text(*failure, language)),
                ],
            )
        })
        .collect();
    ResultShown {
        summary,
        skipped: (outcome.skipped > 0).then(|| {
            Text::RestoreSkipped.fill(
                language,
                &[("files", &files_text(outcome.skipped, language))],
            )
        }),
        full: outcome
            .full
            .then(|| Text::RestoreFull.get(language).to_owned()),
        failed_heading: (!failed.is_empty())
            .then(|| Text::RestoreNotRestored.get(language).to_owned()),
        failed,
    }
}

#[cfg(test)]
mod tests;
