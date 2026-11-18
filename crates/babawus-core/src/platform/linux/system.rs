//! What the system gives on Linux that isn't sticks, identity, flushing or paths: random bytes
//! and the offset of local time from UTC. See "Platform layer" in Requirements.md.

use std::mem::MaybeUninit;
use std::time::SystemTime;

/// Fills `bytes` with random bytes from the system.
pub fn random_bytes(bytes: &mut [u8]) {
    // SAFETY: `bytes` is valid for writes of its whole length.
    let written = unsafe { libc::getrandom(bytes.as_mut_ptr().cast(), bytes.len(), 0) };
    assert_eq!(
        usize::try_from(written).ok(),
        Some(bytes.len()),
        "the system gives a few random bytes without fail"
    );
}

/// How many seconds local time is ahead of UTC at `time`, by the system's time zone. A time
/// before 1970 counts as 1970.
pub fn utc_offset(time: SystemTime) -> i64 {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let seconds = libc::time_t::try_from(seconds).expect("the time fits in a time_t");
    let mut fields = MaybeUninit::<libc::tm>::uninit();
    // SAFETY: `seconds` is valid for reads, and `fields` is valid for writes.
    let result = unsafe { libc::localtime_r(&seconds, fields.as_mut_ptr()) };
    assert!(!result.is_null(), "a time since 1970 can be split up");
    // SAFETY: the conversion succeeded, so it has filled in `fields`.
    unsafe { fields.assume_init() }.tm_gmtoff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_bytes_differ() {
        let (mut first, mut second) = ([0; 16], [0; 16]);
        random_bytes(&mut first);
        random_bytes(&mut second);
        assert_ne!(first, second);
    }
}
