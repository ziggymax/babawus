//! The built-in Help: its text, laid out in blocks, and finding words in it (issues 151 and 154).
//!
//! The Help is one text in each language file. A line starting with "# " is a heading, a line
//! starting with "- " is a bullet, and a line starting with "#" and a number, such as
//! "#14 The stick is full.", begins the entry for that numbered message. Other lines are
//! paragraphs; blank lines only separate them.

use crate::text::Text;

/// One block of the Help.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Heading(String),
    /// The first line of the entry for a numbered message, number included.
    Entry(String),
    Bullet(String),
    Paragraph(String),
}

/// The blocks of the Help `text`, in order.
pub fn blocks(text: &str) -> Vec<Block> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            if let Some(heading) = line.strip_prefix("# ") {
                Block::Heading(heading.trim().to_owned())
            } else if entry_number(line).is_some() {
                Block::Entry(line.to_owned())
            } else if let Some(bullet) = line.strip_prefix("- ") {
                Block::Bullet(bullet.trim().to_owned())
            } else {
                Block::Paragraph(line.trim().to_owned())
            }
        })
        .collect()
}

/// The number of the entry that `line` begins, as in "#14 The stick is full.", if it begins one.
fn entry_number(line: &str) -> Option<u16> {
    let rest = line.strip_prefix('#')?;
    let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits == 0 || !rest[digits..].starts_with(' ') {
        return None;
    }
    rest[..digits].parse().ok()
}

/// The numbered messages that the Help `text` has no entry for, in order.
pub fn missing_entries(text: &str) -> Vec<u16> {
    let present: Vec<u16> = text.lines().filter_map(entry_number).collect();
    let mut numbers: Vec<u16> = Text::ALL.iter().filter_map(|text| text.number()).collect();
    numbers.sort_unstable();
    numbers.retain(|number| !present.contains(number));
    numbers
}

/// Where `needle` is found in `haystack`, ignoring upper and lower case, as the character
/// offset of each match's start and its length in characters, which is what a GTK text buffer
/// counts in. Matches don't overlap. An empty or blank needle finds nothing. A message's number,
/// such as "#1", only finds itself, not "#14".
pub fn find(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    let fold = |c: char| c.to_lowercase().next().unwrap_or(c);
    let needle: Vec<char> = needle.trim().chars().map(fold).collect();
    if needle.is_empty() {
        return Vec::new();
    }
    let number =
        needle.len() > 1 && needle[0] == '#' && needle[1..].iter().all(char::is_ascii_digit);
    let haystack: Vec<char> = haystack.chars().map(fold).collect();
    let mut found = Vec::new();
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        let end = start + needle.len();
        let cut_short = number && haystack.get(end).is_some_and(char::is_ascii_digit);
        if haystack[start..end] == needle[..] && !cut_short {
            found.push((start, needle.len()));
            start += needle.len();
        } else {
            start += 1;
        }
    }
    found
}

/// Which of `found`, the matches of `needle` in `haystack`, to show first: for a message's number,
/// such as "#14", its entry, which starts a line; otherwise the first.
pub fn first_to_show(haystack: &str, needle: &str, found: &[(usize, usize)]) -> usize {
    let needle = needle.trim();
    let is_number = needle.len() > 1
        && needle.starts_with('#')
        && needle[1..].chars().all(|c| c.is_ascii_digit());
    if !is_number {
        return 0;
    }
    let starts: Vec<char> = haystack.chars().collect();
    found
        .iter()
        .position(|&(offset, _)| offset == 0 || starts[offset - 1] == '\n')
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::testing::{english, german};

    #[test]
    fn lines_become_headings_entries_bullets_and_paragraphs() {
        let text = "# Backing up\n\n- Plug in.\n- Press start.\n\nKeep it safe.\n\n#14 The stick is full.\nFree some space.\n";

        assert_eq!(
            blocks(text),
            [
                Block::Heading("Backing up".to_owned()),
                Block::Bullet("Plug in.".to_owned()),
                Block::Bullet("Press start.".to_owned()),
                Block::Paragraph("Keep it safe.".to_owned()),
                Block::Entry("#14 The stick is full.".to_owned()),
                Block::Paragraph("Free some space.".to_owned()),
            ]
        );
    }

    #[test]
    fn only_a_hash_followed_by_digits_and_a_space_begins_an_entry() {
        assert_eq!(entry_number("#14 The stick is full."), Some(14));
        assert_eq!(entry_number("#14"), None);
        assert_eq!(entry_number("#1a Nothing"), None);
        assert_eq!(entry_number("# Heading"), None);
        assert_eq!(entry_number("Search for #14."), None);
    }

    #[test]
    fn the_numbers_without_an_entry_are_found() {
        let text: String = (1..=56)
            .filter(|number| ![3, 40].contains(number))
            .map(|number| format!("#{number} Something.\n"))
            .collect();

        assert_eq!(missing_entries(&text), [3, 40]);
    }

    #[test]
    fn english_and_german_have_an_entry_for_every_number() {
        for language in [english(), german()] {
            let help = Text::Help.get(language);
            assert!(missing_entries(help).is_empty(), "{language:?}");
            assert!(matches!(blocks(help)[0], Block::Heading(_)), "{language:?}");
        }
    }

    #[test]
    fn finding_ignores_case_and_counts_characters() {
        assert_eq!(find("Größe und GRÖSSE, größe", "größe"), [(0, 5), (18, 5)]);
        assert_eq!(find("#14 and #140", "#14"), [(0, 3)]);
        assert_eq!(find("#1 then #14", "#1"), [(0, 2)]);
        assert_eq!(find("aaaa", "aa"), [(0, 2), (2, 2)]);
    }

    #[test]
    fn a_number_search_shows_its_entry_first() {
        let text = "Search for a number, such as #14.\n#13 Full.\n#14 Full too.";
        let found = find(text, "#14");

        assert_eq!(found.len(), 2);
        assert_eq!(first_to_show(text, "#14", &found), 1);
        assert_eq!(first_to_show(text, "full", &find(text, "full")), 0);
    }

    #[test]
    fn nothing_is_found_for_a_blank_search() {
        assert!(find("text", "").is_empty());
        assert!(find("text", "  ").is_empty());
        assert!(find("text", "nothing").is_empty());
    }
}
