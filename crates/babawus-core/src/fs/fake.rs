//! An in-memory file system that can inject faults, for testing.
//!
//! Every change is a numbered *step*. A test can pull the medium out at any step, and then put
//! it back in one of two ways:
//!
//! - [`FakeFs::reinsert`] keeps everything done before that step, which shows the state between
//!   any two steps of an algorithm.
//! - [`FakeFs::reinsert_losing_unflushed`] loses everything that wasn't flushed, the worst that
//!   a real medium can do. Only the content of a flushed file survives, and new names and renames
//!   only survive [`sync_medium`](FileSystem::sync_medium).
//!
//! Real media lie between these two. The fake does not model them losing only some changes.
//!
//! Names behave as on Ext4, or with [`FakeFs::exfat`], as on exFAT, which ignores case.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::io::{self, Cursor, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use super::{FileKind, FileSystem, Metadata, WriteFile};
use crate::names::fold;

const DEFAULT_DEVICE: u64 = 1;
const DEFAULT_FILE_MODE: u32 = 0o644;
const DEFAULT_DIR_MODE: u32 = 0o755;

#[derive(Clone, Debug)]
struct Node {
    kind: FileKind,
    data: Vec<u8>,
    modified: SystemTime,
    mode: u32,
    device: u64,
}

impl Node {
    fn new(kind: FileKind) -> Self {
        let mode = match kind {
            FileKind::Dir => DEFAULT_DIR_MODE,
            _ => DEFAULT_FILE_MODE,
        };
        Self {
            kind,
            data: Vec::new(),
            modified: SystemTime::now(),
            mode,
            device: DEFAULT_DEVICE,
        }
    }
}

/// Names point to nodes, so that a file keeps its identity when it is renamed.
#[derive(Clone, Debug, Default)]
struct Tree {
    names: BTreeMap<PathBuf, u64>,
    nodes: HashMap<u64, Node>,
}

impl Tree {
    fn get(&self, path: &Path) -> Option<&Node> {
        self.nodes.get(self.names.get(path)?)
    }

    fn used(&self) -> u64 {
        let files = self
            .names
            .values()
            .map(|id| self.nodes[id].data.len() as u64);
        files.sum()
    }
}

#[derive(Debug, Default)]
struct State {
    live: Tree,
    flushed: Tree,
    next_id: u64,
    steps: u64,
    pull_at: Option<u64>,
    pulled: bool,
    capacity: Option<u64>,
    failures: HashMap<PathBuf, ErrorKind>,
    ignore_case: bool,
    raise_on_sync: Option<Arc<AtomicBool>>,
}

fn medium_gone() -> io::Error {
    crate::platform::medium_gone_error()
}

impl State {
    /// `path` with each part in the case it has on the file system, if that ignores case.
    fn resolve(&self, path: &Path) -> PathBuf {
        if !self.ignore_case {
            return path.to_owned();
        }
        let mut resolved = PathBuf::new();
        for part in path {
            resolved.push(part);
            if self.live.names.contains_key(&resolved) {
                continue;
            }
            let wanted = fold(&part.to_string_lossy());
            let parent = resolved.parent().map(Path::to_owned);
            let same = self.live.names.keys().find(|name| {
                let name_part = name.file_name().map(|part| fold(&part.to_string_lossy()));
                name.parent() == parent.as_deref() && name_part.as_deref() == Some(&wanted)
            });
            if let Some(same) = same {
                resolved = same.clone();
            }
        }
        resolved
    }

    /// Checks that `path` can be used, without counting a step.
    fn check(&self, path: &Path) -> io::Result<()> {
        if self.pulled {
            return Err(medium_gone());
        }
        match self.failures.get(path) {
            Some(&kind) => Err(kind.into()),
            None => Ok(()),
        }
    }

    /// Counts a step that changes something, and pulls the medium out if it is time to.
    fn step(&mut self) -> io::Result<()> {
        if self.pulled {
            return Err(medium_gone());
        }
        self.steps += 1;
        if self.pull_at == Some(self.steps) {
            self.pulled = true;
            return Err(medium_gone());
        }
        Ok(())
    }

    fn change(&mut self, path: &Path) -> io::Result<()> {
        self.step()?;
        self.check(path)
    }

    fn node(&self, path: &Path) -> io::Result<&Node> {
        self.live
            .get(path)
            .ok_or_else(|| ErrorKind::NotFound.into())
    }

    fn check_parent(&self, path: &Path) -> io::Result<()> {
        let parent = path.parent().ok_or(ErrorKind::InvalidInput)?;
        match self.node(parent)?.kind {
            FileKind::Dir => Ok(()),
            _ => Err(ErrorKind::NotADirectory.into()),
        }
    }

    fn insert(&mut self, path: &Path, node: Node) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.live.nodes.insert(id, node);
        self.live.names.insert(path.to_owned(), id);
        id
    }

    fn check_rename_target(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.check_parent(to)?;
        let Some(target) = self.live.get(to) else {
            return Ok(());
        };
        match (self.node(from)?.kind, target.kind) {
            (FileKind::Dir, FileKind::Dir) if self.children(to).next().is_some() => {
                Err(ErrorKind::DirectoryNotEmpty.into())
            }
            (FileKind::Dir, FileKind::Dir) => Ok(()),
            (FileKind::Dir, _) => Err(ErrorKind::NotADirectory.into()),
            (_, FileKind::Dir) => Err(ErrorKind::IsADirectory.into()),
            _ => Ok(()),
        }
    }

    fn children<'a>(&'a self, path: &'a Path) -> impl Iterator<Item = &'a PathBuf> {
        let names = self.live.names.keys();
        names.filter(move |name| name.parent() == Some(path))
    }

    fn free(&self) -> u64 {
        match self.capacity {
            Some(capacity) => capacity.saturating_sub(self.live.used()),
            None => u64::MAX,
        }
    }
}

/// An in-memory file system. Clones share the same files.
#[derive(Clone, Debug)]
pub struct FakeFs {
    state: Arc<Mutex<State>>,
}

impl Default for FakeFs {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeFs {
    /// An empty file system that holds only the root folder `/`.
    pub fn new() -> Self {
        let fake = Self {
            state: Arc::default(),
        };
        let mut state = fake.state();
        state.insert(Path::new("/"), Node::new(FileKind::Dir));
        state.flushed = state.live.clone();
        drop(state);
        fake
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .expect("no test panics while holding the lock")
    }

    /// An empty file system that ignores case in names, as exFAT does.
    pub fn exfat() -> Self {
        let fake = Self::new();
        fake.state().ignore_case = true;
        fake
    }

    /// Adds a file that is already safely there, with any missing parent folders. This sets up
    /// a test, so it doesn't count as a step.
    pub fn add_file(&self, path: impl AsRef<Path>, data: &[u8], modified: SystemTime) {
        let node = Node {
            data: data.to_vec(),
            modified,
            ..Node::new(FileKind::File)
        };
        self.add(path.as_ref(), node);
    }

    /// Adds a symbolic link or special file, as [`add_file`](Self::add_file) adds a file.
    pub fn add_other(&self, path: impl AsRef<Path>) {
        self.add(path.as_ref(), Node::new(FileKind::Other));
    }

    /// Adds a folder that is on another file system, as [`add_file`](Self::add_file) adds a file.
    pub fn add_dir_on_device(&self, path: impl AsRef<Path>, device: u64) {
        let node = Node {
            device,
            ..Node::new(FileKind::Dir)
        };
        self.add(path.as_ref(), node);
    }

    fn add(&self, path: &Path, node: Node) {
        let mut state = self.state();
        for parent in path.ancestors().skip(1) {
            if state.live.get(parent).is_none() {
                state.insert(parent, Node::new(FileKind::Dir));
            }
        }
        state.insert(path, node);
        state.flushed = state.live.clone();
    }

    /// Limits the bytes the files can use in total. A write beyond it writes what fits, and then
    /// fails with [`ErrorKind::StorageFull`].
    pub fn set_capacity(&self, bytes: u64) {
        self.state().capacity = Some(bytes);
    }

    /// Sets `flag` whenever a file has been flushed, for example to press Cancel while a file is
    /// being flushed.
    pub fn raise_on_sync(&self, flag: Arc<AtomicBool>) {
        self.state().raise_on_sync = Some(flag);
    }

    /// Makes every operation on exactly `path` fail with `kind`, for example to test a file that
    /// can't be read.
    pub fn fail_path(&self, path: impl AsRef<Path>, kind: ErrorKind) {
        self.state().failures.insert(path.as_ref().to_owned(), kind);
    }

    /// Whether the content of the file `path` has reached the medium, whether or not its name
    /// has.
    pub fn content_is_flushed(&self, path: impl AsRef<Path>) -> bool {
        let state = self.state();
        let data = |tree: &Tree, id| tree.nodes.get(id).map(|node: &Node| node.data.clone());
        state
            .live
            .names
            .get(path.as_ref())
            .is_some_and(|id| data(&state.live, id) == data(&state.flushed, id))
    }

    /// The number of steps so far. Run an algorithm once to learn how many steps there are to
    /// try pulling the medium out at.
    pub fn steps(&self) -> u64 {
        self.state().steps
    }

    /// Pulls the medium out when the count of steps reaches `step`, so that step fails and has
    /// no effect. From then on, everything fails until the medium is put back.
    pub fn pull_at_step(&self, step: u64) {
        self.state().pull_at = Some(step);
    }

    /// Puts the medium back, with everything done before it was pulled out.
    pub fn reinsert(&self) {
        let mut state = self.state();
        state.pulled = false;
        state.pull_at = None;
    }

    /// Puts the medium back, without anything that wasn't flushed.
    pub fn reinsert_losing_unflushed(&self) {
        let mut state = self.state();
        state.live = state.flushed.clone();
        state.pulled = false;
        state.pull_at = None;
    }
}

impl FileSystem for FakeFs {
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let state = self.state();
        let path = &state.resolve(path);
        state.check(path)?;
        let node = state.node(path)?;
        Ok(Metadata {
            kind: node.kind,
            len: node.data.len() as u64,
            modified: node.modified,
            mode: node.mode,
            device: node.device,
        })
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        let state = self.state();
        let path = &state.resolve(path);
        state.check(path)?;
        if state.node(path)?.kind != FileKind::Dir {
            return Err(ErrorKind::NotADirectory.into());
        }
        let names = state.children(path).filter_map(|child| child.file_name());
        // The tree is sorted by path, and these paths differ only in their last part.
        Ok(names.map(ToOwned::to_owned).collect())
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read + Send>> {
        let state = self.state();
        let path = &state.resolve(path);
        state.check(path)?;
        let node = state.node(path)?;
        if node.kind == FileKind::Dir {
            return Ok(Box::new(DirReader));
        }
        Ok(Box::new(Cursor::new(node.data.clone())))
    }

    fn create_new(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        let mut state = self.state();
        let path = &state.resolve(path);
        state.change(path)?;
        if state.live.get(path).is_some() {
            return Err(ErrorKind::AlreadyExists.into());
        }
        state.check_parent(path)?;
        let id = state.insert(path, Node::new(FileKind::File));
        Ok(Box::new(self.file(id)))
    }

    fn open_append(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        let mut state = self.state();
        let path = &state.resolve(path);
        state.change(path)?;
        if let Some(&id) = state.live.names.get(path) {
            if state.live.nodes[&id].kind == FileKind::Dir {
                return Err(ErrorKind::IsADirectory.into());
            }
            return Ok(Box::new(self.file(id)));
        }
        state.check_parent(path)?;
        let id = state.insert(path, Node::new(FileKind::File));
        Ok(Box::new(self.file(id)))
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        let mut state = self.state();
        let path = &state.resolve(path);
        state.change(path)?;
        if state.live.get(path).is_some() {
            return Err(ErrorKind::AlreadyExists.into());
        }
        state.check_parent(path)?;
        state.insert(path, Node::new(FileKind::Dir));
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut state = self.state();
        let from = &state.resolve(from);
        let existing = state.resolve(to);
        state.change(from)?;
        state.check(&existing)?;
        state.node(from)?;
        if existing != *from {
            state.check_rename_target(from, &existing)?;
            state.live.names.remove(&existing);
        }
        // The name takes the case it is given, in a folder that keeps its own case.
        let parent = existing.parent().unwrap_or(Path::new("/"));
        let to = &parent.join(to.file_name().ok_or(ErrorKind::InvalidInput)?);
        let moved: Vec<PathBuf> = state
            .live
            .names
            .keys()
            .filter(|name| name.starts_with(from))
            .cloned()
            .collect();
        for old in moved {
            let suffix = old.strip_prefix(from).expect("filtered on this prefix");
            let new = if suffix.as_os_str().is_empty() {
                to.clone()
            } else {
                to.join(suffix)
            };
            let id = state.live.names.remove(&old).expect("listed above");
            state.live.names.insert(new, id);
        }
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let mut state = self.state();
        let path = &state.resolve(path);
        state.change(path)?;
        if state.node(path)?.kind == FileKind::Dir {
            return Err(ErrorKind::IsADirectory.into());
        }
        state.live.names.remove(path);
        Ok(())
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        let mut state = self.state();
        let path = &state.resolve(path);
        state.change(path)?;
        state.node(path)?;
        let id = state.live.names[path];
        state
            .live
            .nodes
            .get_mut(&id)
            .expect("names point to nodes")
            .mode = mode;
        Ok(())
    }

    fn free_space(&self, path: &Path) -> io::Result<u64> {
        let state = self.state();
        let path = &state.resolve(path);
        state.check(path)?;
        state.node(path)?;
        Ok(state.free())
    }

    fn sync_medium(&self, path: &Path) -> io::Result<()> {
        let mut state = self.state();
        let path = &state.resolve(path);
        state.change(path)?;
        state.node(path)?;
        state.flushed = state.live.clone();
        Ok(())
    }
}

impl FakeFs {
    fn file(&self, id: u64) -> FakeFile {
        FakeFile {
            fake: self.clone(),
            id,
        }
    }
}

/// Reading a folder fails, as it does on a real file system, but only once something is read.
struct DirReader;

impl Read for DirReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(ErrorKind::IsADirectory.into())
    }
}

struct FakeFile {
    fake: FakeFs,
    id: u64,
}

impl WriteFile for FakeFile {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        let mut state = self.fake.state();
        state.step()?;
        let free = state.free();
        let fits = data.len().min(usize::try_from(free).unwrap_or(usize::MAX));
        let node = state
            .live
            .nodes
            .get_mut(&self.id)
            .expect("files keep their node");
        node.data.extend_from_slice(&data[..fits]);
        node.modified = SystemTime::now();
        if fits < data.len() {
            return Err(ErrorKind::StorageFull.into());
        }
        Ok(())
    }

    fn set_modified(&mut self, time: SystemTime) -> io::Result<()> {
        let mut state = self.fake.state();
        state.step()?;
        let node = state
            .live
            .nodes
            .get_mut(&self.id)
            .expect("files keep their node");
        node.modified = time;
        Ok(())
    }

    fn sync(&mut self) -> io::Result<()> {
        let mut state = self.fake.state();
        state.step()?;
        let node = state.live.nodes[&self.id].clone();
        state.flushed.nodes.insert(self.id, node);
        if let Some(flag) = &state.raise_on_sync {
            flag.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_TIME: SystemTime = SystemTime::UNIX_EPOCH;

    fn read(fake: &FakeFs, path: &str) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        fake.open(Path::new(path))?.read_to_end(&mut data)?;
        Ok(data)
    }

    /// The steps Babawus uses to replace a file: a flushed temporary file, then a rename.
    fn replace_file(fake: &FakeFs) -> io::Result<()> {
        let mut file = fake.create_new(Path::new("/tmp/new"))?;
        file.write_all(b"new ")?;
        file.write_all(b"content")?;
        file.sync()?;
        drop(file);
        fake.rename(Path::new("/tmp/new"), Path::new("/file"))?;
        fake.sync_medium(Path::new("/"))
    }

    fn medium_with_old_file() -> FakeFs {
        let fake = FakeFs::new();
        fake.add_file("/file", b"old content", OLD_TIME);
        fake.add_dir_on_device("/tmp", DEFAULT_DEVICE);
        fake
    }

    #[test]
    fn counts_each_change_as_a_step() {
        let fake = medium_with_old_file();

        replace_file(&fake).unwrap();

        assert_eq!(fake.steps(), 6);
    }

    #[test]
    fn reading_is_not_a_step() {
        let fake = medium_with_old_file();

        fake.metadata(Path::new("/file")).unwrap();
        read(&fake, "/file").unwrap();

        assert_eq!(fake.steps(), 0);
    }

    #[test]
    fn the_step_that_pulls_the_medium_has_no_effect() {
        let fake = medium_with_old_file();
        fake.pull_at_step(5);

        assert!(replace_file(&fake).is_err());

        fake.reinsert();
        assert_eq!(read(&fake, "/file").unwrap(), b"old content");
        assert_eq!(read(&fake, "/tmp/new").unwrap(), b"new content");
    }

    #[test]
    fn everything_fails_while_the_medium_is_out() {
        let fake = medium_with_old_file();
        fake.pull_at_step(1);
        assert!(fake.create_dir(Path::new("/folder")).is_err());

        assert!(fake.metadata(Path::new("/file")).is_err());
        assert!(fake.read_dir(Path::new("/")).is_err());
        assert!(fake.open(Path::new("/file")).is_err());
        assert!(fake.free_space(Path::new("/")).is_err());
        assert!(fake.sync_medium(Path::new("/")).is_err());
    }

    #[test]
    fn an_open_file_fails_once_the_medium_is_out() {
        let fake = medium_with_old_file();
        let mut file = fake.create_new(Path::new("/tmp/new")).unwrap();
        fake.pull_at_step(2);

        assert!(file.write_all(b"data").is_err());
        assert!(file.sync().is_err());
    }

    #[test]
    fn a_file_replaced_safely_is_never_damaged_at_any_step() {
        let steps = {
            let fake = medium_with_old_file();
            replace_file(&fake).unwrap();
            fake.steps()
        };

        for step in 1..=steps {
            for lose_unflushed in [false, true] {
                let fake = medium_with_old_file();
                fake.pull_at_step(step);
                assert!(replace_file(&fake).is_err());

                if lose_unflushed {
                    fake.reinsert_losing_unflushed();
                } else {
                    fake.reinsert();
                }

                let content = read(&fake, "/file").unwrap();
                assert!(
                    content == b"old content" || content == b"new content",
                    "step {step}: {content:?}"
                );
            }
        }
    }

    #[test]
    fn unflushed_content_is_lost() {
        let fake = medium_with_old_file();
        let mut file = fake.open_append(Path::new("/file")).unwrap();
        file.write_all(b", added").unwrap();
        fake.pull_at_step(fake.steps() + 1);
        assert!(file.sync().is_err());

        fake.reinsert_losing_unflushed();

        assert_eq!(read(&fake, "/file").unwrap(), b"old content");
    }

    #[test]
    fn flushing_a_file_keeps_its_content_but_not_a_new_name() {
        let fake = medium_with_old_file();
        let mut flushed = fake.open_append(Path::new("/file")).unwrap();
        flushed.write_all(b", added").unwrap();
        flushed.sync().unwrap();
        let mut new = fake.create_new(Path::new("/new")).unwrap();
        new.sync().unwrap();

        fake.reinsert_losing_unflushed();

        assert_eq!(read(&fake, "/file").unwrap(), b"old content, added");
        assert_eq!(
            fake.metadata(Path::new("/new")).unwrap_err().kind(),
            ErrorKind::NotFound
        );
    }

    #[test]
    fn a_rename_needs_the_medium_flushed() {
        let fake = medium_with_old_file();
        fake.rename(Path::new("/file"), Path::new("/moved"))
            .unwrap();

        fake.reinsert_losing_unflushed();

        assert_eq!(read(&fake, "/file").unwrap(), b"old content");
        assert!(read(&fake, "/moved").is_err());
    }

    #[test]
    fn flushing_the_medium_keeps_everything() {
        let fake = medium_with_old_file();
        fake.rename(Path::new("/file"), Path::new("/moved"))
            .unwrap();
        fake.sync_medium(Path::new("/")).unwrap();

        fake.reinsert_losing_unflushed();

        assert_eq!(read(&fake, "/moved").unwrap(), b"old content");
    }

    #[test]
    fn a_full_medium_writes_what_fits_and_then_fails() {
        let fake = FakeFs::new();
        fake.add_file("/file", b"12345", OLD_TIME);
        fake.set_capacity(8);
        let mut file = fake.create_new(Path::new("/new")).unwrap();

        let error = file.write_all(b"abcdef").unwrap_err();

        assert_eq!(error.kind(), ErrorKind::StorageFull);
        assert_eq!(read(&fake, "/new").unwrap(), b"abc");
        assert_eq!(fake.free_space(Path::new("/")).unwrap(), 0);
    }

    #[test]
    fn removing_a_file_frees_its_space() {
        let fake = FakeFs::new();
        fake.add_file("/file", b"12345", OLD_TIME);
        fake.set_capacity(8);

        fake.remove_file(Path::new("/file")).unwrap();

        assert_eq!(fake.free_space(Path::new("/")).unwrap(), 8);
    }

    #[test]
    fn a_failing_path_reports_its_error() {
        let fake = medium_with_old_file();
        fake.fail_path("/file", ErrorKind::PermissionDenied);

        let error = read(&fake, "/file").unwrap_err();

        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn other_files_are_reported_as_other() {
        let fake = FakeFs::new();
        fake.add_other("/folder/link");

        let metadata = fake.metadata(Path::new("/folder/link")).unwrap();

        assert_eq!(metadata.kind, FileKind::Other);
    }

    #[test]
    fn a_folder_can_be_on_another_device() {
        let fake = FakeFs::new();
        fake.add_dir_on_device("/home/cloud", 2);

        let home = fake.metadata(Path::new("/home")).unwrap();
        let cloud = fake.metadata(Path::new("/home/cloud")).unwrap();

        assert_ne!(home.device, cloud.device);
    }

    #[test]
    fn a_renamed_folder_takes_its_content_along() {
        let fake = FakeFs::new();
        fake.add_file("/folder/sub/file", b"content", OLD_TIME);

        fake.rename(Path::new("/folder"), Path::new("/moved"))
            .unwrap();

        assert_eq!(read(&fake, "/moved/sub/file").unwrap(), b"content");
        assert!(read(&fake, "/folder/sub/file").is_err());
    }
}
