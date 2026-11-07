//! All Babawus logic: versioning, configuration, log, name rules, verdicts, and the media
//! that udisks2 reports.
//!
//! This crate must stay free of UI code.

pub mod backup_screen;
pub mod computer_lock;
pub mod config;
pub mod copy;
pub mod exclude;
pub mod format;
pub mod fs;
pub mod identity;
pub mod lock;
pub mod log;
pub mod media;
pub mod medium;
pub mod names;
pub mod run;
pub mod scan;
pub mod settings;
pub mod start_screen;
pub mod text;
pub mod time;
pub mod udisks;
pub mod verdict;
pub mod verdict_screen;
pub mod versions;

/// The version of Babawus.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
