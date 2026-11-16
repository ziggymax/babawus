//! The built-in Help: putting it together, its text laid out in blocks, and finding words in it
//! (issues 151, 154 and 158).
//!
//! The Help is one text in each language file. A line starting with "# " is a heading, a line
//! starting with "- " is a bullet, and a line with only "#" and a number, such as "#14", begins
//! the entry for that numbered message. Other lines are paragraphs; blank lines only separate
//! them. [`build`] puts it together when the file is loaded: it writes each entry's message after
//! its number, as in "#14 The stick is full.", fills in references to other texts, such as
//! `{ChangeSetup}`, and shows plain quotation marks as the language's own.

use std::collections::BTreeMap;

use crate::language::placeholders;
use crate::text::Text;

/// The Help, put together, and what was wrong in it that didn't stop it from being shown.
#[derive(Debug, PartialEq, Eq)]
pub struct Built {
    pub text: String,
    pub warnings: Vec<String>,
}

/// Puts the Help `source` together, with `texts`, every other text of the language by its key,
/// and `quotes`, the language's opening and closing quotation marks. Gives why it can't be, if it
/// refers to something that isn't a text without placeholders.
pub fn build(
    source: &str,
    texts: &BTreeMap<String, String>,
    quotes: (&str, &str),
) -> Result<Built, String> {
    let mut warnings = Vec::new();
    let mut wrong = Vec::new();
    let mut lines = Vec::new();
    for line in source.lines() {
        if let Some((number, rest)) = entry(line) {
            if !rest.trim().is_empty() {
                warnings.push(format!(
                    "the Help has text after #{number}, which is left out, as Babawus writes the message there itself"
                ));
            }
            lines.push(format!("#{number} {}", heading(number, texts)));
            continue;
        }
        let quoted = match with_quotes(line, quotes) {
            Some(quoted) => quoted,
            None => {
                warnings.push(format!(
                    "a quotation mark in the Help has no partner, so the line is shown as written: {}",
                    line.trim()
                ));
                line.to_owned()
            }
        };
        lines.push(with_references(&quoted, texts, &mut wrong));
    }
    if !wrong.is_empty() {
        wrong.sort();
        wrong.dedup();
        return Err(format!(
            "the Help refers to what isn't a text without placeholders: {}",
            wrong.join(", ")
        ));
    }
    Ok(Built {
        text: lines.join("\n"),
        warnings,
    })
}

/// The message numbered `number`, as the user sees it, with each placeholder shown as "…", and
/// ending a sentence.
fn heading(number: u16, texts: &BTreeMap<String, String>) -> String {
    Text::message(number)
        .into_iter()
        .filter_map(|text| texts.get(&text.key()))
        .map(|text| {
            let mut shown = text.clone();
            for name in placeholders(text) {
                shown = shown.replace(&format!("{{{name}}}"), "…");
            }
            // "for {user}." becomes "for …", not "for ….".
            let shown = shown.trim_end().replace("….", "…");
            if shown.ends_with(['.', '!', '?', '…', ':']) {
                shown
            } else {
                format!("{shown}.")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `line` with its plain quotation marks shown as `quotes`, or nothing if they don't pair up. A
/// mark with a space, or the line's edge, on both sides is a character of its own, and stays.
fn with_quotes(line: &str, quotes: (&str, &str)) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut shown = String::with_capacity(line.len());
    let mut open = false;
    for (index, &c) in chars.iter().enumerate() {
        let space = |at: Option<&char>| at.is_none_or(|c| c.is_whitespace());
        let alone = space(index.checked_sub(1).and_then(|before| chars.get(before)))
            && space(chars.get(index + 1));
        if c == '"' && !alone {
            shown.push_str(if open { quotes.1 } else { quotes.0 });
            open = !open;
        } else {
            shown.push(c);
        }
    }
    (!open).then_some(shown)
}

/// `line` with each reference, such as `{ChangeSetup}`, replaced by that text. A reference to
/// anything but a text without placeholders is added to `wrong`.
fn with_references(
    line: &str,
    texts: &BTreeMap<String, String>,
    wrong: &mut Vec<String>,
) -> String {
    let mut shown = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find('{') {
        shown.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            shown.push_str(&rest[start..]);
            return shown;
        };
        let key = &after[..end];
        match texts.get(key) {
            Some(text) if key != Text::Help.name() && placeholders(text).is_empty() => {
                shown.push_str(text);
            }
            _ => wrong.push(format!("{{{key}}}")),
        }
        rest = &after[end + 1..];
    }
    shown.push_str(rest);
    shown
}

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

/// The number of the entry that `line` begins, as in "#14" or, once put together,
/// "#14 The stick is full.", if it begins one.
fn entry_number(line: &str) -> Option<u16> {
    entry(line).map(|(number, _)| number)
}

/// The number of the entry that `line` begins, and what follows it.
fn entry(line: &str) -> Option<(u16, &str)> {
    let rest = line.trim_end().strip_prefix('#')?;
    let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let after = &rest[digits..];
    if digits == 0 || !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    Some((rest[..digits].parse().ok()?, after))
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
    fn only_a_hash_followed_by_digits_alone_or_and_a_space_begins_an_entry() {
        assert_eq!(entry_number("#14 The stick is full."), Some(14));
        assert_eq!(entry_number("#14"), Some(14));
        assert_eq!(entry_number("#14  "), Some(14));
        assert_eq!(entry_number("#1a Nothing"), None);
        assert_eq!(entry_number("# Heading"), None);
        assert_eq!(entry_number("Search for #14."), None);
    }

    fn texts(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, text)| ((*key).to_owned(), (*text).to_owned()))
            .collect()
    }

    const QUOTES: (&str, &str) = ("»", "«");

    #[test]
    fn an_entry_gets_its_message_after_its_number() {
        let texts = texts(&[("ReasonMediumFull_14", "The stick is full.")]);

        let built = build(
            "#14
Free some space.",
            &texts,
            QUOTES,
        )
        .unwrap();

        assert_eq!(built.text, "#14 The stick is full.\nFree some space.");
        assert!(built.warnings.is_empty());
    }

    #[test]
    fn a_message_of_a_headline_and_an_advice_is_shown_whole_and_each_ends_a_sentence() {
        let texts = texts(&[
            ("StickNotOpened", "The stick isn't opened"),
            ("StickNotOpenedAdvice_1", "Pull it out and plug it back in."),
        ]);

        let built = build("#1", &texts, QUOTES).unwrap();

        assert_eq!(
            built.text,
            "#1 The stick isn't opened. Pull it out and plug it back in."
        );
    }

    #[test]
    fn a_placeholder_in_a_message_is_shown_as_an_ellipsis() {
        let texts = texts(&[("EjectFailed_34", "{label} could not be ejected")]);

        let built = build("#34", &texts, QUOTES).unwrap();

        assert_eq!(built.text, "#34 … could not be ejected.");
    }

    #[test]
    fn a_placeholder_that_ends_a_sentence_takes_no_second_full_stop() {
        let texts = texts(&[("SetupBusy_51", "It is in use for {user}. Wait.")]);

        let built = build("#51", &texts, QUOTES).unwrap();

        assert_eq!(built.text, "#51 It is in use for … Wait.");
    }

    #[test]
    fn text_after_an_entry_number_is_left_out_with_a_warning() {
        let texts = texts(&[("ReasonMediumFull_14", "The stick is full.")]);

        let built = build("#14 The disk is full.", &texts, QUOTES).unwrap();

        assert_eq!(built.text, "#14 The stick is full.");
        assert_eq!(
            built.warnings,
            [
                "the Help has text after #14, which is left out, as Babawus writes the message there itself"
            ]
        );
    }

    #[test]
    fn a_reference_is_filled_in_and_its_quotes_shown_as_the_languages() {
        let texts = texts(&[("ChangeSetup", "Ændr opsætning…")]);

        let built = build("Brug \"{ChangeSetup}\" her.", &texts, QUOTES).unwrap();

        assert_eq!(built.text, "Brug »Ændr opsætning…« her.");
    }

    #[test]
    fn a_reference_to_a_numbered_text_takes_its_full_key_and_has_no_number() {
        let texts = texts(&[("ReasonMediumFull_14", "The stick is full.")]);

        let built = build("It says {ReasonMediumFull_14}", &texts, QUOTES).unwrap();

        assert_eq!(built.text, "It says The stick is full.");
    }

    #[test]
    fn a_reference_to_anything_but_a_text_without_placeholders_rejects_the_help() {
        let texts = texts(&[
            ("Help", "…"),
            ("ReasonMediumFull_14", "The stick is full."),
            ("EjectNow", "Eject {label}"),
        ]);

        let error = build(
            "{Nonsense} {EjectNow} {Help} {ReasonMediumFull} {Nonsense}",
            &texts,
            QUOTES,
        )
        .unwrap_err();

        assert_eq!(
            error,
            "the Help refers to what isn't a text without placeholders: {EjectNow}, {Help}, {Nonsense}, {ReasonMediumFull}"
        );
    }

    #[test]
    fn a_quotation_mark_with_spaces_on_both_sides_is_a_character_of_its_own() {
        let built = build(
            "The characters \" * : and \"Photo.jpg\"",
            &texts(&[]),
            QUOTES,
        )
        .unwrap();

        assert_eq!(built.text, "The characters \" * : and »Photo.jpg«");
    }

    #[test]
    fn a_line_whose_quotes_dont_pair_up_is_shown_as_written_with_a_warning() {
        let built = build("Press \"Start and wait.", &texts(&[]), QUOTES).unwrap();

        assert_eq!(built.text, "Press \"Start and wait.");
        assert_eq!(
            built.warnings,
            [
                "a quotation mark in the Help has no partner, so the line is shown as written: Press \"Start and wait."
            ]
        );
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
