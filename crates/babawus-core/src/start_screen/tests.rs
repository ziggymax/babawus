use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;
use crate::log::LastBackup;
use crate::media::{Drive, Volume};
use crate::time::utc_clock;

fn en() -> Language {
    crate::language::testing::english()
}
const DAY: Duration = Duration::from_secs(86_400);
const HOUR: Duration = Duration::from_secs(3600);

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
    // 2024-10-04 12:00 UTC.
    SystemTime::UNIX_EPOCH + 20_000 * DAY + 12 * HOUR
}

fn details_of(checked: Option<&Checked>, advanced: bool) -> Details {
    let medium = medium("sdb", "BACKUP-A");
    let shown = Shown {
        medium: &medium,
        checked,
        formattable: true,
    };
    details(&shown, now(), utc_clock, en(), advanced)
}

fn actions(details: &Details) -> Vec<Action> {
    details.buttons.iter().map(|button| button.action).collect()
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
    let row = row(&medium, None, en());
    assert_eq!(row.label, "BACKUP-A");
    assert!(row.checking);
    assert!(!row.babawus);
    assert_eq!(row.tooltip, "Checking…");
    assert_eq!(row.eject_tooltip, "Make this stick safe to pull out");
}

#[test]
fn only_a_babawus_medium_has_a_coloured_stick() {
    let medium = medium("sdb", "BACKUP-A");
    assert!(row(&medium, Some(&checked(Status::NotSetUp)), en()).babawus);
    assert!(row(&medium, Some(&checked(Status::WriteProtected)), en()).babawus);
    assert!(!row(&medium, Some(&checked(Status::NotBackupMedium)), en()).babawus);
}

#[test]
fn the_details_say_what_the_medium_is() {
    let details = details_of(Some(&ready(None)), false);
    assert_eq!(details.label, "BACKUP-A");
    assert_eq!(details.status, "Ready");
    assert_eq!(details.mark, Some(Mark::Ready));
    assert_eq!(
        details.about,
        "Kingston DataTraveler 3.0 · 15.5 GB · 9.8 GB free · exFAT"
    );
    assert_eq!(details.explanation, None);
}

#[test]
fn the_free_space_is_left_out_until_it_is_known() {
    let details = details_of(None, false);
    assert_eq!(details.about, "Kingston DataTraveler 3.0 · 15.5 GB · exFAT");
    assert_eq!(details.status, "Checking…");
}

#[test]
fn file_systems_are_named_as_people_know_them() {
    assert_eq!(file_system_name("exfat"), Some("exFAT"));
    assert_eq!(file_system_name("ext4"), Some("Ext4"));
    assert_eq!(file_system_name("ext3"), Some("Ext3"));
    assert_eq!(file_system_name("ext2"), Some("Ext2"));
    assert_eq!(file_system_name("vfat"), Some("FAT"));
    assert_eq!(file_system_name("ntfs"), Some("NTFS"));
    assert_eq!(file_system_name("btrfs"), Some("btrfs"));
    assert_eq!(file_system_name(""), None);
}

#[test]
fn a_medium_without_a_file_system_names_none() {
    let mut medium = medium("sdb", "BACKUP-A");
    medium.volume.file_system = String::new();
    let shown = Shown {
        medium: &medium,
        checked: None,
        formattable: true,
    };
    let details = details(&shown, now(), utc_clock, en(), false);
    assert_eq!(details.about, "Kingston DataTraveler 3.0 · 15.5 GB");
}

#[test]
fn a_ready_medium_shows_the_last_backup() {
    let at = |days_ago: u32, skipped_files| LastBackup {
        time: now() - days_ago * DAY - Duration::from_secs(60),
        skipped_files,
    };
    let last = |checked: Checked| details_of(Some(&checked), false).last_backup;
    assert_eq!(last(ready(None)), Some(("No backup yet".to_owned(), false)));
    let today = Some(("Last backup: today at 11:59".to_owned(), false));
    assert_eq!(last(ready(Some(at(0, false)))), today);
    let yesterday = Some(("Last backup: yesterday at 11:59".to_owned(), true));
    assert_eq!(last(ready(Some(at(1, true)))), yesterday);
    let long_ago = Some((
        "Last backup: 47 days ago (2024-08-18 at 11:59)".to_owned(),
        false,
    ));
    assert_eq!(last(ready(Some(at(47, false)))), long_ago);
    assert_eq!(last(checked(Status::NotSetUp)), None);
}

#[test]
fn any_other_status_explains_what_to_do() {
    let details = details_of(Some(&checked(Status::NotSetUp)), false);
    assert_eq!(details.status, "Not set up for you");
    assert!(details.explanation.unwrap().contains("under Advanced"));
    let in_progress = checked(Status::InProgress {
        user: "anna".to_owned(),
    });
    let details = details_of(Some(&in_progress), false);
    assert_eq!(details.status, "Backup in progress (by anna)");
    assert!(details.explanation.unwrap().contains("anna"));
}

#[test]
fn start_backup_is_shown_for_a_babawus_medium_but_only_works_when_it_is_ready() {
    let details = details_of(Some(&ready(None)), false);
    let start = details.start_backup.unwrap();
    assert!(start.enabled);
    assert_eq!(start.tooltip, "Back up your files to this stick");

    let details = details_of(Some(&checked(Status::NeedsAttention)), false);
    let start = details.start_backup.unwrap();
    assert!(!start.enabled);
    assert_eq!(start.tooltip, "No backup can start: Needs attention");
}

#[test]
fn start_backup_is_not_shown_for_another_medium_or_until_it_is_checked() {
    assert_eq!(details_of(None, false).start_backup, None);
    let other = checked(Status::NotBackupMedium);
    assert_eq!(details_of(Some(&other), false).start_backup, None);
}

#[test]
fn the_users_actions_appear_when_they_apply() {
    let details = details_of(Some(&ready(None)), false);
    assert_eq!(
        actions(&details),
        [Action::ShowHistory, Action::WhatsBackedUp, Action::Restore]
    );

    let no_history = Checked {
        has_history: false,
        ..checked(Status::NotBackupMedium)
    };
    let details = details_of(Some(&no_history), false);
    assert_eq!(actions(&details), []);
}

#[test]
fn the_advanced_actions_only_appear_with_advanced() {
    let details = details_of(Some(&checked(Status::NotSetUp)), true);
    let all = [Action::ShowHistory, Action::Prepare, Action::ChangeSetup];
    assert_eq!(actions(&details), all);
    let details = details_of(Some(&checked(Status::NotBackupMedium)), true);
    assert!(!actions(&details).contains(&Action::ChangeSetup));
}

#[test]
fn every_action_on_a_ready_medium_has_its_own_tooltip() {
    let details = details_of(Some(&ready(None)), true);
    for button in &details.buttons {
        assert_ne!(button.tooltip, "Not yet available");
        assert!(!button.tooltip.is_empty(), "{:?}", button.action);
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

#[test]
fn a_stick_open_for_another_account_says_so_and_offers_nothing_to_change_it() {
    let other = Checked {
        has_history: false,
        ..checked(Status::OtherAccount)
    };
    let details = details_of(Some(&other), true);

    assert_eq!(details.mark, Some(Mark::Attention));
    assert_eq!(details.status, "Open for another account");
    assert!(
        details
            .explanation
            .as_deref()
            .unwrap()
            .contains("pull it out and plug it back in")
    );
    assert_eq!(actions(&details), []);
    assert_eq!(details.start_backup, None, "what is on it is unknown");

    let medium = medium("sdb", "BACKUP-A");
    let row = row(&medium, Some(&other), en());
    assert!(!row.babawus, "what is on it is unknown");
    assert!(!row.checking);
}

/// A snapshot with the stick of `medium("sdb", "BACKUP-A")`, its whole disk and partition, and
/// its file system mounted or not.
fn stick_snapshot(mounted: bool) -> Snapshot {
    let mut medium = medium("sdb", "BACKUP-A");
    if !mounted {
        medium.volume.mount_point = None;
    }
    let block = |path: &str, size| crate::media::Block {
        object_path: path.to_owned(),
        drive: medium.drive.object_path.clone(),
        size,
        ..crate::media::Block::default()
    };
    Snapshot {
        blocks: vec![
            block("/org/freedesktop/UDisks2/block_devices/sdb", 15_502_147_584),
            block(&medium.volume.object_path, 15_500_000_000),
        ],
        drives: vec![medium.drive],
        volumes: vec![medium.volume],
    }
}

#[test]
fn a_listed_stick_is_not_unlisted() {
    assert_eq!(unlisted(&stick_snapshot(true)), None);
    assert_eq!(unlisted(&Snapshot::default()), None);
}

#[test]
fn a_stick_whose_file_system_is_not_opened_is_unlisted() {
    assert_eq!(unlisted(&stick_snapshot(false)), Some(Unlisted::NotOpened));
}

#[test]
fn a_stick_without_a_file_system_cant_be_used() {
    let mut snapshot = stick_snapshot(false);
    snapshot.volumes.clear();

    assert_eq!(unlisted(&snapshot), Some(Unlisted::CantBeUsed));
}

#[test]
fn a_card_reader_without_a_card_is_not_a_stick() {
    let mut snapshot = stick_snapshot(false);
    snapshot.volumes.clear();
    for block in &mut snapshot.blocks {
        block.size = 0;
    }

    assert_eq!(unlisted(&snapshot), None);
}

#[test]
fn a_drive_that_is_not_a_usb_medium_is_never_unlisted() {
    let mut snapshot = stick_snapshot(false);
    snapshot.drives[0].removable = false;

    assert_eq!(unlisted(&snapshot), None);
}

#[test]
fn a_stick_that_isnt_opened_comes_before_one_that_cant_be_used() {
    let mut snapshot = stick_snapshot(false);
    let mut blank = medium("sdc", "BLANK").drive;
    blank.id.push_str("-blank");
    snapshot.blocks.push(crate::media::Block {
        object_path: "/org/freedesktop/UDisks2/block_devices/sdc".to_owned(),
        drive: blank.object_path.clone(),
        size: 1,
        ..crate::media::Block::default()
    });
    snapshot.drives.insert(0, blank);

    assert_eq!(unlisted(&snapshot), Some(Unlisted::NotOpened));
}

#[test]
fn the_empty_page_says_what_to_do() {
    assert_eq!(empty_text(false, true, None), (Text::NoUdisks, None));
    assert_eq!(
        empty_text(true, true, Some(Unlisted::NotOpened)),
        (Text::ChooseMedium, None)
    );
    assert_eq!(empty_text(true, false, None), (Text::PlugIn, None));
    assert_eq!(
        empty_text(true, false, Some(Unlisted::NotOpened)),
        (Text::StickNotOpened, Some(Text::StickNotOpenedAdvice))
    );
    assert_eq!(
        empty_text(true, false, Some(Unlisted::CantBeUsed)),
        (Text::StickCantBeUsed, Some(Text::StickCantBeUsedAdvice))
    );
}

#[test]
fn a_listed_medium_is_not_listed_again_as_a_stick_that_cant_be_used() {
    assert_eq!(unprepared(&stick_snapshot(true)), []);
    assert_eq!(unprepared(&Snapshot::default()), []);
}

#[test]
fn a_stick_that_isnt_opened_is_listed_by_its_name() {
    let sticks = unprepared(&stick_snapshot(false));

    assert_eq!(sticks.len(), 1);
    assert_eq!(sticks[0].label(en()), "BACKUP-A");
    let row = unprepared_row(&sticks[0], en());
    assert_eq!(row.label, "BACKUP-A");
    assert_eq!(row.tooltip, "The stick isn't opened");
    assert!(!row.babawus);
    assert!(!row.checking);
    let details = unprepared_details(&sticks[0], en(), false);
    assert_eq!(details.label, "BACKUP-A");
    assert_eq!(details.mark, Some(Mark::Unusable));
    assert_eq!(details.status, "The stick isn't opened");
    assert_eq!(
        details.about,
        "Kingston DataTraveler 3.0 · 15.5 GB · BACKUP-A (exFAT), not mounted"
    );
    assert_eq!(
        details.explanation.as_deref(),
        Some("Pull it out and plug it back in. #1")
    );
    assert_eq!(details.start_backup, None);
}

#[test]
fn a_stick_without_a_file_system_is_listed_by_its_size() {
    let mut snapshot = stick_snapshot(false);
    snapshot.volumes.clear();
    let sticks = unprepared(&snapshot);

    assert_eq!(sticks.len(), 1);
    assert_eq!(sticks[0].label(en()), "15.5 GB");
    assert_eq!(unprepared_row(&sticks[0], en()).label, "15.5 GB");
    let details = unprepared_details(&sticks[0], en(), false);
    assert_eq!(details.status, "Not a backup medium");
    assert_eq!(
        details.about,
        "Kingston DataTraveler 3.0 · 15.5 GB · No file system"
    );
    assert!(
        details
            .explanation
            .is_some_and(|explanation| explanation.contains("Prepare…"))
    );
}

#[test]
fn a_stick_that_cant_be_used_only_offers_prepare_and_only_under_advanced() {
    let sticks = unprepared(&stick_snapshot(false));

    assert_eq!(actions(&unprepared_details(&sticks[0], en(), false)), []);
    assert_eq!(
        actions(&unprepared_details(&sticks[0], en(), true)),
        [Action::Prepare]
    );
}

#[test]
fn a_stick_that_may_not_be_formatted_is_not_listed() {
    let mut snapshot = stick_snapshot(false);
    snapshot.blocks[0].hint_system = true;

    assert_eq!(unprepared(&snapshot), []);
}

#[test]
fn prepare_is_only_offered_for_a_medium_whose_drive_may_be_formatted() {
    let medium = medium("sdb", "BACKUP-A");
    let checked = ready(None);
    let shown = |formattable| Shown {
        medium: &medium,
        checked: Some(&checked),
        formattable,
    };

    let may = details(&shown(true), now(), utc_clock, en(), true);
    let may_not = details(&shown(false), now(), utc_clock, en(), true);

    assert!(actions(&may).contains(&Action::Prepare));
    assert!(!actions(&may_not).contains(&Action::Prepare));
}

#[test]
fn a_stick_that_cant_be_used_stays_selected_and_is_selected_when_alone() {
    let sticks = unprepared(&stick_snapshot(false));
    let drive = sticks[0].drive.object_path.clone();
    let media = [(medium("sdc", "BACKUP-B"), None)];

    assert_eq!(selected_row(&[], &sticks, None), Some(drive.clone()));
    assert_eq!(selected_row(&media, &sticks, None), None);
    assert_eq!(
        selected_row(&media, &sticks, Some(&drive)),
        Some(drive.clone())
    );
    let two = [sticks[0].clone(), sticks[0].clone()];
    assert_eq!(selected_row(&[], &two, None), None);
}

#[test]
fn a_medium_is_selected_as_before_beside_a_stick_that_cant_be_used() {
    let sticks = unprepared(&stick_snapshot(false));
    let media = [(medium("sdc", "BACKUP-B"), Some(ready(None)))];
    let path = media[0].0.volume.object_path.clone();

    assert_eq!(selected_row(&media, &sticks, None), Some(path.clone()));
    assert_eq!(
        selected_row(&media, &sticks, Some("/gone")),
        Some(path.clone())
    );
    assert_eq!(selected_row(&media, &[], Some(&path)), Some(path));
}
