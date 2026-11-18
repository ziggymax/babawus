//! Flushing and locking on Windows: a placeholder. See `platform/windows.rs`.

use std::fs;
use std::io;
use std::path::Path;

use super::unsupported;
use crate::fs::FileSystem;

pub fn set_mode(_path: &Path, _mode: u32) -> io::Result<()> {
    Err(unsupported())
}

pub fn mode(_metadata: &fs::Metadata) -> u32 {
    0
}

pub fn device(_metadata: &fs::Metadata) -> u64 {
    0
}

pub fn free_space(_path: &Path) -> io::Result<u64> {
    Err(unsupported())
}

pub fn flush_file_system(_path: &Path) -> io::Result<()> {
    Err(unsupported())
}

pub fn is_medium_gone(_error: &io::Error) -> bool {
    false
}

pub fn medium_gone_error() -> io::Error {
    unsupported()
}

pub fn own_program(_system: &dyn FileSystem) -> io::Result<String> {
    Err(unsupported())
}

pub fn program_of(_system: &dyn FileSystem, _process_id: u32) -> io::Result<String> {
    Err(unsupported())
}

/// Never made, since taking the lock always fails.
#[derive(Debug)]
pub struct ComputerLock {
    _private: (),
}

/// What came of trying to take the lock.
#[derive(Debug)]
pub enum Taken {
    Ours(ComputerLock),
    AlreadyOpen(String),
}

impl ComputerLock {
    pub fn take(_path: &Path, _username: &str) -> io::Result<Taken> {
        Err(unsupported())
    }
}
