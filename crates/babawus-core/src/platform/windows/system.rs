//! What the system gives on Windows: a placeholder. See `platform/windows.rs`.

use std::hash::{BuildHasher, RandomState};
use std::time::SystemTime;

/// Random bytes from the keys of the standard library's hash maps, which it takes from the
/// system.
pub fn random_bytes(bytes: &mut [u8]) {
    for (index, chunk) in bytes.chunks_mut(8).enumerate() {
        let random = RandomState::new().hash_one(index).to_le_bytes();
        chunk.copy_from_slice(&random[..chunk.len()]);
    }
}

/// Local time counts as UTC.
pub fn utc_offset(_time: SystemTime) -> i64 {
    0
}
