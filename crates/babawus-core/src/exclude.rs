//! The exclusions of an account: name patterns and specific folders.

use std::path::{Path, PathBuf};

/// What an account leaves out. An excluded folder is left out with everything in it.
#[derive(Clone, Debug, Default)]
pub struct Exclusions {
    /// Matched against every single name, at any depth.
    pub patterns: Vec<String>,
    /// Exact paths relative to the home folder.
    pub folders: Vec<PathBuf>,
}

impl Exclusions {
    /// Whether `relative`, a path relative to the home folder, is excluded.
    pub fn excludes(&self, relative: &Path) -> bool {
        let folder = |folder: &PathBuf| relative.starts_with(folder);
        let name = |name: &std::ffi::OsStr| {
            let name: Vec<char> = name.to_string_lossy().chars().collect();
            self.patterns.iter().any(|pattern| {
                let pattern: Vec<char> = pattern.chars().collect();
                !pattern.contains(&'/') && matches(&pattern, &name)
            })
        };
        self.folders.iter().any(folder) || relative.iter().any(name)
    }
}

/// Matches a pattern of `*` (any characters), `?` (one character) and `[…]` (one of the listed
/// characters). A `[` with no matching `]` stands for itself.
fn matches(pattern: &[char], name: &[char]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some(('*', rest)) => (0..=name.len()).any(|skip| matches(rest, &name[skip..])),
        Some(('?', rest)) => !name.is_empty() && matches(rest, &name[1..]),
        Some(('[', rest)) => match rest.iter().skip(1).position(|&c| c == ']') {
            Some(end) => {
                let (listed, after) = (&rest[..=end], &rest[end + 2..]);
                name.first().is_some_and(|c| listed.contains(c)) && matches(after, &name[1..])
            }
            None => literal('[', rest, name),
        },
        Some((&character, rest)) => literal(character, rest, name),
    }
}

fn literal(character: char, rest: &[char], name: &[char]) -> bool {
    name.first() == Some(&character) && matches(rest, &name[1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns(patterns: &[&str]) -> Exclusions {
        Exclusions {
            patterns: patterns.iter().map(ToString::to_string).collect(),
            folders: Vec::new(),
        }
    }

    fn excluded(pattern: &str, path: &str) -> bool {
        patterns(&[pattern]).excludes(Path::new(path))
    }

    #[test]
    fn star_matches_any_characters() {
        assert!(excluded("*.tmp", "file.tmp"));
        assert!(excluded("*.tmp", ".tmp"));
        assert!(excluded("*~", "notes.txt~"));
        assert!(excluded("a*c*e", "abcde"));
        assert!(!excluded("*.tmp", "file.tmp2"));
    }

    #[test]
    fn question_mark_matches_one_character() {
        assert!(excluded("file?.txt", "file1.txt"));
        assert!(excluded("?", "é"));
        assert!(!excluded("file?.txt", "file.txt"));
        assert!(!excluded("file?.txt", "file12.txt"));
    }

    #[test]
    fn brackets_match_one_listed_character() {
        assert!(excluded("file[12].txt", "file2.txt"));
        assert!(!excluded("file[12].txt", "file3.txt"));
        assert!(!excluded("file[12].txt", "file.txt"));
    }

    #[test]
    fn a_bracket_can_be_listed_first() {
        assert!(excluded("[]]", "]"));
    }

    #[test]
    fn an_open_bracket_stands_for_itself() {
        assert!(excluded("a[b", "a[b"));
        assert!(!excluded("a[b", "ab"));
    }

    #[test]
    fn patterns_match_at_any_depth() {
        assert!(excluded("node_modules", "project/node_modules/lib/a.js"));
        assert!(excluded("*.tmp", "a/b/c.tmp"));
    }

    #[test]
    fn patterns_match_whole_names() {
        assert!(!excluded("cache", ".cache"));
        assert!(!excluded("cache", "cached"));
    }

    #[test]
    fn patterns_are_case_sensitive() {
        assert!(!excluded("*.tmp", "file.TMP"));
    }

    #[test]
    fn a_pattern_with_a_slash_matches_nothing() {
        assert!(!excluded("a/b", "a/b"));
    }

    #[test]
    fn a_folder_is_excluded_with_everything_in_it() {
        let exclusions = Exclusions {
            patterns: Vec::new(),
            folders: vec![PathBuf::from("Documents/old")],
        };

        assert!(exclusions.excludes(Path::new("Documents/old")));
        assert!(exclusions.excludes(Path::new("Documents/old/a/b.txt")));
        assert!(!exclusions.excludes(Path::new("Documents/older")));
        assert!(!exclusions.excludes(Path::new("Documents")));
    }
}
