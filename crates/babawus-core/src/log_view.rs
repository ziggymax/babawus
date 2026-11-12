//! The log window: a stick's log one row per line, close to the file, with styles for the UI to
//! draw. See "See a stick's log" in Requirements.md and Issues.md item 135.

use std::io;
use std::path::Path;

use crate::fs::FileSystem;
use crate::log::{parse_line, read_log};
use crate::verdict::Verdict;
use crate::verdict_screen::VerdictMark;

/// How many digits of a run ID are shown: enough to tell runs apart by eye.
const SHOWN_RUN_ID: usize = 8;
/// The width of the entry type column, as wide as the longest type, `START` or `ERROR`.
const KIND_WIDTH: usize = 5;
const GAP: &str = "  ";

/// What a piece of a line is, so that the UI can give it a style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Time,
    RunId,
    Start,
    Error,
    /// An end entry, in the colour of its verdict's mark.
    End(VerdictMark),
    Key,
    Value,
    /// The closing English text for anyone reading the file.
    Text,
    /// A line that isn't a valid entry, shown as it is in the file.
    Garbled,
    /// The space between the pieces.
    Plain,
}

/// One line of the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLine {
    /// The whole run ID of a valid entry.
    pub run_id: Option<String>,
    /// The runs' lines are shaded alike, alternating from run to run in the order they started.
    pub shaded: bool,
    /// The pieces in order, which together make the line.
    pub pieces: Vec<(String, Style)>,
}

/// The lines of the log `contents`, including garbled ones and an incomplete last line.
pub fn log_lines(contents: &[u8]) -> Vec<LogLine> {
    let mut runs: Vec<&str> = Vec::new();
    let text = contents.strip_suffix(b"\n").unwrap_or(contents);
    if text.is_empty() {
        return Vec::new();
    }
    let complete = contents.ends_with(b"\n");
    let count = text.split(|&byte| byte == b'\n').count();
    text.split(|&byte| byte == b'\n')
        .enumerate()
        .map(|(index, line)| {
            let whole = complete || index + 1 < count;
            match parse_line(line).filter(|_| whole) {
                Some(parsed) => {
                    let run = match runs.iter().position(|&each| each == parsed.run_id) {
                        Some(run) => run,
                        None => {
                            runs.push(parsed.run_id);
                            runs.len() - 1
                        }
                    };
                    entry(line, run % 2 == 1)
                }
                None => LogLine {
                    run_id: None,
                    shaded: false,
                    pieces: vec![(String::from_utf8_lossy(line).into_owned(), Style::Garbled)],
                },
            }
        })
        .collect()
}

/// The lines of the log on the medium at `root`. A medium with no log has none.
pub fn read_log_lines(medium: &dyn FileSystem, root: &Path) -> io::Result<Vec<LogLine>> {
    Ok(log_lines(&read_log(medium, root)?))
}

/// A line that is a valid entry, which is valid UTF-8 with at least three fields.
fn entry(line: &[u8], shaded: bool) -> LogLine {
    let line = String::from_utf8_lossy(line);
    let mut fields = line.split('\t');
    let time = fields.next().unwrap_or_default();
    let run_id = fields.next().unwrap_or_default();
    let kind = fields.next().unwrap_or_default();
    let rest: Vec<&str> = fields.collect();
    let kind_style = match kind {
        "START" => Style::Start,
        "ERROR" => Style::Error,
        _ => {
            let verdict = rest
                .iter()
                .find_map(|field| field.strip_prefix("verdict="))
                .and_then(Verdict::from_code)
                .unwrap_or(Verdict::NotFinished);
            Style::End(VerdictMark::of(verdict))
        }
    };
    let shown_id: String = run_id.chars().take(SHOWN_RUN_ID).collect();
    let mut pieces = vec![
        (time.to_owned(), Style::Time),
        (GAP.to_owned(), Style::Plain),
        (shown_id, Style::RunId),
        (GAP.to_owned(), Style::Plain),
        (format!("{kind:<KIND_WIDTH$}"), kind_style),
    ];
    for field in rest {
        pieces.push((GAP.to_owned(), Style::Plain));
        match field.split_once('=') {
            Some((key, value)) => {
                pieces.push((key.to_owned(), Style::Key));
                pieces.push((" ".to_owned(), Style::Plain));
                pieces.push((value.to_owned(), Style::Value));
            }
            None => pieces.push((field.to_owned(), Style::Text)),
        }
    }
    LogLine {
        run_id: Some(run_id.to_owned()),
        shaded,
        pieces,
    }
}

impl LogLine {
    /// The line as shown, without its styles.
    pub fn text(&self) -> String {
        self.pieces.iter().map(|(text, _)| text.as_str()).collect()
    }
}

#[cfg(test)]
mod tests;
