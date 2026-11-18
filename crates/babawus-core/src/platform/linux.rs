//! The platform layer on Linux. See "Platform layer" in Requirements.md.

mod flush;
mod identity;
mod sticks;

pub use flush::{
    COMPUTER_LOCK_FILE, ComputerLock, Taken, device, flush_file_system, free_space, is_medium_gone,
    mode, own_program, program_of, set_mode,
};
#[cfg(test)]
pub(crate) use flush::{OWN_PROGRAM_FILE, program_file};
#[cfg(test)]
pub(crate) use identity::{BOOT_ID_FILE, MACHINE_ID_FILE};
pub use identity::{boot_id, hostname, is_administrator, machine_id};
pub use sticks::Sticks;
