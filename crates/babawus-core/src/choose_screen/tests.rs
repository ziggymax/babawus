use std::time::SystemTime;

use super::*;
use crate::fs::fake::FakeFs;

fn en() -> Language {
    crate::language::testing::english()
}
const HOME: &str = "/home/anna";

/// A home folder with documents, an old folder, a temporary file, a cloud folder on another
/// drive, pictures, a hidden folder and a link.
fn system() -> FakeFs {
    let system = FakeFs::new();
    let home = PathBuf::from(HOME);
    system.add_dir_on_device(&home, 1);
    let file = |path: &str, bytes: usize| {
        system.add_file(home.join(path), &vec![b'x'; bytes], SystemTime::UNIX_EPOCH);
    };
    system.add_dir_on_device(home.join("Documents"), 1);
    file("Documents/letter.txt", 100);
    system.add_dir_on_device(home.join("Documents/old"), 1);
    file("Documents/old/draft.txt", 50);
    file("Documents/scratch.tmp", 10);
    system.add_dir_on_device(home.join("Documents/cloud"), 2);
    file("Documents/cloud/shared.txt", 1000);
    system.add_dir_on_device(home.join("Pictures"), 1);
    file("Pictures/cat.jpg", 200);
    system.add_dir_on_device(home.join(".config"), 1);
    file(".config/settings", 5);
    system.add_other(home.join("link"));
    system
}

fn choice(selected: &[&str], folders: &[&str]) -> Choice {
    Choice {
        selected: selected.iter().map(|s| s.to_string()).collect(),
        exclude_folders: folders.iter().map(|s| s.to_string()).collect(),
        exclude_patterns: vec!["*.tmp".to_owned()],
    }
}

fn entry(relative: &str, kind: FileKind) -> Entry {
    Entry {
        name: relative.rsplit('/').next().unwrap().to_owned(),
        relative: PathBuf::from(relative),
        kind,
        text: true,
    }
}

fn dir(relative: &str) -> Entry {
    entry(relative, FileKind::Dir)
}

fn file(relative: &str) -> Entry {
    entry(relative, FileKind::File)
}

fn row(choice: &Choice, entry: &Entry) -> Row {
    choice.row(&system(), Path::new(HOME), entry, en())
}

fn toggle(choice: &mut Choice, entry: &Entry) -> bool {
    choice.toggle(&system(), Path::new(HOME), entry)
}

fn count(choice: &Choice) -> Totals {
    choice
        .count(&system(), Path::new(HOME), &AtomicBool::new(false))
        .unwrap()
}

#[test]
fn a_selected_folder_ticks_everything_in_it() {
    let choice = choice(&["Documents"], &[]);

    assert!(row(&choice, &dir("Documents")).ticked);
    let inside = row(&choice, &file("Documents/letter.txt"));
    assert!(inside.ticked);
    assert!(
        !inside.can_toggle,
        "a file inside can't be left out on its own"
    );
    assert!(row(&choice, &dir("Documents/old")).can_toggle);
    assert!(!row(&choice, &dir("Pictures")).ticked);
}

#[test]
fn ticking_selects_and_unticking_unselects() {
    let mut choice = choice(&[], &[]);

    assert!(toggle(&mut choice, &dir("Pictures")));
    assert_eq!(choice.selected, ["Pictures"]);
    assert!(toggle(&mut choice, &dir("Pictures")));
    assert!(choice.selected.is_empty());
}

#[test]
fn unticking_a_folder_inside_a_ticked_one_leaves_it_out() {
    let mut choice = choice(&["Documents"], &[]);

    toggle(&mut choice, &dir("Documents/old"));

    assert_eq!(choice.exclude_folders, ["Documents/old"]);
    let shown = row(&choice, &dir("Documents/old"));
    assert!(!shown.ticked);
    assert!(shown.crossed);
    assert_eq!(shown.note.as_deref(), Some("Left out"));
    let below = row(&choice, &file("Documents/old/draft.txt"));
    assert!(!below.can_toggle);
    assert_eq!(
        below.note.as_deref(),
        Some("Left out with a folder above it")
    );

    toggle(&mut choice, &dir("Documents/old"));

    assert!(choice.exclude_folders.is_empty());
    assert!(row(&choice, &dir("Documents/old")).ticked);
}

#[test]
fn unticking_a_selected_folder_inside_a_ticked_one_leaves_it_out() {
    let mut choice = choice(&["Documents", "Documents/old"], &[]);

    toggle(&mut choice, &dir("Documents/old"));

    assert_eq!(choice.selected, ["Documents"]);
    assert_eq!(choice.exclude_folders, ["Documents/old"]);
}

#[test]
fn the_whole_home_folder_can_be_ticked() {
    let mut choice = choice(&[], &[]);

    toggle(&mut choice, &home_entry(en()));

    assert_eq!(choice.selected, ["."]);
    assert!(row(&choice, &dir("Pictures")).ticked);
}

#[test]
fn a_pattern_crosses_out_what_it_matches() {
    let choice = choice(&["Documents"], &[]);

    let shown = row(&choice, &file("Documents/scratch.tmp"));

    assert!(!shown.ticked);
    assert!(!shown.can_toggle);
    assert_eq!(
        shown.note.as_deref(),
        Some("Left out by the name pattern *.tmp")
    );
}

#[test]
fn a_folder_on_another_drive_is_only_backed_up_if_ticked_itself() {
    let mut choice = choice(&["Documents"], &[]);

    let shown = row(&choice, &dir("Documents/cloud"));
    assert!(!shown.ticked);
    assert!(shown.can_toggle);
    assert!(shown.note.unwrap().starts_with("On another drive"));

    toggle(&mut choice, &dir("Documents/cloud"));

    assert_eq!(choice.selected, ["Documents", "Documents/cloud"]);
    assert!(row(&choice, &dir("Documents/cloud")).ticked);
}

#[test]
fn a_link_can_not_be_ticked() {
    let shown = row(&choice(&["."], &[]), &entry("link", FileKind::Other));

    assert!(!shown.ticked);
    assert!(!shown.can_toggle);
}

#[test]
fn a_name_that_is_not_text_can_not_be_chosen_on_its_own() {
    let mut not_text = dir("Documents/odd");
    not_text.text = false;

    let alone = row(&choice(&[], &[]), &not_text);
    let inside = row(&choice(&["Documents"], &[]), &not_text);

    assert!(!alone.can_toggle);
    assert!(alone.note.is_some());
    assert!(inside.ticked, "still backed up with the folder above");
    assert!(!inside.can_toggle);
}

#[test]
fn a_left_out_folder_outside_the_selection_can_be_taken_back() {
    let mut choice = choice(&[], &[".local/share/Trash"]);
    let trash = dir(".local/share/Trash");

    assert!(row(&choice, &trash).crossed);
    toggle(&mut choice, &trash);

    assert!(choice.exclude_folders.is_empty());
}

#[test]
fn patterns_can_be_added_and_removed() {
    let mut choice = choice(&[], &[]);

    assert_eq!(choice.add_pattern("  node_modules "), Ok(()));
    assert_eq!(choice.add_pattern(""), Err(PatternProblem::Empty));
    assert_eq!(choice.add_pattern("a/b"), Err(PatternProblem::Slash));
    assert_eq!(choice.add_pattern("*.tmp"), Err(PatternProblem::Duplicate));
    assert_eq!(choice.exclude_patterns, ["*.tmp", "node_modules"]);
    assert!(pattern_problem_text(PatternProblem::Slash, en()).contains("untick it"));

    choice.remove_pattern("*.tmp");

    assert_eq!(choice.exclude_patterns, ["node_modules"]);
}

#[test]
fn the_tree_lists_folders_first_and_hides_hidden_entries() {
    let system = system();
    let home = Path::new(HOME);

    let hidden = children(&system, home, Path::new(""), false, &choice(&[], &[])).unwrap();
    let shown = children(&system, home, Path::new(""), true, &choice(&[], &[])).unwrap();
    let chosen = children(
        &system,
        home,
        Path::new(""),
        false,
        &choice(&[".config"], &[]),
    )
    .unwrap();

    let names = |entries: &[Entry]| entries.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
    assert_eq!(names(&hidden), ["Documents", "Pictures", "link"]);
    assert_eq!(names(&shown), [".config", "Documents", "Pictures", "link"]);
    assert_eq!(names(&chosen), [".config", "Documents", "Pictures", "link"]);
    assert_eq!(shown[3].kind, FileKind::Other);
}

#[test]
fn the_count_follows_the_rules_of_a_backup() {
    // Leaves out the pattern, the left-out folder and the other drive, and counts the
    // overlapping selection once.
    let choice = choice(&["Documents", "Documents/letter.txt"], &["Documents/old"]);

    assert_eq!(
        count(&choice),
        Totals {
            files: 1,
            bytes: 100
        }
    );
}

#[test]
fn a_folder_on_another_drive_counts_when_ticked_itself() {
    let choice = choice(&["Documents", "Documents/cloud", "Pictures"], &[]);

    assert_eq!(
        count(&choice),
        Totals {
            files: 4,
            bytes: 100 + 50 + 1000 + 200
        }
    );
}

#[test]
fn counting_stops_when_cancelled() {
    let choice = choice(&["Documents"], &[]);

    let counted = choice.count(&system(), Path::new(HOME), &AtomicBool::new(true));

    assert_eq!(counted, None);
}

#[test]
fn the_totals_compare_with_the_free_space() {
    let totals = Some(Totals {
        files: 12_345,
        bytes: 4_200_000_000,
    });

    let (fits, no_warning) = totals_text(totals, Some(15_500_000_000), en());
    let (_, warning) = totals_text(totals, Some(1_000_000_000), en());

    assert_eq!(
        fits,
        "Chosen: 12,345 files (4.2 GB) · 15.5 GB free on the stick"
    );
    assert_eq!(no_warning, None);
    assert!(
        warning
            .unwrap()
            .starts_with("That is more than the free space")
    );
    assert_eq!(totals_text(None, None, en()).0, "Counting…");
    assert_eq!(
        totals_text(Some(Totals::default()), None, en()).0,
        "Nothing is chosen yet."
    );
}

#[test]
fn a_ticked_folder_with_a_folder_left_out_is_partly_ticked() {
    let choice = choice(&["Documents"], &["Documents/old"]);

    let documents = row(&choice, &dir("Documents"));
    assert!(documents.ticked);
    assert!(documents.partly);
    assert!(row(&choice, &dir("")).partly, "the home folder too");
    assert!(!row(&choice, &dir("Documents/old")).partly);
    assert!(!row(&choice, &dir("Pictures")).partly);
}

#[test]
fn what_a_name_pattern_leaves_out_doesnt_make_a_folder_partly_ticked() {
    // `*.tmp` leaves out Documents/scratch.tmp.
    let choice = choice(&["Documents"], &[]);

    assert!(!row(&choice, &dir("Documents")).partly);
}

#[test]
fn an_unticked_folder_with_something_ticked_inside_is_partly_ticked() {
    let choice = choice(&["Documents/old"], &[]);

    let documents = row(&choice, &dir("Documents"));
    assert!(!documents.ticked);
    assert!(documents.partly);
    assert!(!row(&choice, &dir("Documents/old")).partly);
    assert!(!row(&choice, &file("Documents/letter.txt")).partly);
}

#[test]
fn a_whole_ticked_folder_is_not_partly_ticked() {
    let choice = choice(&["Documents"], &[]);

    assert!(!row(&choice, &dir("Documents/old")).partly);
    assert!(!row(&choice, &file("Documents/letter.txt")).partly);
}
