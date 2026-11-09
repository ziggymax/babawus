//! Every text the user sees, in every language. See "Translations" under "Technology" in
//! Requirements.md: the language changes at once, without a restart, so the texts are looked up
//! each time they are shown, rather than through gettext.
//!
//! A text may hold placeholders such as `{label}`, which [`Text::fill`] replaces. Every
//! translation of a text has the same placeholders as the English one.

use crate::settings::Language;

macro_rules! texts {
    ($($name:ident: $english:literal, $german:literal;)*) => {
        /// A text the user sees.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Text {
            $($name,)*
        }

        impl Text {
            /// Every text, for the tests.
            pub const ALL: &[Self] = &[$(Self::$name,)*];

            /// The text in each language, in the order of [`Language::ALL`]. An empty text has
            /// not been translated yet.
            fn translations(self) -> [&'static str; Language::ALL.len()] {
                match self {
                    $(Self::$name => [$english, $german],)*
                }
            }
        }
    };
}

// TODO(T-020): Translate into German.
texts! {
    WindowTitle: "Backup", "";
    Menu: "Menu", "";
    MenuLanguage: "Language", "";
    MenuAdvanced: "Advanced", "";
    MenuAbout: "About Babawus", "";
    AdvancedMarker: "Advanced", "";
    AdvancedMarkerTooltip: "The poweruser's actions are shown. Choose Advanced in the menu again to hide them.", "";
    AboutComments: "Basic backup with USB storage", "";
    Logo: "Babawus", "";
    PlugIn: "Plug in your backup stick", "";
    ChooseMedium: "Choose a stick in the list", "";
    AlreadyOpen: "Babawus is already open for {user} on this computer", "";
    NoUdisks: "Babawus can't see the USB sticks, because the udisks2 service isn't running. Contact your poweruser.", "";

    Checking: "Checking…", "";
    Ready: "Ready", "";
    NotSetUp: "Not set up for you", "";
    InProgress: "Backup in progress (by {user})", "";
    NeedsAttention: "Needs attention", "";
    WriteProtected: "Write-protected", "";
    NotBackupMedium: "Not a backup medium", "";

    ExplainNotSetUp: "This stick hasn't been set up for you on this computer. Contact your poweruser.", "";
    ExplainInProgress: "A backup to this stick is running for {user}. Wait until it has finished.", "";
    ExplainNeedsAttention: "The setup on this stick is damaged, or needs a newer Babawus. Contact your poweruser.", "";
    ExplainWriteProtected: "This stick can only be read, so no backup can be written to it. If it has a lock switch, slide it to unlocked. Otherwise contact your poweruser.", "";
    ExplainNotBackupMedium: "This stick isn't set up for backups with Babawus. Your poweruser can prepare it.", "";

    LastBackupToday: "Last backup: today", "";
    LastBackupYesterday: "Last backup: yesterday", "";
    LastBackupDaysAgo: "Last backup: {days} days ago", "";
    NoBackupYet: "No backup yet", "";
    SkippedMark: "Some files were skipped in the last backup", "";

    Free: "{size} free", "";

    StartBackup: "Start backup", "";
    StartBackupTooltip: "Back up your files to this stick", "";
    ShowHistory: "Show history", "";
    ShowHistoryTooltip: "See when backups to this stick ran, and how they went", "";
    WhatsBackedUp: "What's backed up", "";
    WhatsBackedUpTooltip: "See which of your files and folders are backed up to this stick", "";
    Eject: "Eject", "";
    EjectTooltip: "Make this stick safe to pull out", "";
    Prepare: "Prepare…", "";
    PrepareTooltip: "Erase this stick and set it up for backups with Babawus", "";
    ChangeSetup: "Change setup…", "";
    ChangeSetupTooltip: "Change what is backed up to this stick, and for whom", "";
    NotYetAvailable: "Not yet available", "";
    CantStart: "No backup can start: {reason}", "";

    GettingReady: "Finding the files to back up…", "";
    BackingUp: "Backing up…", "";
    Stopping: "Stopping at a safe point…", "";
    Finishing: "Finishing… Don't pull out the stick yet.", "";
    Overview: "To back up: {files} files ({size})", "";
    OverviewOneFile: "To back up: 1 file ({size})", "";
    FilesLeft: "Files left: {files}", "";
    BytesLeft: "Left to copy: {size}", "";
    Cancel: "Cancel", "";
    CancelTooltip: "Stop the backup", "";
    StopQuestion: "Stop the backup?", "";
    StopDetail: "The files copied so far stay on the stick, but the backup won't be finished.", "";
    KeepGoing: "Keep going", "";
    StopBackup: "Stop backup", "";
    Back: "Back", "";
    BackTooltip: "Go back to the list of sticks", "";
    VerdictBusy: "Another backup to this stick is just starting", "";
    NextWait: "Wait until it has finished, then try again.", "";
    NextRunAgain: "Run the backup again. If it happens again, contact your poweruser.", "";
    NextContact: "Contact your poweruser.", "";
    NothingToCopy: "Everything was already up to date.", "";
    Copied: "Copied: {files} ({size})", "";
    OneFile: "1 file", "";
    Files: "{files} files", "";
    SkippedGroup: "{reason}: {files}", "";

    ReasonFilesSkipped: "Some files could not be backed up.", "";
    ReasonCancelled: "You stopped the backup.", "";
    ReasonMediumRemoved: "The stick was removed.", "";
    ReasonInterrupted: "The backup was interrupted.", "";
    ReasonCrashed: "Babawus stopped unexpectedly.", "";
    ReasonFlushFailed: "The last changes could not be saved on the stick.", "";
    ReasonMediumFull: "The stick is full.", "";
    ReasonNotAdded: "This stick isn't set up for you on this computer.", "";
    ReasonConfigDamaged: "The setup on this stick is damaged, or needs a newer Babawus.", "";
    ReasonCannotWrite: "The stick can't be written to.", "";

    SkipUnreadable: "Couldn't be read", "";
    SkipDisappeared: "Disappeared during the backup", "";
    SkipNotWritable: "Couldn't be written to the stick", "";
    SkipTypeConflict: "A folder of the same name is on the stick", "";
    SkipChangedDuringCopy: "Kept changing while being copied", "";
    SkipVersionNameTooLong: "Name too long to keep older versions", "";
    SkipNameInvalidChars: "Name has characters this stick can't store", "";
    SkipNameTrailingDot: "Name ends with a dot, which this stick can't store", "";
    SkipNameNotUtf8: "Name isn't valid text", "";
    SkipPathTooLong: "Name or folder path too long", "";
    SkipVersionFolderForm: "Name looks like a folder of older versions", "";
    SkipCaseClash: "Name differs only in upper and lower case from another, which this stick can't tell apart", "";
    SkipSelectionMissing: "Chosen for backup, but doesn't exist any more", "";
    SkipOutsideHome: "Chosen for backup, but outside your home folder", "";

    DetailsTitle: "Details for your poweruser", "";
    DetailsTooltip: "What happened, in a form your poweruser can use", "";
    CopyDetails: "Copy", "";
    CopyDetailsTooltip: "Copy the details, to paste them into an email or a chat", "";
    DetailsCopied: "Copied. You can paste the details into an email or a chat.", "";
    DetailsVerdict: "Verdict: {verdict} ({code})", "";
    DetailsReason: "Reason: {reason} ({code})", "";
    DetailsTime: "Time: {time}", "";
    DetailsAccount: "Account: {account}", "";
    DetailsMedium: "Medium: {label}", "";
    DetailsCounts: "Files found: {scanned}, copied: {copied} ({size}), skipped: {skipped}, errors: {errors}, older versions removed: {removed}", "";
    DetailsFree: "Free on the medium: {size}", "";
    DetailsProblems: "First problems:", "";
    DetailsMore: "  … and {more} more, listed in the log on the medium", "";
    EjectNow: "Eject {label}", "";
    EjectNowTooltip: "Make the stick safe to pull out now", "";
    VerdictAllGood: "All good", "";
    VerdictSkipped: "Some files were skipped", "";
    VerdictNotFinished: "Backup not finished", "";
    VerdictFailed: "Backup failed", "";

    Ejecting: "Ejecting {label}…", "";
    Ejected: "You can pull out {label} now.", "";
    EjectedUnmountedOnly: "You can pull out {label} safely now, even if its light stays on.", "";
    EjectBusy: "{label} is still in use by a program, for example an open file or a file manager window. Close it and try again.", "";
    EjectGone: "{label} has already been removed.", "";
    EjectFailed: "{label} could not be ejected. Contact your poweruser.", "";

    HistoryHeading: "Backups to this stick", "";
    HistoryEmpty: "No backups to this stick yet.", "";
    HistoryUnreadable: "The history on this stick can't be read. Go back and try again.", "";
    HistoryRunning: "Backup in progress", "";
    HistoryToday: "Today, {time}", "";
    HistoryYesterday: "Yesterday, {time}", "";
    HistoryDate: "{date}, {time}", "";
    HistoryWho: "By {user} on {computer}", "";
    HistorySkipped: "Skipped: {files}", "";

    CoverageHeading: "What's backed up to this stick", "";
    CoverageIntro: "These are backed up from this computer for {user}:", "";
    CoverageNothing: "Nothing has been chosen for backup yet. Contact your poweruser.", "";
    CoverageUnreadable: "The setup on this stick can't be read. Go back and try again.", "";
    CoverageHome: "Your home folder", "";
    CoverageMissing: "Doesn't exist any more, so nothing is backed up from it", "";
    CoverageExcluded: "Not backed up, because it is left out below", "";
    CoverageOutsideHome: "Not backed up, because it is outside your home folder", "";
    CoverageLink: "Not backed up, because it is a link, which Babawus doesn't follow", "";
    CoverageUnreadableItem: "Can't be read, so it can't be backed up", "";
    CoverageLeftOut: "Left out:", "";
    CoveragePatterns: "Anything named {names}, in any folder", "";
    Either: "{first} or {last}", "";
    Loading: "Reading the stick…", "";

    FormatHeading: "Prepare a stick", "";
    FormatIntro: "Formatting erases a stick and sets it up for backups with Babawus. Choose the stick:", "";
    FormatNoDrives: "No stick that Babawus may format is plugged in.", "";
    FormatNoFileSystem: "No file system", "";
    FormatNoLabel: "No name", "";
    FormatWithBackups: "{volume}, with Babawus backups", "";
    FormatNotMounted: "{volume}, not mounted", "";
    FormatLabel: "Name for the stick:", "";
    FormatLabelRule: "Use 1 to 11 letters, digits and dashes, with no spaces.", "";
    FormatFileSystem: "File system:", "";
    FormatExFat: "exFAT", "";
    FormatExFatDetail: "Also works on Windows and Mac", "";
    FormatExt4: "Ext4", "";
    FormatExt4Detail: "Works on Linux only, but allows longer names and copes better with being pulled out", "";
    FormatWarning: "Everything on {drive} will be erased: {contents}. This includes every partition on it. Formatting can't be stopped once it has started.", "";
    FormatUnderstood: "I understand that everything on this stick will be erased", "";
    FormatErase: "Erase and format", "";
    FormatEraseTooltip: "Erase the stick and set it up for backups with Babawus", "";
    FormatWorking: "Formatting {label}…", "";
    FormatWorkingDetail: "This can take a few minutes. Don't pull out the stick.", "";
    FormatDone: "{label} is ready", "";
    FormatDoneDetail: "The stick is empty and set up for Babawus. Before anyone can back up to it, it must be set up for each computer and account that will use it.", "";
    FormatFailed: "Formatting failed", "";
    FormatGone: "The stick was removed.", "";
    FormatNotAllowed: "Babawus may no longer format this stick, for example because it is now in use.", "";
    FormatNotPrepared: "The stick was formatted, but Babawus couldn't set it up for backups.", "";
    FormatFailedReason: "Formatting didn't work.", "";
    FormatNothingErased: "Nothing on the stick was changed.", "";
    FormatPartlyErased: "The stick has been erased, but isn't ready for backups. Format it again.", "";
    FormatMessage: "The system said: {message}", "";
    AddComputer: "Add this computer and account…", "";
    AddComputerTooltip: "Set up this stick for you on this computer", "";
    TryAgain: "Try again", "";
    TryAgainTooltip: "Go back to choosing the stick and how to format it", "";

    SetupHeading: "Setup of this stick", "";
    SetupLoading: "Reading the setup…", "";
    SetupDamaged: "The setup on this stick was changed outside Babawus, or is damaged. Check it, then save it.", "";
    SetupUnreadable: "The setup on this stick can't be read at all. You can start a new setup. The backups on the stick are kept, and their folders stay in use.", "";
    SetupStartNew: "Start a new setup…", "";
    SetupStartNewQuestion: "Start a new setup?", "";
    SetupStartNewDetail: "The old setup is replaced when you save. The backups on the stick are kept.", "";
    SetupNewerFormat: "The setup on this stick needs a newer Babawus. It can't be changed here.", "";
    SetupNotBabawus: "This stick has no Babawus setup any more.", "";
    SetupOlderVersions: "Older versions to keep of each file:", "";
    SetupOlderVersionsDetail: "When a file changes, its earlier copies are kept, up to this many. Fewer leaves more space on the stick.", "";
    SetupHere: "You on this computer", "";
    SetupNoComputerId: "This computer has no ID, so it can't be set up. Contact whoever looks after this computer.", "";
    SetupComputerMissing: "This computer ({hostname}) isn't set up on this stick yet.", "";
    SetupComputerMissingLink: "This computer ({hostname}) isn't set up on this stick yet. Add it, or, if it was set up before under another entry, for example before a reinstall, link it to that entry.", "";
    SetupAccountMissing: "Your account, {user}, isn't set up on this computer ({folder}) yet.", "";
    SetupAccountMissingLink: "Your account, {user}, isn't set up on this computer ({folder}) yet. Add it, or, if your user name has changed, link it to your earlier name.", "";
    SetupHereReady: "Your account, {user}, is set up on this computer ({folder}).", "";
    SetupAddComputer: "Add this computer as {folder}", "";
    SetupLinkComputer: "Link to {folder}…", "";
    SetupAddAccount: "Add {user}", "";
    SetupLinkAccount: "Link to {old}…", "";
    SetupChoose: "Choose what to back up…", "";
    SetupEntries: "Computers and accounts on this stick", "";
    SetupEmpty: "No computers or accounts are set up on this stick yet.", "";
    SetupThisComputer: "{folder} (this computer)", "";
    SetupOtherComputer: "{folder} · last seen as {hostname}", "";
    SetupYou: "{user} (you)", "";
    SetupNothingChosen: "nothing chosen yet", "";
    SetupOneChosen: "1 file or folder chosen", "";
    SetupChosen: "{count} files or folders chosen", "";
    SetupRemove: "Remove…", "";
    SetupRemoveComputerTooltip: "Take this computer and its accounts out of the setup", "";
    SetupRemoveAccountTooltip: "Take this account out of the setup", "";
    SetupRemoveComputerQuestion: "Remove {folder} and its accounts from the setup?", "";
    SetupRemoveAccountQuestion: "Remove {user} on {folder} from the setup?", "";
    SetupRemoveDetail: "Its backups stay on the stick, and you can delete them by hand if you want the space back. Nothing changes until you save.", "";
    SetupRemoveConfirm: "Remove", "";
    SetupLinkComputerQuestion: "Link this computer to {folder}?", "";
    SetupLinkComputerDetail: "Do this only if {folder} was this computer before, for example before a reinstall. Its backups then go on in the same folder. Nothing changes until you save.", "";
    SetupLinkAccountQuestion: "Link your account to {old}?", "";
    SetupLinkAccountDetail: "Do this only if {old} was your user name before. Its backups and history are renamed to {user}, and the setup is saved now.", "";
    SetupLinkConfirm: "Link", "";
    SetupSave: "Save", "";
    SetupSaveTooltip: "Write the changes to the stick", "";
    SetupSaved: "Saved.", "";
    SetupSaving: "Saving…", "";
    SetupDiscardQuestion: "Leave without saving?", "";
    SetupDiscardDetail: "Your changes to the setup of this stick will be lost.", "";
    SetupDiscard: "Leave without saving", "";
    SetupKeepEditing: "Keep editing", "";
    SetupBusy: "The stick is in use by a backup for {user}. Wait until it has finished, then save again.", "";
    SetupInvalid: "That change can't be made, because it clashes with another entry.", "";
    SetupBothFolders: "Both your earlier and your new name have a folder on the stick, so Babawus can't link them. Your poweruser has to sort this out by hand.", "";
    SetupSaveFailed: "The setup couldn't be saved: {error}", "";

    Bytes: "{size} bytes", "";
    Kilobytes: "{size} kB", "";
    Megabytes: "{size} MB", "";
    Gigabytes: "{size} GB", "";
    Terabytes: "{size} TB", "";
}

/// The languages whose texts are complete. The tests check them.
// TODO(T-020): Add German.
pub const COMPLETE: &[Language] = &[Language::English];

impl Text {
    /// The text in `language`. A text that has not been translated yet is shown in English.
    pub fn get(self, language: Language) -> &'static str {
        let translations = self.translations();
        let text = translations[index(language)];
        if text.is_empty() {
            translations[index(Language::English)]
        } else {
            text
        }
    }

    /// The text in `language`, with each placeholder `{name}` in `values` replaced.
    pub fn fill(self, language: Language, values: &[(&str, &str)]) -> String {
        values
            .iter()
            .fold(self.get(language).to_owned(), |text, (name, value)| {
                text.replace(&format!("{{{name}}}"), value)
            })
    }
}

fn index(language: Language) -> usize {
    Language::ALL
        .iter()
        .position(|&each| each == language)
        .expect("every language is in Language::ALL")
}

/// A size in bytes, as on the packaging of a USB stick: in powers of 1000, with one decimal, for
/// example "15.5 GB".
pub fn size(bytes: u64, language: Language) -> String {
    const UNITS: [Text; 4] = [
        Text::Kilobytes,
        Text::Megabytes,
        Text::Gigabytes,
        Text::Terabytes,
    ];
    if bytes < 1000 {
        return Text::Bytes.fill(language, &[("size", &bytes.to_string())]);
    }
    let mut value = bytes as f64 / 1000.0;
    let mut unit = 0;
    // Rounding to one decimal can reach 1000, which then reads better in the next unit.
    while value >= 999.95 && unit + 1 < UNITS.len() {
        value /= 1000.0;
        unit += 1;
    }
    let number = format!("{value:.1}");
    let number = match language {
        Language::English => number,
        Language::German => number.replace('.', ","),
    };
    UNITS[unit].fill(language, &[("size", &number)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(text: &str) -> Vec<&str> {
        let mut found: Vec<&str> = text
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect();
        found.sort_unstable();
        found
    }

    #[test]
    fn every_text_exists_in_every_complete_language() {
        for &language in COMPLETE {
            for &text in Text::ALL {
                let translated = text.translations()[index(language)];
                assert!(!translated.is_empty(), "{text:?} has no {language:?}");
            }
        }
    }

    #[test]
    fn every_translation_has_the_placeholders_of_the_english_text() {
        for &text in Text::ALL {
            let english = placeholders(text.get(Language::English));
            for language in Language::ALL {
                assert_eq!(
                    placeholders(text.get(language)),
                    english,
                    "{text:?} in {language:?}"
                );
            }
        }
    }

    #[test]
    fn a_missing_translation_falls_back_to_english() {
        let text = Text::ALL
            .iter()
            .find(|text| text.translations()[index(Language::German)].is_empty());
        if let Some(text) = text {
            assert_eq!(text.get(Language::German), text.get(Language::English));
        }
    }

    #[test]
    fn placeholders_are_filled() {
        let text = Text::InProgress.fill(Language::English, &[("user", "anna")]);
        assert_eq!(text, "Backup in progress (by anna)");
    }

    #[test]
    fn sizes_are_in_powers_of_1000_with_one_decimal() {
        let en = Language::English;
        assert_eq!(size(0, en), "0 bytes");
        assert_eq!(size(999, en), "999 bytes");
        assert_eq!(size(1000, en), "1.0 kB");
        assert_eq!(size(15_496_000_000, en), "15.5 GB");
        assert_eq!(size(999_960_000, en), "1.0 GB");
        assert_eq!(size(64_000_000_000, en), "64.0 GB");
        assert_eq!(size(2_000_000_000_000_000, en), "2000.0 TB");
        assert_eq!(size(15_496_000_000, Language::German), "15,5 GB");
    }
}
