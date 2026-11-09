//! What the history shows: the runs on a medium, newest first, each with when it ran and how it
//! went. See "View the history of a medium" in Requirements.md. The UI only puts these texts on
//! the screen.

use std::time::SystemTime;

use crate::log::Run;
use crate::media::user_of;
use crate::settings::Language;
use crate::text::Text;
use crate::time::ClockTime;
use crate::verdict::Reason;
use crate::verdict_screen::{VerdictMark, copied_text, files_text, reason_text, verdict_title};

/// One run in the history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRow {
    pub mark: VerdictMark,
    /// The verdict, for example "All good".
    pub verdict: String,
    /// When the run started, for example "Today, 14:03".
    pub when: String,
    /// Whose backup it was, for example "By anna on laptop".
    pub who: String,
    /// The reason in plain words, for any verdict but "All good".
    pub reason: Option<String>,
    /// What was copied, for a run that got that far.
    pub copied: Option<String>,
    /// How many files were skipped, if any.
    pub skipped: Option<String>,
}

/// The rows for `runs`, which are oldest first as in the log, at the time `now`. `in_progress`
/// is the user whose backup to the medium is running, if any: the newest run is that one, rather
/// than an interrupted one. `clock` turns a time into the date and time to show.
pub fn history(
    runs: &[Run],
    in_progress: Option<&str>,
    now: SystemTime,
    clock: fn(SystemTime) -> ClockTime,
    language: Language,
) -> Vec<HistoryRow> {
    let today = clock(now);
    runs.iter()
        .rev()
        .enumerate()
        .map(|(index, run)| {
            let running = index == 0
                && run.end.is_none()
                && in_progress.is_some_and(|user| user == user_of(&run.account));
            let who = who_text(&run.account, language);
            let when = when_text(clock(run.started), today, language);
            if running {
                return HistoryRow {
                    mark: VerdictMark::Attention,
                    verdict: Text::HistoryRunning.get(language).to_owned(),
                    when,
                    who,
                    reason: None,
                    copied: None,
                    skipped: None,
                };
            }
            let verdict = run.verdict();
            let reason = run.end.map_or(Reason::Interrupted, |end| end.reason);
            let stats = run.end.and_then(|end| end.stats);
            let copied = stats.and_then(|stats| copied_text(&stats, verdict, language));
            let skipped = stats.filter(|stats| stats.skipped_files > 0).map(|stats| {
                let files = files_text(stats.skipped_files, language);
                Text::HistorySkipped.fill(language, &[("files", &files)])
            });
            HistoryRow {
                mark: VerdictMark::of(verdict),
                verdict: verdict_title(verdict).get(language).to_owned(),
                when,
                who,
                reason: reason_text(reason).map(|text| text.get(language).to_owned()),
                copied,
                skipped,
            }
        })
        .collect()
}

/// "Today, 14:03", "Yesterday, 09:12" or "2026-11-03, 17:54". Days are calendar days, so that
/// a run just before midnight is yesterday's, however short ago it was.
fn when_text(then: ClockTime, today: ClockTime, language: Language) -> String {
    let time = format!("{:02}:{:02}", then.hour, then.minute);
    match then.days_until(today) {
        0 => Text::HistoryToday.fill(language, &[("time", &time)]),
        1 => Text::HistoryYesterday.fill(language, &[("time", &time)]),
        _ => {
            let date = format!("{:04}-{:02}-{:02}", then.year, then.month, then.day);
            Text::HistoryDate.fill(language, &[("date", &date), ("time", &time)])
        }
    }
}

/// "By anna on laptop", from `username@computer folder`.
fn who_text(account: &str, language: Language) -> String {
    let (user, computer) = account.split_once('@').unwrap_or((account, ""));
    Text::HistoryWho.fill(language, &[("user", user), ("computer", computer)])
}

#[cfg(test)]
mod tests;
