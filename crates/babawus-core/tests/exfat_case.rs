//! Checks [`fold`] against the table of upper-case letters that `mkfs.exfat` really writes, for
//! all 65,536 characters exFAT can store. Two names exFAT sees as the same must never look
//! different to Babawus, or two files could end up as one on the medium.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::Command;

use babawus_core::names::fold;

const ENTRY_SIZE: usize = 32;
const UPCASE_TABLE_ENTRY: u8 = 0x82;

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_at(file: &mut File, offset: u64, length: usize) -> Vec<u8> {
    let mut bytes = vec![0; length];
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.read_exact(&mut bytes).unwrap();
    bytes
}

/// Reads the table from an exFAT image, as a list of the upper case of each UTF-16 unit.
fn read_upcase_table(image: &Path) -> Vec<u16> {
    let mut file = File::open(image).unwrap();
    let boot = read_at(&mut file, 0, 512);
    let heap = u64::from(u32_at(&boot, 88));
    let root_cluster = u64::from(u32_at(&boot, 96));
    let sector_size = 1_u64 << boot[108];
    let cluster_size = sector_size << boot[109];
    let offset = |cluster: u64| heap * sector_size + (cluster - 2) * cluster_size;

    let root = read_at(&mut file, offset(root_cluster), cluster_size as usize);
    let entry = root
        .chunks(ENTRY_SIZE)
        .find(|entry| entry[0] == UPCASE_TABLE_ENTRY)
        .expect("exFAT has an up-case table");
    let cluster = u64::from(u32_at(entry, 20));
    let length = u32_at(entry, 24) as usize;
    let raw = read_at(&mut file, offset(cluster), length);

    let units: Vec<u16> = raw
        .chunks(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let mut table = Vec::with_capacity(0x10000);
    let mut units = units.into_iter();
    while let Some(unit) = units.next() {
        // 0xFFFF marks a run of characters that stay the same.
        if unit == 0xFFFF
            && let Some(run) = units.next()
        {
            for _ in 0..run {
                table.push(table.len() as u16);
            }
        } else {
            table.push(unit);
        }
    }
    table
}

#[test]
#[ignore = "needs mkfs.exfat"]
fn fold_covers_every_pair_in_the_exfat_table() {
    let image = Path::new(env!("CARGO_TARGET_TMPDIR")).join("upcase.img");
    File::create(&image)
        .unwrap()
        .set_len(16 * 1024 * 1024)
        .unwrap();
    let status = Command::new("/usr/sbin/mkfs.exfat")
        .arg(&image)
        .output()
        .unwrap()
        .status;
    assert!(status.success());
    let table = read_upcase_table(&image);
    fs::remove_file(&image).unwrap();
    assert_eq!(table.len(), 0x10000);

    let mut groups: HashMap<u16, Vec<char>> = HashMap::new();
    for (code, &upper) in table.iter().enumerate() {
        if let Some(character) = char::from_u32(code as u32) {
            groups.entry(upper).or_default().push(character);
        }
    }

    let mut pairs = 0;
    for group in groups.values().filter(|group| group.len() > 1) {
        let folded = fold(&group[0].to_string());
        for &character in &group[1..] {
            assert_eq!(fold(&character.to_string()), folded, "{group:?}");
            pairs += 1;
        }
    }
    assert!(pairs > 800, "only {pairs} pairs");
}
