//! Every text the user sees, by name. The texts themselves are in the language files (see
//! [`crate::language`]): the language changes at once, without a restart, so the texts are looked
//! up each time they are shown, rather than through gettext.
//!
//! A text may hold placeholders such as `{label}`, which [`Text::fill`] replaces. Every
//! translation of a text has the same placeholders as the English one.

use crate::language::Language;

macro_rules! texts {
    ($($name:ident,)*) => {
        /// A text the user sees. Its name is its key in the language files.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Text {
            $($name,)*
        }

        impl Text {
            /// Every text, in the order of their values.
            pub const ALL: &[Self] = &[$(Self::$name,)*];

            /// The text's name in the code. Its key in the language files is [`Text::key`].
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$name => stringify!($name),)*
                }
            }
        }
    };
}

texts! {
    WindowTitle,
    Menu,
    MenuLanguage,
    MenuAdvanced,
    MenuAbout,
    AdvancedMarker,
    AdvancedMarkerTooltip,
    AdvancedQuestion,
    AdvancedDetail,
    AdvancedContinue,
    AboutComments,
    Logo,
    PlugIn,
    ChooseMedium,
    StickNotOpened,
    StickNotOpenedAdvice,
    StickCantBeUsed,
    StickCantBeUsedAdvice,
    AlreadyOpenDetail,
    Ok,
    AlreadyOpen,
    NoUdisks,

    Checking,
    Ready,
    NotSetUp,
    InProgress,
    NeedsAttention,
    WriteProtected,
    NotBackupMedium,
    OtherAccount,

    ExplainNotSetUp,
    ExplainInProgress,
    ExplainNeedsAttention,
    ExplainWriteProtected,
    ExplainOtherAccount,
    ExplainNotBackupMedium,

    LastBackupToday,
    LastBackupYesterday,
    LastBackupDaysAgo,
    NoBackupYet,
    SkippedMark,

    Free,

    StartBackup,
    StartBackupTooltip,
    ShowHistory,
    ShowHistoryTooltip,
    WhatsBackedUp,
    WhatsBackedUpTooltip,
    EjectTooltip,
    Prepare,
    PrepareTooltip,
    ChangeSetup,
    ChangeSetupTooltip,
    CantStart,

    GettingReady,
    BackingUp,
    Stopping,
    Finishing,
    Overview,
    OverviewOneFile,
    FilesLeft,
    BytesLeft,
    Cancel,
    CancelTooltip,
    StopQuestion,
    StopDetail,
    KeepGoing,
    StopBackup,
    Back,
    BackTooltip,
    VerdictBusy,
    NextWait,
    NextRunAgain,
    NextRunAgainLater,
    NextLookUp,
    NothingToCopy,
    Copied,
    OneFile,
    Files,
    SkippedGroup,

    ReasonFilesSkipped,
    ReasonCancelled,
    ReasonMediumRemoved,
    ReasonInterrupted,
    ReasonCrashed,
    ReasonFlushFailed,
    ReasonMediumFull,
    ReasonNotAdded,
    ReasonConfigDamaged,
    ReasonCannotWrite,

    SkipUnreadable,
    SkipDisappeared,
    SkipNotWritable,
    SkipTypeConflict,
    SkipChangedDuringCopy,
    SkipVersionNameTooLong,
    SkipNameInvalidChars,
    SkipNameTrailingDot,
    SkipNameNotUtf8,
    SkipPathTooLong,
    SkipVersionFolderForm,
    SkipCaseClash,
    SkipSelectionMissing,
    SkipOutsideHome,

    EjectNow,
    EjectNowTooltip,
    VerdictAllGood,
    VerdictSkipped,
    VerdictNotFinished,
    VerdictFailed,

    Ejecting,
    Ejected,
    EjectedUnmountedOnly,
    EjectBusy,
    EjectGone,
    EjectFailed,

    HistoryHeading,
    HistoryEmpty,
    HistoryUnreadable,
    ShowInLog,
    LogWindowTitle,
    LogUnreadable,
    ReminderDays,
    ReminderQuestion,
    ReminderBackUp,
    ReminderNotNow,
    HistoryRunning,
    HistoryToday,
    HistoryYesterday,
    HistoryDate,
    HistoryWho,
    HistorySkipped,

    CoverageHeading,
    CoverageIntro,
    CoverageNothing,
    CoverageUnreadable,
    CoverageHome,
    CoverageMissing,
    CoverageExcluded,
    CoverageOutsideHome,
    CoverageLink,
    CoverageUnreadableItem,
    CoverageLeftOut,
    CoveragePatterns,
    Either,
    Loading,

    FormatHeading,
    FormatIntro,
    FormatNoDrives,
    FormatNoFileSystem,
    FormatNoLabel,
    FormatWithBackups,
    FormatNotMounted,
    FormatLabel,
    FormatLabelRule,
    FormatFileSystem,
    FormatExFat,
    FormatExFatDetail,
    FormatExt4,
    FormatExt4Detail,
    FormatExt4AsksPassword,
    FormatExt4NeedsAdministrator,
    FormatWarning,
    FormatUnderstood,
    FormatErase,
    FormatEraseTooltip,
    FormatWorking,
    FormatWorkingDetail,
    FormatDone,
    FormatDoneDetail,
    FormatFailed,
    FormatGone,
    FormatNotAllowed,
    FormatNotPrepared,
    FormatFailedReason,
    FormatNothingErased,
    FormatPartlyErased,
    FormatMessage,
    AddComputer,
    AddComputerTooltip,
    TryAgain,
    TryAgainTooltip,

    SetupHeading,
    SetupLoading,
    SetupDamaged,
    SetupUnreadable,
    SetupStartNew,
    SetupStartNewQuestion,
    SetupStartNewDetail,
    SetupNewerFormat,
    SetupNotBabawus,
    SetupOlderVersions,
    SetupRunCheckDays,
    SetupRunCheckDaysDetail,
    SetupOlderVersionsDetail,
    SetupHere,
    SetupNoComputerId,
    SetupComputerMissing,
    SetupComputerMissingLink,
    SetupAccountMissing,
    SetupAccountMissingLink,
    SetupHereReady,
    SetupAddComputer,
    SetupLinkComputer,
    SetupAddAccount,
    SetupLinkAccount,
    SetupChoose,
    SetupChooseTooltip,
    SetupEntries,
    SetupEmpty,
    SetupThisComputer,
    SetupOtherComputer,
    SetupYou,
    SetupNothingChosen,
    SetupOneChosen,
    SetupChosen,
    SetupRemove,
    SetupRemoveComputerTooltip,
    SetupRemoveAccountTooltip,
    SetupRemoveComputerQuestion,
    SetupRemoveAccountQuestion,
    SetupRemoveDetail,
    SetupRemoveConfirm,
    SetupLinkComputerQuestion,
    SetupLinkComputerDetail,
    SetupLinkAccountQuestion,
    SetupLinkAccountDetail,
    SetupLinkConfirm,
    SetupSave,
    SetupSaveTooltip,
    SetupSaved,
    SetupSaving,
    SetupDiscardQuestion,
    SetupDiscardDetail,
    SetupDiscard,
    SetupKeepEditing,
    SetupBusy,
    SetupInvalid,
    SetupBothFolders,
    SetupSaveFailed,

    ChooseHeading,
    ChooseIntro,
    ChooseShowHidden,
    ChooseNoteLeftOut,
    ChooseNoteLeftOutAbove,
    ChooseNotePattern,
    ChooseNoteOtherDrive,
    ChooseNoteLink,
    ChooseNoteNotText,
    ChooseUnreadable,
    ChoosePatternsSection,
    ChoosePatternsDetail,
    ChoosePatternAdd,
    ChoosePatternRemove,
    ChoosePatternEmpty,
    ChoosePatternSlash,
    ChoosePatternDuplicate,
    ChooseCounting,
    ChooseNothing,
    ChooseTotals,
    ChooseTotalsNoFree,
    ChooseTooBig,
    ChooseDone,
    ChooseDoneTooltip,
    ChooseCancelTooltip,

    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,

    MenuHelp,
    HelpTitle,
    HelpSearch,
    HelpSearchTooltip,
    HelpFound,
    HelpNotFound,
    HelpNext,
    HelpPrevious,
    Help,
}

impl Text {
    /// The number of a message that tells of a problem, which Babawus shows after it, as in
    /// "The stick is full. #14", so that it can be looked up in Help. The numbers never change
    /// and are never reused (issues 151 and 153).
    pub fn number(self) -> Option<u16> {
        Some(match self {
            Self::StickNotOpenedAdvice => 1,
            Self::StickCantBeUsedAdvice => 2,
            Self::NoUdisks => 3,
            Self::AlreadyOpenDetail => 4,
            Self::ExplainNotSetUp => 5,
            Self::ExplainNeedsAttention => 6,
            Self::ExplainWriteProtected => 7,
            Self::ExplainOtherAccount => 8,
            Self::ExplainNotBackupMedium => 9,
            Self::ReasonMediumRemoved => 10,
            Self::ReasonInterrupted => 11,
            Self::ReasonCrashed => 12,
            Self::ReasonFlushFailed => 13,
            Self::ReasonMediumFull => 14,
            Self::ReasonNotAdded => 15,
            Self::ReasonConfigDamaged => 16,
            Self::ReasonCannotWrite => 17,
            Self::VerdictBusy => 18,
            Self::SkipUnreadable => 19,
            Self::SkipDisappeared => 20,
            Self::SkipNotWritable => 21,
            Self::SkipTypeConflict => 22,
            Self::SkipChangedDuringCopy => 23,
            Self::SkipVersionNameTooLong => 24,
            Self::SkipNameInvalidChars => 25,
            Self::SkipNameTrailingDot => 26,
            Self::SkipNameNotUtf8 => 27,
            Self::SkipPathTooLong => 28,
            Self::SkipVersionFolderForm => 29,
            Self::SkipCaseClash => 30,
            Self::SkipSelectionMissing => 31,
            Self::SkipOutsideHome => 32,
            Self::EjectBusy => 33,
            Self::EjectFailed => 34,
            Self::HistoryUnreadable => 35,
            Self::LogUnreadable => 36,
            Self::CoverageNothing => 37,
            Self::CoverageUnreadable => 38,
            Self::FormatNoDrives => 39,
            Self::FormatExt4NeedsAdministrator => 40,
            Self::FormatGone => 41,
            Self::FormatNotAllowed => 42,
            Self::FormatNotPrepared => 43,
            Self::FormatFailedReason => 44,
            Self::FormatPartlyErased => 45,
            Self::SetupDamaged => 46,
            Self::SetupUnreadable => 47,
            Self::SetupNewerFormat => 48,
            Self::SetupNotBabawus => 49,
            Self::SetupNoComputerId => 50,
            Self::SetupBusy => 51,
            Self::SetupInvalid => 52,
            Self::SetupBothFolders => 53,
            Self::SetupSaveFailed => 54,
            Self::ChooseUnreadable => 55,
            Self::ChooseTooBig => 56,
            _ => return None,
        })
    }

    /// The text's key in the language files: its name, followed by its number if it has one, as
    /// in `ReasonMediumFull_14`, so that whoever edits a file sees which message each Help entry
    /// explains (issue 158).
    pub fn key(self) -> String {
        match self.number() {
            Some(number) => format!("{}_{number}", self.name()),
            None => self.name().to_owned(),
        }
    }

    /// The texts that make up the message numbered `number`, as the user sees it: for most, the
    /// numbered text alone; for a few, a headline shown above it (issue 158).
    pub fn message(number: u16) -> Vec<Self> {
        match number {
            1 => vec![Self::StickNotOpened, Self::StickNotOpenedAdvice],
            2 => vec![Self::StickCantBeUsed, Self::StickCantBeUsedAdvice],
            4 => vec![Self::AlreadyOpen, Self::AlreadyOpenDetail],
            _ => Self::ALL
                .iter()
                .copied()
                .filter(|text| text.number() == Some(number))
                .collect(),
        }
    }

    /// The text in `language`, without its number.
    pub fn without_number(self, language: Language) -> &'static str {
        language.plain_text(self)
    }

    /// The text in `language`, with its number if it has one.
    pub fn get(self, language: Language) -> &'static str {
        language.text(self)
    }

    /// `parts` in `language`, one after the other, with their numbers together at the end, so
    /// that no number stands in the middle of a message (issue 153).
    pub fn joined(parts: &[Self], language: Language) -> String {
        let mut text = parts
            .iter()
            .map(|part| part.without_number(language))
            .collect::<Vec<_>>()
            .join(" ");
        for number in parts.iter().filter_map(|part| part.number()) {
            text.push_str(&format!(" #{number}"));
        }
        text
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
    let number = format!("{value:.1}").replace('.', language.decimal());
    UNITS[unit].fill(language, &[("size", &number)])
}

/// The name's long form in parts, each marked whether it is one of the letters that spell
/// "Babawus" (issue 132).
const NAME_LONG_FORM: [(&str, bool); 10] = [
    ("Ba", true),
    ("sic ", false),
    ("Ba", true),
    ("ckup ", false),
    ("w", true),
    ("ith ", false),
    ("U", true),
    ("SB ", false),
    ("S", true),
    ("torage", false),
];

/// The comment in About, as Pango markup: the name's long form with the letters that spell
/// "Babawus" in bold, in every language, because it is where the name comes from. In any other
/// language than English, that language's own comment follows on the line below (issue 132).
pub fn about_comments(language: Language) -> String {
    let mut markup: String = NAME_LONG_FORM
        .iter()
        .map(|&(part, bold)| {
            if bold {
                format!("<b>{part}</b>")
            } else {
                part.to_owned()
            }
        })
        .collect();
    if !language.is_english() {
        markup.push('\n');
        markup.push_str(&escape_markup(Text::AboutComments.get(language)));
    }
    markup
}

/// `text` with the characters that Pango markup gives a meaning escaped.
fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::testing::{english, german};

    #[test]
    fn the_numbers_run_from_1_to_56_with_none_twice() {
        let mut numbers: Vec<u16> = Text::ALL.iter().filter_map(|text| text.number()).collect();
        numbers.sort_unstable();

        assert_eq!(numbers, (1..=56).collect::<Vec<_>>());
    }

    #[test]
    fn a_numbered_message_ends_with_its_number_in_every_language() {
        assert_eq!(
            Text::ReasonMediumFull.get(english()),
            "The stick is full. #14"
        );
        assert_eq!(
            Text::ReasonMediumFull.get(german()),
            "Der Stick ist voll. #14"
        );
        assert_eq!(
            Text::ReasonMediumFull.without_number(english()),
            "The stick is full."
        );
        assert_eq!(Text::Ready.get(english()), "Ready");
    }

    #[test]
    fn a_numbered_message_with_placeholders_is_filled_before_its_number() {
        let text = Text::SetupBusy.fill(english(), &[("user", "anna")]);

        assert!(text.starts_with("The stick is in use by a backup for anna."));
        assert!(text.ends_with(" #51"));
    }

    #[test]
    fn joined_messages_put_their_numbers_at_the_end() {
        let text = Text::joined(
            &[Text::FormatFailedReason, Text::FormatPartlyErased],
            english(),
        );

        assert_eq!(
            text,
            "Formatting didn't work. The stick has been erased, but isn't ready for backups. \
             Format it again. #44 #45"
        );
    }

    #[test]
    fn placeholders_are_filled() {
        let text = Text::InProgress.fill(english(), &[("user", "anna")]);
        assert_eq!(text, "Backup in progress (by anna)");
    }

    #[test]
    fn the_long_form_is_the_english_comment_and_its_bold_letters_spell_the_name() {
        let text: String = NAME_LONG_FORM.iter().map(|(part, _)| *part).collect();
        assert_eq!(text, Text::AboutComments.get(english()));
        let name: String = NAME_LONG_FORM
            .iter()
            .filter(|(_, bold)| *bold)
            .map(|(part, _)| *part)
            .collect();
        assert_eq!(name.to_lowercase(), "babawus");
    }

    #[test]
    fn about_shows_the_long_form_and_below_it_the_language_s_own_comment() {
        let long_form = "<b>Ba</b>sic <b>Ba</b>ckup <b>w</b>ith <b>U</b>SB <b>S</b>torage";
        assert_eq!(about_comments(english()), long_form);
        assert_eq!(
            about_comments(german()),
            format!("{long_form}\nEinfache Sicherung auf USB-Sticks")
        );
    }

    #[test]
    fn markup_characters_are_escaped() {
        assert_eq!(escape_markup("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn sizes_are_in_powers_of_1000_with_one_decimal() {
        let en = english();
        assert_eq!(size(0, en), "0 bytes");
        assert_eq!(size(999, en), "999 bytes");
        assert_eq!(size(1000, en), "1.0 kB");
        assert_eq!(size(15_496_000_000, en), "15.5 GB");
        assert_eq!(size(999_960_000, en), "1.0 GB");
        assert_eq!(size(64_000_000_000, en), "64.0 GB");
        assert_eq!(size(2_000_000_000_000_000, en), "2000.0 TB");
        assert_eq!(size(15_496_000_000, german()), "15,5 GB");
    }
}
