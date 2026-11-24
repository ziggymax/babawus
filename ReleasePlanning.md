# Planning the next release

The purpose of this document is to have a place to capture ideas for the next release of Babawus and, in doing so, help in creating and maintaining a general plan for the future work on Babawus.

Some releases may represent so much work and/or complexity that a step-by-step sequence of interim non-released versions is needed to reach the stage where a new release can be made. These non-released versions are an important part of the release planning and hence also something that will be described in this document.

For the remainder of this document the word "version" is used for something that is not planned to be released, and the word "release" is used only for something that is intended for release.

Interim versions are numbered as semver pre-releases of the release they lead to, for example 1.0.0-alpha.1, 1.0.0-alpha.2, and so on. Each interim version is tagged on main, but gets no GitHub release.

When work on a version begins, its items become tasks (T-entries) in the task list.

## Latest release of Babawus

The latest/current release is: 1.1.1 (tag `v1.1.1`), with the Linux-specific code behind a platform layer and no visible change. Babawus follows semantic versioning (major.minor.patch).

The latest interim version is: none since 1.1.0; 1.1.1 had none. The last was 1.1.0-alpha.3 (tag `v1.1.0-alpha.3`).

## Next release of Babawus

This section always describes the next upcoming version/release of Babawus. Some of the ideas may be very specific, and some more speculative in nature. Task(s) created to deal with speculative ideas will be investigative rather than implementation-oriented.

1.1.0's plan, its three interim versions and their reasons, are kept in the issues (159 to 165) and the tasks (T-089 to T-105). Step 0 of bringing Babawus to more platforms was released as 1.1.1; its plan and decisions are kept in issue 166 and the tasks T-106 to T-114, and the platform layer is described under "Platform layer" in [Requirements.md](Requirements.md).

The next releases start bringing Babawus to more platforms: first other Linux distributions, then Windows. They are planned as three steps. Step 0, the platform layer, is done (1.1.1). Step 1 is planned below; step 2, Windows, is under "Future releases".

**One codebase for all platforms.** Every platform, Windows included, is built from the same code on main. There are no long-lived branches per platform. What differs between platforms sits in a small platform layer, with one implementation per platform, chosen when Babawus is built (`#[cfg(target_os = ...)]`). Everything else, such as the backup logic, versioning, the log, the configuration and the language files, is shared. Between the Linux distributions, the differences are expected to be almost entirely in packaging and dependency names, not in the code.

**Containers for building, VMs for the checklist.** Babawus is built and its unit tests run in a container for each supported distribution, locally and in GitHub Actions on every pull request. Containers can't test sticks or a desktop, so each distribution also has a VM. A smoke test installs the package there and tries it on an emulated stick, and the release checklist ([ReleaseChecklist.md](ReleaseChecklist.md)) is run there before each release, with real sticks where a step needs hands.

### Step 1: release 1.2.0, other Linux distributions

1.2.0 runs on these distributions, all with their default desktop:

| Distribution | Versions | Package |
| --- | --- | --- |
| Linux Mint | 22.x (as now) | `.deb` |
| Ubuntu | 24.04 LTS and 26.04 LTS | `.deb` |
| Debian | The current stable version | `.deb` |
| Fedora | The newest version only | `.rpm` |

Ubuntu 24.04 is close to free: Linux Mint 22 is built on it, with the same packages and GTK 4.14. Ubuntu, Debian and Fedora use GNOME, unlike Mint's Cinnamon, which matters in two places:

- **The reminder at login.** GNOME has no "Startup Applications". `babawus --checklastbackup` stays as it is, and under Advanced, Babawus can also add its own entry in `~/.config/autostart/`, which works on every desktop. Whether to use that, or a desktop's own way, is the advanced user's choice.
- **Starting Babawus.** GNOME shows no icons on the desktop by default, so the user guide says to pin Babawus to the dock instead.

**Where the work happens.** From 1.2.0, Babawus is developed on the host, the computer that runs the VMs, with Ubuntu 24.04 or newer. It builds and tests Babawus in a podman container for each distribution, as CI does. The VMs, the Mint one included, are kept for tests on sticks and desktops, and for the release checklists (issue 169).

**The VMs.** The VMs run under KVM, managed by libvirt in the user's session, in place of VirtualBox. One small driver on the host does the same things to every guest: libvirt reverts it to a clean snapshot, takes screenshots and plugs sticks in and out, VNC moves the mouse and types, and SSH runs commands. Because the clicks and screenshots are made below the guest, they work on Wayland too. A stick is usually an image file that the guest sees as a USB stick, which makes tests repeatable; real sticks are passed through for the steps that are about hardware. The guests are installed by hand and described in [DevEnvSetup.md](DevEnvSetup.md). The decisions are in issue 170, and Babawus's own description of the set-up is T-125.

**Packages.** One `.deb`, built on Ubuntu 24.04, the oldest of the distributions, serves Mint, both Ubuntus and Debian, as today. Fedora gets one `.rpm`. GitHub builds both, the checklists run on those files, and the same files are attached to the GitHub release (T-129).

**Checklists.** The release checklist runs on a VM for each distribution. Claude drives the window on every one of them through the driver, GNOME on Wayland included, and checks the rest over SSH. The user does what needs hands, such as pulling out a real stick, and types passwords (issue 170). Before the checklist, a smoke test runs on every guest without Claude: it installs the package as a user would, starts Babawus, plugs in an emulated stick, and runs the tests on real file systems (T-128).

1.2.0 is reached in two interim versions, with the test VMs made between them:

1. **1.2.0-alpha.1: building everywhere.** Development moves to the host. Every distribution gets a container that builds Babawus and runs its tests, locally with podman and in CI on every pull request. Fedora gets an `.rpm`, with its own package names, and its `wheel` group counts as administrator, as `sudo` does on the others. The `.deb` is checked to install on each `.deb` distribution. The tasks are T-118 to T-123.
2. **The test VMs.** Not a version of Babawus, since nothing in it changes. libvirt and the driver are proved on one guest, Ubuntu 26.04 on Wayland. Then the Mint VM moves over from VirtualBox, the other guests are made, and the smoke test is added. They come before alpha.2 so that its GNOME parts can be tried on a GNOME desktop. The tasks are T-124 to T-129.
3. **1.2.0-alpha.2: GNOME.** Under Advanced, Babawus can add the reminder at login in `~/.config/autostart/`, which works on every desktop, Mint's "Startup Applications" included. The user guide says to pin Babawus to the dock on GNOME, and how to add the reminder there. ReleaseChecklist.md gets what differs per distribution. Its tasks are created when work on it begins.

Then the release: the smoke test and the checklist on each distribution's VM, on the packages GitHub built, and the release with both packages.

## Future releases

This section captures ideas for some future, unspecified releases of Babawus. As time and versions progress some of these ideas will migrate to the "Next release of Babawus" section, some will be deleted and some may live here for a long time.

To the extent possible the new features are listed by priority, from highest to lowest.

1. *A runnable release script(?)*. Investigate the possibility of automating releases to the maximum by maintaining the release procedure as some kind of runnable script. Anything that can be done without human intervention should be done automatically, where human intervention is needed the script should prompt the person responsible for doing the release with step-by-step actions to perform (and, where the person must report some result back, provide simple fillable forms). The driver and the smoke test of 1.2.0 (issue 170) are its first parts: the script can build on them once a release has been made with them.
1. *Windows (step 2 of bringing Babawus to more platforms)*. Windows gets its own implementation of the platform layer from 1.1.1, built from the same code on main. Rust builds for Windows directly, without Cygwin or MSYS2. Things to settle first:
    - **The UI toolkit.** GTK 4 runs on Windows, but is heavy to bundle and looks foreign there. Another toolkit would be a second UI crate in the same workspace, on the same `babawus-core`. This is the first decision for Windows.
    - **Sticks** are listed, ejected and formatted through Windows' own interfaces instead of udisks2, and formatting needs administrator rights. Windows has no Ext4, so sticks are exFAT only.
    - **Sticks used on both Linux and Windows:** names that Linux allows and Windows doesn't, paths in the configuration, and how computers and accounts are identified on each.
    - **Development** needs two Windows guests under libvirt, driven like the Linux ones: one with the build tools, and a clean one that only ever gets the finished package, so that a missing file shows up as a failed test. Which Windows versions is to be settled. CI builds and runs the unit tests on GitHub's Windows runner, from the same build script as the build guest; until then, building `babawus-core` for Windows from Linux keeps checking that it still compiles.
    - **The package.** A zip is the simplest, but users expect an installer, and Windows warns about programs that aren't signed. The package form and signing are to be settled.
    - **If GTK 4 is chosen,** the research behind issue 170 points to building GTK with gvsbuild and MSVC, not MSYS2, so that what is tested is what is shipped.
1. *More automation of the test VMs*. Ideas left out of 1.2.0 (issue 170), each to be taken up when its lack is felt: an automated test suite for the window, on the same driver; accessible names for the widgets, which such a suite would find them by and which also help screen-reader users; and building the guests from scripts, so that a VM can be made anew without installing by hand.
