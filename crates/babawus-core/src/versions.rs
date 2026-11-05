//! The names of version folders and older versions, and which older versions to remove.
//!
//! These are plain functions on names. File names on Linux are bytes, so they work on bytes.

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

/// The file system on a medium.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    ExFat,
    Ext4,
}

const MAX_NAME: usize = 255;

/// `YYYY-MM-DD_HHMMSS`
const TIMESTAMP_FORM: &[u8] = b"dddd-dd-dd_dddddd";

/// Splits a name at its last dot. A name with no dot, or with only a leading dot, has no
/// extension.
fn split(name: &OsStr) -> (&[u8], Option<&[u8]>) {
    let bytes = name.as_bytes();
    match bytes.iter().rposition(|&byte| byte == b'.') {
        Some(dot) if dot > 0 => (&bytes[..dot], Some(&bytes[dot + 1..])),
        _ => (bytes, None),
    }
}

/// Whether a source file or folder has a name in the form of a version folder, so that it can't
/// be backed up.
pub fn has_version_folder_form(name: &OsStr) -> bool {
    name.as_bytes().windows(2).any(|pair| pair == b"[]")
}

/// The name of the version folder of the file `name`: `report[]pdf` for `report.pdf`.
pub fn version_folder(name: &OsStr) -> OsString {
    let (base, extension) = split(name);
    let mut folder = base.to_vec();
    folder.extend_from_slice(b"[]");
    folder.extend_from_slice(extension.unwrap_or_default());
    OsString::from_vec(folder)
}

/// The name of an older version of the file `name`: `report[2026-11-03_175400].pdf`, or with
/// `suffix` 2, `report[2026-11-03_175400-2].pdf`. A `suffix` of 1 means no suffix.
pub fn older_version(name: &OsStr, timestamp: &str, suffix: u32) -> OsString {
    let (base, extension) = split(name);
    let mut version = base.to_vec();
    version.push(b'[');
    version.extend_from_slice(timestamp.as_bytes());
    if suffix > 1 {
        version.extend_from_slice(format!("-{suffix}").as_bytes());
    }
    version.push(b']');
    if let Some(extension) = extension {
        version.push(b'.');
        version.extend_from_slice(extension);
    }
    OsString::from_vec(version)
}

/// Whether a single name fits on the medium: 255 bytes on Ext4, 255 characters on exFAT, which
/// counts in UTF-16. A name that isn't valid UTF-8 can't be stored on exFAT at all.
pub fn fits_on(name: &OsStr, format: Format) -> bool {
    match format {
        Format::Ext4 => name.len() <= MAX_NAME,
        Format::ExFat => name
            .to_str()
            .is_some_and(|name| name.encode_utf16().count() <= MAX_NAME),
    }
}

fn is_timestamp(bytes: &[u8]) -> bool {
    bytes.len() == TIMESTAMP_FORM.len()
        && bytes
            .iter()
            .zip(TIMESTAMP_FORM)
            .all(|(byte, form)| match form {
                b'd' => byte.is_ascii_digit(),
                form => byte == form,
            })
}

/// Reads the timestamp and suffix from `version`, if it is an older version of the file `name`.
///
/// The match is exact, because `archive` and `archive.` share the version folder `archive[]`, and
/// each must only count its own versions.
fn parse_version<'a>(version: &'a OsStr, name: &OsStr) -> Option<(&'a [u8], u32)> {
    let (base, extension) = split(name);
    let rest = version.as_bytes().strip_prefix(base)?.strip_prefix(b"[")?;
    let rest = match extension {
        Some(extension) => rest.strip_suffix(extension)?.strip_suffix(b".")?,
        None => rest,
    };
    let rest = rest.strip_suffix(b"]")?;
    let (timestamp, suffix) = rest.split_at_checked(TIMESTAMP_FORM.len())?;
    if !is_timestamp(timestamp) {
        return None;
    }
    if suffix.is_empty() {
        return Some((timestamp, 1));
    }
    let digits = std::str::from_utf8(suffix.strip_prefix(b"-")?).ok()?;
    let suffix = digits.parse().ok().filter(|&suffix| suffix > 1)?;
    digits
        .bytes()
        .all(|byte| byte.is_ascii_digit())
        .then_some((timestamp, suffix))
}

/// The older versions of the file `name` to remove from a version folder that holds `entries`,
/// so that `keep` remain. Anything that isn't an older version of this file is left alone.
///
/// The oldest are decided by the timestamp and suffix, not by sorting the names as text.
pub fn versions_to_remove(entries: &[OsString], name: &OsStr, keep: usize) -> Vec<OsString> {
    let mut versions: Vec<_> = entries
        .iter()
        .filter_map(|entry| parse_version(entry, name).map(|order| (order, entry)))
        .collect();
    versions.sort();
    let excess = versions.len().saturating_sub(keep);
    versions
        .into_iter()
        .take(excess)
        .map(|(_, entry)| entry.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIME: &str = "2026-11-03_175400";

    fn os(name: &str) -> &OsStr {
        OsStr::new(name)
    }

    fn names(names: &[&str]) -> Vec<OsString> {
        names.iter().map(OsString::from).collect()
    }

    #[test]
    fn names_follow_the_table_in_the_requirements() {
        let cases = [
            ("report.pdf", "report[]pdf", "report[2026-11-03_175400].pdf"),
            (
                "archive.tar.gz",
                "archive.tar[]gz",
                "archive.tar[2026-11-03_175400].gz",
            ),
            ("Makefile", "Makefile[]", "Makefile[2026-11-03_175400]"),
            (".bashrc", ".bashrc[]", ".bashrc[2026-11-03_175400]"),
        ];
        for (name, folder, version) in cases {
            assert_eq!(version_folder(os(name)), folder);
            assert_eq!(older_version(os(name), TIME, 1), version);
        }
    }

    #[test]
    fn a_second_version_in_the_same_second_gets_a_suffix() {
        let version = older_version(os("report.pdf"), TIME, 2);

        assert_eq!(version, "report[2026-11-03_175400-2].pdf");
    }

    #[test]
    fn a_name_ending_in_a_dot_keeps_its_dot_in_the_version() {
        assert_eq!(version_folder(os("archive.")), "archive[]");
        assert_eq!(
            older_version(os("archive."), TIME, 1),
            "archive[2026-11-03_175400]."
        );
    }

    #[test]
    fn names_that_are_not_utf8_work() {
        let name = OsString::from_vec(b"caf\xe9.txt".to_vec());

        let folder = version_folder(&name);

        assert_eq!(folder.as_bytes(), b"caf\xe9[]txt");
    }

    #[test]
    fn recognises_the_form_of_a_version_folder() {
        assert!(has_version_folder_form(os("report[]pdf")));
        assert!(has_version_folder_form(os("[]")));
        assert!(!has_version_folder_form(os("report[1].pdf")));
        assert!(!has_version_folder_form(os("report.pdf")));
    }

    #[test]
    fn ext4_counts_bytes() {
        assert!(fits_on(os(&"a".repeat(255)), Format::Ext4));
        assert!(!fits_on(os(&"a".repeat(256)), Format::Ext4));
        assert!(!fits_on(os(&"é".repeat(128)), Format::Ext4));
    }

    #[test]
    fn exfat_counts_characters() {
        assert!(fits_on(os(&"é".repeat(255)), Format::ExFat));
        assert!(!fits_on(os(&"é".repeat(256)), Format::ExFat));
    }

    #[test]
    fn exfat_counts_a_character_outside_the_basic_plane_as_two() {
        assert!(fits_on(os(&"😀".repeat(127)), Format::ExFat));
        assert!(!fits_on(os(&"😀".repeat(128)), Format::ExFat));
    }

    #[test]
    fn a_name_that_is_not_utf8_does_not_fit_on_exfat() {
        let name = OsString::from_vec(b"caf\xe9".to_vec());

        assert!(!fits_on(&name, Format::ExFat));
        assert!(fits_on(&name, Format::Ext4));
    }

    #[test]
    fn removes_the_oldest_until_keep_remain() {
        let entries = names(&[
            "report[2026-11-02_191230].pdf",
            "report[2026-10-20_141002].pdf",
            "report[2026-10-27_080544].pdf",
        ]);

        let remove = versions_to_remove(&entries, os("report.pdf"), 2);

        assert_eq!(remove, ["report[2026-10-20_141002].pdf"]);
    }

    #[test]
    fn removes_nothing_when_within_the_limit() {
        let entries = names(&["report[2026-10-20_141002].pdf"]);

        assert!(versions_to_remove(&entries, os("report.pdf"), 1).is_empty());
    }

    #[test]
    fn keeping_none_removes_all() {
        let entries = names(&["report[2026-10-20_141002].pdf"]);

        assert_eq!(versions_to_remove(&entries, os("report.pdf"), 0).len(), 1);
    }

    #[test]
    fn suffixes_are_numbers_not_text() {
        let entries = names(&[
            "report[2026-11-03_175400-10].pdf",
            "report[2026-11-03_175400-2].pdf",
            "report[2026-11-03_175400].pdf",
            "report[2026-11-03_175400-3].pdf",
        ]);

        let remove = versions_to_remove(&entries, os("report.pdf"), 1);

        assert_eq!(
            remove,
            [
                "report[2026-11-03_175400].pdf",
                "report[2026-11-03_175400-2].pdf",
                "report[2026-11-03_175400-3].pdf",
            ]
        );
    }

    #[test]
    fn leaves_anything_that_is_not_a_version_alone() {
        let entries = names(&[
            "notes.txt",
            "report[not-a-timestamp].pdf",
            "report[abcd-ef-gh_ijklmn].pdf",
            "report[2026-11-03_17540].pdf",
            "report[2026-11-03_175400-1].pdf",
            "report[2026-11-03_175400-x].pdf",
            "report[2026-11-03_175400].doc",
            "other[2026-11-03_175400].pdf",
        ]);

        assert!(versions_to_remove(&entries, os("report.pdf"), 0).is_empty());
    }

    #[test]
    fn files_sharing_a_version_folder_keep_their_own_versions() {
        let entries = names(&["archive[2026-11-03_175400]", "archive[2026-11-03_175400]."]);

        assert_eq!(
            versions_to_remove(&entries, os("archive"), 0),
            ["archive[2026-11-03_175400]"]
        );
        assert_eq!(
            versions_to_remove(&entries, os("archive."), 0),
            ["archive[2026-11-03_175400]."]
        );
    }

    #[test]
    fn a_name_with_brackets_works() {
        let entries = names(&["a[b][2026-11-03_175400].txt"]);

        assert_eq!(versions_to_remove(&entries, os("a[b].txt"), 0).len(), 1);
    }
}
