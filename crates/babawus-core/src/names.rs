//! Which names can be stored on a medium, and when exFAT sees two names as the same.

use std::ffi::OsStr;

use crate::copy::Skip;
use crate::versions::{Format, has_version_folder_form};

/// The system's limit on a full path, in bytes.
pub const MAX_PATH: usize = 4096;

/// Why the source file or folder `name` can't be stored on a medium, if so.
pub fn name_problem(name: &OsStr, format: Format) -> Option<Skip> {
    if has_version_folder_form(name) {
        return Some(Skip::VersionFolderForm);
    }
    if format == Format::Ext4 {
        return None;
    }
    let Some(name) = name.to_str() else {
        return Some(Skip::NameNotUtf8);
    };
    if name.chars().any(is_invalid_on_exfat) {
        return Some(Skip::NameInvalidChars);
    }
    name.ends_with('.').then_some(Skip::NameTrailingDot)
}

fn is_invalid_on_exfat(character: char) -> bool {
    character < ' ' || "\"*:<>?\\|".contains(character)
}

/// A form of `name` that is the same for all names that exFAT treats as the same.
///
/// exFAT compares names using a table of upper-case letters stored on the medium. This covers
/// every pair in the table `mkfs.exfat` writes, which `tests/exfat_case.rs` checks. It also folds
/// some pairs that the table doesn't, such as `ı` and `I`. That can only report a clash where
/// there is none, which is visible and safe, while the opposite could mix up two files.
pub fn fold(name: &str) -> String {
    name.chars().map(fold_char).collect()
}

fn fold_char(character: char) -> char {
    let code = u32::from(character);
    // Greek letters with a subscript iota, which Unicode upper-cases to two letters.
    let greek = match code {
        0x1F80..=0x1F87 | 0x1F90..=0x1F97 | 0x1FA0..=0x1FA7 => Some(code + 8),
        0x1FB3 | 0x1FC3 | 0x1FF3 => Some(code + 9),
        _ => None,
    };
    if let Some(upper) = greek.and_then(char::from_u32) {
        return upper;
    }
    let mut upper = character.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(upper), None) => upper,
        _ => character,
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use super::*;

    fn problem(name: &str, format: Format) -> Option<Skip> {
        name_problem(OsStr::new(name), format)
    }

    #[test]
    fn ordinary_names_are_fine() {
        for name in [
            "report.pdf",
            "a b",
            " lead",
            "trail ",
            "[x]",
            "😀",
            "del\u{7f}",
            "..x",
        ] {
            assert_eq!(problem(name, Format::ExFat), None, "{name}");
        }
    }

    #[test]
    fn exfat_refuses_its_invalid_characters() {
        for name in [
            "a\"b", "a*b", "a:b", "a<b", "a>b", "a?b", "a\\b", "a|b", "a\tb", "a\u{1}b",
        ] {
            assert_eq!(
                problem(name, Format::ExFat),
                Some(Skip::NameInvalidChars),
                "{name}"
            );
            assert_eq!(problem(name, Format::Ext4), None, "{name}");
        }
    }

    #[test]
    fn exfat_refuses_a_trailing_dot() {
        assert_eq!(problem("name.", Format::ExFat), Some(Skip::NameTrailingDot));
        assert_eq!(
            problem("name..", Format::ExFat),
            Some(Skip::NameTrailingDot)
        );
        assert_eq!(problem("name.", Format::Ext4), None);
    }

    #[test]
    fn exfat_refuses_a_name_that_is_not_utf8() {
        let name = OsString::from_vec(b"caf\xe9".to_vec());

        assert_eq!(name_problem(&name, Format::ExFat), Some(Skip::NameNotUtf8));
        assert_eq!(name_problem(&name, Format::Ext4), None);
    }

    #[test]
    fn both_refuse_the_form_of_a_version_folder() {
        for format in [Format::ExFat, Format::Ext4] {
            assert_eq!(
                problem("report[]pdf", format),
                Some(Skip::VersionFolderForm)
            );
        }
    }

    #[test]
    fn folding_ignores_case() {
        assert_eq!(fold("Report.PDF"), fold("report.pdf"));
        assert_eq!(fold("ÉCOLE"), fold("école"));
        assert_ne!(fold("report"), fold("reports"));
    }

    #[test]
    fn folding_does_not_expand_letters() {
        assert_ne!(fold("ß"), fold("SS"));
        assert_ne!(fold("ﬁ"), fold("FI"));
    }

    #[test]
    fn folding_covers_greek_letters_with_a_subscript_iota() {
        assert_eq!(fold("ᾀ"), fold("ᾈ"));
        assert_eq!(fold("ᾳ"), fold("ᾼ"));
    }
}
