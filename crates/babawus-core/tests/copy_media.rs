//! The safe copy and versioning on real exFAT and Ext4.
//!
//! These use sudo and loop devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, SystemTime};

use babawus_core::copy::{CHUNK_SIZE, FileCopier, Pruned, Stop, Visit};
use babawus_core::fs::{FileSystem, RealFs};
use babawus_core::versions::Format;
use support::{LoopMedium, MediumFormat};

const MIB: u64 = 1024 * 1024;
const TARGET: &str = "backup/mint/user/report.pdf";
const COPIED: Visit = Visit::Copied(Pruned {
    removed: 0,
    failed: false,
});
const UNCHANGED: Visit = Visit::Unchanged(Pruned {
    removed: 0,
    failed: false,
});

struct Test {
    medium: LoopMedium,
    format: Format,
    source_dir: PathBuf,
    clock: Mutex<String>,
}

impl Test {
    fn new(format: Format) -> Self {
        let medium_format = match format {
            Format::ExFat => MediumFormat::ExFat,
            Format::Ext4 => MediumFormat::Ext4,
        };
        let medium = LoopMedium::new(medium_format, 32 * MIB);
        fs::create_dir(medium.path().join(".babawus")).unwrap();
        let source_dir = medium.path().with_file_name("source");
        fs::create_dir(&source_dir).unwrap();
        Self {
            medium,
            format,
            source_dir,
            clock: Mutex::new("2026-11-03_175400".to_owned()),
        }
    }

    fn source(&self) -> PathBuf {
        self.source_dir.join("report.pdf")
    }

    fn write_source(&self, data: &[u8], modified: SystemTime) {
        fs::write(self.source(), data).unwrap();
        let file = fs::File::options().write(true).open(self.source()).unwrap();
        file.set_modified(modified).unwrap();
    }

    fn visit(&self) -> Result<Visit, Stop> {
        let now = || self.clock.lock().unwrap().clone();
        let cancel = AtomicBool::new(false);
        let root = self.medium.path().to_owned();
        let mut copier = FileCopier::new(&RealFs, &RealFs, root, self.format, 2, &now, &cancel);
        copier.empty_temp_folder()?;
        let visit = copier.visit(&self.source(), Path::new(TARGET), &mut |_| {})?;
        copier.flush_medium()?;
        Ok(visit)
    }

    fn newest(&self) -> PathBuf {
        self.medium.path().join(TARGET)
    }

    fn versions(&self) -> Vec<String> {
        let folder = self.newest().with_file_name("report[]pdf");
        let entries = RealFs.read_dir(&folder).unwrap_or_default();
        entries
            .iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect()
    }
}

fn time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

fn copies_and_keeps_versions(format: Format) {
    let test = Test::new(format);

    test.write_source(b"first", time(1_600_000_000));
    assert_eq!(test.visit(), Ok(COPIED));
    assert_eq!(test.visit(), Ok(UNCHANGED));

    test.write_source(b"second", time(1_600_000_100));
    *test.clock.lock().unwrap() = "2026-11-04_080000".to_owned();
    assert_eq!(test.visit(), Ok(COPIED));

    assert_eq!(fs::read(test.newest()).unwrap(), b"second");
    assert_eq!(test.versions(), ["report[2026-11-04_080000].pdf"]);
    let version = test
        .newest()
        .with_file_name("report[]pdf/report[2026-11-04_080000].pdf");
    assert_eq!(fs::read(version).unwrap(), b"first");
}

fn prunes_to_the_limit(format: Format) {
    let test = Test::new(format);
    test.write_source(b"0", time(1_600_000_000));
    test.visit().unwrap();

    for version in 1..=4_u64 {
        test.write_source(b"x", time(1_600_000_000 + version * 100));
        *test.clock.lock().unwrap() = format!("2026-11-0{version}_080000");
        test.visit().unwrap();
    }

    assert_eq!(
        test.versions(),
        [
            "report[2026-11-03_080000].pdf",
            "report[2026-11-04_080000].pdf"
        ]
    );
}

fn copies_a_file_of_several_chunks(format: Format) {
    let test = Test::new(format);
    let data = vec![7; CHUNK_SIZE * 2 + 10];
    test.write_source(&data, time(1_600_000_000));

    assert_eq!(test.visit(), Ok(COPIED));

    assert_eq!(fs::read(test.newest()).unwrap(), data);
    assert_eq!(test.visit(), Ok(UNCHANGED));
}

fn stops_when_a_file_does_not_fit(format: Format) {
    let test = Test::new(format);
    test.write_source(&vec![7; 40 * MIB as usize], time(1_600_000_000));

    assert_eq!(test.visit(), Err(Stop::MediumFull));

    assert!(!test.newest().exists());
}

fn a_file_from_before_1980_is_not_copied_again(format: Format) {
    let test = Test::new(format);
    test.write_source(b"old file", time(1000));

    assert_eq!(test.visit(), Ok(COPIED));

    assert_eq!(test.visit(), Ok(UNCHANGED));
}

fn a_file_from_the_far_future_is_not_copied_again(format: Format) {
    let test = Test::new(format);
    // The year 2200.
    test.write_source(b"future file", time(7_258_118_400));

    assert_eq!(test.visit(), Ok(COPIED));

    assert_eq!(test.visit(), Ok(UNCHANGED));
}

macro_rules! on_both_file_systems {
    ($($test:ident: $exfat:ident, $ext4:ident;)*) => {
        $(
            #[test]
            #[ignore = "needs sudo and loop devices"]
            fn $exfat() {
                $test(Format::ExFat);
            }

            #[test]
            #[ignore = "needs sudo and loop devices"]
            fn $ext4() {
                $test(Format::Ext4);
            }
        )*
    };
}

on_both_file_systems! {
    copies_and_keeps_versions: exfat_keeps_versions, ext4_keeps_versions;
    prunes_to_the_limit: exfat_prunes, ext4_prunes;
    copies_a_file_of_several_chunks: exfat_copies_chunks, ext4_copies_chunks;
    stops_when_a_file_does_not_fit: exfat_stops_when_full, ext4_stops_when_full;
    a_file_from_before_1980_is_not_copied_again: exfat_old_file, ext4_old_file;
    a_file_from_the_far_future_is_not_copied_again: exfat_future_file, ext4_future_file;
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4_files_and_folders_are_open_to_everyone() {
    let test = Test::new(Format::Ext4);
    test.write_source(b"first", time(1_600_000_000));
    test.visit().unwrap();
    test.write_source(b"second", time(1_600_000_100));
    test.visit().unwrap();

    let folder = test.newest().with_file_name("report[]pdf");
    assert_eq!(RealFs.metadata(&test.newest()).unwrap().mode, 0o666);
    assert_eq!(RealFs.metadata(&folder).unwrap().mode, 0o777);
    assert_eq!(
        RealFs
            .metadata(test.newest().parent().unwrap())
            .unwrap()
            .mode,
        0o777
    );
}
