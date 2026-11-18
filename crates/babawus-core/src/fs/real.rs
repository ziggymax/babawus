use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::SystemTime;

use super::{FileKind, FileSystem, Metadata, WriteFile};
use crate::platform;

/// The real file systems of the computer. What depends on the operating system is in the
/// platform layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct RealFs;

impl FileSystem for RealFs {
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let metadata = fs::symlink_metadata(path)?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_file() {
            FileKind::File
        } else if file_type.is_dir() {
            FileKind::Dir
        } else {
            FileKind::Other
        };
        Ok(Metadata {
            kind,
            len: metadata.len(),
            modified: metadata.modified()?,
            mode: platform::mode(&metadata),
            device: platform::device(&metadata),
        })
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        let mut names = fs::read_dir(path)?
            .map(|entry| Ok(entry?.file_name()))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort_by(|a, b| a.as_encoded_bytes().cmp(b.as_encoded_bytes()));
        Ok(names)
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read + Send>> {
        Ok(Box::new(File::open(path)?))
    }

    fn create_new(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Box::new(RealFile(file)))
    }

    fn open_append(&self, path: &Path) -> io::Result<Box<dyn WriteFile>> {
        let file = OpenOptions::new().append(true).create(true).open(path)?;
        Ok(Box::new(RealFile(file)))
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        fs::create_dir(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        platform::set_mode(path, mode)
    }

    fn free_space(&self, path: &Path) -> io::Result<u64> {
        platform::free_space(path)
    }

    fn sync_medium(&self, path: &Path) -> io::Result<()> {
        platform::flush_file_system(path)
    }
}

struct RealFile(File);

impl WriteFile for RealFile {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write_all(data)
    }

    fn set_modified(&mut self, time: SystemTime) -> io::Result<()> {
        self.0.set_modified(time)
    }

    fn sync(&mut self) -> io::Result<()> {
        self.0.sync_all()
    }
}
