//! What the verdict screen shows when a run has ended. See "Get a clear verdict", "Provide
//! feedback after a backup operation" and "Offer to eject" in Requirements.md. The UI only puts
//! these texts on the screen.

use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::time::SystemTime;

use crate::backup_screen::number;
use crate::copy::Skip;
use crate::log::{LastBackup, PRUNE_FAILED, Stats, escape};
use crate::media::user_of;
use crate::run::{Ended, Outcome};
use crate::settings::Language;
use crate::text::{Text, size};
use crate::time::{days_since, log_timestamp};
use crate::verdict::{Reason, Verdict};

/// How many problems the details list. The log has them all.
const FIRST_PROBLEMS: usize = 10;

/// The mark next to the verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictMark {
    /// A green tick.
    Good,
    /// An amber "!": the backup is incomplete, but nothing is broken.
    Attention,
    /// A red cross: the backup can't succeed without the poweruser.
    Failed,
}

impl VerdictMark {
    pub fn of(verdict: Verdict) -> Self {
        match verdict {
            Verdict::AllGood => Self::Good,
            Verdict::Skipped | Verdict::NotFinished => Self::Attention,
            Verdict::Failed => Self::Failed,
        }
    }
}

/// The verdict screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerdictShown {
    pub mark: VerdictMark,
    /// The verdict, for example "Some files were skipped".
    pub title: String,
    /// The reason in plain words, for any verdict but "All good".
    pub reason: Option<String>,
    /// The skipped files, grouped by reason, for example "Couldn't be read: 3 files".
    pub skipped: Vec<String>,
    /// What to do next, if anything.
    pub next_step: Option<String>,
    /// What was copied, for a run that got that far.
    pub copied: Option<String>,
    /// For example "Last backup: today".
    pub last_backup: String,
    /// "Details for your poweruser", to copy. Empty if the run did nothing.
    pub details: String,
    /// The run counts as a backup, so Babawus offers to eject the medium.
    pub offer_eject: bool,
}

/// What the screen needs to know besides how the run ended.
pub struct Context<'a> {
    /// The medium's label.
    pub label: &'a str,
    /// The account's last backup before this run.
    pub previous: Option<LastBackup>,
    /// When the run ended.
    pub now: SystemTime,
}

/// The verdict screen for a run that ended as `ended`. Nothing means the worker stopped without
/// saying how, which can only be a crash.
pub fn verdict_screen(
    ended: Option<&Ended>,
    context: &Context,
    language: Language,
) -> VerdictShown {
    match ended {
        Some(Ended::Finished(outcome)) => finished(outcome, context, language),
        Some(Ended::Busy(holder)) => {
            let title = match holder {
                Some(holder) => {
                    let user = user_of(&holder.account);
                    Text::InProgress.fill(language, &[("user", user)])
                }
                None => Text::VerdictBusy.get(language).to_owned(),
            };
            VerdictShown {
                mark: VerdictMark::Attention,
                title,
                reason: None,
                skipped: Vec::new(),
                next_step: Some(Text::NextWait.get(language).to_owned()),
                copied: None,
                last_backup: last_backup_text(context.previous, context.now, language),
                details: String::new(),
                offer_eject: false,
            }
        }
        None => {
            let outcome = Outcome {
                reason: Reason::Interrupted,
                stats: Default::default(),
                skipped: Vec::new(),
                prune_failures: Vec::new(),
                account: None,
            };
            let mut shown = finished(&outcome, context, language);
            shown.reason = Some(Text::ReasonCrashed.get(language).to_owned());
            shown.details = details(&outcome, context, Some(&Text::ReasonCrashed), language);
            shown
        }
    }
}

fn finished(outcome: &Outcome, context: &Context, language: Language) -> VerdictShown {
    let verdict = outcome.verdict();
    let counts = matches!(verdict, Verdict::AllGood | Verdict::Skipped);
    let mark = VerdictMark::of(verdict);
    let next_step = match verdict {
        Verdict::AllGood => None,
        Verdict::NotFinished => Some(Text::NextRunAgain),
        Verdict::Skipped | Verdict::Failed => Some(Text::NextContact),
    };
    let copied = copied_text(&outcome.stats, verdict, language);
    let previous = if counts {
        Some(LastBackup {
            time: context.now,
            skipped_files: verdict == Verdict::Skipped,
        })
    } else {
        context.previous
    };
    VerdictShown {
        mark,
        title: verdict_title(verdict).get(language).to_owned(),
        reason: reason_text(outcome.reason).map(|text| text.get(language).to_owned()),
        skipped: skipped_groups(outcome, language),
        next_step: next_step.map(|text| text.get(language).to_owned()),
        copied,
        last_backup: last_backup_text(previous, context.now, language),
        details: details(outcome, context, None, language),
        offer_eject: counts,
    }
}

/// What a run copied, for a run that got that far: "Copied: 12 files (3.4 MB)", or "Everything
/// was already up to date."
pub(crate) fn copied_text(stats: &Stats, verdict: Verdict, language: Language) -> Option<String> {
    (stats.scanned_files > 0 || stats.copied_files > 0).then(|| {
        if stats.copied_files == 0 && verdict == Verdict::AllGood {
            Text::NothingToCopy.get(language).to_owned()
        } else {
            let files = files_text(stats.copied_files, language);
            let size = size(stats.copied_bytes, language);
            Text::Copied.fill(language, &[("files", &files), ("size", &size)])
        }
    })
}

pub(crate) fn verdict_title(verdict: Verdict) -> Text {
    match verdict {
        Verdict::AllGood => Text::VerdictAllGood,
        Verdict::Skipped => Text::VerdictSkipped,
        Verdict::NotFinished => Text::VerdictNotFinished,
        Verdict::Failed => Text::VerdictFailed,
    }
}

/// The reason in plain words. "All good" needs none.
pub(crate) fn reason_text(reason: Reason) -> Option<Text> {
    Some(match reason {
        Reason::None => return None,
        Reason::FilesSkipped => Text::ReasonFilesSkipped,
        Reason::Cancelled => Text::ReasonCancelled,
        Reason::MediumRemoved => Text::ReasonMediumRemoved,
        Reason::Interrupted => Text::ReasonInterrupted,
        Reason::FlushFailed => Text::ReasonFlushFailed,
        Reason::MediumFull => Text::ReasonMediumFull,
        Reason::NotAdded => Text::ReasonNotAdded,
        Reason::ConfigDamaged => Text::ReasonConfigDamaged,
        Reason::CannotWrite => Text::ReasonCannotWrite,
    })
}

fn skip_text(skip: Skip) -> Text {
    match skip {
        Skip::Unreadable => Text::SkipUnreadable,
        Skip::Disappeared => Text::SkipDisappeared,
        Skip::NotWritable | Skip::WriteError => Text::SkipNotWritable,
        Skip::TypeConflict => Text::SkipTypeConflict,
        Skip::ChangedDuringCopy => Text::SkipChangedDuringCopy,
        Skip::VersionNameTooLong => Text::SkipVersionNameTooLong,
        Skip::NameInvalidChars => Text::SkipNameInvalidChars,
        Skip::NameTrailingDot => Text::SkipNameTrailingDot,
        Skip::NameNotUtf8 => Text::SkipNameNotUtf8,
        Skip::PathTooLong => Text::SkipPathTooLong,
        Skip::VersionFolderForm => Text::SkipVersionFolderForm,
        Skip::CaseClash => Text::SkipCaseClash,
        Skip::SelectionMissing => Text::SkipSelectionMissing,
        Skip::OutsideHome => Text::SkipOutsideHome,
    }
}

/// "1 file" or "3 files".
pub(crate) fn files_text(files: u64, language: Language) -> String {
    if files == 1 {
        Text::OneFile.get(language).to_owned()
    } else {
        Text::Files.fill(language, &[("files", &number(files, language))])
    }
}

/// The skipped files, grouped by reason, in the order the reasons first came up.
fn skipped_groups(outcome: &Outcome, language: Language) -> Vec<String> {
    let mut groups: Vec<(Text, u64)> = Vec::new();
    for skipped in &outcome.skipped {
        let text = skip_text(skipped.reason);
        // A folder that was skipped as a whole counts its files, and at least one.
        let files = skipped.files.max(1);
        match groups.iter_mut().find(|(each, _)| *each == text) {
            Some((_, count)) => *count += files,
            None => groups.push((text, files)),
        }
    }
    groups
        .into_iter()
        .map(|(text, files)| {
            let files = files_text(files, language);
            Text::SkippedGroup.fill(
                language,
                &[("reason", text.get(language)), ("files", &files)],
            )
        })
        .collect()
}

fn last_backup_text(last: Option<LastBackup>, now: SystemTime, language: Language) -> String {
    let Some(last) = last else {
        return Text::NoBackupYet.get(language).to_owned();
    };
    match days_since(last.time, now) {
        0 => Text::LastBackupToday.get(language).to_owned(),
        1 => Text::LastBackupYesterday.get(language).to_owned(),
        days => Text::LastBackupDaysAgo.fill(language, &[("days", &days.to_string())]),
    }
}

/// "Details for your poweruser": the verdict, the reason, the date and time, the account, the
/// medium's label, the counts and the first problems. Codes are added as in the log, so that the
/// poweruser can look them up whatever the language.
fn details(
    outcome: &Outcome,
    context: &Context,
    reason_override: Option<&Text>,
    language: Language,
) -> String {
    let verdict = outcome.verdict();
    let stats = &outcome.stats;
    let mut lines = vec![format!("Babawus {}", crate::VERSION)];
    let verdict_text = verdict_title(verdict).get(language);
    lines.push(Text::DetailsVerdict.fill(
        language,
        &[("verdict", verdict_text), ("code", verdict.code())],
    ));
    let reason = reason_override
        .copied()
        .or_else(|| reason_text(outcome.reason));
    if let Some(reason) = reason {
        lines.push(Text::DetailsReason.fill(
            language,
            &[
                ("reason", reason.get(language)),
                ("code", outcome.reason.code()),
            ],
        ));
    }
    lines.push(Text::DetailsTime.fill(language, &[("time", &log_timestamp(context.now))]));
    if let Some(account) = &outcome.account {
        lines.push(Text::DetailsAccount.fill(language, &[("account", account)]));
    }
    lines.push(Text::DetailsMedium.fill(language, &[("label", context.label)]));
    lines.push(Text::DetailsCounts.fill(
        language,
        &[
            ("scanned", &number(stats.scanned_files, language)),
            ("copied", &number(stats.copied_files, language)),
            ("size", &size(stats.copied_bytes, language)),
            ("skipped", &number(stats.skipped_files, language)),
            ("errors", &number(stats.errors, language)),
            ("removed", &number(stats.versions_removed, language)),
        ],
    ));
    if stats.free_bytes > 0 {
        let free = size(stats.free_bytes, language);
        lines.push(Text::DetailsFree.fill(language, &[("size", &free)]));
    }
    let problems: Vec<String> = outcome
        .skipped
        .iter()
        .map(|skipped| {
            let path = shown_path(&skipped.relative);
            if skipped.files > 1 {
                let files = files_text(skipped.files, language);
                format!("  {} {path} ({files})", skipped.reason.code())
            } else {
                format!("  {} {path}", skipped.reason.code())
            }
        })
        .chain(
            outcome
                .prune_failures
                .iter()
                .map(|path| format!("  {PRUNE_FAILED} {}", shown_path(path))),
        )
        .collect();
    if !problems.is_empty() {
        lines.push(Text::DetailsProblems.get(language).to_owned());
        lines.extend(problems.iter().take(FIRST_PROBLEMS).cloned());
        if problems.len() > FIRST_PROBLEMS {
            let more = number((problems.len() - FIRST_PROBLEMS) as u64, language);
            lines.push(Text::DetailsMore.fill(language, &[("more", &more)]));
        }
    }
    lines.join("\n")
}

/// A path as in the log: escaped, so that any name can be shown.
fn shown_path(path: &Path) -> String {
    escape(path.as_os_str().as_bytes())
}

#[cfg(test)]
mod tests;
