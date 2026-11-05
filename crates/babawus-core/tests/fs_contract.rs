//! The behaviour that [`FileSystem`] promises, tested on every implementation.
//!
//! The fake and a real folder are always tested. The tests on exFAT and Ext4 use sudo and loop
//! devices, so a plain `cargo test` ignores them. See `loop_media.rs`.

#[allow(dead_code)]
mod support;

use std::io::{ErrorKind, Read};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use babawus_core::fs::fake::FakeFs;
use babawus_core::fs::{FileKind, FileSystem, RealFs};
use support::{LoopMedium, MediumFormat};

type Case = (&'static str, fn(&dyn FileSystem, &Path));

/// Modification times are compared with this tolerance, see Requirements.md.
const TOLERANCE: Duration = Duration::from_secs(2);

const CASES: &[Case] = &[
    ("create_write_read", create_write_read),
    ("create_new_refuses_existing", create_new_refuses_existing),
    ("create_new_needs_parent", create_new_needs_parent),
    ("append_adds_to_the_end", append_adds_to_the_end),
    ("append_creates_a_file", append_creates_a_file),
    ("create_dir_refuses_existing", create_dir_refuses_existing),
    ("create_dir_needs_parent", create_dir_needs_parent),
    ("rename_moves_a_file", rename_moves_a_file),
    ("rename_replaces_a_file", rename_replaces_a_file),
    ("rename_moves_a_folder", rename_moves_a_folder),
    ("rename_of_missing_file_fails", rename_of_missing_file_fails),
    (
        "rename_over_full_folder_fails",
        rename_over_full_folder_fails,
    ),
    ("remove_file_removes", remove_file_removes),
    ("remove_file_refuses_a_folder", remove_file_refuses_a_folder),
    ("metadata_of_a_file", metadata_of_a_file),
    ("metadata_of_a_folder", metadata_of_a_folder),
    (
        "metadata_of_missing_file_fails",
        metadata_of_missing_file_fails,
    ),
    ("read_dir_is_sorted_by_bytes", read_dir_is_sorted_by_bytes),
    ("read_dir_of_a_file_fails", read_dir_of_a_file_fails),
    (
        "read_dir_of_missing_folder_fails",
        read_dir_of_missing_folder_fails,
    ),
    ("reading_a_folder_fails", reading_a_folder_fails),
    ("modified_time_is_kept", modified_time_is_kept),
    (
        "modified_time_survives_rename",
        modified_time_survives_rename,
    ),
    ("free_space_is_reported", free_space_is_reported),
    ("sync_medium_succeeds", sync_medium_succeeds),
];

/// For file systems that store permissions, so not exFAT.
const MODE_CASES: &[Case] = &[("set_mode_is_exact", set_mode_is_exact)];

fn run(cases: &[Case], fs: &dyn FileSystem, root: &Path) {
    for (name, case) in cases {
        eprintln!("case: {name}");
        let dir = root.join(name);
        fs.create_dir(&dir).unwrap();
        case(fs, &dir);
    }
}

fn write(fs: &dyn FileSystem, path: &Path, data: &[u8]) {
    let mut file = fs.create_new(path).unwrap();
    file.write_all(data).unwrap();
    file.sync().unwrap();
}

fn read(fs: &dyn FileSystem, path: &Path) -> Vec<u8> {
    let mut data = Vec::new();
    fs.open(path).unwrap().read_to_end(&mut data).unwrap();
    data
}

fn error_kind<T>(result: std::io::Result<T>) -> ErrorKind {
    result.err().expect("this must fail").kind()
}

fn create_write_read(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("file");

    write(fs, &path, b"content");

    assert_eq!(read(fs, &path), b"content");
}

fn create_new_refuses_existing(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("file");
    write(fs, &path, b"content");

    assert_eq!(error_kind(fs.create_new(&path)), ErrorKind::AlreadyExists);

    assert_eq!(read(fs, &path), b"content");
}

fn create_new_needs_parent(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("missing/file");

    assert_eq!(error_kind(fs.create_new(&path)), ErrorKind::NotFound);
}

fn append_adds_to_the_end(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("log");
    write(fs, &path, b"first");

    let mut file = fs.open_append(&path).unwrap();
    file.write_all(b" second").unwrap();
    file.sync().unwrap();

    assert_eq!(read(fs, &path), b"first second");
}

fn append_creates_a_file(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("log");

    let mut file = fs.open_append(&path).unwrap();
    file.write_all(b"first").unwrap();
    file.sync().unwrap();

    assert_eq!(read(fs, &path), b"first");
}

fn create_dir_refuses_existing(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("folder");
    fs.create_dir(&path).unwrap();

    assert_eq!(error_kind(fs.create_dir(&path)), ErrorKind::AlreadyExists);
}

fn create_dir_needs_parent(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("missing/folder");

    assert_eq!(error_kind(fs.create_dir(&path)), ErrorKind::NotFound);
}

fn rename_moves_a_file(fs: &dyn FileSystem, dir: &Path) {
    let (from, to) = (dir.join("from"), dir.join("to"));
    write(fs, &from, b"content");

    fs.rename(&from, &to).unwrap();

    assert_eq!(read(fs, &to), b"content");
    assert_eq!(error_kind(fs.metadata(&from)), ErrorKind::NotFound);
}

fn rename_replaces_a_file(fs: &dyn FileSystem, dir: &Path) {
    let (from, to) = (dir.join("from"), dir.join("to"));
    write(fs, &from, b"new");
    write(fs, &to, b"old");

    fs.rename(&from, &to).unwrap();

    assert_eq!(read(fs, &to), b"new");
    assert_eq!(fs.read_dir(dir).unwrap(), ["to"]);
}

fn rename_moves_a_folder(fs: &dyn FileSystem, dir: &Path) {
    let (from, to) = (dir.join("from"), dir.join("to"));
    fs.create_dir(&from).unwrap();
    write(fs, &from.join("file"), b"content");

    fs.rename(&from, &to).unwrap();

    assert_eq!(read(fs, &to.join("file")), b"content");
}

fn rename_of_missing_file_fails(fs: &dyn FileSystem, dir: &Path) {
    let result = fs.rename(&dir.join("missing"), &dir.join("to"));

    assert_eq!(error_kind(result), ErrorKind::NotFound);
}

fn rename_over_full_folder_fails(fs: &dyn FileSystem, dir: &Path) {
    let (from, to) = (dir.join("from"), dir.join("to"));
    fs.create_dir(&from).unwrap();
    fs.create_dir(&to).unwrap();
    write(fs, &to.join("file"), b"content");

    assert_eq!(
        error_kind(fs.rename(&from, &to)),
        ErrorKind::DirectoryNotEmpty
    );
}

fn remove_file_removes(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("file");
    write(fs, &path, b"content");

    fs.remove_file(&path).unwrap();

    assert_eq!(error_kind(fs.metadata(&path)), ErrorKind::NotFound);
}

fn remove_file_refuses_a_folder(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("folder");
    fs.create_dir(&path).unwrap();

    assert_eq!(error_kind(fs.remove_file(&path)), ErrorKind::IsADirectory);
}

fn metadata_of_a_file(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("file");
    write(fs, &path, b"content");

    let metadata = fs.metadata(&path).unwrap();

    assert_eq!(metadata.kind, FileKind::File);
    assert_eq!(metadata.len, 7);
    assert_eq!(metadata.device, fs.metadata(dir).unwrap().device);
}

fn metadata_of_a_folder(fs: &dyn FileSystem, dir: &Path) {
    assert_eq!(fs.metadata(dir).unwrap().kind, FileKind::Dir);
}

fn metadata_of_missing_file_fails(fs: &dyn FileSystem, dir: &Path) {
    assert_eq!(
        error_kind(fs.metadata(&dir.join("missing"))),
        ErrorKind::NotFound
    );
}

fn read_dir_is_sorted_by_bytes(fs: &dyn FileSystem, dir: &Path) {
    for name in ["b", "a.txt", "Z", "a"] {
        write(fs, &dir.join(name), b"");
    }

    assert_eq!(fs.read_dir(dir).unwrap(), ["Z", "a", "a.txt", "b"]);
}

fn read_dir_of_a_file_fails(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("file");
    write(fs, &path, b"");

    assert_eq!(error_kind(fs.read_dir(&path)), ErrorKind::NotADirectory);
}

fn read_dir_of_missing_folder_fails(fs: &dyn FileSystem, dir: &Path) {
    assert_eq!(
        error_kind(fs.read_dir(&dir.join("missing"))),
        ErrorKind::NotFound
    );
}

fn reading_a_folder_fails(fs: &dyn FileSystem, dir: &Path) {
    let result = fs.open(dir).and_then(|mut file| file.read(&mut [0; 1]));

    assert_eq!(error_kind(result), ErrorKind::IsADirectory);
}

fn assert_close(actual: SystemTime, expected: SystemTime) {
    let difference = actual
        .duration_since(expected)
        .or_else(|_| expected.duration_since(actual))
        .unwrap();
    assert!(difference <= TOLERANCE, "off by {difference:?}");
}

fn test_time() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::new(1_600_000_001, 123_456_789)
}

fn modified_time_is_kept(fs: &dyn FileSystem, dir: &Path) {
    let path = dir.join("file");
    let mut file = fs.create_new(&path).unwrap();
    file.write_all(b"content").unwrap();
    file.set_modified(test_time()).unwrap();
    file.sync().unwrap();
    drop(file);

    assert_close(fs.metadata(&path).unwrap().modified, test_time());
}

fn modified_time_survives_rename(fs: &dyn FileSystem, dir: &Path) {
    let (from, to) = (dir.join("from"), dir.join("to"));
    let mut file = fs.create_new(&from).unwrap();
    file.set_modified(test_time()).unwrap();
    file.sync().unwrap();
    drop(file);

    fs.rename(&from, &to).unwrap();

    assert_close(fs.metadata(&to).unwrap().modified, test_time());
}

fn free_space_is_reported(fs: &dyn FileSystem, dir: &Path) {
    assert!(fs.free_space(dir).unwrap() > 0);
}

fn sync_medium_succeeds(fs: &dyn FileSystem, dir: &Path) {
    write(fs, &dir.join("file"), b"content");

    fs.sync_medium(dir).unwrap();
}

fn set_mode_is_exact(fs: &dyn FileSystem, dir: &Path) {
    let (file, folder) = (dir.join("file"), dir.join("folder"));
    write(fs, &file, b"content");
    fs.create_dir(&folder).unwrap();

    fs.set_mode(&file, 0o666).unwrap();
    fs.set_mode(&folder, 0o777).unwrap();

    assert_eq!(fs.metadata(&file).unwrap().mode, 0o666);
    assert_eq!(fs.metadata(&folder).unwrap().mode, 0o777);
}

/// A new folder below the test's temporary folder, removed when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = format!("{name}-{}", std::process::id());
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(dir);
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn fake() {
    let fake = FakeFs::new();

    run(CASES, &fake, Path::new("/"));
    fake.create_dir(Path::new("/modes")).unwrap();
    run(MODE_CASES, &fake, Path::new("/modes"));
}

#[test]
fn real_folder() {
    let dir = TempDir::new("contract");

    run(CASES, &RealFs, &dir.0);
    run(MODE_CASES, &RealFs, &dir.0);
}

#[test]
fn real_symbolic_links_are_not_followed() {
    let dir = TempDir::new("links");
    let link = dir.0.join("link");
    symlink(&dir.0, &link).unwrap();

    assert_eq!(RealFs.metadata(&link).unwrap().kind, FileKind::Other);
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn exfat() {
    let medium = LoopMedium::new(MediumFormat::ExFat, 32 * 1024 * 1024);

    run(CASES, &RealFs, medium.path());
}

#[test]
#[ignore = "needs sudo and loop devices"]
fn ext4() {
    let medium = LoopMedium::new(MediumFormat::Ext4, 32 * 1024 * 1024);

    run(CASES, &RealFs, medium.path());
    run(MODE_CASES, &RealFs, medium.path());
}
