//! The languages Babawus can show, each read from a language file at start (issues 147 to 149).
//! See "Change the language" and "Translations" in Requirements.md.
//!
//! A language file is TOML, named after its language's code, such as `de.toml`. Its `[language]`
//! table gives the code, the name shown in the Language menu, and the separators for numbers; its
//! `[texts]` table gives every [`Text`] by name. English is a file like the others, and the
//! others are checked against it: a file that lacks a text, has a placeholder that differs from
//! the English one, or can't be read as TOML is rejected, with the reason. Texts a file has that
//! Babawus doesn't know are only warned about, so that a file made for a newer Babawus still
//! works.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::fs::FileSystem;
use crate::medium::read_file;
use crate::text::Text;

/// Where an installed Babawus finds its language files.
pub const INSTALLED_FOLDER: &str = "/usr/share/babawus/languages";

/// The code of the language that every other language is checked against, and that is shown
/// when no other one fits.
pub const ENGLISH: &str = "en";

const EXTENSION: &str = "toml";

/// The folder to read the language files from: in a debug build, the source tree's `languages/`
/// folder, so that `cargo run` works; otherwise [`INSTALLED_FOLDER`] (issue 149).
pub fn folder() -> PathBuf {
    if cfg!(debug_assertions) {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages")
    } else {
        PathBuf::from(INSTALLED_FOLDER)
    }
}

/// A language that was read and checked. It is cheap to copy, and lives as long as Babawus.
#[derive(Clone, Copy)]
pub struct Language(&'static LanguageFile);

struct LanguageFile {
    code: String,
    name: String,
    decimal: String,
    thousands: String,
    /// In the order of [`Text::ALL`], each followed by its number if it has one.
    texts: Vec<String>,
    /// The same, without the numbers.
    plain: Vec<String>,
}

impl Language {
    /// The language's code, such as `de`, as in its file name and in the settings file.
    pub fn code(self) -> &'static str {
        &self.0.code
    }

    /// The name of the language in that language, so that it can be found whatever is showing.
    pub fn name(self) -> &'static str {
        &self.0.name
    }

    /// What separates the decimals in a number, such as `.` in "15.5 GB".
    pub fn decimal(self) -> &'static str {
        &self.0.decimal
    }

    /// What groups the thousands in a number, such as `,` in "1,204".
    pub fn thousands(self) -> &'static str {
        &self.0.thousands
    }

    pub(crate) fn text(self, text: Text) -> &'static str {
        &self.0.texts[text as usize]
    }

    pub(crate) fn plain_text(self, text: Text) -> &'static str {
        &self.0.plain[text as usize]
    }

    pub fn is_english(self) -> bool {
        self.code() == ENGLISH
    }
}

impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        self.code() == other.code()
    }
}

impl Eq for Language {}

impl fmt::Debug for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Language({})", self.code())
    }
}

/// The languages that can be shown.
#[derive(Clone, Debug)]
pub struct Languages {
    /// Sorted by name, as in the Language menu (issue 149).
    all: Vec<Language>,
    english: Language,
}

/// What reading the language files gave.
#[derive(Debug)]
pub struct Loaded {
    pub languages: Languages,
    /// For standard error: each rejected file and why, and each unknown text.
    pub problems: Vec<String>,
}

impl Languages {
    /// Reads every language file in `folder`. Without a usable English file nothing can be
    /// shown, so that is an error, with the reason in English.
    pub fn load(fs: &dyn FileSystem, folder: &Path) -> Result<Loaded, String> {
        let english_path = folder.join(format!("{ENGLISH}.{EXTENSION}"));
        let mut problems = Vec::new();
        let english = read(fs, &english_path, None)
            .map(|(file, warnings)| {
                problems.extend(warnings);
                Language(Box::leak(Box::new(file)))
            })
            .map_err(|reason| format!("{}: {reason}", english_path.display()))?;

        let mut names = fs.read_dir(folder).unwrap_or_default();
        names.sort();
        let mut all = vec![english];
        for name in names {
            let path = folder.join(&name);
            if path == english_path || path.extension().is_none_or(|ext| ext != EXTENSION) {
                continue;
            }
            match read(fs, &path, Some(english)) {
                Ok((file, warnings)) => {
                    problems.extend(warnings);
                    all.push(Language(Box::leak(Box::new(file))));
                }
                Err(reason) => {
                    problems.push(format!("{}: not offered, because {reason}", path.display()))
                }
            }
        }
        all.sort_by_key(|each| each.name().to_lowercase());
        Ok(Loaded {
            languages: Self { all, english },
            problems,
        })
    }

    /// Every language, sorted by name.
    pub fn all(&self) -> &[Language] {
        &self.all
    }

    pub fn english(&self) -> Language {
        self.english
    }

    /// The language with `code`, if it was loaded.
    pub fn find(&self, code: &str) -> Option<Language> {
        self.all.iter().copied().find(|each| each.code() == code)
    }

    /// The first language in `system_names` that is available, otherwise English. The names are
    /// the system's languages, best first, in forms like `de_DE.UTF-8`, `de_AT@euro` or `C`.
    pub fn from_system<'a>(&self, system_names: impl IntoIterator<Item = &'a str>) -> Language {
        system_names
            .into_iter()
            .find_map(|name| self.find(name.split(['_', '.', '@']).next()?))
            .unwrap_or(self.english)
    }
}

#[derive(Deserialize)]
struct Contents {
    language: Header,
    texts: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Header {
    code: String,
    name: String,
    decimal: String,
    thousands: String,
}

/// Reads and checks the language file at `path` against `english`, or, for English itself,
/// against nothing. Gives the file and its warnings, or why it is rejected.
fn read(
    fs: &dyn FileSystem,
    path: &Path,
    english: Option<Language>,
) -> Result<(LanguageFile, Vec<String>), String> {
    let data = read_file(fs, path).map_err(|error| format!("it can't be read ({error})"))?;
    let text = str::from_utf8(&data).map_err(|_| "it isn't UTF-8 text".to_owned())?;
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    check(stem, text, english).map(|(file, unknown)| {
        let warnings = unknown
            .into_iter()
            .map(|name| {
                format!(
                    "{}: the text {name} isn't one Babawus knows, and is left out",
                    path.display()
                )
            })
            .collect();
        (file, warnings)
    })
}

/// Checks the contents of the language file named `stem`. Gives the file and the names of the
/// texts it has that Babawus doesn't know, or why it is rejected.
fn check(
    stem: &str,
    contents: &str,
    english: Option<Language>,
) -> Result<(LanguageFile, Vec<String>), String> {
    let mut parsed: Contents =
        toml::from_str(contents).map_err(|error| format!("it isn't valid: {error}"))?;
    let header = parsed.language;
    if header.code != stem {
        return Err(format!(
            "its code \"{}\" isn't its file name, {stem}.{EXTENSION}",
            header.code
        ));
    }
    for (field, value) in [("name", &header.name), ("decimal", &header.decimal)] {
        if value.trim().is_empty() {
            return Err(format!("its {field} is empty"));
        }
    }

    let mut texts = Vec::with_capacity(Text::ALL.len());
    let mut missing = Vec::new();
    let mut wrong = Vec::new();
    for &text in Text::ALL {
        match parsed.texts.remove(text.name()) {
            Some(value) if !value.trim().is_empty() => {
                if let Some(english) = english
                    && placeholders(&value) != placeholders(english.plain_text(text))
                {
                    wrong.push(text.name());
                }
                texts.push(value);
            }
            _ => missing.push(text.name()),
        }
    }
    if !missing.is_empty() {
        return Err(format!("these texts are missing: {}", missing.join(", ")));
    }
    if !wrong.is_empty() {
        return Err(format!(
            "these texts have other placeholders than in English: {}",
            wrong.join(", ")
        ));
    }

    let numbered = Text::ALL
        .iter()
        .zip(&texts)
        .map(|(text, value)| match text.number() {
            Some(number) => format!("{value} #{number}"),
            None => value.clone(),
        })
        .collect();
    let file = LanguageFile {
        code: header.code,
        name: header.name,
        decimal: header.decimal,
        thousands: header.thousands,
        texts: numbered,
        plain: texts,
    };
    Ok((file, parsed.texts.into_keys().collect()))
}

/// The placeholders in `text`, such as `user` for `{user}`, sorted.
pub(crate) fn placeholders(text: &str) -> Vec<&str> {
    let mut found: Vec<&str> = text
        .split('{')
        .skip(1)
        .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
        .collect();
    found.sort_unstable();
    found
}

/// English and German from the source tree, for the tests.
#[cfg(test)]
pub(crate) mod testing {
    use std::sync::OnceLock;

    use super::*;

    fn parse(stem: &str, contents: &str, english: Option<Language>) -> Language {
        let (file, unknown) = check(stem, contents, english).unwrap();
        assert!(unknown.is_empty(), "{unknown:?}");
        Language(Box::leak(Box::new(file)))
    }

    pub fn english() -> Language {
        static ENGLISH: OnceLock<Language> = OnceLock::new();
        *ENGLISH.get_or_init(|| parse("en", include_str!("../../../languages/en.toml"), None))
    }

    pub fn german() -> Language {
        static GERMAN: OnceLock<Language> = OnceLock::new();
        *GERMAN.get_or_init(|| {
            parse(
                "de",
                include_str!("../../../languages/de.toml"),
                Some(english()),
            )
        })
    }

    /// English and German, as loaded from a folder.
    pub fn languages() -> Languages {
        Languages {
            all: vec![german(), english()],
            english: english(),
        }
    }
}

#[cfg(test)]
mod tests;
