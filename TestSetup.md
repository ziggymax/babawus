# Babawus's development, test and release set-up

This document explains how Babawus is built, tested and released on every distribution it supports, and how the parts fit together. [DevEnvSetup.md](DevEnvSetup.md) has the steps for setting up each machine; this document says what each one is for and why. The decisions are in issues 169 and 170 in [Issues-Resolved.md](projectmgt/Issues-Resolved.md).

The set-up was first described as planned, with each part that had not been tried yet marked with the task that would prove it. Every part has been tried now (T-126 to T-133). A part added later and not tried yet is marked the same way, such as *(not tried yet, T-140)*, and the mark goes when its task makes the part work.

## The parts

| Part | What it is for | Where |
| --- | --- | --- |
| The host | Writing code, building, unit tests, the containers, and running the VMs | Ubuntu 26.04 with GNOME on Wayland |
| The containers | Building Babawus and running its unit tests on each distribution, and building and installing its packages | podman on the host, and the same scripts in CI |
| CI | The same as the containers, plus the tests on real file systems, on every pull request | GitHub Actions |
| The VMs | Installing the package as a user would, sticks, the desktop, and the release checklist | KVM under libvirt on the host |
| The driver | Doing the same things to every VM: snapshots, screenshots, clicks and keys, commands, sticks | `vms/driver.py` on the host |

Babawus is built and unit-tested in one place, the containers, and the finished package is then tried in clean VMs. A VM never gets a build tree, only the package, so that a missing dependency or file shows up as a failure there.

## The host

The host is a computer that matters, so it has no passwordless `sudo` and no polkit rule for udisks2 (issue 169). Claude builds, runs the tests and the containers, and will drive the VMs, without a password. The user does the few steps that need `sudo` or hands.

The tests on real file systems format loop devices through udisks2, which needs those rights, so they don't run on the host: they run in CI and on the VMs. In a VM they need a polkit rule for udisks2, as CI has, which the smoke test installs (T-128).

## The containers

Each supported distribution has a container image, built from `containers/apt.Containerfile` or `containers/dnf.Containerfile`. `containers/common.sh` lists the distributions and their pinned images:

| Distribution | Image | Package |
| --- | --- | --- |
| Ubuntu 24.04 (and Linux Mint 22, which is built on it) | `ubuntu:24.04` | `.deb` |
| Ubuntu 26.04 | `ubuntu:26.04` | `.deb` |
| Fedora 45 | `fedora:45` | `.rpm` |

- `containers/check.sh` runs clippy and the unit tests in each container.
- `containers/package.sh` builds the packages and installs each one, with all its dependencies, in a clean image of every distribution it is for, as a user would. It checks that `ldd` finds every library `/usr/bin/babawus` needs, and that the language files and `THIRD-PARTY-LICENSES` are installed.

Each container keeps its build output and downloaded crates in `target/containers/<name>/`, so a second run is quick. The tools the packages are built with, `cargo-deb` and `cargo-about`, are kept there too, rather than in the images.

Containers have no desktop and no USB, so they can't try the window or a stick. That is what the VMs are for.

## CI

CI runs on every pull request and every push to main, with the same scripts as the host:

- **check:** fmt, clippy, the Windows build of `babawus-core`, and all tests, including those on real file systems, directly on an Ubuntu 24.04 runner, where a polkit rule lets the runner use udisks2.
- **check (ubuntu-26.04, fedora-45):** `containers/check.sh` for each of these. The Fedora job also builds the `.rpm` and installs it in a clean Fedora.
- **package (deb):** builds the `.deb` in the Ubuntu 24.04 container and installs it on Ubuntu 24.04 and 26.04.
- **packages:** keeps the `.deb` and the `.rpm` that the jobs above built, with a `SHA256SUMS` file, as the run's artifact `packages` (T-129). Only the newest of each branch is kept: the job deletes the packages of older runs on the same branch, and its two hand-offs, so that a pull request's packages can be tried on a VM and a release made from main's, without old ones piling up (T-134). `vms/fetch-packages.sh [commit]` downloads them and checks the sums.

The Ubuntu containers set apt up with `containers/apt-setup.sh`: a short timeout and retries, and in CI the runner's own mirror (`UBUNTU_MIRROR`) in place of Ubuntu's, which on a slow day took a job past its 30 minutes (T-133).

## Packages

- **One `.deb`** serves Linux Mint and both Ubuntus. It is built on Ubuntu 24.04, the oldest of them, so that it works on the newer ones too (issue 169). Debian is no longer supported (T-133).
- **One `.rpm`** serves Fedora, with Fedora's package names for its dependencies.
- Both install Babawus's MIT licence and `THIRD-PARTY-LICENSES`, the licences of the crates it is built from, made with `cargo-about` (issue 171).
- An interim version such as 1.2.0-alpha.1 is numbered `1.2.0~alpha.1` in both, so that it sorts before 1.2.0.

For a release, GitHub builds both packages, and the smoke test and the checklist run on those very files, which are then attached to the GitHub release, with their `SHA256SUMS` (T-129).

## The VMs

There are three VMs, each with the desktop its distribution ships: Linux Mint 22 with Cinnamon on X11, Ubuntu 26.04 and Fedora 45, both with GNOME on Wayland. Two VMs are enough for the `.deb` (T-133). Mint is built on Ubuntu 24.04's own archive, so it tries the `.deb` on 24.04's exact libraries, and Ubuntu 26.04 on the newest ones; that the package installs on clean Ubuntu 24.04 and 26.04 is checked in the containers on every pull request. Ubuntu 26.04 came first, to prove the set-up (T-126). Mint 22.3 followed, installed anew, since VirtualBox and its Mint VM were removed from the host on 2026-11-25 (T-127); the driver drives it unchanged, on Cinnamon on X11. Fedora 45 came last, installed through the driver the same way (T-124), with guests for Ubuntu 24.04 and Debian 13, which T-133 removed again.

**libvirt in the user session.** The VMs run under KVM, managed by libvirt in the user's own session (`qemu:///session`), not the system one. Nothing about them runs as root, and the user needs no membership of the `libvirt` group, which is close to root on a computer that matters. The user is in the `kvm` group, which is all KVM needs. The user session has two limits:

- **The network is QEMU's own** (slirp), which reaches the internet through the host. The driver forwards a port on the host, `127.0.0.1` only, to each guest's SSH when it starts the guest. libvirt forwards ports itself only for passt, its newer network for the user session, but passt can't be used there on Ubuntu 26.04: its AppArmor profile lets neither libvirt nor the user stop it, so it outlives its guest and keeps the port, and the guest got no address from it (T-126).
- **Passing a real stick through** needs the user to have access to its USB device on the host, which root owns. `vms/70-babawus-usb-storage.rules`, installed on the host once, gives the user logged in at the computer access to every USB storage device (T-126).

**Each guest is defined** by its file in `vms/`, such as `vms/ubuntu-26.04.xml`, which `vms/driver.py define` turns into a libvirt guest with an empty 25 GB disk. The disks, the installers' ISOs and the stick images are kept in `~/.local/share/libvirt/images/`. A guest uses BIOS rather than UEFI, because libvirt can't take an internal snapshot of a guest with UEFI firmware, and an internal snapshot of the running guest lets a revert return straight to a desktop that is logged in. **Each guest has** the same settings, so that the driver needs nothing per guest but its name and the commands it runs:

- automatic login for the test user, so that the desktop is there after a revert;
- OpenSSH, with the host's key;
- VNC graphics, listening only on the host, so that the user can watch and type in virt-manager, at a fixed resolution of 1280×800, so that screenshots can be compared between runs;
- a USB tablet as pointer, so that clicks land on exact coordinates;
- a USB 3 (`qemu-xhci`) controller, for plugging in sticks;
- a snapshot named `clean`, taken once the guest is set up.

The guests are installed by hand, with the driver doing the clicking and typing, and the user typing the test user's password; DevEnvSetup.md describes it. Building them from scripts is an idea for later (issue 170).

## The driver

Every VM is driven the same way, from the host, by `vms/driver.py`, through two channels:

| Channel | Used for |
| --- | --- |
| libvirt | Starting and stopping, reverting to the `clean` snapshot, taking screenshots, moving the pointer, clicking, typing and pressing keys, and plugging in and unplugging sticks |
| SSH | Running commands, copying the package in, and reading results |

Screenshots, clicks and keys go to QEMU through libvirt (its `screendump`, `input-send-event` and `send-key`), below the guest, so they work the same on Cinnamon and on GNOME on Wayland, where a program inside the guest can't take screenshots of other windows or move the mouse. This was proved by installing Ubuntu 26.04 through the driver (T-126). Adding a distribution then means adding a VM, not changing the driver. The driver uses only Python's standard library and the distribution's `python3-libvirt` (issue 170); `vms/driver.py --help` lists its commands.

Typing is for a US keyboard layout, which every guest is installed with. The guest draws a moment behind: a screenshot taken right after typing may show only part of the text, so wait a second first.

Claude uses the driver to drive the window in every guest, and the smoke test uses it without Claude.

**Starting Babawus from SSH.** A command run over SSH is not part of the desktop session, so a window started from it wouldn't appear. `vms/driver.py launch` starts it through `systemd-run --user`, in the user's own systemd, which the desktop has given its `WAYLAND_DISPLAY` and `DISPLAY`, so the window appears on the desktop (T-126).

## Sticks

- **Emulated sticks** are image files on the host, formatted there or left blank for Babawus to format, which the driver plugs into a guest as a USB disk. The guest sees a real USB stick arrive and leave, and the same image gives the same result every time. udisks2 in the guest reports them as removable USB sticks, as it does a real one. They are used for the smoke test and for most checklist steps (T-126).
- **Real sticks** are passed through from the host for the steps that are about hardware, such as speed or pulling a stick out during a backup. The driver finds a stick by its serial number (`vms/driver.py sticks` lists them), so that two identical sticks are told apart wherever they are plugged in. While a VM has it, the stick disappears from the host, and only one VM can have it at a time (T-126).

## Testing

| Test | Runs where | Run by | When |
| --- | --- | --- | --- |
| Unit tests, clippy | The host, each container, CI | Claude, CI | Every change |
| Tests on real file systems | CI, the VMs | CI, Claude over SSH | Every pull request, and when needed |
| Packages install and find their libraries | Clean container images, CI | `containers/package.sh` | Every pull request |
| Smoke test | Every VM | `vms/smoke.py`, without Claude | Before each release |
| Release checklist ([ReleaseChecklist.md](projectmgt/ReleaseChecklist.md)) | Each distribution's VM | Claude, with the user | Before each release |

**The smoke test**, `vms/smoke.py`, tries the packages on every guest, one after another, without Claude (T-128). For each guest it:

1. reverts it to `clean`, and sets its clock, which stood still in the snapshot;
2. installs the package as a user would (`apt-get install ./babawus….deb` or `dnf install ./babawus….rpm`), after reading apt's package lists again, up to four times a minute apart, since a mirror in the middle of a sync refuses them for a while;
3. starts Babawus with `--debug=sticks`, plugs in an emulated exFAT stick labelled `SMOKE`, checks that Babawus lists it, saves a screenshot, pulls the stick out, and checks that Babawus no longer lists it;
4. installs the tools that the tests on real file systems use, and a polkit rule that lets `tester` use udisks2 from SSH, outside the desktop session;
5. runs the tests on real file systems, built in that distribution's container by `containers/build-tests.sh` (Mint's in Ubuntu 24.04's).

The tools and the rule come only after Babawus has been tried, so that they can't hide a dependency missing from a package, and the `clean` snapshot has neither, so that the checklist runs with no rights a user wouldn't have (issue 170). It takes the packages from `target/containers/packages/`, where `containers/package.sh` leaves them, or from the folder given with `--packages`. Each guest is left running as the test left it, and its log and screenshot go to `target/smoke/`. The three guests take about 3 minutes.

**`--debug`** lets a test ask Babawus what it sees, rather than compare screenshots, which differ between desktops. `babawus --debug=sticks` prints a line to standard error each time the start screen reads the list of sticks, such as `Babawus debug sticks: listed "SMOKE"`. More topics can be added to the same option, as a comma-separated list; an unknown topic is reported and ignored. It is for testing, so the Help and the README don't mention it (issue 170).

**The release checklist** is run on each distribution's VM. Claude drives the window through the driver, GNOME on Wayland included, and checks the rest over SSH. The user does what needs hands, such as pulling out a real stick, and types passwords (issue 170).

There is no automated test suite for the window: the logic behind each screen is tested in `babawus-core`, and the checklist covers the window itself. A suite, and accessible names for the widgets that it would find them by, can be added later on the same driver.

## A release, step by step

1. The interim versions are done and tagged on main ([ReleasePlanning.md](projectmgt/ReleasePlanning.md)), and the release's version is merged to main. **In the same pull request, the README's "Supported platforms" is brought up to date** (T-139, issue 174): the distributions, desktops and packages the release supports, and how its packages are tested, so that the commit that is tagged carries it. A note such as "Until 1.2.0 is released, …" goes once that release is made. **The same pull request adds the release at the top of `<releases>` in `crates/babawus/assets/io.github.ziggymax.Babawus.metainfo.xml`**, `<release version="<version>" date="<the day it is released>"/>` (issue 175); `tests/metainfo.rs` fails without it. Interim versions get none.
2. CI builds the `.deb` and the `.rpm` for main's commit and keeps them, with `SHA256SUMS` (T-129). `vms/fetch-packages.sh` downloads them into `target/github-packages/<commit>/` and checks the sums.
3. The smoke test runs on every VM, on those packages: `vms/smoke.py --packages target/github-packages/<commit>`.
4. The release checklist runs on each distribution's VM, on the same packages.
5. That commit is tagged, and the GitHub release gets the same three files: `gh release create v<version> --target <commit> --title "Babawus <version>" --notes-file <notes> target/github-packages/<commit>/*`. Nothing is built again, so the released files are the ones that were tested.

**The release message** always has these parts, after what is new (T-139, issue 174), so that a user sees which distributions the release is for, and what its very files went through:

```markdown
## Supported platforms

| Distribution | Desktop | Package |
| --- | --- | --- |
(the same rows as the README's "Supported platforms")

## How these files were tested

- Built by GitHub, in CI run <link to the run> for <commit>, which also ran the automated tests on every distribution.
- Installed as a user would, and tried by the smoke test, on <each VM, such as Linux Mint 22.3 (Cinnamon, X11)>, on <date>.
- The whole release checklist, by hand: <link to each distribution's checklist issue>.

## Checking the download

`sha256sum --check SHA256SUMS`, in the folder with the downloaded files.
```

A distribution the checklist was not run on is not listed as supported, in the README or in the message.

## Not part of the set-up

- **macOS** is not planned (issue 170).
- **Windows** will get a VM in the same set-up, driven the same way, once its UI toolkit is chosen; that choice is still open in ReleasePlanning.md.
- **Building the guests from scripts**, a test suite for the window, and accessible names for its widgets are ideas for later (issue 170).
