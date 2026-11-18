# Planning the next release

The purpose of this document is to have a place to capture ideas for the next release of Babawus and, in doing so, help in creating and maintaining a general plan for the future work on Babawus.

Some releases may represent so much work and/or complexity that a step-by-step sequence of interim non-released versions is needed to reach the stage where a new release can be made. These non-released versions are an important part of the release planning and hence also something that will be described in this document.

For the remainder of this document the word "version" is used for something that is not planned to be released, and the word "release" is used only for something that is intended for release.

Interim versions are numbered as semver pre-releases of the release they lead to, for example 1.0.0-alpha.1, 1.0.0-alpha.2, and so on. Each interim version is tagged on main, but gets no GitHub release.

When work on a version begins, its items become tasks (T-entries) in the task list.

## Latest release of Babawus

The latest/current release is: 1.1.0 (tag `v1.1.0`), with a Restore window and leaving out single files. Babawus follows semantic versioning (major.minor.patch).

The latest interim version is: none since 1.1.0. The last was 1.1.0-alpha.3 (tag `v1.1.0-alpha.3`).

## Next release of Babawus

This section always describes the next upcoming version/release of Babawus. Some of the ideas may be very specific, and some more speculative in nature. Task(s) created to deal with speculative ideas will be investigative rather than implementation-oriented.

1.1.0's plan, its three interim versions and their reasons, are kept in the issues (159 to 165) and the tasks (T-089 to T-105).

The next releases start bringing Babawus to more platforms: first other Linux distributions, then Windows. They are planned as three steps. Steps 0 and 1 are planned below; step 2, Windows, is under "Future releases".

**One codebase for all platforms.** Every platform, Windows included, is built from the same code on main. There are no long-lived branches per platform. What differs between platforms sits in a small platform layer, with one implementation per platform, chosen when Babawus is built (`#[cfg(target_os = ...)]`). Everything else, such as the backup logic, versioning, the log, the configuration and the language files, is shared. Between the Linux distributions, the differences are expected to be almost entirely in packaging and dependency names, not in the code.

**Containers for building, VMs for the checklist.** Babawus is built and its unit tests run in a container for each supported distribution, locally and in GitHub Actions on every pull request. Containers can't test real sticks or a desktop, so the release checklist ([ReleaseChecklist.md](ReleaseChecklist.md)) is run on real sticks in one VM per distribution before each release.

### Step 0: release 1.1.1, the platform layer

1.1.1 moves the Linux-specific code behind the platform layer, without changing what Babawus does. It is released as a normal version for Linux Mint 22.x, so that the change is tested and in use before any new platform is added. The platform layer covers:

- **Sticks:** listing, mounting, ejecting and formatting them, and setting their permissions. This is udisks2, `mkfs.exfat`, `mkfs.ext4` and `setfacl` today.
- **Identity:** what identifies the computer and its boot (`/etc/machine-id`, `/proc`), and the computer's name.
- **Flushing and locking:** `syncfs` and `fsync`, and how the lock checks whether its process is still running.
- **Paths:** where the settings, the note of the last backup and the installed language files are.

From 1.1.1 on, CI also checks that `babawus-core` compiles for Windows, with a placeholder Windows implementation of the platform layer. That way, Linux-only code can't slip back into the shared parts while the other platforms are still to come.

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

Interim versions (1.2.0-alpha.N) are added as needed. Their tasks are created when work on 1.2.0 begins.

## Future releases

This section captures ideas for some future, unspecified releases of Babawus. As time and versions progress some of these ideas will migrate to the "Next release of Babawus" section, some will be deleted and some may live here for a long time.

To the extent possible the new features are listed by priority, from highest to lowest.

1. *A runnable release script(?)*. Investigate the possibility of automating releases to the maximum by maintaining the release procedure as some kind of runnable script. Anything that can be done without human intervention should be done automatically, where human intervention is needed the script should prompt the person responsible for doing the release with step-by-step actions to perform (and, where the person must report some result back, provide simple fillable forms).
1. *Windows (step 2 of bringing Babawus to more platforms)*. Windows gets its own implementation of the platform layer from 1.1.1, built from the same code on main. Rust builds for Windows directly, without Cygwin or MSYS2. Things to settle first:
    - **The UI toolkit.** GTK 4 runs on Windows, but is heavy to bundle and looks foreign there. Another toolkit would be a second UI crate in the same workspace, on the same `babawus-core`. This is the first decision for Windows.
    - **Sticks** are listed, ejected and formatted through Windows' own interfaces instead of udisks2, and formatting needs administrator rights. Windows has no Ext4, so sticks are exFAT only.
    - **Sticks used on both Linux and Windows:** names that Linux allows and Windows doesn't, paths in the configuration, and how computers and accounts are identified on each.
    - **Development** needs a Windows VM for real testing. Building for Windows from Linux can keep CI checking that the Windows build still compiles.
