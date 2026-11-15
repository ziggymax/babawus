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

            /// The text's key in the language files.
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
    NextContact,
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
}

impl Text {
    /// The text in `language`.
    pub fn get(self, language: Language) -> &'static str {
        language.text(self)
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
