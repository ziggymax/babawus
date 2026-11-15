# Babawus development environment setup

This guide explains how to rebuild the Babawus development and test VM from nothing, and records what was learned while setting it up. It describes the VM as it was on 2026-11-05.

The steps are split between **You** (things that need the VirtualBox host, a password, an account or a physical USB stick) and **Claude** (everything that can be done from a session on the VM). Claude can't see the host, so the host settings below are recommendations or were worked out from inside the VM.

## The VM

| Item | Value |
| --- | --- |
| Host | `bigdog`, which also runs the Claude desktop app |
| Hypervisor | VirtualBox |
| Guest name | `steen-VirtualBox` |
| Operating system | Linux Mint 22.3 Cinnamon on X11, the supported platform |
| CPUs and memory | 16 CPUs, about 20 GB |
| Disk | 25 GB, of which 24 GB is the root partition |
| Firmware and graphics | EFI, VMSVGA |
| Network | NAT (guest address `10.0.2.15`) |
| USB | USB 3 (xHCI) |
| Claude access | Over SSH from the Claude desktop app on the host |

## Overview

| Step | Who | What |
| --- | --- | --- |
| 1 | You | Create the VM and install Mint |
| 2 | You | Enable SSH and passwordless `sudo` |
| 3 | You | Connect Claude to the VM |
| 4 | You | Sign in to GitHub |
| 5 | Claude | Install packages and updates |
| 6 | Claude | Install Rust and `cargo-deb` |
| 7 | Claude | Configure Git and clone the repository |
| 8 | Claude | Verify the setup |
| 9 | You | Take a snapshot |
| 10 | You | Set up USB for manual testing |

## Step by step

### 1. Create the VM and install Mint (You)

1. Create a VirtualBox VM with EFI enabled, VMSVGA graphics, and NAT networking.
2. Give it at least the CPUs and memory listed above. Use a disk of 40 GB or more: 25 GB works, but only 7.5 GB was left after the first builds.
3. Enable USB 3 (xHCI) now, so that step 10 is simpler.
4. Install Linux Mint 22.x Cinnamon, create your user account and install the updates that are offered.
5. Add a NAT port forward to guest port 22, so that the host can reach the VM over SSH.

### 2. Enable SSH and passwordless sudo (You)

Run these in a terminal on the VM. The first command installs the SSH server:

```bash
sudo apt install -y openssh-server
```

The integration tests and Claude's sessions can't type a password, so `sudo` must work without one:

```bash
echo "$USER ALL=(ALL) NOPASSWD:ALL" | sudo tee /etc/sudoers.d/babawus-dev
```

```bash
sudo chmod 440 /etc/sudoers.d/babawus-dev
```

Then add the host's public key to `~/.ssh/authorized_keys` on the VM, and check that you can log in from the host without a password. Only do this on a VM that holds nothing important: see "Security notes".

### 3. Connect Claude to the VM (You)

Add the VM as an SSH connection in the Claude desktop app on the host, and start a session on it. Claude Code itself is managed by the app, so `claude` isn't on the `PATH` on the VM.

### 4. Sign in to GitHub (You)

Run this on the VM, and sign in as `ziggymax`:

```bash
sudo apt install -y git gh
```

```bash
gh auth login
```

Use a fine-grained token limited to this repository, with read and write access to contents, pull requests and workflows, and read access to Actions and commit statuses. Without workflows access, GitHub rejects pushes that change `.github/workflows/`.

### 5. Install packages and updates (Claude)

```bash
sudo apt-get update
```

```bash
sudo apt-get install -y build-essential pkg-config libgtk-4-dev curl exfatprogs e2fsprogs acl udisks2 xdotool scrot
```

```bash
sudo apt-get dist-upgrade -y
```

`xdotool` and `scrot` are for driving and capturing the GUI from a session. Everything else is needed to build or test Babawus. `xvfb` isn't needed, because GUI testing is manual. `lintian` is optional, for extra checks on the `.deb` package.

### 6. Install Rust and cargo-deb (Claude)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

```bash
"$HOME/.cargo/bin/cargo" install cargo-deb
```

The installer adds Cargo to `~/.bashrc` and `~/.profile`, so new shells find it. The stable toolchain already includes `clippy` and `rustfmt`.

### 7. Configure Git and clone the repository (Claude)

```bash
git config --global user.name 'Steen K'
```

```bash
git config --global user.email 'steen@kroeyer.net'
```

```bash
gh repo clone ziggymax/babawus ~/Projects/babawus
```

### 8. Verify the setup (Claude)

Run these from `~/Projects/babawus`. All of them should succeed:

```bash
cargo fmt --check
```

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
cargo test -- --include-ignored
```

```bash
cargo deb -p babawus
```

Then check that nothing was left behind. This should print nothing:

```bash
losetup -a; findmnt -rn -o TARGET | grep -E 'medium-|/media/'
```

Finally, check the GUI. Sessions run over SSH, so they have to name the display:

```bash
DISPLAY=:0 XAUTHORITY=$HOME/.Xauthority cargo run -p babawus
```

A window titled "Backup" should appear on the VM's desktop. Close it to finish the command.

### 9. Take a snapshot (You)

Take a VirtualBox snapshot of the working VM, and take a new one before testing formatting on real devices.

### 10. Set up USB for manual testing (You)

The manual checklist needs a real stick. A 16 GB Kingston DataTraveler 3.0 passed through this way ran at USB 3 speed (5000M in `lsusb -t`).

1. If you skipped it in step 1, shut the VM down and enable USB 3 (xHCI). Without it, sticks run at USB 2.0 speed at best, and measurements of `fsync` won't reflect real use.
2. Add a USB filter for a stick that you are happy to wipe.
3. Attach the stick and tell Claude, who can check that it appears as removable with `lsblk -o NAME,SIZE,TRAN,RM`.

## Discoveries

### Cinnamon and loop devices

- **Automounting:** an image that already held a file system was mounted under `/media/steen/` as soon as `udisksctl loop-setup` attached it. Scripts that mount it themselves then fail with `AlreadyMounted`.
- **What works:** attaching a blank image with `sudo losetup`, then formatting and mounting it. This doesn't trigger the automount, and it is what the test helper does. Re-attaching an image that already has a file system this way hasn't been tested.
- **Open windows:** the desktop treats anything mounted below the home folder as a user's volume. With `org.cinnamon.desktop.media-handling automount-open` on, a mount below `target/` opened a file manager window, which sometimes kept the mount busy so that `umount` failed and the loop device stayed behind. This happened at random, also in passing runs. So the test helper mounts in the system's temporary folder, and unmounts with `--lazy`. After that change, 26 runs opened no windows and left nothing behind.
- **Detaching:** after `udisksctl unmount`, a loop device set up through udisks2 can detach by itself. A following `udisksctl loop-delete` then fails with `No such device or address`, which is harmless.
- **Leaks:** if a script stops before detaching, the loop device stays attached even after its image is deleted. `losetup -a` shows these with `(deleted)`.

### File systems

- **Ext4 ACLs:** `/etc/mke2fs.conf` sets `default_mntopts = acl,user_xattr`, and a new file system reports `acl` as a default mount option. A format through udisks2 on a real stick confirmed this, so the `tune2fs -o acl` step and its `pkexec` fallback aren't needed on Mint 22.
- **Ownership comes first:** `setfacl` on the root folder of a new Ext4 file system fails with `Operation not permitted` while root owns it, and works after a change of owner. This matches the order in Requirements.md, where `take-ownership` comes first.
- **Full media:** tiny images really do fill up, and writes fail with `StorageFull` on both exFAT and Ext4.
- **exFAT names:** a trailing dot is silently dropped, and a trailing space is kept. Only control characters below `0x20` are refused.
- **exFAT case:** comparing names uses a table stored on the medium, not Unicode. Of 1,052 related pairs tested, 275 did not count as the same (for example `ı` and `I`), and 27 Greek pairs count as the same although simple upper-casing keeps them apart. `tests/exfat_case.rs` reads the table that `mkfs.exfat` writes, and checks all 65,536 characters.
- **exFAT dates:** exFAT silently moves times before 1980 to 1980-01-01 00:00:00 UTC (315532800), and times after 2107 to 2107-12-31 23:59:59 UTC (4354819199). Only tests on real exFAT showed this.

### Formatting through udisks2

These results come from the Kingston stick, using the `Format` and `CreatePartition` D-Bus methods.

- **`mkfs` options:** the `mkfs-args` option of `Format` passes arguments on to `mkfs.ext4`. With `-m 0` and the initialisation options, the file system had 0 reserved blocks, and all 116 block groups were marked `ITABLE_ZEROED`. Without them, it had 189,222 reserved blocks and no zeroed groups.
- **Background initialisation:** no `ext4lazyinit` thread appeared in the controlled check, right after mounting or 20 seconds later. One thread was counted in an earlier, less controlled run, and that wasn't explained.
- **Ownership:** `take-ownership` leaves the root folder owned by the user with mode `0700`. The user can then set `0777` and the default ACL without root.
- **Format times:** Ext4 took 50–62 seconds with the options and 29 seconds without, and exFAT took 0.3 seconds.
- **D-Bus timeout:** GIO's default call timeout is 25 seconds, which is shorter than every Ext4 format here. The test used 900 seconds, so Babawus must set a long timeout on the call. Testing with the default timeout was not tried.
- **Partition types:** the test used `0x83` for Ext4 and `0x07` for exFAT on the MBR. The integration tests have since written both GPT types on loop devices.
- **No automount after formatting:** Cinnamon didn't mount the stick after a format, so Babawus has to mount it itself.
- **Finding block groups:** when counting groups in `dumpe2fs` output, match `^Group [0-9]`. A plain `^Group ` also matches the "Group descriptor size" line.

### Formatting in Babawus

Babawus's own formatting, through udisks2, on the Kingston stick on 2026-11-07:

- **Times:** Ext4 took 57.7 seconds and exFAT 3.0 seconds, including unmounting, partitioning, mounting and creating the configuration.
- **Ext4 result:** the root folder, `.babawus` and `backup` were owned by the user with mode `0777`, the default ACL was set, there were no reserved blocks, and every block group was initialised. No `ext4lazyinit` thread was running afterwards.
- **The old file system came back:** when an old partition started where the new one did, the new one showed the old file system, and Cinnamon mounted it while udisks2 was wiping it. Babawus now wipes the old partitions first (see issue 61).
- **Loop devices detach themselves:** when udisks2 mounts a loop device for the user who set it up, it sets the device to detach itself once unmounted. Reformatting one in a test then fails, because unmounting it removes the device. The test turns that off again with the `SetAutoclear` method.
- **Setting up loop devices at once:** two `udisksctl loop-setup` calls at the same time can pick the same free device, and one fails with "Device or resource busy". The test helper sets them up one at a time.
- **Ejecting:** unmounting and powering off the stick took 0.2 seconds. udisks2 sent it `SYNCHRONIZE CACHE` and `START STOP UNIT`, and removed it from the USB bus, so it left the VM. It came back once it was attached again from the host. With a file open on the stick, unmounting failed with udisks2's `DeviceBusy` error.
- **Permissions in CI:** polkit lets the user of the active desktop session use udisks2. GitHub's runner has no such session, so CI adds a polkit rule for its own user.

### Speed on a real stick

Measured once on the Kingston stick, copying through `.babawus/tmp/` and renaming, as Babawus will. Directories weren't synced, so a real run may be a little slower.

| Test | Ext4 | exFAT |
| --- | --- | --- |
| 4 KiB files, `fsync` per file | 59 files/s | 67 files/s |
| 4 KiB files, one sync at the end | 78 files/s | 228 files/s |
| 64 KiB files, `fsync` per file | 31 files/s | 54 files/s |
| 64 KiB files, one sync at the end | 38 files/s | 125 files/s |
| One 512 MiB file | 16.6 MiB/s | 21.8 MiB/s |

Two further results show that writes mostly reach memory first, not the stick:

- **Large files:** on exFAT, all 512 MiB were "written" in 0.6 seconds, and `fsync` then took 20.1 seconds.
- **After `fsync`:** on Ext4, a final sync still took 8.6 seconds after 1000 files had each had their own `fsync`. On exFAT it took no time.

Flushing in chunks costs nothing measurable. Copying 256 MiB to exFAT, twice for the smaller sizes:

| Chunk size | Speed | Longest wait for the counter |
| --- | --- | --- |
| 1 MiB | 20–22 MiB/s | 0.9 seconds |
| 4 MiB | 22 MiB/s | 1.4 seconds |
| 16 MiB | 22 MiB/s | 1.6 seconds |
| 64 MiB | 21 MiB/s | 3.5 seconds |
| 256 MiB, so no chunks | 22 MiB/s | 11.9 seconds |

### USB sticks

- **Detecting removable devices:** udisks2 reports the stick as `Removable: true` but `MediaRemovable: false`, because the stick and its medium are one unit. To decide what to offer for formatting, Babawus must use `Removable`, together with `ConnectionBus: usb`.
- **Other udisks2 properties:** the stick also reports `HintSystem: false`, `Ejectable: true` and `CanPowerOff: true`, so eject and power off should be possible.
- **Automounting:** Cinnamon mounts a passed-through stick under `/media/steen/` as usual, also when the VM boots with the stick attached.
- **Read-only mounts:** udisks2's `ReadOnly` is about the device, so a stick mounted with `-o ro` still reports `false`. Babawus checks the mount itself with `statvfs`.
- **Change signals:** a single unmount or mount reached Babawus as up to seven separate D-Bus messages, spread over more than one pass of the main loop. Gathering them for 100 ms turned each change into one update.
- **Loop devices:** udisks2 reports a loop device's file system, label and mount points like any other, with `/` as its drive. So the integration tests can check what udisks2 reports, but a loop device never counts as a USB medium.
- **Device names change:** after a reboot with the stick attached, the stick was `/dev/sda` and the system disk `/dev/sdb`, the reverse of before. Never use a device name from an earlier session: look the stick up by model first, for example with `lsblk -o NAME,TRAN,MODEL`. A model check is what stopped a test script here from using the wrong device.

### Build and packaging

- **GTK:** Mint 22.3 has GTK 4.14.5, so the `gtk4` crate uses the `v4_14` feature. Don't raise it above the supported platform.
- **Build times:** about 45 seconds for a first debug build and about a minute for a release build.
- **Disk use:** `target/` reaches 1.1 GB even for the skeleton.
- **Packaging:** `cargo deb` must be given `-p babawus`, because the project is a workspace. Installing, updating and uninstalling it were first run on 2026-11-10 (T-025).
- **Test images:** these go in the system's temporary folder, as `babawus-medium-*`, so that the desktop ignores them (see "Cinnamon and loop devices").

### System quirks

- **Restarting Cinnamon over SSH:** `cinnamon --replace`, run from an SSH session, leaves Cinnamon running in that session. Afterwards, popovers didn't show on the first try, new windows opened behind others, and Captain's "Install Package" did nothing. Logging out and in again put it right. Restart Cinnamon at the computer instead, or log out.

- **GUI runs over SSH have no session bus:** a second start then can't find the first, so it shows "Babawus is already open" instead of raising the window. To test that, run both inside `dbus-run-session`.
- **Package flags:** after the original setup, `build-essential`, `g++` and `g++-x86-64-linux-gnu` were installed but marked for removal (`ri` in `dpkg -l`). The cause wasn't found. If it happens again, run `printf 'build-essential install\ng++ install\ng++-x86-64-linux-gnu install\n' | sudo dpkg --set-selections`.
- **Held-back updates:** `apt` may hold back a package with `(phased 40%)` in `apt-cache policy`. Ubuntu is rolling it out gradually, and it arrives by itself later.
- **Guest Additions:** the Guest Additions packages aren't installed. The kernel's own `vboxguest` module is loaded, and nothing so far has needed more.
- **Slow package mirror in CI:** Ubuntu's mirror once delivered at 24 kB/s, which took a run from one minute to nearly thirteen. CI now caches the downloaded packages and uses a short `apt` timeout. `apt-get update` still needs the mirror.
- **Graphics warnings:** the app prints `libEGL warning: DRI3 error` on this VM, and its window still works. This is probably because the VM has no accelerated graphics, but that hasn't been confirmed.
- **Polkit:** the VM has a rule, `/etc/polkit-1/rules.d/49-babawus-dev.rules`, that lets `steen` use every udisks2 action without a password, so that tests and Claude's sessions over SSH can format and unmount. That, not SSH, is why formatting the real stick never asked for a password here, Ext4 included. Without the rule, Mint asks the desktop user for nothing on exFAT, and asks for an administrator's password on Ext4 and for unmounting another account's stick (T-024, issues 108 to 110). To see what an ordinary computer would ask, move the rule away for a moment, ask `pkcheck --action-id <action> --process <pid>` about a Babawus started from the desktop menu (one started over SSH belongs to the SSH session), and put the rule back. The rule:

  ```js
  // Development VM only: let the dev account use udisks2 from SSH sessions.
  polkit.addRule(function(action, subject) {
      if (action.id.indexOf("org.freedesktop.udisks2.") == 0 && subject.user == "steen") {
          return polkit.Result.YES;
      }
  });
  ```
- **A second account:** `testuser`, a Standard account made in Users and Groups on 2026-11-10 for T-024. A mount stays after its account logs out, under `/media/<that account>/`, which no other account can open, so pull the stick out and plug it back in after changing accounts.

## Security notes

These settings are fine for a disposable test VM, and wrong for a computer that matters:

- Your account can run anything as root without a password, and so can any process or Claude session running as you.
- SSH still accepts passwords. Set `PasswordAuthentication no` if the VM ever becomes reachable from outside the host.
- The GitHub token is stored in the keyring on the VM, so revoke it if you discard the VM.

## Not kept in the repository

If the VM is lost, these are gone unless they are backed up:

- Claude's memory, in `~/.claude/projects/`. This guide covers everything that matters from it.
- Work that hasn't been pushed to GitHub.
