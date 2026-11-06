//! The fixed layout of a medium, and replacing Babawus's own files on it safely.
//!
//! See "Layout of a Babawus medium" and "Permissions on Ext4 media" in Requirements.md.

use std::io::{self, ErrorKind, Read};
use std::path::Path;

use crate::fs::FileSystem;
use crate::versions::Format;

pub const CONFIG_FILE: &str = ".babawus/babawus_config";
pub const LOG_FILE: &str = ".babawus/babawus_log.txt";
pub const LOCK_FILE: &str = ".babawus/lock";
pub const TEMP_FOLDER: &str = ".babawus/tmp";
pub const BACKUP_FOLDER: &str = "backup";

pub(crate) const FILE_MODE: u32 = 0o666;
pub(crate) const DIR_MODE: u32 = 0o777;

const REPLACEMENT: &str = "replacement";

/// Reads a whole file.
pub(crate) fn read_file(fs: &dyn FileSystem, path: &Path) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    fs.open(path)?.read_to_end(&mut data)?;
    Ok(data)
}

/// Sets `mode` on Ext4. exFAT stores no permissions.
pub(crate) fn set_mode(
    medium: &dyn FileSystem,
    format: Format,
    path: &Path,
    mode: u32,
) -> io::Result<()> {
    match format {
        Format::Ext4 => medium.set_mode(path, mode),
        Format::ExFat => Ok(()),
    }
}

/// Replaces the file `relative`, which is relative to `root`, without ever writing it in place.
/// An interruption leaves either the old file or the new one. The caller holds the medium's lock.
pub(crate) fn replace_file(
    medium: &dyn FileSystem,
    root: &Path,
    format: Format,
    relative: &str,
    data: &[u8],
) -> io::Result<()> {
    let folder = root.join(TEMP_FOLDER);
    match medium.create_dir(&folder) {
        Ok(()) => set_mode(medium, format, &folder, DIR_MODE)?,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let temp = folder.join(REPLACEMENT);
    match medium.remove_file(&temp) {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    let result = write_temp(medium, format, &temp, data)
        .and_then(|()| medium.rename(&temp, &root.join(relative)));
    if result.is_err() {
        // If this fails too, the next run empties the folder anyway.
        let _ = medium.remove_file(&temp);
    }
    result?;
    medium.sync_medium(root)
}

fn write_temp(medium: &dyn FileSystem, format: Format, temp: &Path, data: &[u8]) -> io::Result<()> {
    let mut file = medium.create_new(temp)?;
    set_mode(medium, format, temp, FILE_MODE)?;
    file.write_all(data)?;
    file.sync()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use super::*;
    use crate::fs::fake::FakeFs;

    const FILE: &str = ".babawus/file";

    fn medium() -> FakeFs {
        let medium = FakeFs::new();
        medium.add_file("/medium/.babawus/file", b"old", SystemTime::UNIX_EPOCH);
        medium
    }

    fn root() -> PathBuf {
        PathBuf::from("/medium")
    }

    fn content(medium: &FakeFs) -> Vec<u8> {
        read_file(medium, &root().join(FILE)).unwrap()
    }

    fn replace(medium: &FakeFs, format: Format) -> io::Result<()> {
        replace_file(medium, &root(), format, FILE, b"new")
    }

    #[test]
    fn replaces_the_file_and_leaves_nothing_behind() {
        let medium = medium();

        replace(&medium, Format::ExFat).unwrap();

        assert_eq!(content(&medium), b"new");
        let temp = medium.read_dir(&root().join(TEMP_FOLDER)).unwrap();
        assert!(temp.is_empty());
    }

    #[test]
    fn a_leftover_temporary_file_does_not_get_in_the_way() {
        let medium = medium();
        let leftover = root().join(TEMP_FOLDER).join(REPLACEMENT);
        medium.add_file(&leftover, b"junk", SystemTime::UNIX_EPOCH);

        replace(&medium, Format::ExFat).unwrap();

        assert_eq!(content(&medium), b"new");
    }

    #[test]
    fn on_ext4_everyone_can_read_and_write_the_result() {
        let medium = medium();

        replace(&medium, Format::Ext4).unwrap();

        let path = root().join(FILE);
        assert_eq!(medium.metadata(&path).unwrap().mode, FILE_MODE);
        let folder = root().join(TEMP_FOLDER);
        assert_eq!(medium.metadata(&folder).unwrap().mode, DIR_MODE);
    }

    #[test]
    fn a_full_medium_leaves_the_old_file_and_no_temporary_file() {
        let medium = medium();
        medium.set_capacity(4);

        let error = replace(&medium, Format::ExFat).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::StorageFull);
        assert_eq!(content(&medium), b"old");
        assert!(
            medium
                .read_dir(&root().join(TEMP_FOLDER))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn pulling_the_medium_at_any_step_leaves_a_whole_file() {
        let steps = {
            let medium = medium();
            replace(&medium, Format::Ext4).unwrap();
            medium.steps()
        };
        for step in 1..=steps {
            for lose_unflushed in [false, true] {
                let medium = medium();
                medium.pull_at_step(step);

                assert!(replace(&medium, Format::Ext4).is_err(), "step {step}");

                if lose_unflushed {
                    medium.reinsert_losing_unflushed();
                } else {
                    medium.reinsert();
                }
                let content = content(&medium);
                assert!(content == b"old" || content == b"new", "step {step}");
            }
        }
    }

    #[test]
    fn the_temporary_file_is_flushed_before_it_is_renamed() {
        let steps = {
            let medium = medium();
            replace(&medium, Format::Ext4).unwrap();
            medium.steps()
        };
        let flushed_at = |step| {
            let medium = medium();
            medium.pull_at_step(step);
            let _ = replace(&medium, Format::Ext4);
            medium.reinsert();
            let temp = root().join(TEMP_FOLDER).join(REPLACEMENT);
            let written = read_file(&medium, &temp).is_ok_and(|data| data == b"new");
            written && medium.content_is_flushed(&temp)
        };

        assert!((1..=steps).any(flushed_at));
    }

    #[test]
    fn once_it_has_returned_the_new_file_survives_being_pulled() {
        let medium = medium();

        replace(&medium, Format::ExFat).unwrap();
        medium.pull_at_step(medium.steps() + 1);
        let _ = medium.remove_file(&root().join(FILE));
        medium.reinsert_losing_unflushed();

        assert_eq!(content(&medium), b"new");
    }
}
