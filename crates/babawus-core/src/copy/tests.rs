use std::ffi::OsString;
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use super::*;
use crate::fs::WriteFile;
use crate::fs::fake::FakeFs;

const SOURCE: &str = "/home/user/Projects/report.pdf";
const TARGET: &str = "backup/mint/user/Projects/report.pdf";
const ROOT: &str = "/medium";
const TEMP: &str = "/medium/.babawus/tmp";
const NEWEST: &str = "/medium/backup/mint/user/Projects/report.pdf";
const VERSIONS: &str = "/medium/backup/mint/user/Projects/report[]pdf";

fn time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + seconds)
}

fn read(fs: &dyn FileSystem, path: &str) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    fs.open(Path::new(path))?.read_to_end(&mut data)?;
    Ok(data)
}

fn names(fs: &dyn FileSystem, path: &str) -> Vec<String> {
    let entries = fs.read_dir(Path::new(path)).unwrap_or_default();
    entries
        .iter()
        .map(|name| name.to_string_lossy().into_owned())
        .collect()
}

struct Setup {
    source: FakeFs,
    medium: FakeFs,
    format: Format,
    keep: usize,
    clock: Mutex<String>,
    cancel: AtomicBool,
}

impl Setup {
    fn new() -> Self {
        let medium = FakeFs::new();
        medium.add_dir_on_device(TEMP, 1);
        Self {
            source: FakeFs::new(),
            medium,
            format: Format::ExFat,
            keep: 3,
            clock: Mutex::new("2026-11-03_175400".to_owned()),
            cancel: AtomicBool::new(false),
        }
    }

    /// A source file with new content, and its old newest copy on the medium.
    fn changed() -> Self {
        let setup = Self::new();
        setup.source.add_file(SOURCE, b"new", time(100));
        setup.medium.add_file(NEWEST, b"old", time(0));
        setup
    }

    fn set_clock(&self, time: &str) {
        *self.clock.lock().unwrap() = time.to_owned();
    }

    fn visit_with(
        &self,
        source: &dyn FileSystem,
        progress: &mut dyn FnMut(u64),
    ) -> Result<Visit, Stop> {
        let now = || self.clock.lock().unwrap().clone();
        let root = PathBuf::from(ROOT);
        let mut copier = FileCopier::new(
            source,
            &self.medium,
            root,
            self.format,
            self.keep,
            &now,
            &self.cancel,
        );
        copier.visit(Path::new(SOURCE), Path::new(TARGET), progress)
    }

    fn visit(&self) -> Result<Visit, Stop> {
        self.visit_with(&self.source, &mut |_| {})
    }

    fn start_run(&self) {
        let now = String::new;
        let root = PathBuf::from(ROOT);
        let copier = FileCopier::new(
            &self.source,
            &self.medium,
            root,
            self.format,
            self.keep,
            &now,
            &self.cancel,
        );
        copier.empty_temp_folder().unwrap();
    }

    fn newest(&self) -> Vec<u8> {
        read(&self.medium, NEWEST).unwrap()
    }

    fn versions(&self) -> Vec<String> {
        names(&self.medium, VERSIONS)
    }

    fn temp_files(&self) -> Vec<String> {
        names(&self.medium, TEMP)
    }
}

const COPIED: Result<Visit, Stop> = Ok(Visit::Copied(Pruned {
    removed: 0,
    failed: false,
}));
const UNCHANGED: Result<Visit, Stop> = Ok(Visit::Unchanged(Pruned {
    removed: 0,
    failed: false,
}));

#[test]
fn a_new_file_is_copied_with_its_modification_time() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"content", time(0));

    assert_eq!(setup.visit(), COPIED);

    assert_eq!(setup.newest(), b"content");
    assert_eq!(
        setup.medium.metadata(Path::new(NEWEST)).unwrap().modified,
        time(0)
    );
    assert!(setup.temp_files().is_empty());
}

#[test]
fn a_new_file_gets_no_version_folder() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"content", time(0));

    setup.visit().unwrap();

    assert!(setup.medium.metadata(Path::new(VERSIONS)).is_err());
}

#[test]
fn a_small_new_file_needs_a_single_flush() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"content", time(0));
    setup
        .medium
        .add_dir_on_device("/medium/backup/mint/user/Projects", 1);

    setup.visit().unwrap();

    // Create, write, set the time, flush, rename.
    assert_eq!(setup.medium.steps(), 5);
}

#[test]
fn an_unchanged_file_changes_nothing() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"same", time(0));
    setup.medium.add_file(NEWEST, b"same", time(0));

    assert_eq!(setup.visit(), UNCHANGED);

    assert_eq!(setup.medium.steps(), 0);
}

#[test]
fn a_time_within_the_tolerance_counts_as_unchanged() {
    for copy_time in [time(8), time(10), time(12)] {
        let setup = Setup::new();
        setup.source.add_file(SOURCE, b"same", time(10));
        setup.medium.add_file(NEWEST, b"same", copy_time);

        assert_eq!(setup.visit(), UNCHANGED);
    }
}

#[test]
fn a_time_beyond_the_tolerance_counts_as_changed() {
    for copy_time in [time(7), time(13)] {
        let setup = Setup::new();
        setup.source.add_file(SOURCE, b"same", time(10));
        setup.medium.add_file(NEWEST, b"same", copy_time);

        assert_eq!(setup.visit(), COPIED);
    }
}

#[test]
fn on_exfat_a_time_it_cannot_store_counts_as_the_nearest_it_can() {
    let before_1980 = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
    let earliest = SystemTime::UNIX_EPOCH + Duration::from_secs(EXFAT_EARLIEST);
    let after_2107 = SystemTime::UNIX_EPOCH + Duration::from_secs(7_258_118_400);
    let latest = SystemTime::UNIX_EPOCH + Duration::from_secs(EXFAT_LATEST);
    for (source_time, copy_time) in [(before_1980, earliest), (after_2107, latest)] {
        let setup = Setup::new();
        setup.source.add_file(SOURCE, b"same", source_time);
        setup.medium.add_file(NEWEST, b"same", copy_time);

        assert_eq!(setup.visit(), UNCHANGED);
    }
}

#[test]
fn on_ext4_a_time_before_1980_is_compared_as_it_is() {
    let mut setup = Setup::new();
    setup.format = Format::Ext4;
    let before_1980 = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
    let earliest = SystemTime::UNIX_EPOCH + Duration::from_secs(EXFAT_EARLIEST);
    setup.source.add_file(SOURCE, b"same", before_1980);
    setup.medium.add_file(NEWEST, b"same", earliest);

    assert_eq!(setup.visit(), COPIED);
}

#[test]
fn a_different_size_counts_as_changed() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"longer", time(0));
    setup.medium.add_file(NEWEST, b"short", time(0));

    assert_eq!(setup.visit(), COPIED);
}

#[test]
fn a_changed_file_keeps_the_old_copy_as_an_older_version() {
    let setup = Setup::changed();

    assert_eq!(setup.visit(), COPIED);

    assert_eq!(setup.newest(), b"new");
    assert_eq!(setup.versions(), ["report[2026-11-03_175400].pdf"]);
    let version = format!("{VERSIONS}/report[2026-11-03_175400].pdf");
    assert_eq!(read(&setup.medium, &version).unwrap(), b"old");
    assert_eq!(
        setup.medium.metadata(Path::new(&version)).unwrap().modified,
        time(0)
    );
}

#[test]
fn a_second_version_in_the_same_second_gets_a_suffix() {
    let setup = Setup::changed();
    setup.medium.add_file(
        format!("{VERSIONS}/report[2026-11-03_175400].pdf"),
        b"",
        time(0),
    );

    setup.visit().unwrap();

    assert_eq!(
        setup.versions(),
        [
            "report[2026-11-03_175400-2].pdf",
            "report[2026-11-03_175400].pdf"
        ]
    );
}

#[test]
fn the_oldest_version_goes_when_the_limit_is_passed() {
    let setup = Setup::changed();
    for day in ["01", "02", "03"] {
        let version = format!("{VERSIONS}/report[2026-10-{day}_120000].pdf");
        setup.medium.add_file(version, b"", time(0));
    }

    let visit = setup.visit();

    assert_eq!(
        visit,
        Ok(Visit::Copied(Pruned {
            removed: 1,
            failed: false
        }))
    );
    assert_eq!(
        setup.versions(),
        [
            "report[2026-10-02_120000].pdf",
            "report[2026-10-03_120000].pdf",
            "report[2026-11-03_175400].pdf",
        ]
    );
}

#[test]
fn an_unchanged_file_is_pruned_too() {
    let mut setup = Setup::new();
    setup.keep = 1;
    setup.source.add_file(SOURCE, b"same", time(0));
    setup.medium.add_file(NEWEST, b"same", time(0));
    for day in ["01", "02"] {
        let version = format!("{VERSIONS}/report[2026-10-{day}_120000].pdf");
        setup.medium.add_file(version, b"", time(0));
    }

    let visit = setup.visit();

    assert_eq!(
        visit,
        Ok(Visit::Unchanged(Pruned {
            removed: 1,
            failed: false
        }))
    );
    assert_eq!(setup.versions(), ["report[2026-10-02_120000].pdf"]);
}

#[test]
fn pruning_leaves_other_files_alone() {
    let mut setup = Setup::changed();
    setup.keep = 1;
    setup
        .medium
        .add_file(format!("{VERSIONS}/notes.txt"), b"", time(0));
    setup.medium.add_file(
        format!("{VERSIONS}/report[2026-10-01_120000].pdf"),
        b"",
        time(0),
    );

    setup.visit().unwrap();

    assert_eq!(
        setup.versions(),
        ["notes.txt", "report[2026-11-03_175400].pdf"]
    );
}

#[test]
fn a_failed_prune_does_not_undo_the_copy() {
    let mut setup = Setup::changed();
    setup.keep = 1;
    let old = format!("{VERSIONS}/report[2026-10-01_120000].pdf");
    setup.medium.add_file(&old, b"", time(0));
    setup.medium.fail_path(&old, ErrorKind::PermissionDenied);

    let visit = setup.visit();

    assert_eq!(
        visit,
        Ok(Visit::Copied(Pruned {
            removed: 0,
            failed: true
        }))
    );
    assert_eq!(setup.newest(), b"new");
}

#[test]
fn a_file_that_became_a_folder_in_the_source_becomes_an_older_version() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"content", time(0));
    let old_file = "/medium/backup/mint/user/Projects";
    setup.medium.add_file(old_file, b"old file", time(0));

    assert_eq!(setup.visit(), COPIED);

    assert_eq!(setup.newest(), b"content");
    let version = "/medium/backup/mint/user/Projects[]/Projects[2026-11-03_175400]";
    assert_eq!(read(&setup.medium, version).unwrap(), b"old file");
}

#[test]
fn a_folder_that_became_a_file_in_the_source_is_skipped() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"content", time(0));
    setup
        .medium
        .add_file(format!("{NEWEST}/inner"), b"backup", time(0));

    assert_eq!(setup.visit(), Ok(Visit::Skipped(Skip::TypeConflict)));

    assert_eq!(
        read(&setup.medium, &format!("{NEWEST}/inner")).unwrap(),
        b"backup"
    );
}

#[test]
fn on_ext4_everything_created_is_open_to_everyone() {
    let mut setup = Setup::changed();
    setup.format = Format::Ext4;
    setup
        .medium
        .add_dir_on_device("/medium/backup/mint/user/Projects", 1);

    setup.visit().unwrap();

    assert_eq!(
        setup.medium.metadata(Path::new(NEWEST)).unwrap().mode,
        0o666
    );
    assert_eq!(
        setup.medium.metadata(Path::new(VERSIONS)).unwrap().mode,
        0o777
    );
}

#[test]
fn on_ext4_new_parent_folders_are_open_to_everyone() {
    let mut setup = Setup::new();
    setup.format = Format::Ext4;
    setup.source.add_file(SOURCE, b"content", time(0));

    setup.visit().unwrap();

    let folder = setup
        .medium
        .metadata(Path::new("/medium/backup/mint"))
        .unwrap();
    assert_eq!(folder.mode, 0o777);
}

#[test]
fn on_exfat_no_modes_are_set() {
    let setup = Setup::changed();

    setup.visit().unwrap();

    assert_ne!(
        setup.medium.metadata(Path::new(NEWEST)).unwrap().mode,
        0o666
    );
}

#[test]
fn a_version_name_that_is_too_long_leaves_the_newest_copy_alone() {
    let name = format!("{}.txt", "a".repeat(240));
    let setup = Setup::new();
    let source = format!("/home/user/{name}");
    let target = format!("backup/{name}");
    setup.source.add_file(&source, b"new", time(100));
    setup
        .medium
        .add_file(format!("/medium/{target}"), b"old", time(0));
    let now = || "2026-11-03_175400".to_owned();
    let mut copier = FileCopier::new(
        &setup.source,
        &setup.medium,
        PathBuf::from(ROOT),
        Format::Ext4,
        3,
        &now,
        &setup.cancel,
    );

    let visit = copier.visit(Path::new(&source), Path::new(&target), &mut |_| {});

    assert_eq!(visit, Ok(Visit::Skipped(Skip::VersionNameTooLong)));
    assert_eq!(
        read(&setup.medium, &format!("/medium/{target}")).unwrap(),
        b"old"
    );
    assert_eq!(setup.medium.steps(), 0);
}

#[test]
fn a_file_that_does_not_fit_stops_the_run_and_changes_nothing() {
    let setup = Setup::changed();
    setup.medium.set_capacity(5);

    assert_eq!(setup.visit(), Err(Stop::MediumFull));

    assert_eq!(setup.newest(), b"old");
    assert_eq!(setup.medium.steps(), 0);
}

#[test]
fn running_out_of_space_while_writing_stops_the_run_safely() {
    let setup = Setup::changed();
    let temp = format!("{TEMP}/copy-1");
    setup.medium.fail_path(&temp, ErrorKind::StorageFull);

    assert_eq!(setup.visit(), Err(Stop::MediumFull));

    assert_eq!(setup.newest(), b"old");
    assert!(setup.versions().is_empty());
}

#[test]
fn a_source_that_cannot_be_read_is_skipped() {
    let setup = Setup::changed();
    setup.source.fail_path(SOURCE, ErrorKind::PermissionDenied);

    assert_eq!(setup.visit(), Ok(Visit::Skipped(Skip::Unreadable)));

    assert_eq!(setup.newest(), b"old");
}

/// A source that misbehaves in ways the fake can't: its reads fail once they have started, or it
/// reports a new modification time on each look, up to a limit.
struct TrickySource {
    fake: FakeFs,
    broken_reads: bool,
    changes: Option<u32>,
    looks: AtomicU32,
}

impl TrickySource {
    fn new(fake: &FakeFs) -> Self {
        Self {
            fake: fake.clone(),
            broken_reads: false,
            changes: None,
            looks: AtomicU32::new(0),
        }
    }
}

struct BrokenReader;

impl Read for BrokenReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::from_raw_os_error(libc::EIO))
    }
}

impl FileSystem for TrickySource {
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let metadata = self.fake.metadata(path)?;
        let Some(limit) = self.changes else {
            return Ok(metadata);
        };
        let looks = self.looks.fetch_add(1, Ordering::Relaxed).min(limit);
        Ok(Metadata {
            modified: time(1000 + u64::from(looks)),
            ..metadata
        })
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read + Send>> {
        if self.broken_reads {
            return Ok(Box::new(BrokenReader));
        }
        self.fake.open(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        self.fake.read_dir(path)
    }

    fn create_new(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        self.fake.create_new(path)
    }

    fn open_append(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        self.fake.open_append(path)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.fake.create_dir(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.fake.rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.fake.remove_file(path)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.fake.set_mode(path, mode)
    }

    fn free_space(&self, path: &Path) -> io::Result<u64> {
        self.fake.free_space(path)
    }

    fn sync_medium(&self, path: &Path) -> io::Result<()> {
        self.fake.sync_medium(path)
    }
}

#[test]
fn a_read_error_during_the_copy_skips_the_file_and_removes_the_temporary_file() {
    let setup = Setup::changed();
    let source = TrickySource {
        broken_reads: true,
        ..TrickySource::new(&setup.source)
    };

    let visit = setup.visit_with(&source, &mut |_| {});

    assert_eq!(visit, Ok(Visit::Skipped(Skip::Unreadable)));
    assert_eq!(setup.newest(), b"old");
    assert!(setup.temp_files().is_empty());
}

#[test]
fn a_file_that_changes_during_the_copy_is_tried_once_more() {
    let setup = Setup::changed();
    // The first look and the check after the first copy differ. After that it stays the same.
    let source = TrickySource {
        changes: Some(1),
        ..TrickySource::new(&setup.source)
    };
    let mut progress = Vec::new();

    let visit = setup.visit_with(&source, &mut |bytes| progress.push(bytes));

    assert_eq!(visit, COPIED);
    assert_eq!(setup.newest(), b"new");
    assert_eq!(progress, [3, 0, 3]);
    assert!(setup.temp_files().is_empty());
}

#[test]
fn a_file_that_keeps_changing_is_skipped() {
    let setup = Setup::changed();
    let source = TrickySource {
        changes: Some(u32::MAX),
        ..TrickySource::new(&setup.source)
    };

    let visit = setup.visit_with(&source, &mut |_| {});

    assert_eq!(visit, Ok(Visit::Skipped(Skip::ChangedDuringCopy)));
    assert_eq!(setup.newest(), b"old");
    assert!(setup.versions().is_empty());
    assert!(setup.temp_files().is_empty());
}

#[test]
fn progress_counts_each_flushed_chunk() {
    let setup = Setup::new();
    let size = CHUNK_SIZE * 2 + 10;
    setup.source.add_file(SOURCE, &vec![7; size], time(0));
    let mut progress = Vec::new();

    setup
        .visit_with(&setup.source, &mut |bytes| progress.push(bytes))
        .unwrap();

    let chunk = CHUNK_SIZE as u64;
    assert_eq!(progress, [chunk, chunk * 2, chunk * 2 + 10]);
    assert_eq!(setup.newest().len(), size);
}

#[test]
fn progress_is_only_reported_after_the_flush() {
    let setup = Setup::new();
    setup
        .source
        .add_file(SOURCE, &vec![7; CHUNK_SIZE + 10], time(0));
    setup
        .medium
        .add_dir_on_device("/medium/backup/mint/user/Projects", 1);
    let mut steps = Vec::new();

    setup
        .visit_with(&setup.source, &mut |_| steps.push(setup.medium.steps()))
        .unwrap();

    // Create, write, flush. Then write, set the time, flush.
    assert_eq!(steps, [3, 6]);
}

#[test]
fn a_file_of_exactly_one_chunk_keeps_its_modification_time() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, &vec![7; CHUNK_SIZE], time(0));
    let mut progress = Vec::new();

    setup
        .visit_with(&setup.source, &mut |bytes| progress.push(bytes))
        .unwrap();

    assert_eq!(progress, [CHUNK_SIZE as u64]);
    assert_eq!(
        setup.medium.metadata(Path::new(NEWEST)).unwrap().modified,
        time(0)
    );
}

#[test]
fn an_empty_file_is_copied() {
    let setup = Setup::new();
    setup.source.add_file(SOURCE, b"", time(0));

    assert_eq!(setup.visit(), COPIED);

    assert_eq!(setup.newest(), b"");
}

#[test]
fn cancelling_before_the_copy_changes_nothing() {
    let setup = Setup::changed();
    setup.cancel.store(true, Ordering::Relaxed);

    assert_eq!(setup.visit(), Ok(Visit::Cancelled));

    assert_eq!(setup.medium.steps(), 0);
}

#[test]
fn cancelling_during_a_large_file_stops_after_the_chunk() {
    let setup = Setup::changed();
    setup
        .source
        .add_file(SOURCE, &vec![7; CHUNK_SIZE * 3], time(100));
    let mut chunks = 0;

    let visit = setup.visit_with(&setup.source, &mut |_| {
        chunks += 1;
        setup.cancel.store(true, Ordering::Relaxed);
    });

    assert_eq!(visit, Ok(Visit::Cancelled));
    assert_eq!(chunks, 1);
    assert_eq!(setup.newest(), b"old");
    assert!(setup.versions().is_empty());
    assert!(setup.temp_files().is_empty());
}

#[test]
fn a_temporary_folder_that_cannot_be_written_stops_the_run() {
    let setup = Setup::changed();
    setup
        .medium
        .fail_path(format!("{TEMP}/copy-1"), ErrorKind::PermissionDenied);

    assert_eq!(setup.visit(), Err(Stop::CannotWrite));
}

#[test]
fn a_folder_that_cannot_be_written_skips_the_file() {
    let setup = Setup::changed();
    setup
        .medium
        .fail_path(VERSIONS, ErrorKind::PermissionDenied);

    assert_eq!(setup.visit(), Ok(Visit::Skipped(Skip::NotWritable)));

    assert_eq!(setup.newest(), b"old");
    assert!(setup.temp_files().is_empty());
}

#[test]
fn another_write_error_skips_the_file() {
    let setup = Setup::changed();
    setup
        .medium
        .fail_path(format!("{TEMP}/copy-1"), ErrorKind::InvalidInput);

    assert_eq!(setup.visit(), Ok(Visit::Skipped(Skip::WriteError)));
}

#[test]
fn a_read_only_medium_stops_the_run() {
    let setup = Setup::changed();
    setup
        .medium
        .fail_path(format!("{TEMP}/copy-1"), ErrorKind::ReadOnlyFilesystem);

    assert_eq!(setup.visit(), Err(Stop::CannotWrite));
}

#[test]
fn the_temporary_folder_is_emptied_at_the_start_of_a_run() {
    let setup = Setup::new();
    setup
        .medium
        .add_file(format!("{TEMP}/left-over"), b"", time(0));

    setup.start_run();

    assert!(setup.temp_files().is_empty());
}

#[test]
fn a_missing_temporary_folder_is_created() {
    let setup = Setup::new();
    let medium = FakeFs::new();
    medium.add_dir_on_device("/medium/.babawus", 1);
    let setup = Setup { medium, ..setup };

    setup.start_run();

    assert!(setup.medium.metadata(Path::new(TEMP)).is_ok());
}

#[test]
fn flushing_the_medium_makes_everything_safe() {
    let setup = Setup::changed();
    setup.visit().unwrap();
    let now = String::new;
    let copier = FileCopier::new(
        &setup.source,
        &setup.medium,
        PathBuf::from(ROOT),
        setup.format,
        setup.keep,
        &now,
        &setup.cancel,
    );

    copier.flush_medium().unwrap();
    setup.medium.reinsert_losing_unflushed();

    assert_eq!(setup.newest(), b"new");
}

#[test]
fn a_medium_that_cannot_be_flushed_is_gone() {
    let setup = Setup::new();
    setup.medium.pull_at_step(1);
    let now = String::new;
    let copier = FileCopier::new(
        &setup.source,
        &setup.medium,
        PathBuf::from(ROOT),
        setup.format,
        setup.keep,
        &now,
        &setup.cancel,
    );

    assert_eq!(copier.flush_medium(), Err(Stop::MediumGone));
}

/// Everything that must hold after the medium was pulled out, and after the next run.
fn check_interrupted_run(format: Format, step: u64, lose_unflushed: bool) {
    let mut setup = Setup::changed();
    setup.format = format;
    setup.keep = 1;
    let oldest = format!("{VERSIONS}/report[2026-10-01_120000].pdf");
    setup.medium.add_file(&oldest, b"oldest", time(0));
    setup.medium.pull_at_step(step);
    let context = format!("{format:?}, step {step}, lose_unflushed {lose_unflushed}");

    assert_eq!(setup.visit(), Err(Stop::MediumGone), "{context}");

    if lose_unflushed {
        setup.medium.reinsert_losing_unflushed();
    } else {
        setup.medium.reinsert();
    }

    // No half-written copy, and the old content is somewhere.
    let version = format!("{VERSIONS}/report[2026-11-03_175400].pdf");
    let newest = read(&setup.medium, NEWEST);
    let version_content = read(&setup.medium, &version);
    match newest.as_deref() {
        Ok(b"old") | Ok(b"new") => {}
        Err(_) => assert_eq!(
            version_content.as_deref().ok(),
            Some(&b"old"[..]),
            "{context}"
        ),
        Ok(other) => panic!("{context}: half-written copy {other:?}"),
    }
    if newest.as_deref().ok() == Some(b"new") {
        assert_eq!(
            version_content.as_deref().ok(),
            Some(&b"old"[..]),
            "{context}"
        );
    }

    // The next run catches up without anything being cleaned up by hand.
    setup.set_clock("2026-11-04_080000");
    setup.start_run();
    let next = setup.visit();
    assert!(
        matches!(next, Ok(Visit::Copied(_) | Visit::Unchanged(_))),
        "{context}: {next:?}"
    );
    assert_eq!(setup.newest(), b"new", "{context}");
    assert!(setup.temp_files().is_empty(), "{context}");
    let old_kept = setup.versions().iter().any(|name| {
        read(&setup.medium, &format!("{VERSIONS}/{name}"))
            .as_deref()
            .ok()
            == Some(&b"old"[..])
    });
    assert!(old_kept, "{context}: {:?}", setup.versions());
}

#[test]
fn the_medium_can_be_pulled_out_at_every_step() {
    for format in [Format::ExFat, Format::Ext4] {
        let steps = {
            let mut setup = Setup::changed();
            setup.format = format;
            setup.keep = 1;
            let oldest = format!("{VERSIONS}/report[2026-10-01_120000].pdf");
            setup.medium.add_file(oldest, b"oldest", time(0));
            setup.visit().unwrap();
            setup.medium.steps()
        };
        assert!(steps >= 6, "{steps}");
        for step in 1..=steps {
            check_interrupted_run(format, step, false);
            check_interrupted_run(format, step, true);
        }
    }
}
