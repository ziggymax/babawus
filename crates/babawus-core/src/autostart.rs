//! The reminder at login, turned on and off from Babawus: an entry in the user's autostart folder
//! that runs `babawus --checklastbackup` at each login, which every desktop reads, GNOME and
//! Cinnamon alike (issue 174, T-136). See "Remind me at login when my last backup is too old" in
//! Requirements.md.
//!
//! The user may have made such an entry themselves, for example in Mint's "Startup Applications",
//! which writes to the same folder. Any entry that runs `babawus --checklastbackup` counts. Turning
//! the reminder off deletes Babawus's own entry, and switches any other off with
//! `X-GNOME-Autostart-enabled=false`, as "Startup Applications" does, so that nothing of the
//! user's is deleted. An entry with `Hidden=true` counts as deleted, as the desktops treat it.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::fs::FileSystem;
use crate::medium::read_file;
use crate::reminder::CHECK_OPTION;
use crate::settings::save_user_file;

/// The name of Babawus's own entry, after its application ID.
pub const ENTRY_FILE: &str = "io.github.ziggymax.Babawus-reminder.desktop";

const GROUP: &str = "[Desktop Entry]";
const ENABLED_KEY: &str = "X-GNOME-Autostart-enabled";

/// Babawus's own entry. The name is the one the user guide gave for an entry made by hand.
const OWN_ENTRY: &str = "[Desktop Entry]
Type=Application
Name=Babawus reminder
Comment=Reminds you at login when your last backup is too long ago
Exec=babawus --checklastbackup
Icon=/usr/share/icons/hicolor/scalable/apps/io.github.ziggymax.Babawus.svg
Terminal=false
X-GNOME-Autostart-enabled=true
";

/// An entry in the autostart folder that runs the reminder.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    path: PathBuf,
    own: bool,
    enabled: bool,
    text: String,
}

/// Whether the reminder at login is on: whether an entry in `folder` runs it, and isn't
/// switched off.
pub fn is_on(fs: &dyn FileSystem, folder: &Path) -> io::Result<bool> {
    Ok(entries(fs, folder)?.iter().any(|entry| entry.enabled))
}

/// Turns the reminder at login on. An entry that is there but switched off, Babawus's own first,
/// is switched back on; only if there is none does Babawus write its own.
pub fn turn_on(fs: &dyn FileSystem, folder: &Path) -> io::Result<()> {
    let entries = entries(fs, folder)?;
    if entries.iter().any(|entry| entry.enabled) {
        return Ok(());
    }
    match entries.iter().find(|entry| entry.own).or(entries.first()) {
        Some(entry) => save_user_file(fs, &entry.path, with_enabled(&entry.text, true).as_bytes()),
        None => save_user_file(fs, &folder.join(ENTRY_FILE), OWN_ENTRY.as_bytes()),
    }
}

/// Turns the reminder at login off: deletes Babawus's own entry, and switches off any other that
/// is on.
pub fn turn_off(fs: &dyn FileSystem, folder: &Path) -> io::Result<()> {
    for entry in entries(fs, folder)? {
        if entry.own {
            fs.remove_file(&entry.path)?;
        } else if entry.enabled {
            save_user_file(fs, &entry.path, with_enabled(&entry.text, false).as_bytes())?;
        }
    }
    Ok(())
}

/// The entries in `folder` that run the reminder, sorted by name. A missing folder has none, and
/// a file that can't be read, or isn't text, isn't one.
fn entries(fs: &dyn FileSystem, folder: &Path) -> io::Result<Vec<Entry>> {
    let mut names = match fs.read_dir(folder) {
        Ok(names) => names,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    names.sort();
    let mut found = Vec::new();
    for name in names {
        if !Path::new(&name)
            .extension()
            .is_some_and(|extension| extension == "desktop")
        {
            continue;
        }
        let path = folder.join(&name);
        let Ok(text) = read_file(fs, &path).map(String::from_utf8) else {
            continue;
        };
        let Ok(text) = text else { continue };
        let keys = |key: &str| value(&text, key);
        let runs_reminder = keys("Exec").is_some_and(|exec| runs_reminder(&exec));
        if !runs_reminder || keys("Hidden").as_deref() == Some("true") {
            continue;
        }
        found.push(Entry {
            own: name == ENTRY_FILE,
            enabled: keys(ENABLED_KEY).as_deref() != Some("false"),
            path,
            text,
        });
    }
    Ok(found)
}

/// Whether the command in an `Exec` key runs Babawus with the reminder's option.
fn runs_reminder(exec: &str) -> bool {
    let words: Vec<&str> = exec
        .split_whitespace()
        .map(|word| word.trim_matches(|quote| quote == '"' || quote == '\''))
        .collect();
    words
        .first()
        .is_some_and(|program| Path::new(program).file_name() == Some("babawus".as_ref()))
        && words.contains(&CHECK_OPTION)
}

/// The value of `key` in the `[Desktop Entry]` group, without a language suffix.
fn value(text: &str, key: &str) -> Option<String> {
    let mut in_group = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_group = line == GROUP;
        } else if in_group
            && let Some((name, value)) = line.split_once('=')
            && name.trim() == key
        {
            return Some(value.trim().to_owned());
        }
    }
    None
}

/// The entry `text`, with `X-GNOME-Autostart-enabled` set to `enabled` in its `[Desktop Entry]`
/// group, and everything else as it was.
fn with_enabled(text: &str, enabled: bool) -> String {
    let setting = format!("{ENABLED_KEY}={enabled}");
    let mut lines: Vec<String> = Vec::new();
    let mut in_group = false;
    let mut set = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if in_group && !set {
                insert_before_blank_lines(&mut lines, setting.clone());
                set = true;
            }
            in_group = trimmed == GROUP;
        } else if in_group
            && trimmed
                .split_once('=')
                .is_some_and(|(name, _)| name.trim() == ENABLED_KEY)
        {
            if !set {
                lines.push(setting.clone());
                set = true;
            }
            continue;
        }
        lines.push(line.to_owned());
    }
    if in_group && !set {
        insert_before_blank_lines(&mut lines, setting);
    }
    let mut result = lines.join("\n");
    result.push('\n');
    result
}

/// Adds `line` at the end of the group so far, before the blank lines that end it.
fn insert_before_blank_lines(lines: &mut Vec<String>, line: String) {
    let at = lines
        .iter()
        .rposition(|each| !each.trim().is_empty())
        .map_or(0, |last| last + 1);
    lines.insert(at, line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::fake::FakeFs;

    fn folder() -> PathBuf {
        PathBuf::from("/home/anna/.config/autostart")
    }

    fn write(fs: &FakeFs, name: &str, text: &str) {
        save_user_file(fs, &folder().join(name), text.as_bytes()).unwrap();
    }

    fn read(fs: &FakeFs, name: &str) -> String {
        String::from_utf8(read_file(fs, &folder().join(name)).unwrap()).unwrap()
    }

    const USERS: &str = "[Desktop Entry]\nType=Application\nExec=babawus --checklastbackup\nName=Babawus reminder\n";

    #[test]
    fn without_a_folder_the_reminder_is_off() {
        let fs = FakeFs::new();

        assert!(!is_on(&fs, &folder()).unwrap());
    }

    #[test]
    fn turning_it_on_writes_babawus_own_entry_and_the_folder() {
        let fs = FakeFs::new();

        turn_on(&fs, &folder()).unwrap();

        assert!(is_on(&fs, &folder()).unwrap());
        assert_eq!(read(&fs, ENTRY_FILE), OWN_ENTRY);
    }

    #[test]
    fn turning_it_off_deletes_babawus_own_entry() {
        let fs = FakeFs::new();
        turn_on(&fs, &folder()).unwrap();

        turn_off(&fs, &folder()).unwrap();

        assert!(!is_on(&fs, &folder()).unwrap());
        assert_eq!(
            fs.read_dir(&folder()).unwrap(),
            Vec::<std::ffi::OsString>::new()
        );
    }

    #[test]
    fn an_entry_the_user_made_counts_and_is_switched_off_not_deleted() {
        let fs = FakeFs::new();
        write(&fs, "babawus reminder.desktop", USERS);
        assert!(is_on(&fs, &folder()).unwrap());

        turn_off(&fs, &folder()).unwrap();

        assert!(!is_on(&fs, &folder()).unwrap());
        assert_eq!(
            read(&fs, "babawus reminder.desktop"),
            "[Desktop Entry]\nType=Application\nExec=babawus --checklastbackup\nName=Babawus reminder\nX-GNOME-Autostart-enabled=false\n"
        );
    }

    #[test]
    fn turning_it_on_again_switches_the_users_entry_back_on_rather_than_adding_one() {
        let fs = FakeFs::new();
        write(
            &fs,
            "mine.desktop",
            "[Desktop Entry]\nExec=/usr/bin/babawus --checklastbackup\nX-GNOME-Autostart-enabled=false\nName=Mine\n",
        );

        turn_on(&fs, &folder()).unwrap();

        assert!(is_on(&fs, &folder()).unwrap());
        assert_eq!(
            read(&fs, "mine.desktop"),
            "[Desktop Entry]\nExec=/usr/bin/babawus --checklastbackup\nX-GNOME-Autostart-enabled=true\nName=Mine\n"
        );
        assert_eq!(fs.read_dir(&folder()).unwrap().len(), 1);
    }

    #[test]
    fn turning_it_on_when_it_is_on_changes_nothing() {
        let fs = FakeFs::new();
        write(&fs, "mine.desktop", USERS);

        turn_on(&fs, &folder()).unwrap();

        assert_eq!(read(&fs, "mine.desktop"), USERS);
        assert_eq!(fs.read_dir(&folder()).unwrap().len(), 1);
    }

    #[test]
    fn babawus_own_entry_switched_off_elsewhere_is_switched_back_on_first() {
        let fs = FakeFs::new();
        write(&fs, ENTRY_FILE, &with_enabled(OWN_ENTRY, false));
        write(&fs, "a-mine.desktop", &with_enabled(USERS, false));
        assert!(!is_on(&fs, &folder()).unwrap());

        turn_on(&fs, &folder()).unwrap();

        assert_eq!(read(&fs, ENTRY_FILE), OWN_ENTRY);
        assert_eq!(read(&fs, "a-mine.desktop"), with_enabled(USERS, false));
    }

    #[test]
    fn other_programs_hidden_entries_and_other_groups_dont_count() {
        let fs = FakeFs::new();
        write(&fs, "other.desktop", "[Desktop Entry]\nExec=babawus\n");
        write(&fs, "notes.txt", USERS);
        write(&fs, "hidden.desktop", &format!("{USERS}Hidden=true\n"));
        write(
            &fs,
            "babawusx.desktop",
            "[Desktop Entry]\nExec=babawusx --checklastbackup\n",
        );
        write(
            &fs,
            "action.desktop",
            "[Desktop Entry]\nExec=other\n\n[Desktop Action check]\nExec=babawus --checklastbackup\n",
        );

        assert!(!is_on(&fs, &folder()).unwrap());
        turn_off(&fs, &folder()).unwrap();
        assert_eq!(read(&fs, "hidden.desktop"), format!("{USERS}Hidden=true\n"));
    }

    #[test]
    fn the_setting_goes_into_the_desktop_entry_group_before_its_blank_lines() {
        let text = "# made by hand\n[Desktop Entry]\nExec='babawus' --checklastbackup\n\n[Desktop Action x]\nExec=other\n";

        assert_eq!(
            with_enabled(text, false),
            "# made by hand\n[Desktop Entry]\nExec='babawus' --checklastbackup\nX-GNOME-Autostart-enabled=false\n\n[Desktop Action x]\nExec=other\n"
        );
        assert!(runs_reminder("'babawus' --checklastbackup"));
    }
}
