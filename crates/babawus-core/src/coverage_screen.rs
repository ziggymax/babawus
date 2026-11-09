//! What "What's backed up" shows: which files and folders on this computer are backed up to a
//! medium for the user's account, and what is left out. See "See which files and folders a
//! medium covers" in Requirements.md. The UI only puts these texts on the screen.

use std::io::ErrorKind;
use std::path::{Component, Path};

use crate::config::{self, Loaded};
use crate::fs::{FileKind, FileSystem};
use crate::media::Viewer;
use crate::settings::Language;
use crate::text::Text;

/// What a selected file or folder is, on this computer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The whole home folder is selected.
    Home,
    Folder,
    File,
    /// It doesn't exist any more.
    Missing,
    /// An exclusion leaves it out.
    Excluded,
    /// It is outside the home folder.
    OutsideHome,
    /// A symbolic link or special file, which Babawus doesn't follow or copy.
    Link,
    Unreadable,
}

/// A file or folder selected for backup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    /// Relative to the home folder, as in the configuration.
    pub path: String,
    pub kind: Kind,
}

/// What the account backs up, as read from the medium and this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coverage {
    /// In the order of the configuration.
    pub selected: Vec<Selected>,
    pub exclude_patterns: Vec<String>,
    pub exclude_folders: Vec<String>,
}

/// What reading the medium found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gathered {
    Covered(Coverage),
    /// This computer or account is no longer in the configuration.
    NotSetUp,
    /// The configuration can't be read or used.
    Unreadable,
}

impl Viewer<'_> {
    /// What the account backs up to the medium at `root`, whose file system is reached through
    /// `files`, from the home folder `home`. This only reads.
    pub fn coverage(&self, files: &dyn FileSystem, root: &Path, home: &Path) -> Gathered {
        let config = match config::load(files, root) {
            Ok(Loaded::Ready(config)) => config,
            _ => return Gathered::Unreadable,
        };
        let Some((_, account)) = config.account(&self.this.computer_id, self.username) else {
            return Gathered::NotSetUp;
        };
        let exclusions = account.selection(home).exclusions;
        let selected = account
            .selected
            .iter()
            .map(|path| {
                let relative = Path::new(path);
                let kind = if relative.components().all(|part| part == Component::CurDir) {
                    Kind::Home
                } else if !relative
                    .components()
                    .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
                {
                    Kind::OutsideHome
                } else if exclusions.excludes(relative) {
                    Kind::Excluded
                } else {
                    match self.system.metadata(&home.join(relative)) {
                        Ok(metadata) => match metadata.kind {
                            FileKind::Dir => Kind::Folder,
                            FileKind::File => Kind::File,
                            FileKind::Other => Kind::Link,
                        },
                        Err(error) if error.kind() == ErrorKind::NotFound => Kind::Missing,
                        Err(_) => Kind::Unreadable,
                    }
                };
                Selected {
                    path: path.clone(),
                    kind,
                }
            })
            .collect();
        Gathered::Covered(Coverage {
            selected,
            exclude_patterns: account.exclude_patterns.clone(),
            exclude_folders: account.exclude_folders.clone(),
        })
    }
}

/// Which picture goes next to a selected file or folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Picture {
    Folder,
    File,
    /// Selected, but nothing is backed up from it.
    NotBackedUp,
}

/// A selected file or folder, as shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub picture: Picture,
    pub name: String,
    /// Why nothing is backed up from it, if so.
    pub note: Option<String>,
}

/// "What's backed up".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageShown {
    /// "These are backed up from this computer for anna:", or what went wrong.
    pub intro: String,
    pub items: Vec<Item>,
    /// "Left out:", if anything is.
    pub left_out_heading: Option<String>,
    /// The excluded folders, then the name patterns in one line.
    pub left_out: Vec<String>,
}

/// "What's backed up" for `gathered`, for the user `username`.
pub fn coverage_screen(gathered: &Gathered, username: &str, language: Language) -> CoverageShown {
    let coverage = match gathered {
        Gathered::Covered(coverage) => coverage,
        Gathered::NotSetUp | Gathered::Unreadable => {
            let text = match gathered {
                Gathered::NotSetUp => Text::ExplainNotSetUp,
                _ => Text::CoverageUnreadable,
            };
            return CoverageShown {
                intro: text.get(language).to_owned(),
                items: Vec::new(),
                left_out_heading: None,
                left_out: Vec::new(),
            };
        }
    };
    let intro = if coverage.selected.is_empty() {
        Text::CoverageNothing.get(language).to_owned()
    } else {
        Text::CoverageIntro.fill(language, &[("user", username)])
    };
    let items = coverage
        .selected
        .iter()
        .map(|selected| {
            let (picture, note) = match selected.kind {
                Kind::Home | Kind::Folder => (Picture::Folder, None),
                Kind::File => (Picture::File, None),
                Kind::Missing => (Picture::NotBackedUp, Some(Text::CoverageMissing)),
                Kind::Excluded => (Picture::NotBackedUp, Some(Text::CoverageExcluded)),
                Kind::OutsideHome => (Picture::NotBackedUp, Some(Text::CoverageOutsideHome)),
                Kind::Link => (Picture::NotBackedUp, Some(Text::CoverageLink)),
                Kind::Unreadable => (Picture::NotBackedUp, Some(Text::CoverageUnreadableItem)),
            };
            let name = match selected.kind {
                Kind::Home => Text::CoverageHome.get(language).to_owned(),
                _ => selected.path.clone(),
            };
            Item {
                picture,
                name,
                note: note.map(|note| note.get(language).to_owned()),
            }
        })
        .collect();
    let mut left_out: Vec<String> = coverage.exclude_folders.clone();
    if !coverage.exclude_patterns.is_empty() {
        let names = either(&coverage.exclude_patterns, language);
        left_out.push(Text::CoveragePatterns.fill(language, &[("names", &names)]));
    }
    CoverageShown {
        intro,
        items,
        left_out_heading: (!left_out.is_empty())
            .then(|| Text::CoverageLeftOut.get(language).to_owned()),
        left_out,
    }
}

/// "a", "a or b", or "a, b or c".
fn either(names: &[String], language: Language) -> String {
    match names.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => {
            let first = rest.join(", ");
            Text::Either.fill(language, &[("first", &first), ("last", last)])
        }
    }
}

#[cfg(test)]
mod tests;
