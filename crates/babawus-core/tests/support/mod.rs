//! Real file systems on loop devices, for integration tests.
//!
//! This is the only place in the tests that runs `mkfs`. Automated tests must never be able to
//! format anything except a loop device they created themselves, so [`LoopMedium::new`] always
//! creates its own device, and [`check_device`] must pass before anything is formatted.
//!
//! The image is blank when it is attached, and is attached with `losetup` rather than udisks2.
//! Cinnamon automounts a loop device set up through udisks2, which would interfere with the test.
//!
//! Everything lives in the system's temporary folder, not below `target`. The desktop treats a
//! mount below the home folder as a user's volume and opens a file manager window on it, which
//! can also keep the mount busy.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

const LABEL: &str = "BACKUP-A";

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, Debug)]
pub enum MediumFormat {
    ExFat,
    Ext4,
}

/// A mounted file system in an image file. Dropping it unmounts and removes everything.
pub struct LoopMedium {
    dir: PathBuf,
    mount_point: PathBuf,
    device: String,
    mounted: bool,
}

impl LoopMedium {
    /// Creates an image of `size` bytes, formats it as `format` and mounts it so that the
    /// current user can write to it.
    pub fn new(format: MediumFormat, size: u64) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let name = format!("babawus-medium-{}-{id}", std::process::id());
        let dir = std::env::temp_dir().join(name);
        let mount_point = dir.join("mnt");
        fs::create_dir_all(&mount_point).expect("create the medium folder");

        let image = dir.join("medium.img");
        File::create(&image)
            .and_then(|file| file.set_len(size))
            .expect("create the image file");
        let image = image.canonicalize().expect("resolve the image path");

        let device = sudo_output(&["losetup", "--find", "--show", path_str(&image)]);
        // From here on, Drop cleans up, also if the test panics.
        let mut medium = Self {
            dir,
            mount_point,
            device,
            mounted: false,
        };

        let backing_file = sudo_output(&[
            "losetup",
            "--noheadings",
            "--output",
            "BACK-FILE",
            &medium.device,
        ]);
        check_device(&medium.device, &backing_file, &image).expect("refusing to format");

        medium.format_and_mount(format);
        medium
    }

    /// The root folder of the mounted medium.
    pub fn path(&self) -> &Path {
        &self.mount_point
    }

    /// Mounts the file system again, read-only.
    #[allow(dead_code)] // Not every test file uses it.
    pub fn remount_read_only(&self) {
        sudo(&["mount", "-o", "remount,ro", path_str(&self.mount_point)]);
    }

    fn format_and_mount(&mut self, format: MediumFormat) {
        let device = self.device.clone();
        let mount_point = path_str(&self.mount_point).to_owned();
        let owner = format!(
            "{}:{}",
            command_output("id", &["-u"]),
            command_output("id", &["-g"])
        );

        match format {
            MediumFormat::ExFat => {
                sudo(&["mkfs.exfat", "-L", LABEL, &device]);
                let (uid, gid) = owner.split_once(':').expect("owner is uid:gid");
                let options = format!("uid={uid},gid={gid}");
                sudo(&["mount", "-o", &options, &device, &mount_point]);
                self.mounted = true;
            }
            MediumFormat::Ext4 => {
                sudo(&[
                    "mkfs.ext4",
                    "-q",
                    "-L",
                    LABEL,
                    "-m",
                    "0",
                    "-E",
                    "lazy_itable_init=0,lazy_journal_init=0",
                    &device,
                ]);
                sudo(&["mount", &device, &mount_point]);
                self.mounted = true;
                sudo(&["chown", &owner, &mount_point]);
                sudo(&["chmod", "0777", &mount_point]);
            }
        }
    }
}

impl Drop for LoopMedium {
    fn drop(&mut self) {
        let mount_point = path_str(&self.mount_point).to_owned();
        if self.mounted {
            report_failure(
                "umount",
                Command::new("sudo").args(["-n", "umount", "--lazy", &mount_point]),
            );
        }
        let device = self.device.clone();
        report_failure(
            "losetup",
            Command::new("sudo").args(["-n", "losetup", "-d", &device]),
        );
        if let Err(error) = fs::remove_dir_all(&self.dir) {
            eprintln!("could not remove {}: {error}", self.dir.display());
        }
    }
}

/// Checks that `device` is a loop device backed by `image`, as reported by `losetup`.
pub fn check_device(device: &str, backing_file: &str, image: &Path) -> Result<(), String> {
    let number = device.strip_prefix("/dev/loop").unwrap_or_default();
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{device} is not a loop device"));
    }
    if Path::new(backing_file) != image {
        return Err(format!(
            "{device} is backed by {backing_file}, not by {}",
            image.display()
        ));
    }
    Ok(())
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("test paths are valid UTF-8")
}

fn sudo(args: &[&str]) {
    sudo_output(args);
}

fn sudo_output(args: &[&str]) -> String {
    let mut sudo_args = vec!["-n"];
    sudo_args.extend_from_slice(args);
    command_output("sudo", &sudo_args)
}

fn command_output(program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("could not run {program}: {error}"));
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn report_failure(name: &str, command: &mut Command) {
    match command.output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => eprintln!("{name} failed: {}", String::from_utf8_lossy(&output.stderr)),
        Err(error) => eprintln!("could not run {name}: {error}"),
    }
}
