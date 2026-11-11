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
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

const LABEL: &str = "BACKUP-A";

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

static SETTING_UP: Mutex<()> = Mutex::new(());

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
    pub fn remount_read_only(&self) {
        sudo(&["mount", "-o", "remount,ro", path_str(&self.mount_point)]);
    }

    fn format_and_mount(&mut self, format: MediumFormat) {
        let device = self.device.clone();
        self.format(format);
        // udev looks at a device again when whoever wrote to it closes it, and udisks2 reports
        // what udev found. On a busy machine that look can come too early, or be lost, and
        // udisks2 then reports no label and no file system for good. So ask udev to look again,
        // now that the file system is there, and wait until it has (T-044).
        sudo(&["udevadm", "trigger", "--action=change", "--settle", &device]);
        self.mount(format);
    }

    fn format(&self, format: MediumFormat) {
        let device = self.device.clone();
        match format {
            MediumFormat::ExFat => sudo(&["mkfs.exfat", "-L", LABEL, &device]),
            MediumFormat::Ext4 => sudo(&[
                "mkfs.ext4",
                "-q",
                "-L",
                LABEL,
                "-m",
                "0",
                "-E",
                "lazy_itable_init=0,lazy_journal_init=0",
                &device,
            ]),
        }
    }

    fn mount(&mut self, format: MediumFormat) {
        let device = self.device.clone();
        let mount_point = path_str(&self.mount_point).to_owned();
        let owner = format!(
            "{}:{}",
            command_output("id", &["-u"]),
            command_output("id", &["-g"])
        );

        match format {
            MediumFormat::ExFat => {
                let (uid, gid) = owner.split_once(':').expect("owner is uid:gid");
                let options = format!("uid={uid},gid={gid}");
                sudo(&["mount", "-o", &options, &device, &mount_point]);
                self.mounted = true;
            }
            MediumFormat::Ext4 => {
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

/// A blank image attached as a loop device through udisks2, for tests that format it through
/// udisks2. udisks2 lets the user who set up a loop device change it without asking, and scans
/// it for partitions. A blank image has no file system, so the desktop doesn't mount it.
/// Dropping it unmounts and removes everything.
pub struct BlankLoop {
    dir: PathBuf,
    device: String,
}

impl BlankLoop {
    pub fn new(size: u64) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let name = format!("babawus-medium-{}-blank-{id}", std::process::id());
        let dir = std::env::temp_dir().join(name);
        fs::create_dir_all(&dir).expect("create the medium folder");
        let image = dir.join("medium.img");
        File::create(&image)
            .and_then(|file| file.set_len(size))
            .expect("create the image file");
        let image = image.canonicalize().expect("resolve the image path");

        // udisks2 picks a free loop device, and two set-ups at once can pick the same one.
        let setting_up = SETTING_UP.lock().unwrap_or_else(|error| error.into_inner());
        let output = command_output(
            "udisksctl",
            &[
                "loop-setup",
                "--no-user-interaction",
                "--file",
                path_str(&image),
            ],
        );
        // "Mapped file /tmp/.../medium.img as /dev/loop8."
        let device = output
            .rsplit(' ')
            .next()
            .map(|device| device.trim_end_matches('.').to_owned())
            .unwrap_or_default();
        drop(setting_up);
        let blank = Self { dir, device };
        let backing_file = command_output(
            "losetup",
            &["--noheadings", "--output", "BACK-FILE", &blank.device],
        );
        check_device(&blank.device, &backing_file, &image).expect("refusing to format");
        blank
    }

    /// Keeps the loop device attached when its file system is unmounted. udisks2 sets the
    /// device to detach itself when it mounts it for the user who set it up.
    pub fn keep_attached(&self) {
        command_output(
            "gdbus",
            &[
                "call",
                "--system",
                "--dest",
                "org.freedesktop.UDisks2",
                "--object-path",
                &self.object_path(),
                "--method",
                "org.freedesktop.UDisks2.Loop.SetAutoclear",
                "false",
                "{}",
            ],
        );
    }

    /// For example `/dev/loop8`.
    pub fn device(&self) -> &str {
        &self.device
    }

    /// The udisks2 object path of the whole device.
    pub fn object_path(&self) -> String {
        let name = self.device.trim_start_matches("/dev/");
        format!("/org/freedesktop/UDisks2/block_devices/{name}")
    }
}

impl Drop for BlankLoop {
    fn drop(&mut self) {
        // Whatever formatting left mounted.
        if let Ok(mounts) = Command::new("findmnt")
            .args(["-rn", "-o", "SOURCE,TARGET"])
            .output()
        {
            for line in String::from_utf8_lossy(&mounts.stdout).lines() {
                let mut fields = line.split(' ');
                let (Some(source), Some(target)) = (fields.next(), fields.next()) else {
                    continue;
                };
                if source.starts_with(&format!("{}p", self.device)) {
                    report_failure(
                        "umount",
                        Command::new("sudo").args(["-n", "umount", "--lazy", target]),
                    );
                }
            }
        }
        report_failure(
            "losetup",
            Command::new("sudo").args(["-n", "losetup", "-d", &self.device]),
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
