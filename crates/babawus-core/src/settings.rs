//! The settings of a user on this computer. See "Change the language" in Requirements.md.
//!
//! Settings are a convenience, so a settings file that can't be read never stops Babawus: it is
//! simply as if no choice had been made.

use std::ffi::OsString;
use std::io::{self, ErrorKind};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::fs::FileSystem;
use crate::language::{Language, Languages};
use crate::medium::read_file;

/// The settings file, in the user's configuration folder.
pub const SETTINGS_FILE: &str = "babawus/settings.toml";

const TEMP_SUFFIX: &str = ".tmp";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The code of the language the user has chosen, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
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

    /// The language to show: the user's choice, if it is still there, otherwise the system's.
    /// See [`Languages::from_system`].
    pub fn language<'a>(
        &self,
        languages: &Languages,
        system_names: impl IntoIterator<Item = &'a str>,
    ) -> Language {
        self.language
            .as_deref()
            .and_then(|code| languages.find(code))
            .unwrap_or_else(|| languages.from_system(system_names))
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
    use crate::language::testing;
    use crate::log::new_run_id;

    fn chosen(code: &str) -> Settings {
        Settings {
            language: Some(code.to_owned()),
        }
    }

    fn german() -> Settings {
        chosen("de")
    }

    fn english() -> Settings {
        chosen("en")
    }

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
    fn the_users_choice_goes_before_the_system_language() {
        let languages = testing::languages();

        assert_eq!(
            english().language(&languages, ["de_DE"]),
            testing::english()
        );
    }

    #[test]
    fn without_a_choice_the_system_language_is_used() {
        let languages = testing::languages();

        let language = Settings::default().language(&languages, ["de_DE"]);

        assert_eq!(language, testing::german());
    }

    #[test]
    fn a_chosen_language_that_is_gone_gives_the_system_language() {
        let languages = testing::languages();

        assert_eq!(
            chosen("fr").language(&languages, ["de_DE"]),
            testing::german()
        );
        assert_eq!(chosen("fr").language(&languages, ["C"]), testing::english());
    }

    #[test]
    fn saved_settings_can_be_read_back() {
        for settings in [german(), english(), chosen("fr"), Settings::default()] {
            let fake = fake();

            settings.save(&fake, &path()).unwrap();

            assert_eq!(Settings::load(&fake, &path()), settings);
        }
    }

    #[test]
    fn the_file_has_the_form_in_the_requirements() {
        let fake = fake();

        german().save(&fake, &path()).unwrap();

        assert_eq!(read_file(&fake, &path()).unwrap(), b"language = \"de\"\n");
    }

    #[test]
    fn saving_replaces_earlier_settings_and_leaves_no_temporary_file() {
        let fake = fake();
        german().save(&fake, &path()).unwrap();

        english().save(&fake, &path()).unwrap();

        assert_eq!(Settings::load(&fake, &path()), english());
        let names = fake.read_dir(path().parent().unwrap()).unwrap();
        assert_eq!(names, ["settings.toml"]);
    }

    #[test]
    fn a_temporary_file_left_behind_does_not_stop_a_save() {
        let fake = fake();
        let temp = "/home/usera/.config/babawus/settings.toml.tmp";
        fake.add_file(temp, b"half", SystemTime::UNIX_EPOCH);

        german().save(&fake, &path()).unwrap();

        assert_eq!(Settings::load(&fake, &path()), german());
    }

    #[test]
    fn no_file_means_no_choice() {
        assert_eq!(Settings::load(&fake(), &path()), Settings::default());
    }

    #[test]
    fn a_file_that_cannot_be_understood_means_no_choice() {
        let garbled: [&[u8]; 3] = [b"nonsense", b"language = 3\n", b"\xff\xfe"];
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

        assert_eq!(Settings::load(&fake, &path()), german());
    }

    #[test]
    fn a_failed_save_keeps_the_old_settings_and_is_reported() {
        let fake = fake();
        german().save(&fake, &path()).unwrap();
        let temp = "/home/usera/.config/babawus/settings.toml.tmp";
        fake.fail_path(temp, ErrorKind::PermissionDenied);

        assert!(english().save(&fake, &path()).is_err());

        assert_eq!(Settings::load(&fake, &path()), german());
    }

    #[test]
    fn a_failed_write_leaves_no_temporary_file() {
        let fake = fake();
        german().save(&fake, &path()).unwrap();
        fake.set_capacity(0);

        assert!(english().save(&fake, &path()).is_err());

        let names = fake.read_dir(path().parent().unwrap()).unwrap();
        assert_eq!(names, ["settings.toml"]);
    }

    #[test]
    fn a_folder_that_cannot_be_created_is_reported() {
        let fake = fake();
        fake.add_file("/home/usera/.config", b"", SystemTime::UNIX_EPOCH);

        assert!(german().save(&fake, &path()).is_err());
    }

    #[test]
    fn an_interrupted_save_leaves_the_old_settings_or_the_new_ones() {
        let steps = {
            let fake = fake();
            german().save(&fake, &path()).unwrap();
            let before = fake.steps();
            english().save(&fake, &path()).unwrap();
            fake.steps() - before
        };
        assert!(steps >= 3);

        for step in 1..=steps {
            let fake = fake();
            german().save(&fake, &path()).unwrap();
            fake.pull_at_step(fake.steps() + step);

            assert!(english().save(&fake, &path()).is_err(), "step {step}");

            fake.reinsert();
            let settings = Settings::load(&fake, &path());
            assert!([german(), english()].contains(&settings), "step {step}");
            english().save(&fake, &path()).unwrap();
            assert_eq!(Settings::load(&fake, &path()), english(), "step {step}");
        }
    }

    #[test]
    fn the_new_settings_are_flushed_before_they_replace_the_old_ones() {
        let fake = fake();
        german().save(&fake, &path()).unwrap();

        english().save(&fake, &path()).unwrap();

        assert!(fake.content_is_flushed(path()));
    }

    #[test]
    fn saving_works_on_a_real_file_system() {
        let folder = std::env::temp_dir().join(format!("babawus-settings-{}", new_run_id()));
        let path = folder.join("deep/er/babawus/settings.toml");

        german().save(&RealFs, &path).unwrap();
        english().save(&RealFs, &path).unwrap();

        let settings = Settings::load(&RealFs, &path);
        std::fs::remove_dir_all(&folder).unwrap();
        assert_eq!(settings, english());
    }
}
