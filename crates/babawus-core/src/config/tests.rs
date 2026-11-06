use std::path::PathBuf;
use std::time::SystemTime;

use super::*;
use crate::fs::fake::FakeFs;

const ID: &str = "0123456789abcdef0123456789abcdef";
const OTHER_ID: &str = "fedcba9876543210fedcba9876543210";

fn root() -> PathBuf {
    PathBuf::from("/medium")
}

fn config() -> Config {
    let mut config = Config::new();
    config.add_computer(ID, "mint", "mint", &[]).unwrap();
    config.add_account(ID, "usera").unwrap();
    config
}

/// A file with a correct checksum, whatever its fields say.
fn sealed(fields: &str) -> String {
    let body = format!("{MARKER}{HEADER}{fields}");
    let checksum = checksum(&body);
    format!("{body}{CHECKSUM_PREFIX}{checksum}\n")
}

fn parse_text(text: &str) -> Loaded {
    parse(text.as_bytes())
}

fn account(config: &Config) -> &Account {
    &config.computers[0].accounts[0]
}

fn account_mut(config: &mut Config) -> &mut Account {
    &mut config.computers[0].accounts[0]
}

fn os_names(names: &[&str]) -> Vec<OsString> {
    names.iter().map(OsString::from).collect()
}

#[test]
fn a_written_file_looks_as_the_requirements_describe() {
    let mut config = config();
    let account = account_mut(&mut config);
    account.selected = vec!["Documents".to_owned(), "notes.txt".to_owned()];

    let text = config.to_text();

    let expected = "\
# Babawus configuration
# This file is written by Babawus. Change it with the Advanced functions in Babawus.
# If it is edited by hand, the medium needs attention until it has been saved in Babawus.
format = 1
older_versions = 10

[[computer]]
id = \"0123456789abcdef0123456789abcdef\"
folder = \"mint\"
hostname = \"mint\"

[[computer.account]]
username = \"usera\"
selected = [\"Documents\", \"notes.txt\"]
exclude_patterns = [\".cache\", \"*.tmp\", \"*~\"]
exclude_folders = [\".local/share/Trash\"]

";
    let (body, checksum_line) = text.split_at(expected.len());
    assert_eq!(body, expected);
    assert_eq!(checksum_line.len(), CHECKSUM_PREFIX.len() + 16 + 1);
    assert!(checksum_line.starts_with(CHECKSUM_PREFIX));
    assert!(checksum_line.ends_with('\n'));
}

#[test]
fn the_checksum_is_the_first_16_digits_of_sha256() {
    // Worked out separately, with `sha256sum`.
    assert_eq!(checksum("abc\n"), "edeaaff3f1774ad2");
}

#[test]
fn what_is_written_reads_back_the_same() {
    let mut config = config();
    config.older_versions = 99;
    account_mut(&mut config).selected = vec!["Bilder \"gut\"/é.txt".to_owned()];
    config
        .add_computer(OTHER_ID, "mint-2", "Mint", &[])
        .unwrap();

    assert_eq!(parse_text(&config.to_text()), Loaded::Ready(config));
}

#[test]
fn a_new_configuration_with_no_computers_reads_back() {
    let config = Config::new();

    assert_eq!(config.older_versions, 10);
    assert_eq!(parse_text(&config.to_text()), Loaded::Ready(config));
}

#[test]
fn an_edit_makes_the_file_damaged_but_the_configuration_can_be_checked() {
    let config = config();
    let edited = config
        .to_text()
        .replace("older_versions = 10", "older_versions = 11");

    let Loaded::Damaged(Some(found)) = parse_text(&edited) else {
        panic!("the edit went unnoticed");
    };

    assert_eq!(found.older_versions, 11);
    assert_eq!(found.computers, config.computers);
}

#[test]
fn saving_a_checked_configuration_clears_the_damage() {
    let edited = config()
        .to_text()
        .replace("mint\"\nhostname", "mint\"\nhostname ");
    let Loaded::Damaged(Some(found)) = parse_text(&edited) else {
        panic!("the edit went unnoticed");
    };

    assert_eq!(parse_text(&found.to_text()), Loaded::Ready(found));
}

#[test]
fn the_checksum_may_be_in_upper_case() {
    let config = config();
    let text = config.to_text();
    let (body, last) = text.rsplit_once(CHECKSUM_PREFIX).unwrap();
    let upper = format!("{body}{CHECKSUM_PREFIX}{}", last.to_uppercase());

    assert_eq!(parse_text(&upper), Loaded::Ready(config));
}

#[test]
fn a_changed_or_missing_checksum_line_is_damage() {
    let config = config();
    let text = config.to_text();
    let without = text.rsplit_once(CHECKSUM_PREFIX).unwrap().0.to_owned();
    let wrong = format!("{without}{CHECKSUM_PREFIX}0000000000000000\n");
    let no_line_ending = text.strip_suffix('\n').unwrap().to_owned();
    let extra_line = format!("{text}\n");
    let crlf = text.replace('\n', "\r\n");

    for damaged in [without, wrong, no_line_ending, extra_line, crlf] {
        let expected = Loaded::Damaged(Some(config.clone()));
        assert_eq!(parse_text(&damaged), expected, "{damaged}");
    }
}

#[test]
fn a_file_without_the_marker_is_damaged_even_with_a_correct_checksum() {
    let body = "format = 1\nolder_versions = 10\n";
    let text = format!("{body}{CHECKSUM_PREFIX}{}\n", checksum(body));

    assert_eq!(parse_text(&text), Loaded::Damaged(Some(Config::new())));
}

#[test]
fn a_file_that_can_not_be_parsed_can_only_be_replaced() {
    let halves = config().to_text();
    let cut = halves.split_at(halves.len() / 2).0;

    for text in ["", "\n", "not toml", cut, "# Babawus configuration\n"] {
        assert_eq!(parse_text(text), Loaded::Damaged(None), "{text}");
    }
    assert_eq!(parse(b"\xff\xfe"), Loaded::Damaged(None));
}

#[test]
fn a_newer_format_is_recognised_whatever_the_rest_of_the_file_holds() {
    for rest in [
        "older_versions = 10\n",
        "new_field = \"something\"\n",
        "this isn't even toml\n",
    ] {
        let text = format!("{MARKER}{HEADER}format = 2\n{rest}");
        assert_eq!(parse_text(&text), Loaded::NewerFormat, "{rest}");
    }
}

#[test]
fn a_newer_format_needs_the_marker() {
    let text = "format = 2\nolder_versions = 10\n";

    assert_eq!(parse_text(text), Loaded::Damaged(None));
}

#[test]
fn format_is_only_looked_for_before_the_first_table() {
    let text = sealed("older_versions = 10\n\n[[computer]]\nformat = 2\n");

    assert_eq!(parse_text(&text), Loaded::Damaged(None));
}

#[test]
fn values_that_break_the_rules_make_the_file_unusable() {
    let computer = |id: &str, folder: &str| {
        format!("[[computer]]\nid = \"{id}\"\nfolder = \"{folder}\"\nhostname = \"h\"\n")
    };
    let account = |username: &str, patterns: &str, folders: &str| {
        format!(
            "[[computer.account]]\nusername = \"{username}\"\nselected = []\n\
             exclude_patterns = {patterns}\nexclude_folders = {folders}\n"
        )
    };
    let good = computer(ID, "mint");
    let cases = [
        ("too few versions", "0".to_owned(), String::new()),
        ("too many versions", "100".to_owned(), String::new()),
        ("no number", "\"ten\"".to_owned(), String::new()),
        ("bad id", "10".to_owned(), computer("xyz", "mint")),
        (
            "upper-case id",
            "10".to_owned(),
            computer(&ID.to_uppercase(), "mint"),
        ),
        ("dots", "10".to_owned(), computer(ID, "..")),
        ("path", "10".to_owned(), computer(ID, "../mint")),
        ("slash", "10".to_owned(), computer(ID, "a/b")),
        ("empty folder", "10".to_owned(), computer(ID, "")),
        ("upper case", "10".to_owned(), computer(ID, "Mint")),
        ("not storable", "10".to_owned(), computer(ID, "a:b")),
        ("version form", "10".to_owned(), computer(ID, "a[]b")),
        ("too long", "10".to_owned(), computer(ID, &"a".repeat(256))),
        (
            "same id",
            "10".to_owned(),
            format!("{good}{}", computer(ID, "other")),
        ),
        (
            "same folder",
            "10".to_owned(),
            format!("{good}{}", computer(OTHER_ID, "mint")),
        ),
        (
            "unknown field",
            "10".to_owned(),
            format!("{good}colour = \"red\"\n"),
        ),
        (
            "user with a path",
            "10".to_owned(),
            format!("{good}{}", account("../x", "[]", "[]")),
        ),
        (
            "same user in another case",
            "10".to_owned(),
            format!(
                "{good}{}{}",
                account("anna", "[]", "[]"),
                account("Anna", "[]", "[]")
            ),
        ),
        (
            "pattern with a slash",
            "10".to_owned(),
            format!("{good}{}", account("a", "[\"a/b\"]", "[]")),
        ),
        (
            "empty pattern",
            "10".to_owned(),
            format!("{good}{}", account("a", "[\"\"]", "[]")),
        ),
        (
            "empty excluded folder",
            "10".to_owned(),
            format!("{good}{}", account("a", "[]", "[\"\"]")),
        ),
        (
            "absolute excluded folder",
            "10".to_owned(),
            format!("{good}{}", account("a", "[]", "[\"/home\"]")),
        ),
        (
            "excluded folder with dots",
            "10".to_owned(),
            format!("{good}{}", account("a", "[]", "[\"a/../b\"]")),
        ),
        (
            "missing field",
            "10".to_owned(),
            format!("{good}[[computer.account]]\nusername = \"a\"\n"),
        ),
    ];
    for (name, versions, tables) in cases {
        let text = sealed(&format!(
            "format = 1\nolder_versions = {versions}\n\n{tables}\n"
        ));
        assert_eq!(parse_text(&text), Loaded::Damaged(None), "{name}");
    }
}

#[test]
fn format_0_is_not_valid() {
    let text = sealed("format = 0\nolder_versions = 10\n\n");

    assert_eq!(parse_text(&text), Loaded::Damaged(None));
}

#[test]
fn a_selection_with_dots_is_left_for_the_run_to_report() {
    let mut config = config();
    account_mut(&mut config).selected = vec!["../other".to_owned()];

    assert_eq!(parse_text(&config.to_text()), Loaded::Ready(config));
}

#[test]
fn a_new_account_starts_with_the_suggested_exclusions() {
    let config = config();
    let account = account(&config);

    assert!(account.selected.is_empty());
    assert_eq!(account.exclude_patterns, [".cache", "*.tmp", "*~"]);
    assert_eq!(account.exclude_folders, [".local/share/Trash"]);
}

#[test]
fn the_suggested_exclusions_work() {
    let config = config();
    let selection = account(&config).selection(Path::new("/home/usera"));

    for path in [
        ".cache/x",
        "a/.cache",
        "a.tmp",
        "notes~",
        ".local/share/Trash/x",
    ] {
        assert!(selection.exclusions.excludes(Path::new(path)), "{path}");
    }
    assert!(!selection.exclusions.excludes(Path::new(".local/share/x")));
}

#[test]
fn an_account_becomes_what_the_scan_needs() {
    let mut config = config();
    let account = account_mut(&mut config);
    account.selected = vec!["Documents".to_owned()];

    let selection = account.selection(Path::new("/home/usera"));

    assert_eq!(selection.home, Path::new("/home/usera"));
    assert_eq!(selection.selected, [PathBuf::from("Documents")]);
}

#[test]
fn an_entry_is_found_by_computer_id_and_exact_user_name() {
    let config = config();

    let (computer, account) = config.account(ID, "usera").unwrap();
    assert_eq!(computer.folder, "mint");
    assert_eq!(account.username, "usera");
    assert!(config.account(OTHER_ID, "usera").is_none());
    assert!(config.account(ID, "userb").is_none());
    assert!(config.account(ID, "UserA").is_none());
}

#[test]
fn the_suggested_folder_is_the_hostname_in_lower_case() {
    assert_eq!(Config::new().suggest_folder("Mint", &[]), "mint");
}

#[test]
fn a_name_in_the_configuration_counts_as_used() {
    let mut config = config();

    assert_eq!(config.suggest_folder("mint", &[]), "mint-2");

    config
        .add_computer(OTHER_ID, "mint-2", "mint", &[])
        .unwrap();
    assert_eq!(config.suggest_folder("MINT", &[]), "mint-3");
}

#[test]
fn a_folder_on_the_medium_counts_as_used_without_regard_to_case() {
    let on_medium = os_names(&["MINT", "mint-2"]);

    assert_eq!(Config::new().suggest_folder("mint", &on_medium), "mint-3");
}

#[test]
fn a_hostname_that_is_no_valid_folder_gets_a_plain_name() {
    assert_eq!(Config::new().suggest_folder("", &[]), "computer");
    assert_eq!(Config::new().suggest_folder("bad:name", &[]), "computer");
    let on_medium = os_names(&["computer"]);
    assert_eq!(Config::new().suggest_folder("", &on_medium), "computer-2");
}

#[test]
fn a_computer_can_not_use_a_folder_that_is_taken() {
    let mut config = config();

    let in_config = config.add_computer(OTHER_ID, "MINT", "mint", &[]);
    let on_medium = config.add_computer(OTHER_ID, "old", "old", &os_names(&["old"]));

    assert!(matches!(in_config, Err(EntryError::Clash)));
    assert!(matches!(on_medium, Err(EntryError::Clash)));
    assert_eq!(config.computers.len(), 1);
}

#[test]
fn a_computer_can_not_be_added_twice() {
    let mut config = config();

    let result = config.add_computer(ID, "other", "other", &[]);

    assert!(matches!(result, Err(EntryError::Clash)));
    assert_eq!(config.computers.len(), 1);
}

#[test]
fn a_computer_needs_a_valid_id_and_folder() {
    let mut config = Config::new();

    let bad_id = config.add_computer("xyz", "mint", "mint", &[]);
    let bad_folder = config.add_computer(ID, "a/b", "mint", &[]);

    assert!(matches!(bad_id, Err(EntryError::Invalid)));
    assert!(matches!(bad_folder, Err(EntryError::Invalid)));
    assert!(config.computers.is_empty());
}

#[test]
fn an_account_can_not_be_added_twice_or_to_a_missing_computer() {
    let mut config = config();

    let twice = config.add_account(ID, "UserA");
    let missing = config.add_account(OTHER_ID, "usera");

    assert!(matches!(twice, Err(EntryError::Clash)));
    assert!(matches!(missing, Err(EntryError::NotFound)));
    assert_eq!(config.computers[0].accounts.len(), 1);
}

#[test]
fn an_account_needs_a_valid_name() {
    let mut config = config();

    let result = config.add_account(ID, "a/b");

    assert!(matches!(result, Err(EntryError::Invalid)));
}

#[test]
fn linking_a_computer_keeps_its_folder_and_accounts() {
    let mut config = config();

    config
        .link_computer("mint", OTHER_ID, "new-laptop")
        .unwrap();

    let (computer, _) = config.account(OTHER_ID, "usera").unwrap();
    assert_eq!(computer.folder, "mint");
    assert_eq!(computer.hostname, "new-laptop");
    assert!(config.computer(ID).is_none());
}

#[test]
fn linking_needs_an_existing_entry_and_a_free_id() {
    let mut config = config();
    config
        .add_computer(OTHER_ID, "mint-2", "mint", &[])
        .unwrap();
    let before = config.clone();

    let missing = config.link_computer("nothing", OTHER_ID, "x");
    let taken = config.link_computer("mint", OTHER_ID, "x");

    assert!(matches!(missing, Err(EntryError::NotFound)));
    assert!(matches!(taken, Err(EntryError::Clash)));
    assert_eq!(config, before);
}

#[test]
fn removing_entries_only_touches_those_entries() {
    let mut config = config();
    config.add_account(ID, "userb").unwrap();
    config
        .add_computer(OTHER_ID, "mint-2", "mint", &[])
        .unwrap();

    config.remove_account(ID, "usera").unwrap();
    assert!(config.account(ID, "usera").is_none());
    assert!(config.account(ID, "userb").is_some());

    config.remove_computer(ID).unwrap();
    assert!(config.computer(ID).is_none());
    assert!(config.computer(OTHER_ID).is_some());
}

#[test]
fn removing_a_missing_entry_is_reported() {
    let mut config = config();

    let computer = config.remove_computer(OTHER_ID);
    let account = config.remove_account(ID, "nobody");

    assert!(matches!(computer, Err(EntryError::NotFound)));
    assert!(matches!(account, Err(EntryError::NotFound)));
}

struct Medium {
    fake: FakeFs,
}

impl Medium {
    fn new() -> Self {
        let fake = FakeFs::new();
        fake.add_dir_on_device("/medium/.babawus", 0);
        Self { fake }
    }

    fn add_backup(&self, path: &str) {
        let path = root().join("backup").join(path);
        self.fake.add_file(path, b"data", SystemTime::UNIX_EPOCH);
    }

    fn has_backup(&self, path: &str) -> bool {
        let path = root().join("backup").join(path);
        self.fake.metadata(&path).is_ok()
    }

    fn save(&self, config: &Config) {
        save(&self.fake, &root(), Format::ExFat, config).unwrap();
    }

    fn load(&self) -> Loaded {
        load(&self.fake, &root()).unwrap()
    }

    fn link(&self, config: &mut Config, old: &str, new: &str) -> Result<(), EntryError> {
        link_account(&self.fake, &root(), Format::ExFat, config, ID, old, new)
    }
}

#[test]
fn a_medium_with_no_configuration_is_not_a_babawus_medium() {
    assert_eq!(Medium::new().load(), Loaded::Missing);
    assert_eq!(load(&FakeFs::new(), &root()).unwrap(), Loaded::Missing);
}

#[test]
fn a_read_error_is_not_taken_for_a_missing_file() {
    let medium = Medium::new();
    let path = root().join(CONFIG_FILE);
    medium.fake.fail_path(&path, ErrorKind::PermissionDenied);

    assert!(load(&medium.fake, &root()).is_err());
}

#[test]
fn a_saved_configuration_can_be_loaded() {
    let medium = Medium::new();
    let config = config();

    medium.save(&config);

    assert_eq!(medium.load(), Loaded::Ready(config));
}

#[test]
fn saving_replaces_the_old_configuration() {
    let medium = Medium::new();
    let mut config = config();
    medium.save(&config);

    config.older_versions = 5;
    medium.save(&config);

    assert_eq!(medium.load(), Loaded::Ready(config));
}

#[test]
fn an_invalid_configuration_is_never_saved() {
    let medium = Medium::new();
    let mut config = config();
    config.older_versions = 0;

    let result = save(&medium.fake, &root(), Format::ExFat, &config);

    assert!(matches!(result, Err(EntryError::Invalid)));
    assert_eq!(medium.load(), Loaded::Missing);
}

#[test]
fn pulling_the_medium_while_saving_never_damages_the_configuration() {
    let old = config();
    let mut new = old.clone();
    new.older_versions = 5;
    let steps = {
        let medium = Medium::new();
        medium.save(&old);
        let before = medium.fake.steps();
        medium.save(&new);
        medium.fake.steps() - before
    };
    for step in 1..=steps {
        for lose_unflushed in [false, true] {
            let medium = Medium::new();
            medium.save(&old);
            medium.fake.pull_at_step(medium.fake.steps() + step);

            assert!(save(&medium.fake, &root(), Format::ExFat, &new).is_err());

            if lose_unflushed {
                medium.fake.reinsert_losing_unflushed();
            } else {
                medium.fake.reinsert();
            }
            let loaded = medium.load();
            assert!(
                loaded == Loaded::Ready(old.clone()) || loaded == Loaded::Ready(new.clone()),
                "step {step}: {loaded:?}"
            );
        }
    }
}

#[test]
fn the_folders_on_a_medium_are_listed() {
    let medium = Medium::new();
    assert!(folders_on_medium(&medium.fake, &root()).unwrap().is_empty());

    medium.add_backup("mint/usera/file");
    medium.add_backup("old/userb/file");

    let folders = folders_on_medium(&medium.fake, &root()).unwrap();
    assert_eq!(folders, os_names(&["mint", "old"]));
}

#[test]
fn linking_an_account_moves_its_backups_and_renames_the_entry() {
    let medium = Medium::new();
    let mut config = config();
    medium.add_backup("mint/usera/Documents/report.pdf");
    medium.add_backup("mint/usera/Documents/report[]pdf/report[2026-11-03_175400].pdf");

    medium.link(&mut config, "usera", "Anna").unwrap();

    assert!(medium.has_backup("mint/anna/Documents/report.pdf"));
    assert!(medium.has_backup("mint/anna/Documents/report[]pdf/report[2026-11-03_175400].pdf"));
    assert!(!medium.has_backup("mint/usera"));
    assert!(config.account(ID, "Anna").is_some());
    assert!(config.account(ID, "usera").is_none());
    assert_eq!(medium.load(), Loaded::Ready(config));
}

#[test]
fn linking_a_change_of_case_only_needs_no_rename() {
    let medium = Medium::new();
    let mut config = config();
    medium.add_backup("mint/usera/file");

    medium.link(&mut config, "usera", "UserA").unwrap();

    assert!(medium.has_backup("mint/usera/file"));
    assert!(config.account(ID, "UserA").is_some());
}

#[test]
fn an_account_that_never_made_a_backup_can_be_linked() {
    let medium = Medium::new();
    let mut config = config();

    medium.link(&mut config, "usera", "anna").unwrap();

    assert!(config.account(ID, "anna").is_some());
}

#[test]
fn linking_refuses_to_overwrite_an_existing_folder() {
    let medium = Medium::new();
    let mut config = config();
    medium.add_backup("mint/usera/mine");
    medium.add_backup("mint/anna/theirs");
    let before = config.clone();

    let result = medium.link(&mut config, "usera", "anna");

    assert!(matches!(result, Err(EntryError::Clash)));
    assert!(medium.has_backup("mint/usera/mine"));
    assert!(medium.has_backup("mint/anna/theirs"));
    assert_eq!(config, before);
    assert_eq!(medium.load(), Loaded::Missing);
}

#[test]
fn linking_needs_a_valid_free_name_and_an_existing_entry() {
    let medium = Medium::new();
    let mut config = config();
    config.add_account(ID, "taken").unwrap();
    let before = config.clone();

    let bad = medium.link(&mut config, "usera", "a/b");
    let taken = medium.link(&mut config, "usera", "taken");
    let missing = medium.link(&mut config, "nobody", "anna");

    assert!(matches!(bad, Err(EntryError::Invalid)));
    assert!(matches!(taken, Err(EntryError::Clash)));
    assert!(matches!(missing, Err(EntryError::NotFound)));
    assert_eq!(config, before);
}

#[test]
fn an_interrupted_link_can_be_repeated() {
    let steps = {
        let medium = Medium::new();
        let mut config = config();
        medium.add_backup("mint/usera/file");
        medium.save(&config);
        let before = medium.fake.steps();
        medium.link(&mut config, "usera", "anna").unwrap();
        medium.fake.steps() - before
    };
    for step in 1..=steps {
        for lose_unflushed in [false, true] {
            let medium = Medium::new();
            let mut config = config();
            medium.add_backup("mint/usera/file");
            medium.save(&config);
            medium.fake.pull_at_step(medium.fake.steps() + step);

            assert!(medium.link(&mut config, "usera", "anna").is_err());

            if lose_unflushed {
                medium.fake.reinsert_losing_unflushed();
            } else {
                medium.fake.reinsert();
            }
            let Loaded::Ready(mut loaded) = medium.load() else {
                panic!("step {step}: the configuration was damaged");
            };
            if loaded.account(ID, "usera").is_some() {
                medium.link(&mut loaded, "usera", "anna").unwrap();
            }
            assert!(loaded.account(ID, "anna").is_some(), "step {step}");
            assert!(medium.has_backup("mint/anna/file"), "step {step}");
        }
    }
}
