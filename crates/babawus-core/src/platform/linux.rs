//! The platform layer on Linux. See "Platform layer" in Requirements.md.

mod flush;
mod identity;
mod paths;
mod sticks;
mod system;

pub use flush::{
    ComputerLock, Taken, device, flush_file_system, free_space, is_medium_gone, medium_gone_error,
    mode, own_program, program_of, set_mode,
};
#[cfg(test)]
pub(crate) use flush::{OWN_PROGRAM_FILE, program_file};
#[cfg(test)]
pub(crate) use identity::{BOOT_ID_FILE, MACHINE_ID_FILE};
pub use identity::{boot_id, hostname, is_administrator, machine_id};
pub use paths::{
    computer_lock_file, home_folder, installed_languages_folder, last_backup_file, name_from_bytes,
    settings_file,
};
pub use sticks::Sticks;
pub use system::{random_bytes, utc_offset};
