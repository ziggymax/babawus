//! Identity on Windows: a placeholder. See `platform/windows.rs`.

use std::io;

use super::unsupported;
use crate::fs::FileSystem;

pub fn machine_id(_system: &dyn FileSystem) -> io::Result<String> {
    Err(unsupported())
}

pub fn hostname(_system: &dyn FileSystem) -> io::Result<String> {
    Err(unsupported())
}

pub fn boot_id(_system: &dyn FileSystem) -> io::Result<String> {
    Err(unsupported())
}

pub fn is_administrator() -> bool {
    false
}
