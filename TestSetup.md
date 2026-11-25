# Babawus's development, test and release set-up

This document explains how Babawus is built, tested and released on every distribution it supports, and how the parts fit together. [DevEnvSetup.md](DevEnvSetup.md) has the steps for setting up each machine; this document says what each one is for and why. The decisions are in issues 169 and 170 in [Issues-Resolved.md](projectmgt/Issues-Resolved.md).

The set-up is described as planned. A part that has not been tried yet is marked with the task that proves it, such as *(not tried yet, T-126)*. The mark goes when the task makes the part work, and this document changes if the task finds that something works differently.

## The parts

| Part | What it is for | Where |
| --- | --- | --- |
| The host | Writing code, building, unit tests, the containers, and running the VMs | Ubuntu 26.04 with GNOME on Wayland |
| The containers | Building Babawus and running its unit tests on each distribution, and building and installing its packages | podman on the host, and the same scripts in CI |
| CI | The same as the containers, plus the tests on real file systems, on every pull request | GitHub Actions |
| The VMs | Installing the package as a user would, sticks, the desktop, and the release checklist | KVM under libvirt on the host *(not tried yet, T-126)* |
| The driver | Doing the same things to every VM: snapshots, screenshots, clicks and keys, commands, sticks | A small command-line program on the host *(not tried yet, T-126)* |

Babawus is built and unit-tested in one place, the containers, and the finished package is then tried in clean VMs. A VM never gets a build tree, only the package, so that a missing dependency or file shows up as a failure there.

## The host

The host is a computer that matters, so it has no passwordless `sudo` and no polkit rule for udisks2 (issue 169). Claude builds, runs the tests and the containers, and will drive the VMs, without a password. The user does the few steps that need `sudo` or hands.

The tests on real file systems format loop devices through udisks2, which needs those rights, so they don't run on the host: they run in CI and on the VMs.

## The containers

Each supported distribution has a container image, built from `containers/apt.Containerfile` or `containers/dnf.Containerfile`. `containers/common.sh` lists the distributions and their pinned images:

| Distribution | Image | Package |
| --- | --- | --- |
| Ubuntu 24.04 (and Linux Mint 22, which is built on it) | `ubuntu:24.04` | `.deb` |
| Ubuntu 26.04 | `ubuntu:26.04` | `.deb` |
| Debian 13 | `debian:13` | `.deb` |
| Fedora 45 | `fedora:45` | `.rpm` |

- `containers/check.sh` runs clippy and the unit tests in each container.
- `containers/package.sh` builds the packages and installs each one, with all its dependencies, in a clean image of every distribution it is for, as a user would. It checks that `ldd` finds every library `/usr/bin/babawus` needs, and that the language files and `THIRD-PARTY-LICENSES` are installed.

Each container keeps its build output and downloaded crates in `target/containers/<name>/`, so a second run is quick. The tools the packages are built with, `cargo-deb` and `cargo-about`, are kept there too, rather than in the images.

Containers have no desktop and no USB, so they can't try the window or a stick. That is what the VMs are for.

## CI

CI runs on every pull request and every push to main, with the same scripts as the host:

- **check:** fmt, clippy, the Windows build of `babawus-core`, and all tests, including those on real file systems, directly on an Ubuntu 24.04 runner, where a polkit rule lets the runner use udisks2.
- **check (ubuntu-26.04, debian-13, fedora-45):** `containers/check.sh` for each of these. The Fedora job also builds the `.rpm` and installs it in a clean Fedora.
- **package (deb):** builds the `.deb` in the Ubuntu 24.04 container and installs it on Ubuntu 24.04 and 26.04 and Debian 13.

## Packages

- **One `.deb`** serves Linux Mint, both Ubuntus and Debian. It is built on Ubuntu 24.04, the oldest of them, so that it works on the newer ones too (issue 169).
- **One `.rpm`** serves Fedora, with Fedora's package names for its dependencies.
- Both install Babawus's MIT licence and `THIRD-PARTY-LICENSES`, the licences of the crates it is built from, made with `cargo-about` (issue 171).
- An interim version such as 1.2.0-alpha.1 is numbered `1.2.0~alpha.1` in both, so that it sorts before 1.2.0.

For a release, GitHub builds both packages, and the smoke test and the checklist run on those very files, which are then attached to the GitHub release *(not tried yet, T-129)*.

## The VMs

There is one VM per distribution: Linux Mint 22 with Cinnamon, Ubuntu 24.04, Ubuntu 26.04, Debian 13 and Fedora 45, each with the desktop it ships. All but Mint use GNOME on Wayland. Ubuntu 26.04 comes first, to prove the set-up *(not tried yet, T-126)*. The Mint VM moves over from VirtualBox *(not tried yet, T-127)*, and the others follow *(not tried yet, T-124)*. Until T-127, the Mint VM is still the VirtualBox one that DevEnvSetup.md describes.

**libvirt in the user session.** The VMs run under KVM, managed by libvirt in the user's own session (`qemu:///session`), not the system one. Nothing about them runs as root, and the user needs no membership of the `libvirt` group, which is close to root on a computer that matters. The user is in the `kvm` group, which is all KVM needs. The user session has two limits, which T-126 settles:

- The network is QEMU's own, with a port on the host forwarded to each guest's SSH.
- Passing a real stick through needs the user to have access to its device on the host, likely through one udev rule.

**Each guest has** the same settings, so that the driver needs nothing per guest but its name and the commands it runs:

- automatic login for the test user, so that the desktop is there after a revert;
- OpenSSH, with the host's key;
- VNC graphics, listening only on the host, at a fixed resolution, so that screenshots can be compared between runs;
- a USB tablet as pointer, so that clicks land on exact coordinates;
- a USB 3 (`qemu-xhci`) controller, for plugging in sticks;
- a snapshot named `clean`, taken once the guest is set up.

The guests are installed by hand, as DevEnvSetup.md will describe. Building them from scripts is an idea for later (issue 170).

## The driver

Every VM is driven the same way, from the host, through three channels:

| Channel | Used for |
| --- | --- |
| libvirt | Reverting to the `clean` snapshot, taking screenshots, and plugging in and unplugging sticks |
| VNC | Moving the mouse, clicking, typing and pressing keys |
| SSH | Running commands, copying the package in, and reading results |

Because screenshots, clicks and keys are made below the guest, by the hypervisor, they work the same on Cinnamon and on GNOME on Wayland, where a program inside the guest can't take screenshots of other windows or move the mouse. Adding a distribution then means adding a VM, not changing the driver.

The driver is a small command-line program on the host, with commands such as revert, screenshot, click, type, key, run, plug and unplug *(not tried yet, T-126)*. Claude uses it to drive the window in every guest, and the smoke test uses it without Claude.

**Starting Babawus from SSH.** A command run over SSH is not part of the desktop session, so a window started from it wouldn't appear. The driver starts Babawus in the logged-in user's session, by giving it that session's `WAYLAND_DISPLAY` or `DISPLAY` and `XDG_RUNTIME_DIR`, or through `systemd-run --user` *(not tried yet, T-126)*.

## Sticks

- **Emulated sticks** are image files on the host, formatted there or left blank for Babawus to format, which the driver plugs into a guest as a USB disk. The guest sees a real USB stick arrive and leave, and the same image gives the same result every time. They are used for the smoke test and for most checklist steps *(not tried yet, T-126)*.
- **Real sticks** are passed through from the host for the steps that are about hardware, such as speed or pulling a stick out during a backup. While a VM has it, the stick disappears from the host, and only one VM can have it at a time. Two identical sticks are told apart by their USB port rather than by vendor and product *(not tried yet, T-126)*.

## Testing

| Test | Runs where | Run by | When |
| --- | --- | --- | --- |
| Unit tests, clippy | The host, each container, CI | Claude, CI | Every change |
| Tests on real file systems | CI, the VMs | CI, Claude over SSH | Every pull request, and when needed |
| Packages install and find their libraries | Clean container images, CI | `containers/package.sh` | Every pull request |
| Smoke test | Every VM | The driver, without Claude | Before each release *(not tried yet, T-128)* |
| Release checklist ([ReleaseChecklist.md](projectmgt/ReleaseChecklist.md)) | Each distribution's VM | Claude, with the user | Before each release |

**The smoke test** reverts a VM to `clean`, installs the package as a user would (`apt install ./babawus.deb` or `dnf install ./babawus.rpm`), starts Babawus, plugs in an emulated stick and checks that it shows up, and runs the tests on real file systems, built in that distribution's container *(not tried yet, T-128)*.

**The release checklist** is run on each distribution's VM. Claude drives the window through the driver, GNOME on Wayland included, and checks the rest over SSH. The user does what needs hands, such as pulling out a real stick, and types passwords (issue 170).

There is no automated test suite for the window: the logic behind each screen is tested in `babawus-core`, and the checklist covers the window itself. A suite, and accessible names for the widgets that it would find them by, can be added later on the same driver.

## A release, step by step

1. The interim versions are done and tagged on main ([ReleasePlanning.md](projectmgt/ReleasePlanning.md)).
2. GitHub builds the `.deb` and the `.rpm` *(not tried yet, T-129)*.
3. The smoke test runs on every VM, on those packages *(not tried yet, T-128)*.
4. The release checklist runs on each distribution's VM, on the same packages.
5. The version is tagged, and the GitHub release gets both packages.

## Not part of the set-up

- **macOS** is not planned (issue 170).
- **Windows** will get a VM in the same set-up, driven the same way, once its UI toolkit is chosen; that choice is still open in ReleasePlanning.md.
- **Building the guests from scripts**, a test suite for the window, and accessible names for its widgets are ideas for later (issue 170).
