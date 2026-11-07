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
    BusyStarting: "Another backup to this stick is just starting. Try again in a moment.", "";
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
