use std::time::SystemTime;

use super::testing::{english, german};
use super::*;
use crate::fs::RealFs;
use crate::fs::fake::FakeFs;

const ENGLISH_FILE: &str = include_str!("../../../../languages/en.toml");
const FOLDER: &str = "/usr/share/babawus/languages";

/// The English file made into the language `code`, called `name`.
fn other(code: &str, name: &str) -> String {
    ENGLISH_FILE
        .replace("code = \"en\"", &format!("code = \"{code}\""))
        .replace("name = \"English\"", &format!("name = \"{name}\""))
}

fn folder_with(files: &[(&str, &str)]) -> FakeFs {
    let fake = FakeFs::new();
    for (name, contents) in files {
        fake.add_file(
            format!("{FOLDER}/{name}"),
            contents.as_bytes(),
            SystemTime::UNIX_EPOCH,
        );
    }
    fake
}

fn load(files: &[(&str, &str)]) -> Result<Loaded, String> {
    Languages::load(&folder_with(files), Path::new(FOLDER))
}

fn codes(loaded: &Loaded) -> Vec<&'static str> {
    loaded
        .languages
        .all()
        .iter()
        .map(|each| each.code())
        .collect()
}

#[test]
fn every_file_in_the_source_tree_is_complete_and_offered_without_problems() {
    let loaded = Languages::load(&RealFs, &folder()).unwrap();

    let files = RealFs.read_dir(&folder()).unwrap();
    let toml_files = files
        .iter()
        .filter(|name| name.to_string_lossy().ends_with(".toml"));
    assert_eq!(loaded.languages.all().len(), toml_files.count());
    assert!(loaded.problems.is_empty(), "{:?}", loaded.problems);
    assert_eq!(loaded.languages.english(), english());
    assert_eq!(loaded.languages.find("de"), Some(german()));
}

#[test]
fn a_language_file_gives_its_name_and_separators() {
    assert_eq!(
        (english().name(), english().decimal(), english().thousands()),
        ("English", ".", ",")
    );
    assert_eq!(
        (german().name(), german().decimal(), german().thousands()),
        ("Deutsch", ",", ".")
    );
}

#[test]
fn the_menu_is_sorted_by_the_languages_own_names() {
    let loaded = load(&[
        ("en.toml", ENGLISH_FILE),
        ("es.toml", &other("es", "Español")),
        ("da.toml", &other("da", "dansk")),
        ("de.toml", &other("de", "Deutsch")),
    ])
    .unwrap();

    assert_eq!(codes(&loaded), ["da", "de", "en", "es"]);
}

#[test]
fn without_an_english_file_nothing_can_be_shown() {
    let error = load(&[("de.toml", &other("de", "Deutsch"))]).unwrap_err();

    assert!(error.contains("en.toml"), "{error}");
}

#[test]
fn a_broken_english_file_is_an_error_with_the_reason() {
    let broken = ENGLISH_FILE.replace("StartBackup = ", "StartBackup ");

    let error = load(&[("en.toml", &broken)]).unwrap_err();

    assert!(error.contains("en.toml: it isn't valid"), "{error}");
    assert!(error.contains("line"), "{error}");
}

#[test]
fn a_file_with_a_missing_text_is_not_offered_and_says_which() {
    let incomplete = other("fr", "Français").replace("StartBackup = \"Start backup\"\n", "");

    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", &incomplete)]).unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert_eq!(
        loaded.problems,
        [format!(
            "{FOLDER}/fr.toml: not offered, because these texts are missing: StartBackup"
        )]
    );
}

#[test]
fn an_empty_text_counts_as_missing() {
    let empty = other("fr", "Français").replace("\"Start backup\"", "\"\"");

    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", &empty)]).unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert!(loaded.problems[0].contains("missing: StartBackup"));
}

#[test]
fn a_file_with_other_placeholders_is_not_offered() {
    let wrong = other("fr", "Français").replace("by {user}", "by {usr}");

    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", &wrong)]).unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert!(
        loaded.problems[0].ends_with("other placeholders than in English: InProgress"),
        "{:?}",
        loaded.problems
    );
}

#[test]
fn a_file_that_isnt_valid_toml_is_not_offered() {
    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", "[language\n")]).unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert!(loaded.problems[0].contains("fr.toml: not offered, because it isn't valid"));
}

#[test]
fn a_file_whose_code_isnt_its_name_is_not_offered() {
    let loaded = load(&[
        ("en.toml", ENGLISH_FILE),
        ("fr.toml", &other("de", "Français")),
    ])
    .unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert!(loaded.problems[0].ends_with("its code \"de\" isn't its file name, fr.toml"));
}

#[test]
fn a_file_without_a_name_is_not_offered() {
    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", &other("fr", " "))]).unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert!(loaded.problems[0].ends_with("its name is empty"));
}

#[test]
fn unknown_texts_are_warned_about_but_the_file_is_offered() {
    let newer = other("fr", "Français") + "Teleport = \"Teleport\"\n";

    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", &newer)]).unwrap();

    assert_eq!(codes(&loaded), ["en", "fr"]);
    assert_eq!(
        loaded.problems,
        [format!(
            "{FOLDER}/fr.toml: the text Teleport isn't one Babawus knows, and is left out"
        )]
    );
}

#[test]
fn files_that_arent_language_files_are_left_alone() {
    let loaded = load(&[
        ("en.toml", ENGLISH_FILE),
        ("README", "Not a language"),
        ("fr.toml~", "Not a language"),
    ])
    .unwrap();

    assert_eq!(codes(&loaded), ["en"]);
    assert!(loaded.problems.is_empty());
}

#[test]
fn german_is_found_in_the_forms_systems_use() {
    let languages = testing::languages();
    for name in ["de", "de_DE", "de_DE.UTF-8", "de_AT@euro", "de.UTF-8"] {
        assert_eq!(languages.from_system([name]), german(), "{name}");
    }
}

#[test]
fn an_unavailable_system_language_gives_english() {
    let languages = testing::languages();
    for names in [&["fr_FR.UTF-8"][..], &["C"], &["den_XX"], &[""], &[]] {
        let language = languages.from_system(names.iter().copied());
        assert_eq!(language, english(), "{names:?}");
    }
}

#[test]
fn the_first_available_system_language_wins() {
    let languages = testing::languages();

    assert_eq!(
        languages.from_system(["fr_FR", "fr", "de_DE", "en_GB", "C"]),
        german()
    );
    assert_eq!(languages.from_system(["en_GB", "de_DE"]), english());
}

#[test]
fn a_help_without_an_entry_for_a_number_is_warned_about_but_offered() {
    let without = other("fr", "Français")
        .replace("\n#3 Babawus can't see", "\nBabawus can't see")
        .replace("\n#40 Only an administrator", "\nOnly an administrator");

    let loaded = load(&[("en.toml", ENGLISH_FILE), ("fr.toml", &without)]).unwrap();

    assert_eq!(codes(&loaded), ["en", "fr"]);
    assert_eq!(
        loaded.problems,
        [format!(
            "{FOLDER}/fr.toml: the Help has no entry for #3, #40"
        )]
    );
}
