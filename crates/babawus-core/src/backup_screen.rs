//! What the backup screen shows while a run works, and when it ends. See "Provide feedback
//! during a backup operation" and "Finishing a run" in Requirements.md. The UI only puts these
//! texts on the screen.

use crate::run::{Ended, Progress};
use crate::settings::Language;
use crate::text::{Text, size};
use crate::verdict::Verdict;

/// Where a run is, gathered from its progress messages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tracker {
    /// The latest message, if any has come yet.
    last: Option<Progress>,
    /// What the run is going to copy, once it is known.
    total: Option<(u64, u64)>,
    /// The user asked to stop.
    cancelling: bool,
}

/// The backup screen at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    /// What is going on, for example "Backing up…".
    pub status: String,
    /// A spinner is shown next to the status: the run is busy, but has no counters to show.
    pub busy: bool,
    /// What the run is going to copy, for example "To back up: 1,204 files (3.2 GB)".
    pub overview: Option<String>,
    pub files_left: Option<String>,
    pub bytes_left: Option<String>,
    /// How far the copying has got, from 0 to 1, by bytes.
    pub fraction: Option<f64>,
    /// Cancel can be pressed.
    pub can_cancel: bool,
}

impl Tracker {
    pub fn update(&mut self, progress: Progress) {
        if let Progress::Overview { files, bytes } = progress {
            self.total = Some((files, bytes));
        }
        self.last = Some(progress);
    }

    /// The user asked to stop. The run stops at the next safe point.
    pub fn cancel(&mut self) {
        self.cancelling = true;
    }

    pub fn cancelling(&self) -> bool {
        self.cancelling
    }

    pub fn shown(&self, language: Language) -> Shown {
        let overview = self
            .total
            .map(|(files, bytes)| overview(files, bytes, language));
        let (files_left, bytes_left) = match self.last {
            Some(Progress::Overview { files, bytes }) => (files, bytes),
            Some(Progress::Copying {
                files_left,
                bytes_left,
            }) => (files_left, bytes_left),
            _ => (0, 0),
        };
        let counting = matches!(
            self.last,
            Some(Progress::Overview { .. } | Progress::Copying { .. })
        );
        let finishing = self.last == Some(Progress::Finishing);
        let status = if finishing {
            Text::Finishing
        } else if self.cancelling {
            Text::Stopping
        } else if counting {
            Text::BackingUp
        } else {
            Text::GettingReady
        };
        let fraction = self.total.map(|(files, bytes)| {
            if finishing {
                1.0
            } else if bytes > 0 {
                (bytes - bytes_left.min(bytes)) as f64 / bytes as f64
            } else if files > 0 {
                (files - files_left.min(files)) as f64 / files as f64
            } else {
                1.0
            }
        });
        Shown {
            status: status.get(language).to_owned(),
            busy: !counting,
            overview,
            files_left: counting.then(|| {
                Text::FilesLeft.fill(language, &[("files", &number(files_left, language))])
            }),
            bytes_left: counting
                .then(|| Text::BytesLeft.fill(language, &[("size", &size(bytes_left, language))])),
            fraction,
            can_cancel: !finishing && !self.cancelling,
        }
    }
}

fn overview(files: u64, bytes: u64, language: Language) -> String {
    let size = size(bytes, language);
    if files == 1 {
        Text::OverviewOneFile.fill(language, &[("size", &size)])
    } else {
        let files = number(files, language);
        Text::Overview.fill(language, &[("files", &files), ("size", &size)])
    }
}

/// A whole number with its thousands marked, for example "1,204".
pub fn number(value: u64, language: Language) -> String {
    let separator = match language {
        Language::English => ',',
        Language::German => '.',
    };
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(separator);
        }
        grouped.push(digit);
    }
    grouped
}

/// The verdict in plain words.
pub fn verdict_text(verdict: Verdict, language: Language) -> &'static str {
    match verdict {
        Verdict::AllGood => Text::VerdictAllGood,
        Verdict::Skipped => Text::VerdictSkipped,
        Verdict::NotFinished => Text::VerdictNotFinished,
        Verdict::Failed => Text::VerdictFailed,
    }
    .get(language)
}

/// What the screen says when a run has ended. The worker may have stopped without saying how,
/// which can only be a crash, and leaves the run unfinished.
// TODO(T-015): Replace this with the verdict screen.
pub fn ended_text(ended: Option<&Ended>, language: Language) -> String {
    match ended {
        Some(Ended::Finished(outcome)) => verdict_text(outcome.verdict(), language).to_owned(),
        Some(Ended::Busy(Some(holder))) => {
            let user = crate::media::user_of(&holder.account);
            Text::InProgress.fill(language, &[("user", user)])
        }
        Some(Ended::Busy(None)) => Text::BusyStarting.get(language).to_owned(),
        None => verdict_text(Verdict::NotFinished, language).to_owned(),
    }
}

#[cfg(test)]
mod tests;
