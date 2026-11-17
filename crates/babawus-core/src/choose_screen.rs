//! "Choose what to back up…": a tree of the home folder with a tick box on each row, the name
//! patterns, and live totals. See "Configure USB media" and "Exclusions" in Requirements.md, and
//! issues 82 and 86-89. The UI only puts these rows and texts on the screen.

use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::backup_screen::number;
use crate::config::Account;
use crate::exclude::Exclusions;
use crate::fs::{FileKind, FileSystem};
use crate::language::Language;
use crate::text::{Text, size};

/// What an account backs up, while it is being chosen. Paths are relative to the home folder,
/// as in the configuration, and `.` is the home folder itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    pub selected: Vec<String>,
    pub exclude_folders: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// A file or folder in the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The name as shown. A name that isn't valid text is shown as well as it can be.
    pub name: String,
    /// Relative to the home folder. Empty for the home folder itself.
    pub relative: PathBuf,
    pub kind: FileKind,
    /// The whole path is valid text, so it can be stored in the configuration.
    pub text: bool,
}

/// How a row of the tree looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub ticked: bool,
    pub can_toggle: bool,
    /// A folder whose tick box is partly ticked: it is ticked with something inside it
    /// unticked by hand, or unticked with something inside it ticked. See issue 123.
    pub partly: bool,
    /// It is left out, so its name is crossed out.
    pub crossed: bool,
    /// Why it is as it is, if that isn't plain.
    pub note: Option<String>,
}

/// Why a name pattern can't be added.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternProblem {
    Empty,
    Slash,
    Duplicate,
}

/// The files and bytes chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub files: u64,
    pub bytes: u64,
}

/// A path from the configuration, without `.` parts, so that paths can be compared.
fn plain(path: &str) -> PathBuf {
    Path::new(path)
        .components()
        .filter(|part| !matches!(part, Component::CurDir))
        .collect()
}

/// A path as the configuration stores it. The home folder is `.`.
fn stored(relative: &Path) -> Option<String> {
    let text = relative.to_str()?;
    Some(if text.is_empty() {
        ".".to_owned()
    } else {
        text.to_owned()
    })
}

/// Whether `path` is the home folder or below it, with no `..` in it.
fn inside_home(path: &Path) -> bool {
    path.components()
        .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
}

impl Choice {
    pub fn of(account: &Account) -> Self {
        Self {
            selected: account.selected.clone(),
            exclude_folders: account.exclude_folders.clone(),
            exclude_patterns: account.exclude_patterns.clone(),
        }
    }

    /// Writes this choice into `account`.
    pub fn apply(&self, account: &mut Account) {
        account.selected = self.selected.clone();
        account.exclude_folders = self.exclude_folders.clone();
        account.exclude_patterns = self.exclude_patterns.clone();
    }

    fn exclusions(&self) -> Exclusions {
        Exclusions {
            patterns: self.exclude_patterns.clone(),
            folders: self
                .exclude_folders
                .iter()
                .map(|folder| plain(folder))
                .collect(),
            // TODO(T-092): The tree shows the left-out files too.
            files: Vec::new(),
        }
    }

    fn is_selected(&self, relative: &Path) -> bool {
        self.selected.iter().any(|path| plain(path) == relative)
    }

    fn is_left_out(&self, relative: &Path) -> bool {
        self.exclude_folders
            .iter()
            .any(|path| plain(path) == relative)
    }

    /// Whether one of `paths` lies strictly inside the folder `relative`.
    fn has_inside(&self, paths: &[String], relative: &Path) -> bool {
        paths
            .iter()
            .map(|path| plain(path))
            .any(|path| inside_home(&path) && path != relative && path.starts_with(relative))
    }

    /// The deepest selected folder strictly above `relative`.
    fn selected_above(&self, relative: &Path) -> Option<PathBuf> {
        self.selected
            .iter()
            .map(|path| plain(path))
            .filter(|path| inside_home(path) && relative != path && relative.starts_with(path))
            .max_by_key(|path| path.components().count())
    }

    /// Whether going from the selected folder `above` down to `relative` enters another file
    /// system, which a backup doesn't do.
    fn crosses_device(system: &dyn FileSystem, home: &Path, above: &Path, relative: &Path) -> bool {
        let Ok(start) = system.metadata(&home.join(above)) else {
            return false;
        };
        let below = relative.strip_prefix(above).unwrap_or(relative);
        let mut path = home.join(above);
        below.components().any(|part| {
            path.push(part);
            system
                .metadata(&path)
                .is_ok_and(|metadata| metadata.device != start.device)
        })
    }

    /// How the row for `entry` looks. `system` is this computer's file system, and `home` the
    /// home folder.
    pub fn row(
        &self,
        system: &dyn FileSystem,
        home: &Path,
        entry: &Entry,
        language: Language,
    ) -> Row {
        self.look(system, home, entry, Some(language))
    }

    /// The row for `entry`, with its note in `language`, or without notes if there is none.
    fn look(
        &self,
        system: &dyn FileSystem,
        home: &Path,
        entry: &Entry,
        language: Option<Language>,
    ) -> Row {
        let relative = entry.relative.as_path();
        let is_dir = entry.kind == FileKind::Dir;
        let note = |text: Text| language.map(|language| text.get(language).to_owned());
        if entry.kind == FileKind::Other {
            return Row {
                ticked: false,
                can_toggle: false,
                partly: false,
                crossed: false,
                note: note(Text::ChooseNoteLink),
            };
        }
        let exclusions = self.exclusions();
        if self.is_left_out(relative) {
            return Row {
                ticked: false,
                can_toggle: entry.text,
                partly: is_dir && self.has_inside(&self.selected, relative),
                crossed: true,
                note: note(Text::ChooseNoteLeftOut),
            };
        }
        if exclusions
            .folders
            .iter()
            .any(|folder| relative.starts_with(folder))
        {
            return Row {
                ticked: false,
                can_toggle: false,
                partly: false,
                crossed: true,
                note: note(Text::ChooseNoteLeftOutAbove),
            };
        }
        if let Some(pattern) = relative
            .iter()
            .find_map(|name| exclusions.matching_pattern(name))
        {
            return Row {
                ticked: false,
                can_toggle: false,
                partly: false,
                crossed: true,
                note: language.map(|language| {
                    Text::ChooseNotePattern.fill(language, &[("pattern", pattern)])
                }),
            };
        }
        let not_text = if entry.text {
            None
        } else {
            note(Text::ChooseNoteNotText)
        };
        if self.is_selected(relative) {
            return Row {
                ticked: true,
                can_toggle: true,
                partly: is_dir && self.has_inside(&self.exclude_folders, relative),
                crossed: false,
                note: None,
            };
        }
        if let Some(above) = self.selected_above(relative) {
            if Self::crosses_device(system, home, &above, relative) {
                return Row {
                    ticked: false,
                    can_toggle: entry.text,
                    partly: is_dir && self.has_inside(&self.selected, relative),
                    crossed: false,
                    note: not_text.or_else(|| note(Text::ChooseNoteOtherDrive)),
                };
            }
            return Row {
                ticked: true,
                // A file inside a ticked folder can't be left out on its own.
                can_toggle: entry.text && entry.kind == FileKind::Dir,
                partly: is_dir && self.has_inside(&self.exclude_folders, relative),
                crossed: false,
                note: not_text,
            };
        }
        Row {
            ticked: false,
            can_toggle: entry.text,
            partly: is_dir && self.has_inside(&self.selected, relative),
            crossed: false,
            note: not_text,
        }
    }

    /// Ticks or unticks `entry`, as described in issue 86. Returns whether anything
    /// changed.
    pub fn toggle(&mut self, system: &dyn FileSystem, home: &Path, entry: &Entry) -> bool {
        let relative = entry.relative.as_path();
        let Some(path) = stored(relative) else {
            return false;
        };
        let row = self.look(system, home, entry, None);
        if !row.can_toggle {
            return false;
        }
        if self.is_left_out(relative) {
            self.exclude_folders
                .retain(|folder| plain(folder) != relative);
            return true;
        }
        let inherited = self
            .selected_above(relative)
            .is_some_and(|above| !Self::crosses_device(system, home, &above, relative));
        if self.is_selected(relative) {
            self.selected.retain(|selected| plain(selected) != relative);
            // Still ticked through the folder above, so unticking leaves it out.
            if inherited && entry.kind == FileKind::Dir {
                self.exclude_folders.push(path);
            }
            return true;
        }
        if row.ticked {
            self.exclude_folders.push(path);
        } else {
            self.selected.push(path);
        }
        true
    }

    /// Adds a name pattern, if it can be one.
    pub fn add_pattern(&mut self, pattern: &str) -> Result<(), PatternProblem> {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return Err(PatternProblem::Empty);
        }
        if pattern.contains('/') {
            return Err(PatternProblem::Slash);
        }
        if self.exclude_patterns.iter().any(|each| each == pattern) {
            return Err(PatternProblem::Duplicate);
        }
        self.exclude_patterns.push(pattern.to_owned());
        Ok(())
    }

    pub fn remove_pattern(&mut self, pattern: &str) {
        self.exclude_patterns.retain(|each| each != pattern);
    }

    /// Whether a hidden entry at `relative` is shown although hidden entries are not: it, or
    /// something below it, is selected or left out. See issue 87.
    pub fn keeps_in_sight(&self, relative: &Path) -> bool {
        self.selected
            .iter()
            .chain(&self.exclude_folders)
            .any(|path| plain(path).starts_with(relative))
    }

    /// Counts the files and bytes chosen, as a backup would find them. Returns `None` if
    /// `cancel` was set meanwhile.
    pub fn count(
        &self,
        system: &dyn FileSystem,
        home: &Path,
        cancel: &AtomicBool,
    ) -> Option<Totals> {
        let exclusions = self.exclusions();
        let mut totals = Totals::default();
        let mut selected: Vec<PathBuf> = self.selected.iter().map(|path| plain(path)).collect();
        selected.sort();
        selected.dedup();
        for relative in &selected {
            if !inside_home(relative) || exclusions.excludes(relative) {
                continue;
            }
            // Counted already with a selected folder above it.
            let reached = selected.iter().any(|above| {
                above != relative
                    && relative.starts_with(above)
                    && !Self::crosses_device(system, home, above, relative)
            });
            if reached {
                continue;
            }
            let Ok(metadata) = system.metadata(&home.join(relative)) else {
                continue;
            };
            match metadata.kind {
                FileKind::File => {
                    totals.files += 1;
                    totals.bytes += metadata.len;
                }
                FileKind::Dir => {
                    count_folder(
                        system,
                        home,
                        relative,
                        metadata.device,
                        &exclusions,
                        cancel,
                        &mut totals,
                    )?;
                }
                FileKind::Other => {}
            }
        }
        Some(totals)
    }
}

fn count_folder(
    system: &dyn FileSystem,
    home: &Path,
    relative: &Path,
    device: u64,
    exclusions: &Exclusions,
    cancel: &AtomicBool,
    totals: &mut Totals,
) -> Option<()> {
    if cancel.load(Ordering::Relaxed) {
        return None;
    }
    let Ok(names) = system.read_dir(&home.join(relative)) else {
        return Some(());
    };
    for name in names {
        let child = relative.join(&name);
        if exclusions.excludes(&child) {
            continue;
        }
        let Ok(metadata) = system.metadata(&home.join(&child)) else {
            continue;
        };
        match metadata.kind {
            FileKind::File => {
                totals.files += 1;
                totals.bytes += metadata.len;
            }
            FileKind::Dir if metadata.device == device => {
                count_folder(system, home, &child, device, exclusions, cancel, totals)?;
            }
            FileKind::Dir | FileKind::Other => {}
        }
    }
    Some(())
}

/// The home folder itself, the root of the tree.
pub fn home_entry(language: Language) -> Entry {
    Entry {
        name: Text::CoverageHome.get(language).to_owned(),
        relative: PathBuf::new(),
        kind: FileKind::Dir,
        text: true,
    }
}

/// What is in the folder `relative`: folders first, then files, each by name without regard
/// to case. Hidden entries are left out unless `show_hidden`, or `choice` keeps them in sight.
pub fn children(
    system: &dyn FileSystem,
    home: &Path,
    relative: &Path,
    show_hidden: bool,
    choice: &Choice,
) -> io::Result<Vec<Entry>> {
    let parent_text = relative.to_str().is_some();
    let mut entries: Vec<Entry> = system
        .read_dir(&home.join(relative))?
        .into_iter()
        .filter_map(|name| {
            let child = relative.join(&name);
            let hidden = name.as_encoded_bytes().starts_with(b".");
            if hidden && !show_hidden && !choice.keeps_in_sight(&child) {
                return None;
            }
            let kind = system.metadata(&home.join(&child)).ok()?.kind;
            Some(Entry {
                name: name.to_string_lossy().into_owned(),
                text: parent_text && OsStr::to_str(&name).is_some(),
                relative: child,
                kind,
            })
        })
        .collect();
    entries.sort_by_cached_key(|entry| (entry.kind != FileKind::Dir, entry.name.to_lowercase()));
    Ok(entries)
}

/// The totals, as shown, and a warning if they are more than the free space. `None` means
/// they are still being counted.
pub fn totals_text(
    totals: Option<Totals>,
    free: Option<u64>,
    language: Language,
) -> (String, Option<String>) {
    let Some(totals) = totals else {
        return (Text::ChooseCounting.get(language).to_owned(), None);
    };
    if totals.files == 0 {
        return (Text::ChooseNothing.get(language).to_owned(), None);
    }
    let files = if totals.files == 1 {
        Text::OneFile.get(language).to_owned()
    } else {
        Text::Files.fill(language, &[("files", &number(totals.files, language))])
    };
    let chosen = size(totals.bytes, language);
    let text = match free {
        Some(free) => Text::ChooseTotals.fill(
            language,
            &[
                ("files", &files),
                ("size", &chosen),
                ("free", &size(free, language)),
            ],
        ),
        None => Text::ChooseTotalsNoFree.fill(language, &[("files", &files), ("size", &chosen)]),
    };
    let warning = free
        .is_some_and(|free| totals.bytes > free)
        .then(|| Text::ChooseTooBig.get(language).to_owned());
    (text, warning)
}

/// Why a pattern can't be added, in words.
pub fn pattern_problem_text(problem: PatternProblem, language: Language) -> &'static str {
    match problem {
        PatternProblem::Empty => Text::ChoosePatternEmpty,
        PatternProblem::Slash => Text::ChoosePatternSlash,
        PatternProblem::Duplicate => Text::ChoosePatternDuplicate,
    }
    .get(language)
}

#[cfg(test)]
mod tests;
