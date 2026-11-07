use super::*;

const EN: Language = Language::English;

fn tracker(messages: &[Progress]) -> Tracker {
    let mut tracker = Tracker::default();
    for &message in messages {
        tracker.update(message);
    }
    tracker
}

const OVERVIEW: Progress = Progress::Overview {
    files: 1204,
    bytes: 3_200_000_000,
};

#[test]
fn before_the_overview_the_run_is_getting_ready() {
    for messages in [&[][..], &[Progress::Scanning]] {
        let shown = tracker(messages).shown(EN);
        assert_eq!(shown.status, "Finding the files to back up…");
        assert!(shown.busy);
        assert_eq!(shown.overview, None);
        assert_eq!(shown.files_left, None);
        assert_eq!(shown.fraction, None);
        assert!(shown.can_cancel);
    }
}

#[test]
fn the_overview_comes_before_any_file_is_copied() {
    let shown = tracker(&[Progress::Scanning, OVERVIEW]).shown(EN);
    assert_eq!(shown.status, "Backing up…");
    assert!(!shown.busy);
    let overview = "To back up: 1,204 files (3.2 GB)";
    assert_eq!(shown.overview.as_deref(), Some(overview));
    assert_eq!(shown.files_left.as_deref(), Some("Files left: 1,204"));
    assert_eq!(shown.bytes_left.as_deref(), Some("Left to copy: 3.2 GB"));
    assert_eq!(shown.fraction, Some(0.0));
}

#[test]
fn the_counters_count_down_while_copying() {
    let copying = Progress::Copying {
        files_left: 300,
        bytes_left: 800_000_000,
    };
    let shown = tracker(&[OVERVIEW, copying]).shown(EN);
    assert_eq!(shown.files_left.as_deref(), Some("Files left: 300"));
    assert_eq!(shown.bytes_left.as_deref(), Some("Left to copy: 800.0 MB"));
    assert_eq!(shown.fraction, Some(0.75));
    // The overview stays.
    assert!(shown.overview.is_some());
}

#[test]
fn without_bytes_the_files_show_how_far_it_has_got() {
    let overview = Progress::Overview { files: 4, bytes: 0 };
    let copying = Progress::Copying {
        files_left: 1,
        bytes_left: 0,
    };
    assert_eq!(tracker(&[overview, copying]).shown(EN).fraction, Some(0.75));
    let nothing = Progress::Overview { files: 0, bytes: 0 };
    let shown = tracker(&[nothing]).shown(EN);
    assert_eq!(shown.fraction, Some(1.0));
    assert_eq!(
        shown.overview.as_deref(),
        Some("To back up: 0 files (0 bytes)")
    );
}

#[test]
fn one_file_is_not_files() {
    let one = Progress::Overview {
        files: 1,
        bytes: 2000,
    };
    let overview = tracker(&[one]).shown(EN).overview;
    assert_eq!(overview.as_deref(), Some("To back up: 1 file (2.0 kB)"));
}

#[test]
fn finishing_cannot_be_cancelled() {
    let shown = tracker(&[OVERVIEW, Progress::Finishing]).shown(EN);
    assert_eq!(shown.status, "Finishing… Don't pull out the stick yet.");
    assert!(shown.busy);
    assert!(!shown.can_cancel);
    assert_eq!(shown.fraction, Some(1.0));
}

#[test]
fn cancel_is_pressed_once() {
    let mut tracker = tracker(&[OVERVIEW]);
    tracker.cancel();
    let shown = tracker.shown(EN);
    assert_eq!(shown.status, "Stopping at a safe point…");
    assert!(!shown.can_cancel);
    tracker.update(Progress::Finishing);
    assert_eq!(
        tracker.shown(EN).status,
        "Finishing… Don't pull out the stick yet."
    );
}

#[test]
fn numbers_have_their_thousands_marked() {
    assert_eq!(number(0, EN), "0");
    assert_eq!(number(999, EN), "999");
    assert_eq!(number(1000, EN), "1,000");
    assert_eq!(number(1_234_567, EN), "1,234,567");
    assert_eq!(number(1_234_567, Language::German), "1.234.567");
}
