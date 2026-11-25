//! What the verdict screen shows when a run has ended. See "Get a clear verdict", "Provide
//! feedback after a backup operation" and "Offer to eject" in Requirements.md. The UI only puts
//! these texts on the screen.

use std::time::SystemTime;

use crate::backup_screen::number;
use crate::copy::Skip;
use crate::language::Language;
use crate::log::{LastBackup, Stats};
use crate::media::user_of;
use crate::run::{Ended, Outcome};
use crate::text::{Text, size};
use crate::time::ClockTime;
use crate::verdict::{Reason, Verdict};

/// The mark next to the verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictMark {
    /// A green tick.
    Good,
    /// An amber "!": the backup is incomplete, but nothing is broken.
    Attention,
    /// A red cross: the backup can't succeed until something is fixed.
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
    /// Turns a time into the date and time to show.
    pub clock: fn(SystemTime) -> ClockTime,
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
                last_backup: last_backup_text(
                    context.previous,
                    context.now,
                    context.clock,
                    language,
                ),
                offer_eject: false,
            }
        }
        None => {
            let outcome = Outcome {
                reason: Reason::Interrupted,
                stats: Default::default(),
                skipped: Vec::new(),
                prune_failures: Vec::new(),
            };
            let mut shown = finished(&outcome, context, language);
            shown.reason = Some(Text::ReasonCrashed.get(language).to_owned());
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
        // The user stopped it on purpose, so nothing is wrong (`ui-review/UiReview.md` V05 at
        // ccbfb96).
        Verdict::NotFinished if outcome.reason == Reason::Cancelled => {
            Some(Text::NextRunAgainLater)
        }
        Verdict::NotFinished => Some(Text::NextRunAgain),
        Verdict::Skipped | Verdict::Failed => Some(Text::NextLookUp),
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
        // "Some files were skipped" says it already (`ui-review/UiReview.md` V03 at ccbfb96).
        reason: (outcome.reason != Reason::FilesSkipped)
            .then(|| reason_text(outcome.reason))
            .flatten()
            .map(|text| text.get(language).to_owned()),
        skipped: skipped_groups(outcome, language),
        next_step: next_step.map(|text| text.get(language).to_owned()),
        copied,
        last_backup: last_backup_text(previous, context.now, context.clock, language),
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
            let line = Text::SkippedGroup.fill(
                language,
                &[("reason", text.without_number(language)), ("files", &files)],
            );
            // The number ends the line rather than the reason (issue 153).
            match text.number() {
                Some(number) => format!("{line} #{number}"),
                None => line,
            }
        })
        .collect()
}

/// "Last backup: today at 14:03", "Last backup: yesterday at 09:12" or "Last backup: 47 days
/// ago (2026-09-23 at 17:54)", from when it finished, at the time `now`. Days are calendar days,
/// as `clock` shows them, and a time in the future, from a wrong clock, counts as today. See
/// issue 93.
pub(crate) fn last_backup_text(
    last: Option<LastBackup>,
    now: SystemTime,
    clock: fn(SystemTime) -> ClockTime,
    language: Language,
) -> String {
    let Some(last) = last else {
        return Text::NoBackupYet.get(language).to_owned();
    };
    let then = clock(last.time);
    let time = format!("{:02}:{:02}", then.hour, then.minute);
    match then.days_until(clock(now)).max(0) {
        0 => Text::LastBackupToday.fill(language, &[("time", &time)]),
        1 => Text::LastBackupYesterday.fill(language, &[("time", &time)]),
        days => {
            let date = format!("{:04}-{:02}-{:02}", then.year, then.month, then.day);
            Text::LastBackupDaysAgo.fill(
                language,
                &[
                    ("days", &days.to_string()),
                    ("date", &date),
                    ("time", &time),
                ],
            )
        }
    }
}

#[cfg(test)]
mod tests;
