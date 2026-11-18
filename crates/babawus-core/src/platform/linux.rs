//! The platform layer on Linux. See "Platform layer" in Requirements.md.

mod identity;
mod sticks;

#[cfg(test)]
pub(crate) use identity::{BOOT_ID_FILE, MACHINE_ID_FILE};
pub use identity::{boot_id, hostname, is_administrator, machine_id};
pub use sticks::Sticks;
