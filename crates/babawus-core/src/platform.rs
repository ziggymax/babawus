//! The platform layer: everything in `babawus-core` that depends on the operating system.
//!
//! See "Platform layer" in Requirements.md. Each operating system has a submodule, chosen at
//! compile time, that offers the same plain types and functions. The types that every
//! submodule shares are defined here, so that the rest of Babawus is written once.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

use crate::format::Refusal;

/// A step of formatting. Everything before [`Step::Partition`] leaves the data on the drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Unmount,
    Partition,
    Format,
    Mount,
    /// Setting permissions, and creating the folders and the configuration.
    Prepare,
}

/// Why formatting failed.
#[derive(Debug)]
pub enum FormatError {
    /// The label is not allowed. Nothing was changed.
    InvalidLabel,
    /// The drive may not be formatted, or has gone. Nothing was changed.
    Refused(Refusal),
    /// A step failed, for example because the system or the administrator said no.
    Failed { step: Step, message: String },
}

impl FormatError {
    /// Whether the drive still holds what it held before.
    pub fn nothing_erased(&self) -> bool {
        match self {
            Self::InvalidLabel | Self::Refused(_) => true,
            Self::Failed { step, .. } => *step == Step::Unmount,
        }
    }
}

/// How far ejecting got.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ejected {
    /// The medium was unmounted and powered off, so it can be pulled out.
    PoweredOff,
    /// The medium was unmounted, but powering it off was not possible, so its light may stay
    /// on. It can still be pulled out safely. `reason` is for the details, if the system gave
    /// one.
    UnmountedOnly { reason: Option<String> },
}

/// Why ejecting failed. The medium is still mounted, at least in part.
#[derive(Debug, PartialEq, Eq)]
pub enum EjectError {
    /// The drive has gone, or isn't a USB medium.
    Gone,
    /// A program is still using a file system on it, such as an open file or a file manager
    /// window.
    Busy,
    Failed(String),
}
