use super::*;

const START_A: &str = "2026-11-12 16:24:07 +0100\t3c8d0c3f4316f9e0\tSTART\taccount=steen@laptop\tversion=0.1.0\tBackup started\n";
const ERROR_A: &str = "2026-11-12 16:24:07 +0100\t3c8d0c3f4316f9e0\tERROR\tcode=UNREADABLE\tpath=BabawusTest/secret.txt\n";
const END_A: &str =
    "2026-11-12 16:24:08 +0100\t3c8d0c3f4316f9e0\tEND\tverdict=SKIPPED\treason=FILES_SKIPPED\n";
const START_B: &str = "2026-11-12 17:00:00 +0100\t00112233445566aa\tSTART\taccount=anna@laptop\n";
const END_B: &str =
    "2026-11-12 17:01:00 +0100\t00112233445566aa\tEND\tverdict=ALL_GOOD\treason=NONE\n";

fn styles(line: &LogLine) -> Vec<Style> {
    line.pieces
        .iter()
        .map(|(_, style)| *style)
        .filter(|style| *style != Style::Plain)
        .collect()
}

#[test]
fn an_entry_shows_its_columns_with_short_run_ids_and_the_pairs_as_key_and_value() {
    let lines = log_lines(START_A.as_bytes());
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].text(),
        "2026-11-12 16:24:07 +0100  3c8d0c3f  START  account steen@laptop  version 0.1.0  Backup started"
    );
    assert_eq!(lines[0].run_id.as_deref(), Some("3c8d0c3f4316f9e0"));
    assert_eq!(
        styles(&lines[0]),
        [
            Style::Time,
            Style::RunId,
            Style::Start,
            Style::Key,
            Style::Value,
            Style::Key,
            Style::Value,
            Style::Text
        ]
    );
}

#[test]
fn the_entry_types_line_up_and_end_takes_its_verdict_s_colour() {
    let log = format!("{START_A}{ERROR_A}{END_A}{START_B}{END_B}");
    let lines = log_lines(log.as_bytes());
    assert!(lines[1].text().contains("  ERROR  code UNREADABLE  "));
    assert!(lines[2].text().contains("  END    verdict SKIPPED  "));
    assert_eq!(lines[1].pieces[4].1, Style::Error);
    assert_eq!(lines[2].pieces[4].1, Style::End(VerdictMark::Attention));
    assert_eq!(lines[4].pieces[4].1, Style::End(VerdictMark::Good));
}

#[test]
fn runs_are_shaded_alternately_in_the_order_they_started() {
    // The closing entry for an interrupted run can come later, among another run's lines.
    let log = format!("{START_A}{START_B}{END_B}{END_A}");
    let shaded: Vec<bool> = log_lines(log.as_bytes())
        .iter()
        .map(|line| line.shaded)
        .collect();
    assert_eq!(shaded, [false, true, true, false]);
}

#[test]
fn garbled_and_incomplete_lines_are_shown_as_they_are() {
    let log = format!("{START_A}not an entry\n{}", &END_A[..30]);
    let lines = log_lines(log.as_bytes());
    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[1].pieces,
        [("not an entry".to_owned(), Style::Garbled)]
    );
    assert_eq!(lines[1].run_id, None);
    // Without its line ending, the last line is incomplete, even if it looked whole.
    assert_eq!(lines[2].pieces[0].1, Style::Garbled);
    let whole_but_unended = START_B.trim_end();
    let lines = log_lines(whole_but_unended.as_bytes());
    assert_eq!(lines[0].pieces[0].1, Style::Garbled);
}

#[test]
fn an_empty_log_has_no_lines() {
    assert!(log_lines(b"").is_empty());
}

#[test]
fn a_theme_that_writes_in_a_light_colour_is_dark() {
    assert!(is_dark_theme((1.0, 1.0, 1.0)));
    assert!(is_dark_theme((0.93, 0.93, 0.93)));
    assert!(!is_dark_theme((0.0, 0.0, 0.0)));
    assert!(!is_dark_theme((0.2, 0.2, 0.2)));
}

/// The contrast between two "#rrggbb" colours, from 1 to 21, as WCAG 2 defines it.
fn contrast(a: &str, b: &str) -> f32 {
    let rgb = |colour: &str| {
        let channel =
            |at: usize| u8::from_str_radix(&colour[at..at + 2], 16).unwrap() as f32 / 255.0;
        (channel(1), channel(3), channel(5))
    };
    let (a, b) = (luminance(rgb(a)), luminance(rgb(b)));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn the_values_can_be_read_on_every_background_of_either_theme() {
    // The values are in the theme's own text colour: near white on a dark theme, near black on
    // a light one (issue 189). 4.5 is what WCAG asks of normal text.
    for (dark, text, window) in [(true, "#eeeeee", "#1e1e1e"), (false, "#1e1e1e", "#ffffff")] {
        let colours = Colours::of(dark);
        for background in [window, colours.shaded, colours.highlight] {
            assert!(
                contrast(text, background) >= 4.5,
                "dark {dark}, background {background}"
            );
        }
    }
}

#[test]
fn the_coloured_words_can_be_read_on_every_background_of_either_theme() {
    for (dark, window) in [(true, "#1e1e1e"), (false, "#ffffff")] {
        let colours = Colours::of(dark);
        let words = [
            colours.time,
            colours.run_id,
            colours.start,
            colours.error,
            colours.end_good,
            colours.end_attention,
            colours.end_failed,
            colours.key,
            colours.text,
        ];
        for background in [window, colours.shaded, colours.highlight] {
            for word in words {
                assert!(
                    contrast(word, background) >= 3.0,
                    "dark {dark}, {word} on {background}"
                );
            }
        }
    }
}
