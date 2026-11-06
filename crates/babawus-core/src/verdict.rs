//! Verdicts and their reasons. See "Get a clear verdict" in Requirements.md.

use crate::copy::Stop;

/// The outcome of a run, from best to worst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    AllGood,
    Skipped,
    NotFinished,
    Failed,
}

impl Verdict {
    const ALL: [Self; 4] = [
        Self::AllGood,
        Self::Skipped,
        Self::NotFinished,
        Self::Failed,
    ];

    /// The fixed code in the log.
    pub fn code(self) -> &'static str {
        match self {
            Self::AllGood => "ALL_GOOD",
            Self::Skipped => "SKIPPED",
            Self::NotFinished => "NOT_FINISHED",
            Self::Failed => "FAILED",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|verdict| verdict.code() == code)
    }
}

/// Why a run ended as it did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    None,
    FilesSkipped,
    Cancelled,
    MediumRemoved,
    /// Found by a later run: this run has no end entry.
    Interrupted,
    FlushFailed,
    MediumFull,
    NotAdded,
    /// There is a configuration file, but it can't be used: it is damaged, or needs a newer
    /// Babawus. Only in a verdict, never in the log, because a run doesn't touch such a medium.
    ConfigDamaged,
    CannotWrite,
}

impl Reason {
    const ALL: [Self; 10] = [
        Self::None,
        Self::FilesSkipped,
        Self::Cancelled,
        Self::MediumRemoved,
        Self::Interrupted,
        Self::FlushFailed,
        Self::MediumFull,
        Self::NotAdded,
        Self::ConfigDamaged,
        Self::CannotWrite,
    ];

    /// The fixed code in the log.
    pub fn code(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::FilesSkipped => "FILES_SKIPPED",
            Self::Cancelled => "CANCELLED",
            Self::MediumRemoved => "MEDIUM_REMOVED",
            Self::Interrupted => "INTERRUPTED",
            Self::FlushFailed => "FLUSH_FAILED",
            Self::MediumFull => "MEDIUM_FULL",
            Self::NotAdded => "NOT_ADDED",
            Self::ConfigDamaged => "CONFIG_DAMAGED",
            Self::CannotWrite => "CANNOT_WRITE",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|reason| reason.code() == code)
    }

    /// Verdicts are defined by what the user should do next.
    pub fn verdict(self) -> Verdict {
        match self {
            Self::None => Verdict::AllGood,
            Self::FilesSkipped => Verdict::Skipped,
            Self::Cancelled | Self::MediumRemoved | Self::Interrupted | Self::FlushFailed => {
                Verdict::NotFinished
            }
            Self::MediumFull | Self::NotAdded | Self::ConfigDamaged | Self::CannotWrite => {
                Verdict::Failed
            }
        }
    }
}

impl From<Stop> for Reason {
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::MediumFull => Self::MediumFull,
            Stop::MediumGone => Self::MediumRemoved,
            Stop::CannotWrite => Self::CannotWrite,
        }
    }
}

/// The reason that gives the worst verdict: if several things happen in one run, the worst wins.
/// Among equals, the first stays, because that is what went wrong first.
pub fn worst(reasons: impl IntoIterator<Item = Reason>) -> Reason {
    reasons.into_iter().fold(Reason::None, |worst, reason| {
        if reason.verdict() > worst.verdict() {
            reason
        } else {
            worst
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_codes_are_the_ones_in_the_requirements() {
        let verdicts = Verdict::ALL.map(Verdict::code);
        let reasons = Reason::ALL.map(Reason::code);

        assert_eq!(verdicts, ["ALL_GOOD", "SKIPPED", "NOT_FINISHED", "FAILED"]);
        assert_eq!(
            reasons,
            [
                "NONE",
                "FILES_SKIPPED",
                "CANCELLED",
                "MEDIUM_REMOVED",
                "INTERRUPTED",
                "FLUSH_FAILED",
                "MEDIUM_FULL",
                "NOT_ADDED",
                "CONFIG_DAMAGED",
                "CANNOT_WRITE",
            ]
        );
    }

    #[test]
    fn every_code_reads_back() {
        for verdict in Verdict::ALL {
            assert_eq!(Verdict::from_code(verdict.code()), Some(verdict));
        }
        for reason in Reason::ALL {
            assert_eq!(Reason::from_code(reason.code()), Some(reason));
        }
        assert_eq!(Verdict::from_code("all_good"), None);
        assert_eq!(Reason::from_code("MEDIUM"), None);
    }

    #[test]
    fn each_reason_leads_to_the_verdict_in_the_requirements() {
        let cases = [
            (Reason::None, Verdict::AllGood),
            (Reason::FilesSkipped, Verdict::Skipped),
            (Reason::Cancelled, Verdict::NotFinished),
            (Reason::MediumRemoved, Verdict::NotFinished),
            (Reason::Interrupted, Verdict::NotFinished),
            (Reason::FlushFailed, Verdict::NotFinished),
            (Reason::MediumFull, Verdict::Failed),
            (Reason::NotAdded, Verdict::Failed),
            (Reason::ConfigDamaged, Verdict::Failed),
            (Reason::CannotWrite, Verdict::Failed),
        ];
        for (reason, verdict) in cases {
            assert_eq!(reason.verdict(), verdict, "{reason:?}");
        }
    }

    #[test]
    fn a_stop_becomes_a_reason() {
        assert_eq!(Reason::from(Stop::MediumFull), Reason::MediumFull);
        assert_eq!(Reason::from(Stop::MediumGone), Reason::MediumRemoved);
        assert_eq!(Reason::from(Stop::CannotWrite), Reason::CannotWrite);
    }

    #[test]
    fn the_worst_verdict_wins() {
        let reasons = [Reason::FilesSkipped, Reason::MediumFull, Reason::Cancelled];

        assert_eq!(worst(reasons), Reason::MediumFull);
        assert_eq!(
            worst([Reason::FilesSkipped, Reason::None]),
            Reason::FilesSkipped
        );
    }

    #[test]
    fn nothing_going_wrong_is_all_good() {
        assert_eq!(worst([]), Reason::None);
    }

    #[test]
    fn among_equals_the_first_reason_stays() {
        let reasons = [Reason::Cancelled, Reason::MediumRemoved];

        assert_eq!(worst(reasons), Reason::Cancelled);
    }
}
