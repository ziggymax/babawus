//! All Babawus logic: versioning, configuration, log, name rules and verdicts.
//!
//! This crate must stay free of UI code.

pub mod fs;

/// The version of Babawus.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
