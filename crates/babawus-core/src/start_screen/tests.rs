use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;
use crate::log::LastBackup;
use crate::media::{Drive, Volume};

const EN: Language = Language::English;
const DAY: Duration = Duration::from_secs(86_400);

fn medium(name: &str, label: &str) -> Medium {
    Medium {
        drive: Drive {
            object_path: format!("/org/freedesktop/UDisks2/drives/{name}"),
            id: format!("Kingston-DataTraveler-3.0-{name}"),
            vendor: "Kingston".to_owned(),
            model: "DataTraveler 3.0".to_owned(),
            size: 15_502_147_584,
            removable: true,
            connection_bus: "usb".to_owned(),
            can_power_off: true,
        },
        volume: Volume {
            object_path: format!("/org/freedesktop/UDisks2/block_devices/{name}1"),
            drive: format!("/org/freedesktop/UDisks2/drives/{name}"),
            label: label.to_owned(),
            file_system: "exfat".to_owned(),
            mount_point: Some(PathBuf::from(format!("/media/usera/{label}"))),
            read_only: false,
            size: 15_500_000_000,
        },
        root: PathBuf::from(format!("/media/usera/{label}")),
    }
}

fn checked(status: Status) -> Checked {
    Checked {
        status,
        free: Some(9_800_000_000),
        has_history: true,
    }
}

fn ready(last_backup: Option<LastBackup>) -> Checked {
    checked(Status::Ready { last_backup })
}

fn now() -> SystemTime {
    SystemTime::UNIX_EPOCH + 20_000 * DAY
}

fn details_of(checked: Option<&Checked>, advanced: bool) -> Details {
    let medium = medium("sdb", "BACKUP-A");
    let shown = Shown {
        medium: &medium,
        checked,
        formattable: true,
    };
    details(&shown, now(), EN, advanced)
}

fn actions(details: &Details) -> Vec<Action> {
    details.buttons.iter().map(|button| button.action).collect()
}

fn button(details: &Details, action: Action) -> &Button {
    details
        .buttons
        .iter()
        .find(|button| button.action == action)
        .unwrap()
}

#[test]
fn marks_group_the_statuses_by_what_can_be_done() {
    assert_eq!(Mark::of(&Status::Ready { last_backup: None }), Mark::Ready);
    assert_eq!(Mark::of(&Status::NotSetUp), Mark::Attention);
    let in_progress = Status::InProgress {
        user: "anna".to_owned(),
    };
    assert_eq!(Mark::of(&in_progress), Mark::Attention);
    assert_eq!(Mark::of(&Status::NeedsAttention), Mark::Attention);
    assert_eq!(Mark::of(&Status::WriteProtected), Mark::Unusable);
    assert_eq!(Mark::of(&Status::NotBackupMedium), Mark::Unusable);
}

#[test]
fn a_row_shows_a_spinner_and_a_grey_stick_until_it_is_checked() {
    let medium = medium("sdb", "BACKUP-A");
    let row = row(&medium, None, EN);
    assert_eq!(row.label, "BACKUP-A");
    assert_eq!(row.mark, None);
    assert!(!row.babawus);
    assert_eq!(row.tooltip, "Checking…");
}

#[test]
fn only_a_babawus_medium_has_a_coloured_stick() {
    let medium = medium("sdb", "BACKUP-A");
    assert!(row(&medium, Some(&checked(Status::NotSetUp)), EN).babawus);
    assert!(row(&medium, Some(&checked(Status::WriteProtected)), EN).babawus);
    assert!(!row(&medium, Some(&checked(Status::NotBackupMedium)), EN).babawus);
}

#[test]
fn the_details_say_what_the_medium_is() {
    let details = details_of(Some(&ready(None)), false);
    assert_eq!(details.label, "BACKUP-A");
    assert_eq!(details.status, "Ready");
    assert_eq!(details.mark, Some(Mark::Ready));
    assert_eq!(
        details.about,
        "Kingston DataTraveler 3.0 · 15.5 GB · 9.8 GB free"
    );
    assert_eq!(details.explanation, None);
}

#[test]
fn the_free_space_is_left_out_until_it_is_known() {
    let details = details_of(None, false);
    assert_eq!(details.about, "Kingston DataTraveler 3.0 · 15.5 GB");
    assert_eq!(details.status, "Checking…");
}

#[test]
fn a_ready_medium_shows_the_last_backup() {
    let at = |days_ago: u32, skipped_files| LastBackup {
        time: now() - days_ago * DAY - Duration::from_secs(60),
        skipped_files,
    };
    let last = |checked: Checked| details_of(Some(&checked), false).last_backup;
    assert_eq!(last(ready(None)), Some(("No backup yet".to_owned(), false)));
    let today = Some(("Last backup: today".to_owned(), false));
    assert_eq!(last(ready(Some(at(0, false)))), today);
    let yesterday = Some(("Last backup: yesterday".to_owned(), true));
    assert_eq!(last(ready(Some(at(1, true)))), yesterday);
    let long_ago = Some(("Last backup: 47 days ago".to_owned(), false));
    assert_eq!(last(ready(Some(at(47, false)))), long_ago);
    assert_eq!(last(checked(Status::NotSetUp)), None);
}

#[test]
fn any_other_status_explains_what_to_do() {
    let details = details_of(Some(&checked(Status::NotSetUp)), false);
    assert_eq!(details.status, "Not set up for you");
    assert!(
        details
            .explanation
            .unwrap()
            .contains("Contact your poweruser")
    );
    let in_progress = checked(Status::InProgress {
        user: "anna".to_owned(),
    });
    let details = details_of(Some(&in_progress), false);
    assert_eq!(details.status, "Backup in progress (by anna)");
    assert!(details.explanation.unwrap().contains("anna"));
}

#[test]
fn start_backup_is_always_shown_but_only_works_on_a_ready_medium() {
    let details = details_of(Some(&checked(Status::NeedsAttention)), false);
    let start = button(&details, Action::StartBackup);
    assert!(!start.enabled);
    assert_eq!(start.tooltip, "No backup can start: Needs attention");
    let details = details_of(None, false);
    assert!(!button(&details, Action::StartBackup).enabled);
}

#[test]
fn the_users_actions_appear_when_they_apply() {
    let details = details_of(Some(&ready(None)), false);
    let all = vec![
        Action::StartBackup,
        Action::ShowHistory,
        Action::WhatsBackedUp,
        Action::Eject,
    ];
    assert_eq!(actions(&details), all);

    let no_history = Checked {
        has_history: false,
        ..checked(Status::NotBackupMedium)
    };
    let details = details_of(Some(&no_history), false);
    assert_eq!(actions(&details), [Action::StartBackup, Action::Eject]);
}

#[test]
fn the_poweruser_actions_only_appear_with_advanced() {
    let details = details_of(Some(&checked(Status::NotSetUp)), true);
    let all = [
        Action::StartBackup,
        Action::ShowHistory,
        Action::Eject,
        Action::Prepare,
        Action::ChangeSetup,
    ];
    assert_eq!(actions(&details), all);
    let details = details_of(Some(&checked(Status::NotBackupMedium)), true);
    assert!(!actions(&details).contains(&Action::ChangeSetup));
}

#[test]
fn only_change_setup_is_not_yet_available() {
    let details = details_of(Some(&ready(None)), true);
    for button in &details.buttons {
        let available = button.action != Action::ChangeSetup;
        assert_eq!(button.enabled, available, "{:?}", button.action);
        if !available {
            assert_eq!(button.tooltip, "Not yet available");
        }
    }
}

#[test]
fn the_selection_follows_the_medium() {
    let a = medium("sda", "BACKUP-A");
    let b = medium("sdb", "BACKUP-B");
    let media = vec![(a, Some(ready(None))), (b.clone(), Some(ready(None)))];
    let selected = selection(&media, Some(&b.volume.object_path));
    assert_eq!(selected, Some(&b));
}

#[test]
fn a_single_ready_medium_is_selected_by_itself() {
    let a = medium("sda", "BACKUP-A");
    let b = medium("sdb", "PHOTOS");
    let media = vec![
        (a.clone(), Some(ready(None))),
        (b, Some(checked(Status::NotBackupMedium))),
    ];
    assert_eq!(selection(&media, None), Some(&a));
    // The selected medium was removed.
    assert_eq!(selection(&media, Some("/gone")), Some(&a));
}

#[test]
fn nothing_is_selected_when_the_choice_is_not_clear() {
    let a = medium("sda", "BACKUP-A");
    let b = medium("sdb", "BACKUP-B");
    let two_ready = vec![(a.clone(), Some(ready(None))), (b, Some(ready(None)))];
    assert_eq!(selection(&two_ready, None), None);
    let unchecked = vec![(a, None)];
    assert_eq!(selection(&unchecked, None), None);
    assert_eq!(selection(&[], None), None);
}
