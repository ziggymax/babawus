//! The platform layer on Windows: a placeholder that compiles but says Windows isn't supported
//! yet, so that CI can check that the rest of `babawus-core` builds for Windows. See "Platform
//! layer" in Requirements.md.

mod flush;
mod identity;
mod paths;
mod sticks;
mod system;

use std::io::{self, ErrorKind};

pub use flush::{
    ComputerLock, Taken, device, flush_file_system, free_space, is_medium_gone, medium_gone_error,
    mode, own_program, program_of, set_mode,
};
pub use identity::{boot_id, hostname, is_administrator, machine_id};
pub use paths::{
    computer_lock_file, home_folder, installed_languages_folder, last_backup_file, name_from_bytes,
    settings_file,
};
pub use sticks::Sticks;
pub use system::{random_bytes, utc_offset};

/// The error of everything that doesn't work on Windows yet.
fn unsupported() -> io::Error {
    io::Error::new(ErrorKind::Unsupported, "Windows isn't supported yet")
}
