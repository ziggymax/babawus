//! Getting files back from a stick: the backup profile's files as a tree, picked with ticks as in
//! "Choose what to back up", each file in its newest copy or a picked older version, and copied
//! back to where they were in the home folder, or into another folder that the user chose
//! (issue 161).
//!
//! This only reads the stick. Everything it writes goes to the target folder.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::io::{self, ErrorKind, Read as _};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::SystemTime;

use crate::copy::CHUNK_SIZE;
use crate::fs::{FileKind, FileSystem};
use crate::time::ClockTime;
use crate::versions::{
    Format, file_of_version_folder, has_version_folder_form, older_versions, version_folder,
};

/// A backup profile's files on a stick, and the home folder they came from.
pub struct Backup<'a> {
    /// The stick's file system.
    pub medium: &'a dyn FileSystem,
    /// The profile's folder on the stick, `<stick>/backup/<computer>/<user>`.
    pub root: PathBuf,
    pub format: Format,
    /// This computer's file system.
    pub system: &'a dyn FileSystem,
    pub home: PathBuf,
}

/// A file or folder in the backup. Paths are relative to the profile's folder, which is the
/// home folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The name as shown. A name that isn't valid text is shown as well as it can be.
    pub name: String,
    pub relative: PathBuf,
    /// [`FileKind::Dir`] or [`FileKind::File`].
    pub kind: FileKind,
    /// Something of this name is in the home folder now.
    pub on_computer: bool,
    /// A file whose newest copy isn't on the stick, only older versions. Babawus never removes a
    /// newest copy, so this happens when a file became a folder, or was removed by hand.
    pub only_older: bool,
}

/// Which copy of a file to restore.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Copy {
    Newest,
    /// The name of an older version in the file's version folder.
    Older(OsString),
}

/// A copy of a file on the stick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub copy: Copy,
    /// When the newest copy was last changed, or when an older version was replaced.
    pub time: SystemTime,
    pub len: u64,
}

impl Backup<'_> {
    /// What is in the folder `relative` of the backup: folders first, then files, each by name
    /// without regard to case. Version folders show as their files' versions, not as folders.
    pub fn children(&self, relative: &Path) -> io::Result<Vec<Item>> {
        let names = self.medium.read_dir(&self.root.join(relative))?;
        let mut items = Vec::new();
        for name in &names {
            if has_version_folder_form(name) {
                // A version folder whose file is gone shows the file, with its older versions.
                let Some(file) = file_of_version_folder(name) else {
                    continue;
                };
                let taken = names.iter().any(|other| self.same_name(other, &file));
                if !taken && !self.versions(&relative.join(&file))?.is_empty() {
                    items.push(self.item(relative, &file, FileKind::File, true));
                }
                continue;
            }
            match self.medium.metadata(&self.root.join(relative).join(name)) {
                Ok(metadata) if metadata.kind != FileKind::Other => {
                    items.push(self.item(relative, name, metadata.kind, false));
                }
                Ok(_) => {}
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        items.sort_by_cached_key(|item| (item.kind != FileKind::Dir, item.name.to_lowercase()));
        items.dedup_by(|a, b| a.relative == b.relative);
        Ok(items)
    }

    fn same_name(&self, a: &OsStr, b: &OsStr) -> bool {
        match self.format {
            Format::Ext4 => a == b,
            Format::ExFat => {
                crate::names::fold(&a.to_string_lossy()) == crate::names::fold(&b.to_string_lossy())
            }
        }
    }

    fn item(&self, folder: &Path, name: &OsStr, kind: FileKind, only_older: bool) -> Item {
        let relative = folder.join(name);
        Item {
            name: name.to_string_lossy().into_owned(),
            on_computer: self.system.metadata(&self.home.join(&relative)).is_ok(),
            relative,
            kind,
            only_older,
        }
    }

    /// The copies of the file `relative` on the stick: its newest copy, if there is one, then
    /// its older versions, newest first.
    pub fn versions(&self, relative: &Path) -> io::Result<Vec<Version>> {
        let Some(name) = relative.file_name() else {
            return Ok(Vec::new());
        };
        let folder = self.root.join(relative.parent().unwrap_or(Path::new("")));
        let mut versions = Vec::new();
        match self.medium.metadata(&folder.join(name)) {
            Ok(metadata) if metadata.kind == FileKind::File => versions.push(Version {
                copy: Copy::Newest,
                time: metadata.modified,
                len: metadata.len,
            }),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let version_folder = folder.join(version_folder(name));
        let entries = match self.medium.read_dir(&version_folder) {
            Ok(entries) => entries,
            Err(error)
                if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
            {
                Vec::new()
            }
            Err(error) => return Err(error),
        };
        for (entry, time) in older_versions(&entries, name, self.format) {
            let Ok(metadata) = self.medium.metadata(&version_folder.join(&entry)) else {
                continue;
            };
            if metadata.kind == FileKind::File {
                versions.push(Version {
                    copy: Copy::Older(entry),
                    time,
                    len: metadata.len,
                });
            }
        }
        Ok(versions)
    }

    /// Where `copy` of the file `relative` is on the stick.
    fn source(&self, relative: &Path, copy: &Copy) -> PathBuf {
        let path = self.root.join(relative);
        match copy {
            Copy::Newest => path,
            Copy::Older(entry) => {
                let name = relative.file_name().unwrap_or_default();
                path.with_file_name(version_folder(name)).join(entry)
            }
        }
    }
}

/// How a row of the tree looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub ticked: bool,
    pub can_toggle: bool,
    /// A folder whose tick box is partly ticked: it is ticked with something inside it left
    /// out, or unticked with something inside it ticked.
    pub partly: bool,
    /// Left out by hand, so its name is crossed out.
    pub crossed: bool,
    /// Left out with a folder above it, so it can't be ticked.
    pub left_out_above: bool,
}

/// What is to be restored, while it is being picked. Paths are relative to the home folder,
/// and the empty path is the home folder itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    pub ticked: Vec<PathBuf>,
    pub left_out: Vec<PathBuf>,
    /// The older version picked for a file. A file without one gets its newest copy.
    pub picked: BTreeMap<PathBuf, OsString>,
}

impl Choice {
    fn is_ticked(&self, relative: &Path) -> bool {
        self.ticked.iter().any(|path| path == relative)
    }

    fn is_left_out(&self, relative: &Path) -> bool {
        self.left_out.iter().any(|path| path == relative)
    }

    fn ticked_above(&self, relative: &Path) -> bool {
        self.ticked
            .iter()
            .any(|path| path != relative && relative.starts_with(path))
    }

    fn left_out_above(&self, relative: &Path) -> bool {
        self.left_out
            .iter()
            .any(|path| path != relative && relative.starts_with(path))
    }

    fn has_inside(paths: &[PathBuf], relative: &Path) -> bool {
        paths
            .iter()
            .any(|path| path != relative && path.starts_with(relative))
    }

    /// How the row for `relative` looks. `is_dir` says whether it is a folder.
    pub fn row(&self, relative: &Path, is_dir: bool) -> Row {
        let row = |ticked, can_toggle, partly, crossed, left_out_above| Row {
            ticked,
            can_toggle,
            partly: is_dir && partly,
            crossed,
            left_out_above,
        };
        if self.is_left_out(relative) {
            return row(
                false,
                true,
                Self::has_inside(&self.ticked, relative),
                true,
                false,
            );
        }
        if self.left_out_above(relative) {
            return row(false, false, false, true, true);
        }
        if self.is_ticked(relative) || self.ticked_above(relative) {
            return row(
                true,
                true,
                Self::has_inside(&self.left_out, relative),
                false,
                false,
            );
        }
        row(
            false,
            true,
            Self::has_inside(&self.ticked, relative),
            false,
            false,
        )
    }

    /// Ticks or unticks `relative`, as in "Choose what to back up". Returns whether anything
    /// changed.
    pub fn toggle(&mut self, relative: &Path, is_dir: bool) -> bool {
        let row = self.row(relative, is_dir);
        if !row.can_toggle {
            return false;
        }
        if self.is_left_out(relative) {
            self.left_out.retain(|path| path != relative);
            return true;
        }
        let inherited = self.ticked_above(relative);
        if self.is_ticked(relative) {
            self.ticked.retain(|path| path != relative);
            if inherited {
                self.left_out.push(relative.to_owned());
            }
            return true;
        }
        if row.ticked {
            self.left_out.push(relative.to_owned());
        } else {
            self.ticked.push(relative.to_owned());
        }
        true
    }

    /// Picks `copy` of the file `relative`, and ticks it if it isn't. Returns whether it is now
    /// ticked.
    pub fn pick(&mut self, relative: &Path, copy: Copy) -> bool {
        match copy {
            Copy::Newest => {
                self.picked.remove(relative);
            }
            Copy::Older(entry) => {
                self.picked.insert(relative.to_owned(), entry);
            }
        }
        if !self.row(relative, false).ticked {
            self.toggle(relative, false);
        }
        self.row(relative, false).ticked
    }

    /// The copy of the file `relative` that a restore takes.
    pub fn copy_of(&self, relative: &Path) -> Copy {
        self.picked
            .get(relative)
            .map_or(Copy::Newest, |entry| Copy::Older(entry.clone()))
    }

    /// Whether anything is ticked.
    pub fn is_empty(&self) -> bool {
        self.ticked.is_empty()
    }
}

/// What is in the way of a restored file in the home folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Existing {
    /// A file of that name: the user decides.
    File,
    /// A folder, link or special file of that name: the file can't go there.
    Other,
}

/// A file to restore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub relative: PathBuf,
    pub copy: Copy,
    /// When this copy was last changed, or replaced, for "Keep both".
    pub time: SystemTime,
    pub len: u64,
    pub existing: Option<Existing>,
}

/// Everything a restore will do, worked out before anything is copied.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// Where the files go: the home folder, or another folder that the user chose, below which
    /// they keep their folders.
    pub target: PathBuf,
    /// Folders to create if they are missing, parents first.
    pub folders: Vec<PathBuf>,
    pub files: Vec<Planned>,
    /// Files and folders on the stick that couldn't be read, so they can't be restored.
    pub unreadable: Vec<PathBuf>,
}

impl Plan {
    /// How many files are already in the target folder, which the user is asked about.
    pub fn already_there(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.existing == Some(Existing::File))
            .count()
    }

    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|file| file.len).sum()
    }
}

impl Backup<'_> {
    /// The top folder of the stick, which holds `backup`.
    fn stick(&self) -> &Path {
        self.root.ancestors().nth(3).unwrap_or(&self.root)
    }

    /// Whether files may be restored into the folder `target`: an absolute path that isn't on the
    /// stick, because restoring never writes to the stick.
    pub fn may_restore_to(&self, target: &Path) -> bool {
        target.is_absolute() && !target.starts_with(self.stick())
    }

    /// A new folder in the home folder to suggest for restoring into: `<name> 2026-11-17 15.02`,
    /// or with `, 2` and so on if that is taken. `name` is in the user's language, for example
    /// "Restored from BACKUP-A".
    pub fn suggested_folder(&self, name: &str, now: ClockTime) -> PathBuf {
        let stamp = format!(
            "{name} {:04}-{:02}-{:02} {:02}.{:02}",
            now.year, now.month, now.day, now.hour, now.minute
        );
        (1..)
            .map(|number| match number {
                1 => self.home.join(&stamp),
                _ => self.home.join(format!("{stamp}, {number}")),
            })
            .find(|path| self.system.metadata(path).is_err())
            .expect("some number is free")
    }

    /// Works out what restoring `choice` into the folder `target` does: the home folder, or
    /// another one. Something that can't be read on the stick is noted, and the rest goes on.
    /// Returns `None` if `cancel` was set meanwhile.
    pub fn plan(&self, choice: &Choice, target: &Path, cancel: &AtomicBool) -> Option<Plan> {
        let mut plan = Plan {
            target: target.to_owned(),
            ..Plan::default()
        };
        let mut roots = choice.ticked.clone();
        roots.sort();
        roots.dedup();
        for root in &roots {
            if choice.left_out_above(root) || choice.is_left_out(root) {
                continue;
            }
            // Reached already from a ticked folder above it.
            if roots
                .iter()
                .any(|above| above != root && root.starts_with(above))
            {
                continue;
            }
            self.plan_path(choice, root, &mut plan, cancel)?;
        }
        Some(plan)
    }

    fn plan_path(
        &self,
        choice: &Choice,
        relative: &Path,
        plan: &mut Plan,
        cancel: &AtomicBool,
    ) -> Option<()> {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        let kind = match self.medium.metadata(&self.root.join(relative)) {
            Ok(metadata) => metadata.kind,
            // A file with only older versions.
            Err(error) if error.kind() == ErrorKind::NotFound => FileKind::File,
            Err(_) => {
                plan.unreadable.push(relative.to_owned());
                return Some(());
            }
        };
        match kind {
            FileKind::Dir => {
                if !relative.as_os_str().is_empty() {
                    plan.folders.push(relative.to_owned());
                }
                let Ok(items) = self.children(relative) else {
                    plan.unreadable.push(relative.to_owned());
                    return Some(());
                };
                for item in items {
                    if !choice.is_left_out(&item.relative) {
                        self.plan_path(choice, &item.relative, plan, cancel)?;
                    }
                }
            }
            FileKind::File => {
                let Ok(versions) = self.versions(relative) else {
                    plan.unreadable.push(relative.to_owned());
                    return Some(());
                };
                let wanted = choice.copy_of(relative);
                // A picked version that is gone falls back to the newest copy there is.
                let version = versions
                    .iter()
                    .find(|version| version.copy == wanted)
                    .or(versions.first());
                if let Some(version) = version {
                    let existing =
                        self.system
                            .metadata(&plan.target.join(relative))
                            .ok()
                            .map(|metadata| match metadata.kind {
                                FileKind::File => Existing::File,
                                _ => Existing::Other,
                            });
                    plan.files.push(Planned {
                        relative: relative.to_owned(),
                        copy: version.copy.clone(),
                        time: version.time,
                        len: version.len,
                        existing,
                    });
                }
            }
            FileKind::Other => {}
        }
        Some(())
    }
}

/// What to do with files that are already in the target folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Replace,
    /// Restore next to the file, with the copy's date and time in the name.
    KeepBoth,
    Skip,
}

/// Why a file couldn't be restored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// A folder, link or special file has its name, or a file has the name of a folder above it.
    InTheWay,
    /// It couldn't be read from the stick, for example because the stick was pulled out.
    Unreadable,
    /// It couldn't be written to the target folder.
    NotWritable,
    /// The target folder is on the stick, which restoring never writes to.
    OnTheStick,
}

/// How a restore went.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub restored: u64,
    pub bytes: u64,
    /// Files left as they were, because the user chose to skip them.
    pub skipped: u64,
    pub failed: Vec<(PathBuf, Failure)>,
    /// The target folder's disk is full: the restore stopped there.
    pub full: bool,
    pub cancelled: bool,
}

/// The name for a restored file kept next to `name`: `report (2026-11-03 17.54).pdf`, with
/// `number` 2 `report (2026-11-03 17.54, 2).pdf`. The date goes before the extension, which is
/// split at the last dot, as for older versions.
pub fn kept_name(name: &OsStr, time: ClockTime, number: u32) -> OsString {
    let bytes = name.as_bytes();
    let (base, extension) = match bytes.iter().rposition(|&byte| byte == b'.') {
        Some(dot) if dot > 0 => (&bytes[..dot], Some(&bytes[dot..])),
        _ => (bytes, None),
    };
    let mut stamp = format!(
        " ({:04}-{:02}-{:02} {:02}.{:02}",
        time.year, time.month, time.day, time.hour, time.minute
    );
    if number > 1 {
        stamp.push_str(&format!(", {number}"));
    }
    stamp.push(')');
    let mut kept = base.to_vec();
    kept.extend_from_slice(stamp.as_bytes());
    kept.extend_from_slice(extension.unwrap_or_default());
    OsString::from_vec(kept)
}

/// Counts what has been restored so far, for the progress shown.
#[derive(Debug, Default)]
pub struct Progress {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
}

impl Backup<'_> {
    /// Restores everything in `plan`, deciding about files already in place by `resolution`.
    /// `clock` gives the local time for the names of kept files. Stops at once if `cancel` is
    /// set, or if the home folder is full.
    pub fn restore(
        &self,
        plan: &Plan,
        resolution: Resolution,
        clock: &dyn Fn(SystemTime) -> ClockTime,
        progress: &Progress,
        cancel: &AtomicBool,
    ) -> Outcome {
        let mut outcome = Outcome::default();
        for relative in &plan.unreadable {
            outcome.failed.push((relative.clone(), Failure::Unreadable));
        }
        let ready = if self.may_restore_to(&plan.target) {
            self.make_target(&plan.target)
        } else {
            Err(Failure::OnTheStick)
        };
        if let Err(failure) = ready {
            for file in &plan.files {
                outcome.failed.push((file.relative.clone(), failure));
            }
            return outcome;
        }
        for folder in &plan.folders {
            // A folder that can't be made shows up as a failure of the files in it.
            let _ = self.make_folder(&plan.target, folder);
        }
        self.remove_left_overs(plan);
        let mut temp_number = 0;
        for file in &plan.files {
            if cancel.load(Ordering::Relaxed) {
                outcome.cancelled = true;
                break;
            }
            let target = plan.target.join(&file.relative);
            let name = target.file_name().unwrap_or_default().to_owned();
            let existing = self
                .system
                .metadata(&target)
                .ok()
                .map(|metadata| metadata.kind);
            let target = match (existing, resolution) {
                (None, _) => target,
                (Some(FileKind::File), Resolution::Skip) => {
                    outcome.skipped += 1;
                    continue;
                }
                (Some(FileKind::File), Resolution::Replace) => target,
                (Some(_), Resolution::KeepBoth) => self.free_name(&target, &name, clock(file.time)),
                (Some(_), _) => {
                    outcome
                        .failed
                        .push((file.relative.clone(), Failure::InTheWay));
                    continue;
                }
            };
            if let Some(parent) = file.relative.parent()
                && let Err(failure) = self.make_folder(&plan.target, parent)
            {
                outcome.failed.push((file.relative.clone(), failure));
                continue;
            }
            temp_number += 1;
            match self.copy_back(file, &target, temp_number, progress, cancel) {
                Ok(()) => {
                    outcome.restored += 1;
                    outcome.bytes += file.len;
                    progress.files.fetch_add(1, Ordering::Relaxed);
                }
                Err(CopyError::Cancelled) => {
                    outcome.cancelled = true;
                    break;
                }
                Err(CopyError::Full) => {
                    outcome.full = true;
                    outcome
                        .failed
                        .push((file.relative.clone(), Failure::NotWritable));
                    break;
                }
                Err(CopyError::Failed(failure)) => {
                    outcome.failed.push((file.relative.clone(), failure));
                }
            }
        }
        outcome
    }

    /// Removes the temporary files that an earlier restore left in the folders this one writes
    /// to, because it was stopped in the middle of a file by a crash or a power cut. Only one
    /// Babawus runs on a computer, so a temporary file of another process is left over. One that
    /// can't be removed is left, and is tried again next time.
    fn remove_left_overs(&self, plan: &Plan) {
        let mut folders: BTreeSet<PathBuf> = BTreeSet::new();
        folders.insert(plan.target.clone());
        folders.extend(plan.folders.iter().map(|folder| plan.target.join(folder)));
        folders.extend(
            plan.files
                .iter()
                .filter_map(|file| file.relative.parent())
                .map(|parent| plan.target.join(parent)),
        );
        let own = std::process::id();
        for folder in folders {
            let Ok(names) = self.system.read_dir(&folder) else {
                continue;
            };
            for name in names.iter().filter(|name| is_left_over(name, own)) {
                let path = folder.join(name);
                if self
                    .system
                    .metadata(&path)
                    .is_ok_and(|metadata| metadata.kind == FileKind::File)
                {
                    let _ = self.system.remove_file(&path);
                }
            }
        }
    }

    /// Makes the target folder, and any folders above it that are missing.
    fn make_target(&self, target: &Path) -> Result<(), Failure> {
        let mut missing = Vec::new();
        for path in target.ancestors() {
            match self.system.metadata(path) {
                Ok(metadata) if metadata.kind == FileKind::Dir => break,
                Ok(_) => return Err(Failure::InTheWay),
                Err(error) if error.kind() == ErrorKind::NotFound => missing.push(path),
                Err(_) => return Err(Failure::NotWritable),
            }
        }
        for path in missing.into_iter().rev() {
            match self.system.create_dir(path) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(_) => return Err(Failure::NotWritable),
            }
        }
        Ok(())
    }

    /// Makes the folder `relative` in `base`, and the folders above it.
    fn make_folder(&self, base: &Path, relative: &Path) -> Result<(), Failure> {
        let mut path = base.to_owned();
        for part in relative.components() {
            path.push(part);
            match self.system.metadata(&path) {
                Ok(metadata) if metadata.kind == FileKind::Dir => continue,
                Ok(_) => return Err(Failure::InTheWay),
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(_) => return Err(Failure::NotWritable),
            }
            match self.system.create_dir(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(_) => return Err(Failure::NotWritable),
            }
        }
        Ok(())
    }

    /// The first free name for a kept file next to `target`.
    fn free_name(&self, target: &Path, name: &OsStr, time: ClockTime) -> PathBuf {
        (1..)
            .map(|number| target.with_file_name(kept_name(name, time, number)))
            .find(|path| self.system.metadata(path).is_err())
            .expect("some number is free")
    }

    /// Copies `file` to a temporary name next to `target`, and renames it to `target` once it
    /// is all there, keeping its modification time.
    fn copy_back(
        &self,
        file: &Planned,
        target: &Path,
        temp_number: u32,
        progress: &Progress,
        cancel: &AtomicBool,
    ) -> Result<(), CopyError> {
        let source = self.source(&file.relative, &file.copy);
        let modified = self
            .medium
            .metadata(&source)
            .map_err(|_| CopyError::Failed(Failure::Unreadable))?
            .modified;
        let mut reader = self
            .medium
            .open(&source)
            .map_err(|_| CopyError::Failed(Failure::Unreadable))?;
        let temp = target.with_file_name(temp_name(std::process::id(), temp_number));
        let result = (|| {
            let mut writer = self.system.create_new(&temp).map_err(CopyError::writing)?;
            let mut chunk = vec![0; CHUNK_SIZE];
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Err(CopyError::Cancelled);
                }
                let read = reader
                    .read(&mut chunk)
                    .map_err(|_| CopyError::Failed(Failure::Unreadable))?;
                if read == 0 {
                    break;
                }
                writer
                    .write_all(&chunk[..read])
                    .map_err(CopyError::writing)?;
                progress.bytes.fetch_add(read as u64, Ordering::Relaxed);
            }
            writer.set_modified(modified).map_err(CopyError::writing)?;
            writer.sync().map_err(CopyError::writing)?;
            drop(writer);
            self.system
                .rename(&temp, target)
                .map_err(CopyError::writing)
        })();
        if result.is_err() {
            let _ = self.system.remove_file(&temp);
        }
        result
    }
}

/// The start of the name of a file being restored, before it is renamed.
const TEMP_PREFIX: &str = ".babawus-restore-";

/// The temporary name of the `number`th file restored by the process `process`:
/// `.babawus-restore-12345-3`.
fn temp_name(process: u32, number: u32) -> String {
    format!("{TEMP_PREFIX}{process}-{number}")
}

/// Whether `name` is the temporary name of a file that another process than `own` was
/// restoring, so that it was left over.
pub fn is_left_over(name: &OsStr, own: u32) -> bool {
    let Some(rest) = name
        .to_str()
        .and_then(|name| name.strip_prefix(TEMP_PREFIX))
    else {
        return false;
    };
    let Some((process, number)) = rest.split_once('-') else {
        return false;
    };
    let digits = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    digits(process) && digits(number) && process.parse::<u32>().is_ok_and(|process| process != own)
}

enum CopyError {
    Cancelled,
    /// The home folder is full.
    Full,
    Failed(Failure),
}

impl CopyError {
    fn writing(error: io::Error) -> Self {
        if error.kind() == ErrorKind::StorageFull {
            Self::Full
        } else {
            Self::Failed(Failure::NotWritable)
        }
    }
}

#[cfg(test)]
mod tests;
