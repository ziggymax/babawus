//! The log of a medium: `.babawus/babawus_log.txt`.
//!
//! See "Maintain a log", "Format of the log" and "Interrupted runs in the log" in Requirements.md.
//!
//! Every function that changes the medium expects the caller to hold the medium's lock.

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fmt::Write;
use std::io::{self, ErrorKind};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::fs::FileSystem;
use crate::identity::hex;
use crate::medium::{FILE_MODE, LOG_FILE, read_file, replace_file, set_mode};
use crate::time::parse_log_timestamp;
use crate::verdict::{Reason, Verdict};
use crate::versions::Format;

/// A longer log is trimmed to this at the start of a run, which leaves room for the run itself
/// below the limit of 1,000,000 bytes.
const TRIM_LIMIT: usize = 900_000;

/// The errors of a run that are logged one by one. The rest are only counted: many errors most
/// likely share a cause, and the first ones show it (Issues.md item 124).
const MAX_LOGGED_ERRORS: u64 = 100;

/// The code for an older version that could not be removed. This doesn't change the verdict.
pub const PRUNE_FAILED: &str = "PRUNE_FAILED";
const MULTIPLE_ERRORS: &str = "MULTIPLE_ERRORS";

const START: &str = "START";
const ERROR: &str = "ERROR";
const END: &str = "END";

const RUN_ID_BYTES: usize = 8;

/// Who runs a backup, for its start entry.
#[derive(Clone, Copy, Debug)]
pub struct Runner<'s> {
    /// `username@computer folder`.
    pub account: &'s str,
    /// The computer's hostname, which the configuration only keeps as "last seen as".
    pub hostname: &'s str,
}

/// The numbers of a finished run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub scanned_files: u64,
    pub copied_files: u64,
    pub copied_bytes: u64,
    pub skipped_files: u64,
    /// Filled in by [`RunLog::end`], which has counted them.
    pub errors: u64,
    pub versions_removed: u64,
    pub free_bytes: u64,
}

/// An anomaly during a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggedError {
    pub code: String,
    /// Relative to the home folder.
    pub path: Option<PathBuf>,
    /// The number of files in a skipped folder.
    pub files: Option<u64>,
}

/// The end entry of a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct End {
    pub time: SystemTime,
    pub verdict: Verdict,
    pub reason: Reason,
    /// Not known for an interrupted run.
    pub stats: Option<Stats>,
}

/// One run, as read from the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub id: String,
    /// `username@computer folder`.
    pub account: String,
    pub started: SystemTime,
    pub errors: Vec<LoggedError>,
    pub end: Option<End>,
}

impl Run {
    /// A run with no end entry was interrupted.
    pub fn verdict(&self) -> Verdict {
        self.end.map_or(Verdict::NotFinished, |end| end.verdict)
    }

    /// Whether this run counts for "Last backup": it finished, even if files were skipped.
    pub fn counts_as_backup(&self) -> bool {
        matches!(self.verdict(), Verdict::AllGood | Verdict::Skipped)
    }
}

/// When an account last made a backup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LastBackup {
    pub time: SystemTime,
    /// The last backup skipped files, so a warning mark is shown.
    pub skipped_files: bool,
}

/// `username@computer folder`, which identifies an account in the log.
pub fn account_name(username: &str, computer_folder: &str) -> String {
    format!("{username}@{computer_folder}")
}

/// The most recent backup of `account`.
pub fn last_backup(runs: &[Run], account: &str) -> Option<LastBackup> {
    let run = runs
        .iter()
        .rev()
        .find(|run| run.account == account && run.counts_as_backup())?;
    let end = run.end?;
    Some(LastBackup {
        time: end.time,
        skipped_files: end.verdict == Verdict::Skipped,
    })
}

/// A new run ID: 16 random lower-case hexadecimal digits.
pub fn new_run_id() -> String {
    let mut bytes = [0; RUN_ID_BYTES];
    // SAFETY: `bytes` is valid for writes of its whole length.
    let written = unsafe { libc::getrandom(bytes.as_mut_ptr().cast(), bytes.len(), 0) };
    assert_eq!(
        written, RUN_ID_BYTES as isize,
        "the system gives a few random bytes without fail"
    );
    hex(&bytes, RUN_ID_BYTES)
}

fn is_run_id(id: &str) -> bool {
    id.len() == 2 * RUN_ID_BYTES
        && id
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// Writes a value so that it stays on one line, in valid UTF-8, and can be read back exactly.
pub(crate) fn escape(value: &[u8]) -> String {
    let mut text = String::new();
    for chunk in value.utf8_chunks() {
        for character in chunk.valid().chars() {
            match character {
                '\t' => text.push_str("\\t"),
                '\n' => text.push_str("\\n"),
                '\\' => text.push_str("\\\\"),
                character if character.is_ascii_control() => {
                    write!(text, "\\x{:02x}", u32::from(character))
                        .expect("writing to a string can't fail");
                }
                character => text.push(character),
            }
        }
        for byte in chunk.invalid() {
            write!(text, "\\x{byte:02x}").expect("writing to a string can't fail");
        }
    }
    text
}

/// Reads an escaped value. Returns `None` if it can't have been written by [`escape`].
fn unescape(text: &str) -> Option<Vec<u8>> {
    let mut value = Vec::with_capacity(text.len());
    let mut bytes = text.bytes();
    while let Some(byte) = bytes.next() {
        if byte != b'\\' {
            value.push(byte);
            continue;
        }
        match bytes.next()? {
            b't' => value.push(b'\t'),
            b'n' => value.push(b'\n'),
            b'\\' => value.push(b'\\'),
            b'x' => {
                let digit = |byte: u8| char::from(byte).to_digit(16);
                let high = digit(bytes.next()?)?;
                let low = digit(bytes.next()?)?;
                value.push(u8::try_from(high * 16 + low).ok()?);
            }
            _ => return None,
        }
    }
    Some(value)
}

/// One line of the log, with its line ending. Values must already be escaped.
fn entry(
    timestamp: &str,
    run_id: &str,
    kind: &str,
    pairs: &[(&str, String)],
    text: Option<&str>,
) -> String {
    let mut line = format!("{timestamp}\t{run_id}\t{kind}");
    for (key, value) in pairs {
        write!(line, "\t{key}={value}").expect("writing to a string can't fail");
    }
    if let Some(text) = text {
        line.push('\t');
        line.push_str(text);
    }
    line.push('\n');
    line
}

/// What a line of the log holds.
enum Parsed {
    Start {
        account: String,
    },
    Error(LoggedError),
    End {
        reason: Reason,
        stats: Option<Stats>,
    },
}

pub(crate) struct Line<'a> {
    time: SystemTime,
    pub(crate) run_id: &'a str,
    parsed: Parsed,
}

/// The `key=value` pairs among `fields`. Anything else is the optional text.
fn pairs<'a>(fields: impl Iterator<Item = &'a str>) -> HashMap<&'a str, &'a str> {
    fields.filter_map(|field| field.split_once('=')).collect()
}

fn parse_stats(pairs: &HashMap<&str, &str>) -> Option<Stats> {
    let number = |key: &str| pairs.get(key)?.parse().ok();
    Some(Stats {
        scanned_files: number("scanned_files")?,
        copied_files: number("copied_files")?,
        copied_bytes: number("copied_bytes")?,
        skipped_files: number("skipped_files")?,
        errors: number("errors")?,
        versions_removed: number("versions_removed")?,
        free_bytes: number("free_bytes")?,
    })
}

/// Reads a line, without its line ending. Returns `None` for a garbled line.
pub(crate) fn parse_line(line: &[u8]) -> Option<Line<'_>> {
    let mut fields = str::from_utf8(line).ok()?.split('\t');
    let time = parse_log_timestamp(fields.next()?)?;
    let run_id = fields.next().filter(|id| is_run_id(id))?;
    let kind = fields.next()?;
    let pairs = pairs(fields);
    let parsed = match kind {
        START => {
            let account = unescape(pairs.get("account")?)?;
            Parsed::Start {
                account: String::from_utf8(account).ok()?,
            }
        }
        ERROR => {
            let path = match pairs.get("path") {
                Some(path) => Some(PathBuf::from(OsString::from_vec(unescape(path)?))),
                None => None,
            };
            let files = match pairs.get("files") {
                Some(files) => Some(files.parse().ok()?),
                None => None,
            };
            Parsed::Error(LoggedError {
                code: (*pairs.get("code")?).to_owned(),
                path,
                files,
            })
        }
        END => {
            let reason = Reason::from_code(pairs.get("reason")?)?;
            Verdict::from_code(pairs.get("verdict")?)?;
            Parsed::End {
                reason,
                stats: parse_stats(&pairs),
            }
        }
        _ => return None,
    };
    Some(Line {
        time,
        run_id,
        parsed,
    })
}

/// The lines of the log, without their line endings. A last line with no line ending is always
/// incomplete, even if it looks whole.
fn complete_lines(contents: &[u8]) -> impl Iterator<Item = &[u8]> {
    contents
        .split_inclusive(|&byte| byte == b'\n')
        .filter_map(|line| line.strip_suffix(b"\n"))
}

/// Reads the runs in a log, oldest first. Incomplete and garbled lines are ignored, and so are
/// entries of runs whose start entry is gone.
pub fn parse_runs(contents: &[u8]) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    let mut positions = HashMap::new();
    for line in complete_lines(contents).filter_map(parse_line) {
        let position = positions.get(line.run_id).copied();
        match (line.parsed, position) {
            (Parsed::Start { account }, None) => {
                positions.insert(line.run_id.to_owned(), runs.len());
                runs.push(Run {
                    id: line.run_id.to_owned(),
                    account,
                    started: line.time,
                    errors: Vec::new(),
                    end: None,
                });
            }
            (Parsed::Error(error), Some(position)) => runs[position].errors.push(error),
            (Parsed::End { reason, stats }, Some(position)) => {
                runs[position].end = Some(End {
                    time: line.time,
                    verdict: reason.verdict(),
                    reason,
                    stats,
                });
            }
            _ => {}
        }
    }
    runs
}

/// Reads the runs on the medium at `root`. A medium with no log has no runs yet.
pub fn read_runs(medium: &dyn FileSystem, root: &Path) -> io::Result<Vec<Run>> {
    Ok(parse_runs(&read_log(medium, root)?))
}

pub(crate) fn read_log(medium: &dyn FileSystem, root: &Path) -> io::Result<Vec<u8>> {
    match read_file(medium, &root.join(LOG_FILE)) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        contents => contents,
    }
}

/// The end entries for runs that have none, except the run whose lock is still live.
fn closing_entries(contents: &[u8], timestamp: &str, live_run: Option<&str>) -> String {
    let pairs = [
        ("verdict", Verdict::NotFinished.code().to_owned()),
        ("reason", Reason::Interrupted.code().to_owned()),
    ];
    let open_runs = parse_runs(contents)
        .into_iter()
        .filter(|run| run.end.is_none() && live_run != Some(run.id.as_str()));
    open_runs
        .map(|run| {
            entry(
                timestamp,
                &run.id,
                END,
                &pairs,
                Some("The run was interrupted"),
            )
        })
        .collect()
}

/// The log without its oldest runs, if it has grown beyond the limit. Each account's most recent
/// backup and a live run are always kept, and garbled lines go.
fn trimmed(contents: &[u8], live_run: Option<&str>) -> Option<Vec<u8>> {
    if contents.len() <= TRIM_LIMIT {
        return None;
    }
    let runs = parse_runs(contents);
    let mut protected: HashSet<&str> = live_run.into_iter().collect();
    let mut accounts = HashSet::new();
    for run in runs.iter().rev().filter(|run| run.counts_as_backup()) {
        if accounts.insert(&run.account) {
            protected.insert(&run.id);
        }
    }

    let mut sizes: HashMap<&str, usize> = HashMap::new();
    for line in complete_lines(contents) {
        if let Some(parsed) = parse_line(line) {
            *sizes.entry(parsed.run_id).or_default() += line.len() + 1;
        }
    }
    let mut kept: HashSet<&str> = runs.iter().map(|run| run.id.as_str()).collect();
    let mut size: usize = kept.iter().map(|id| sizes[id]).sum();
    for run in &runs {
        if size <= TRIM_LIMIT {
            break;
        }
        if !protected.contains(run.id.as_str()) {
            kept.remove(run.id.as_str());
            size -= sizes[run.id.as_str()];
        }
    }

    let mut result = Vec::with_capacity(size);
    for line in complete_lines(contents) {
        if parse_line(line).is_some_and(|parsed| kept.contains(parsed.run_id)) {
            result.extend_from_slice(line);
            result.push(b'\n');
        }
    }
    Some(result)
}

/// Adds to the end of the log and flushes it.
fn append(medium: &dyn FileSystem, root: &Path, format: Format, text: &str) -> io::Result<()> {
    let path = root.join(LOG_FILE);
    let mut file = match medium.create_new(&path) {
        Ok(file) => {
            set_mode(medium, format, &path, FILE_MODE)?;
            file
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => medium.open_append(&path)?,
        Err(error) => return Err(error),
    };
    file.write_all(text.as_bytes())?;
    file.sync()
}

/// A field without the tab or line ending that follows it, and that ending.
fn split_field(field: &str) -> (&str, &str) {
    let length = field.trim_end_matches(['\t', '\n']).len();
    field.split_at(length)
}

/// Renames an account in the log, when a user account is linked to its existing entry, so that
/// its earlier runs still count. Doing this twice does no harm.
pub(crate) fn rename_account(
    medium: &dyn FileSystem,
    root: &Path,
    format: Format,
    old: &str,
    new: &str,
) -> io::Result<()> {
    let contents = read_log(medium, root)?;
    let old_field = format!("account={}", escape(old.as_bytes()));
    let new_field = format!("account={}", escape(new.as_bytes()));
    let mut renamed = Vec::with_capacity(contents.len());
    for line in contents.split_inclusive(|&byte| byte == b'\n') {
        // Only a start entry has an `account` field. Lines that aren't text stay as they are.
        match str::from_utf8(line) {
            Ok(line) => {
                let fields = line.split_inclusive('\t').map(|field| {
                    let (value, ending) = split_field(field);
                    if value == old_field {
                        format!("{new_field}{ending}")
                    } else {
                        field.to_owned()
                    }
                });
                renamed.extend(fields.flat_map(String::into_bytes));
            }
            Err(_) => renamed.extend_from_slice(line),
        }
    }
    if renamed == contents {
        return Ok(());
    }
    replace_file(medium, root, format, LOG_FILE, &renamed)
}

/// Writes the log of a run. The caller holds the medium's lock until after [`end`](Self::end).
pub struct RunLog<'a> {
    medium: &'a dyn FileSystem,
    root: &'a Path,
    format: Format,
    run_id: String,
    /// The current time for the log. See [`log_timestamp`](crate::time::log_timestamp).
    now: &'a dyn Fn() -> String,
    errors: u64,
}

impl<'a> RunLog<'a> {
    /// Starts the log of a run: closes interrupted runs, trims the log, and writes the start
    /// entry, which says who runs it, where, and with which Babawus. `.babawus/tmp/` has already
    /// been emptied. `live_run` is the run whose lock is still live, if any, and is never closed.
    pub fn start(
        medium: &'a dyn FileSystem,
        root: &'a Path,
        format: Format,
        run_id: String,
        runner: &Runner,
        now: &'a dyn Fn() -> String,
        live_run: Option<&str>,
    ) -> io::Result<Self> {
        let mut contents = read_log(medium, root)?;
        let mut addition = String::new();
        if contents.last().is_some_and(|&last| last != b'\n') {
            addition.push('\n');
        }
        addition.push_str(&closing_entries(&contents, &now(), live_run));
        if !addition.is_empty() {
            append(medium, root, format, &addition)?;
            contents.extend_from_slice(addition.as_bytes());
        }
        if let Some(trimmed) = trimmed(&contents, live_run) {
            // Trimming is housekeeping. If it fails, for example on a full medium, the next run
            // tries again. If the medium is gone, the start entry below finds out.
            let _ = replace_file(medium, root, format, LOG_FILE, &trimmed);
        }
        let log = Self {
            medium,
            root,
            format,
            run_id,
            now,
            errors: 0,
        };
        let pairs = [
            ("account", escape(runner.account.as_bytes())),
            ("version", escape(crate::VERSION.as_bytes())),
            ("hostname", escape(runner.hostname.as_bytes())),
        ];
        log.write(START, &pairs, Some("Backup started"))?;
        Ok(log)
    }

    fn write(&self, kind: &str, pairs: &[(&str, String)], text: Option<&str>) -> io::Result<()> {
        let line = entry(&(self.now)(), &self.run_id, kind, pairs, text);
        append(self.medium, self.root, self.format, &line)
    }

    /// The number of anomalies so far.
    pub fn errors(&self) -> u64 {
        self.errors
    }

    /// Records an anomaly. `path` is relative to the home folder, and `files` is the number of
    /// files it stands for. The first errors are logged one by one, and the rest only counted.
    pub fn error(&mut self, code: &str, path: &Path, files: u64) -> io::Result<()> {
        self.errors += 1;
        if self.errors > MAX_LOGGED_ERRORS + 1 {
            return Ok(());
        }
        if self.errors > MAX_LOGGED_ERRORS {
            let pairs = [("code", MULTIPLE_ERRORS.to_owned())];
            let text = "Multiple errors, further errors are only counted";
            return self.write(ERROR, &pairs, Some(text));
        }
        let mut pairs = vec![
            ("code", code.to_owned()),
            ("path", escape(path.as_os_str().as_bytes())),
        ];
        if files > 1 {
            pairs.push(("files", files.to_string()));
        }
        self.write(ERROR, &pairs, None)
    }

    /// Writes the end entry. The medium has already been flushed. If this fails, a later run
    /// closes this run as interrupted.
    pub fn end(self, reason: Reason, stats: Stats) -> io::Result<()> {
        let pairs = [
            ("verdict", reason.verdict().code().to_owned()),
            ("reason", reason.code().to_owned()),
            ("scanned_files", stats.scanned_files.to_string()),
            ("copied_files", stats.copied_files.to_string()),
            ("copied_bytes", stats.copied_bytes.to_string()),
            ("skipped_files", stats.skipped_files.to_string()),
            ("errors", self.errors.to_string()),
            ("versions_removed", stats.versions_removed.to_string()),
            ("free_bytes", stats.free_bytes.to_string()),
        ];
        self.write(END, &pairs, None)
    }
}

#[cfg(test)]
mod tests;
