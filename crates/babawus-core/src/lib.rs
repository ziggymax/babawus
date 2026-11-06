//! All Babawus logic: versioning, configuration, log, name rules and verdicts.
//!
//! This crate must stay free of UI code.

pub mod computer_lock;
pub mod config;
pub mod copy;
pub mod exclude;
pub mod fs;
pub mod identity;
pub mod lock;
pub mod log;
pub mod medium;
pub mod names;
pub mod run;
pub mod scan;
pub mod settings;
pub mod time;
pub mod verdict;
pub mod versions;

/// The version of Babawus.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
