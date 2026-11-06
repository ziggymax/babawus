//! The computer ID. See "Identifying a computer" in Requirements.md.

use std::fmt::Write;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::fs::FileSystem;
use crate::medium::read_file;

const MACHINE_ID_FILE: &str = "/etc/machine-id";
const HOSTNAME_FILE: &str = "/proc/sys/kernel/hostname";

/// The message of the HMAC. This must never change, because that would disconnect every computer
/// from every medium.
const APPLICATION_ID: &[u8] = b"io.github.ziggymax.Babawus";

const ID_DIGITS: usize = 32;
const SHA256_BLOCK: usize = 64;

/// The first `bytes` bytes of `data` as lower-case hexadecimal digits.
pub(crate) fn hex(data: &[u8], bytes: usize) -> String {
    data.iter()
        .take(bytes)
        .fold(String::new(), |mut text, byte| {
            write!(text, "{byte:02x}").expect("writing to a string can't fail");
            text
        })
}

/// HMAC-SHA256, as defined in RFC 2104.
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut block = [0; SHA256_BLOCK];
    if key.len() > SHA256_BLOCK {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let padded = |pad: u8| block.map(|byte| byte ^ pad);
    let inner = Sha256::new()
        .chain_update(padded(0x36))
        .chain_update(message)
        .finalize();
    Sha256::new()
        .chain_update(padded(0x5c))
        .chain_update(inner)
        .finalize()
        .into()
}

/// Whether `id` has the form of a computer ID.
pub(crate) fn is_computer_id(id: &str) -> bool {
    id.len() == ID_DIGITS
        && id
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// The computer ID that belongs to the contents of `/etc/machine-id`, or `None` if they don't
/// hold a machine ID.
pub fn computer_id(machine_id: &str) -> Option<String> {
    let key = machine_id.strip_suffix('\n').unwrap_or(machine_id);
    let is_machine_id = key.len() == ID_DIGITS && key.bytes().all(|byte| byte.is_ascii_hexdigit());
    is_machine_id.then(|| hex(&hmac_sha256(key.as_bytes(), APPLICATION_ID), ID_DIGITS / 2))
}

/// The ID of this computer, or `None` if it has no machine ID.
pub fn this_computer_id(fs: &dyn FileSystem) -> io::Result<Option<String>> {
    let contents = read_file(fs, Path::new(MACHINE_ID_FILE))?;
    Ok(str::from_utf8(&contents).ok().and_then(computer_id))
}

/// The hostname of this computer.
pub fn this_hostname(fs: &dyn FileSystem) -> io::Result<String> {
    let contents = read_file(fs, Path::new(HOSTNAME_FILE))?;
    Ok(String::from_utf8_lossy(&contents).trim_end().to_owned())
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::fs::fake::FakeFs;

    const MACHINE_ID: &str = "0123456789abcdef0123456789abcdef";
    /// Worked out separately, with Python's `hmac` module. This value must never change.
    const COMPUTER_ID: &str = "203a901d9caa4803d310150d35b859a5";

    #[test]
    fn the_hostname_is_read_without_its_line_ending() {
        let fake = FakeFs::new();
        fake.add_file(HOSTNAME_FILE, b"mint\n", SystemTime::UNIX_EPOCH);

        assert_eq!(this_hostname(&fake).unwrap(), "mint");
    }

    #[test]
    fn this_computer_has_a_hostname() {
        assert!(!this_hostname(&crate::fs::RealFs).unwrap().is_empty());
    }

    fn hmac_hex(key: &[u8], message: &[u8]) -> String {
        hex(&hmac_sha256(key, message), 32)
    }

    #[test]
    fn hmac_matches_rfc_4231() {
        assert_eq!(
            hmac_hex(&[0x0b; 20], b"Hi There"),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            hmac_hex(b"Jefe", b"what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn hmac_hashes_a_key_longer_than_a_block_as_rfc_4231_says() {
        assert_eq!(
            hmac_hex(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            ),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn the_computer_id_never_changes() {
        assert_eq!(computer_id(MACHINE_ID).unwrap(), COMPUTER_ID);
    }

    #[test]
    fn the_line_ending_of_the_machine_id_is_not_part_of_the_key() {
        let with_line_ending = format!("{MACHINE_ID}\n");

        assert_eq!(computer_id(&with_line_ending).unwrap(), COMPUTER_ID);
    }

    #[test]
    fn a_computer_id_has_the_form_the_configuration_expects() {
        assert!(is_computer_id(&computer_id(MACHINE_ID).unwrap()));
    }

    #[test]
    fn different_machine_ids_give_different_computer_ids() {
        let other = "fedcba9876543210fedcba9876543210";

        assert_ne!(computer_id(MACHINE_ID), computer_id(other));
    }

    #[test]
    fn anything_else_is_not_a_machine_id() {
        for contents in ["", "\n", "uninitialized\n", "0123", &"g".repeat(32)] {
            assert_eq!(computer_id(contents), None, "{contents}");
        }
    }

    #[test]
    fn the_id_of_this_computer_comes_from_the_machine_id_file() {
        let fs = FakeFs::new();
        let contents = format!("{MACHINE_ID}\n");
        fs.add_file(MACHINE_ID_FILE, contents.as_bytes(), SystemTime::UNIX_EPOCH);

        assert_eq!(this_computer_id(&fs).unwrap().unwrap(), COMPUTER_ID);
    }

    #[test]
    fn a_machine_id_file_with_junk_gives_no_id() {
        let fs = FakeFs::new();
        fs.add_file(MACHINE_ID_FILE, b"\xff\xfe", SystemTime::UNIX_EPOCH);

        assert_eq!(this_computer_id(&fs).unwrap(), None);
    }

    #[test]
    fn checking_an_id_only_accepts_lower_case_digits() {
        assert!(is_computer_id("0123456789abcdef0123456789abcdef"));
        assert!(!is_computer_id("0123456789ABCDEF0123456789ABCDEF"));
        assert!(!is_computer_id("0123"));
    }
}
