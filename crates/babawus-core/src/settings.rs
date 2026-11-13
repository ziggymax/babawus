//! The settings of a user on this computer. See "Change the language" in Requirements.md.
//!
//! Settings are a convenience, so a settings file that can't be read never stops Babawus: it is
//! simply as if no choice had been made.

use std::ffi::OsString;
use std::io::{self, ErrorKind};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::fs::FileSystem;
use crate::medium::read_file;

/// The settings file, in the user's configuration folder.
pub const SETTINGS_FILE: &str = "babawus/settings.toml";

const TEMP_SUFFIX: &str = ".tmp";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "de")]
    German,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::English, Self::German];

    /// The name of the language in that language, so that it can be found whatever is showing.
    pub fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::German => "Deutsch",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::German => "de",
        }
    }

    /// The first language in `system_names` that is available, otherwise English. The names are
    /// the system's languages, best first, in forms like `de_DE.UTF-8`, `de_AT@euro` or `C`.
    pub fn from_system<'a>(system_names: impl IntoIterator<Item = &'a str>) -> Self {
        system_names
            .into_iter()
            .find_map(|name| {
                let code = name.split(['_', '.', '@']).next()?;
                Self::ALL
                    .into_iter()
                    .find(|language| language.code() == code)
            })
            .unwrap_or(Self::English)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The language the user has chosen, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
}

impl Settings {
    /// Reads the settings file at `path`. A file that isn't there, or can't be understood, gives
    /// the default settings.
    pub fn load(fs: &dyn FileSystem, path: &Path) -> Self {
        let contents = read_file(fs, path).unwrap_or_default();
        str::from_utf8(&contents)
            .ok()
            .and_then(|text| toml::from_str(text).ok())
            .unwrap_or_default()
    }

    /// The language to show: the user's choice, otherwise the system's. See
    /// [`Language::from_system`].
    pub fn language<'a>(&self, system_names: impl IntoIterator<Item = &'a str>) -> Language {
        self.language
            .unwrap_or_else(|| Language::from_system(system_names))
    }

    /// Writes the settings file at `path`, and any folders it needs. The file is never written in
    /// place, so an interruption leaves either the old settings or the new ones.
    pub fn save(&self, fs: &dyn FileSystem, path: &Path) -> io::Result<()> {
        let text = toml::to_string(self).map_err(io::Error::other)?;
        save_user_file(fs, path, text.as_bytes())
    }
}

/// Writes a small file of the user's at `path`, and any folders it needs. The file is never
/// written in place, so an interruption leaves either the old file or the new one.
pub(crate) fn save_user_file(fs: &dyn FileSystem, path: &Path, data: &[u8]) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        create_folders(fs, folder)?;
    }
    let mut temp = OsString::from(path);
    temp.push(TEMP_SUFFIX);
    let temp = Path::new(&temp);
    match fs.remove_file(temp) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    let result = write_file(fs, temp, data).and_then(|()| fs.rename(temp, path));
    if result.is_err() {
        // If this fails too, the next save removes it.
        let _ = fs.remove_file(temp);
    }
    result
}

fn write_file(fs: &dyn FileSystem, path: &Path, data: &[u8]) -> io::Result<()> {
    let mut file = fs.create_new(path)?;
    file.write_all(data)?;
    file.sync()
}

/// Creates the folder `path`, and those above it, as far as they are missing.
fn create_folders(fs: &dyn FileSystem, path: &Path) -> io::Result<()> {
    match fs.create_dir(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                create_folders(fs, parent)?;
            }
            fs.create_dir(path)
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use super::*;
    use crate::fs::RealFs;
    use crate::fs::fake::FakeFs;
    use crate::log::new_run_id;

    const GERMAN: Settings = Settings {
        language: Some(Language::German),
    };
    const ENGLISH: Settings = Settings {
        language: Some(Language::English),
    };

    fn path() -> PathBuf {
        PathBuf::from("/home/usera/.config/babawus/settings.toml")
    }

    fn fake() -> FakeFs {
        let fake = FakeFs::new();
        fake.add_file("/home/usera/file", b"", SystemTime::UNIX_EPOCH);
        fake
    }

    fn fake_with(contents: &[u8]) -> FakeFs {
        let fake = fake();
        fake.add_file(path(), contents, SystemTime::UNIX_EPOCH);
        fake
    }

    #[test]
    fn german_is_found_in_the_forms_systems_use() {
        for name in ["de", "de_DE", "de_DE.UTF-8", "de_AT@euro", "de.UTF-8"] {
            assert_eq!(Language::from_system([name]), Language::German, "{name}");
        }
    }

    #[test]
    fn an_unavailable_system_language_gives_english() {
        for names in [&["fr_FR.UTF-8"][..], &["C"], &["den_XX"], &[""], &[]] {
            let language = Language::from_system(names.iter().copied());
            assert_eq!(language, Language::English, "{names:?}");
        }
    }

    #[test]
    fn the_first_available_system_language_wins() {
        let names = ["fr_FR", "fr", "de_DE", "en_GB", "C"];
        assert_eq!(Language::from_system(names), Language::German);

        let names = ["en_GB", "de_DE"];
        assert_eq!(Language::from_system(names), Language::English);
    }

    #[test]
    fn the_users_choice_goes_before_the_system_language() {
        assert_eq!(ENGLISH.language(["de_DE"]), Language::English);
    }

    #[test]
    fn without_a_choice_the_system_language_is_used() {
        assert_eq!(Settings::default().language(["de_DE"]), Language::German);
    }

    #[test]
    fn languages_have_their_own_names() {
        let names = Language::ALL.map(Language::name);

        assert_eq!(names, ["English", "Deutsch"]);
    }

    #[test]
    fn saved_settings_can_be_read_back() {
        for settings in [GERMAN, ENGLISH, Settings::default()] {
            let fake = fake();

            settings.save(&fake, &path()).unwrap();

            assert_eq!(Settings::load(&fake, &path()), settings);
        }
    }

    #[test]
    fn the_file_has_the_form_in_the_requirements() {
        let fake = fake();

        GERMAN.save(&fake, &path()).unwrap();

        assert_eq!(read_file(&fake, &path()).unwrap(), b"language = \"de\"\n");
    }

    #[test]
    fn saving_replaces_earlier_settings_and_leaves_no_temporary_file() {
        let fake = fake();
        GERMAN.save(&fake, &path()).unwrap();

        ENGLISH.save(&fake, &path()).unwrap();

        assert_eq!(Settings::load(&fake, &path()), ENGLISH);
        let names = fake.read_dir(path().parent().unwrap()).unwrap();
        assert_eq!(names, ["settings.toml"]);
    }

    #[test]
    fn a_temporary_file_left_behind_does_not_stop_a_save() {
        let fake = fake();
        let temp = "/home/usera/.config/babawus/settings.toml.tmp";
        fake.add_file(temp, b"half", SystemTime::UNIX_EPOCH);

        GERMAN.save(&fake, &path()).unwrap();

        assert_eq!(Settings::load(&fake, &path()), GERMAN);
    }

    #[test]
    fn no_file_means_no_choice() {
        assert_eq!(Settings::load(&fake(), &path()), Settings::default());
    }

    #[test]
    fn a_file_that_cannot_be_understood_means_no_choice() {
        let garbled: [&[u8]; 4] = [
            b"nonsense",
            b"language = \"fr\"\n",
            b"language = 3\n",
            b"\xff\xfe",
        ];
        for contents in garbled {
            let fake = fake_with(contents);

            assert_eq!(Settings::load(&fake, &path()), Settings::default());
        }
    }

    #[test]
    fn a_file_that_cannot_be_read_means_no_choice() {
        let fake = fake_with(b"language = \"de\"\n");
        fake.fail_path(path(), ErrorKind::PermissionDenied);

        assert_eq!(Settings::load(&fake, &path()), Settings::default());
    }

    #[test]
    fn settings_from_a_newer_babawus_keep_the_language() {
        let fake = fake_with(b"language = \"de\"\ntheme = \"dark\"\n");

        assert_eq!(Settings::load(&fake, &path()), GERMAN);
    }

    #[test]
    fn a_failed_save_keeps_the_old_settings_and_is_reported() {
        let fake = fake();
        GERMAN.save(&fake, &path()).unwrap();
        let temp = "/home/usera/.config/babawus/settings.toml.tmp";
        fake.fail_path(temp, ErrorKind::PermissionDenied);

        assert!(ENGLISH.save(&fake, &path()).is_err());

        assert_eq!(Settings::load(&fake, &path()), GERMAN);
    }

    #[test]
    fn a_failed_write_leaves_no_temporary_file() {
        let fake = fake();
        GERMAN.save(&fake, &path()).unwrap();
        fake.set_capacity(0);

        assert!(ENGLISH.save(&fake, &path()).is_err());

        let names = fake.read_dir(path().parent().unwrap()).unwrap();
        assert_eq!(names, ["settings.toml"]);
    }

    #[test]
    fn a_folder_that_cannot_be_created_is_reported() {
        let fake = fake();
        fake.add_file("/home/usera/.config", b"", SystemTime::UNIX_EPOCH);

        assert!(GERMAN.save(&fake, &path()).is_err());
    }

    #[test]
    fn an_interrupted_save_leaves_the_old_settings_or_the_new_ones() {
        let steps = {
            let fake = fake();
            GERMAN.save(&fake, &path()).unwrap();
            let before = fake.steps();
            ENGLISH.save(&fake, &path()).unwrap();
            fake.steps() - before
        };
        assert!(steps >= 3);

        for step in 1..=steps {
            let fake = fake();
            GERMAN.save(&fake, &path()).unwrap();
            fake.pull_at_step(fake.steps() + step);

            assert!(ENGLISH.save(&fake, &path()).is_err(), "step {step}");

            fake.reinsert();
            let settings = Settings::load(&fake, &path());
            assert!([GERMAN, ENGLISH].contains(&settings), "step {step}");
            ENGLISH.save(&fake, &path()).unwrap();
            assert_eq!(Settings::load(&fake, &path()), ENGLISH, "step {step}");
        }
    }

    #[test]
    fn the_new_settings_are_flushed_before_they_replace_the_old_ones() {
        let fake = fake();
        GERMAN.save(&fake, &path()).unwrap();

        ENGLISH.save(&fake, &path()).unwrap();

        assert!(fake.content_is_flushed(path()));
    }

    #[test]
    fn saving_works_on_a_real_file_system() {
        let folder = std::env::temp_dir().join(format!("babawus-settings-{}", new_run_id()));
        let path = folder.join("deep/er/babawus/settings.toml");

        GERMAN.save(&RealFs, &path).unwrap();
        ENGLISH.save(&RealFs, &path).unwrap();

        let settings = Settings::load(&RealFs, &path);
        std::fs::remove_dir_all(&folder).unwrap();
        assert_eq!(settings, ENGLISH);
    }
}
