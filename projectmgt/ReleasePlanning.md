# Planning the next release

The purpose of this document is to have a place to capture ideas for the next release of Babawus and, in doing so, help in creating and maintaining a general plan for the future work on Babawus.

Some releases may represent so much work and/or complexity that a step-by-step sequence of interim non-released versions is needed to reach the stage where a new release can be made. These non-released versions are an important part of the release planning and hence also something that will be described in this document.

For the remainder of this document the word "version" is used for something that is not planned to be released, and the word "release" is used only for something that is intended for release.

Interim versions are numbered as semver pre-releases of the release they lead to, for example 1.0.0-alpha.1, 1.0.0-alpha.2, and so on. Each interim version is tagged on main, but gets no GitHub release.

When work on a version begins, its items become tasks (T-entries) in the task list.

## Latest release of Babawus

The latest/current release is: 1.2.0 (tag `v1.2.0`), for Linux Mint, Ubuntu and Fedora, with a `.deb` and an `.rpm`. Babawus follows semantic versioning (major.minor.patch).

The latest interim version is: none since 1.2.0. The last was 1.2.0-alpha.2 (tag `v1.2.0-alpha.2`).

## Next release of Babawus

This section always describes the next upcoming version/release of Babawus. Some of the ideas may be very specific, and some more speculative in nature. Task(s) created to deal with speculative ideas will be investigative rather than implementation-oriented.

1.1.0's plan, its three interim versions and their reasons, are kept in the issues (159 to 165) and the tasks (T-089 to T-105). Step 0 of bringing Babawus to more platforms was released as 1.1.1; its plan and decisions are kept in issue 166 and the tasks T-106 to T-114, and the platform layer is described under "Platform layer" in [Requirements.md](Requirements.md). Step 1, other Linux distributions, was released as 1.2.0; its plan, its two interim versions and their decisions are kept in the issues (169, 170 and 174 to 181) and the tasks (T-118 to T-149), and the set-up it was built and tested with is described in [TestSetup.md](../TestSetup.md).

The next releases start bringing Babawus to more platforms: first other Linux distributions, then Windows. They are planned as three steps. Step 0, the platform layer, is done (1.1.1), and so is step 1, other Linux distributions (1.2.0). Step 2, Windows, is under "Future releases".

**One codebase for all platforms.** Every platform, Windows included, is built from the same code on main. There are no long-lived branches per platform. What differs between platforms sits in a small platform layer, with one implementation per platform, chosen when Babawus is built (`#[cfg(target_os = ...)]`). Everything else, such as the backup logic, versioning, the log, the configuration and the language files, is shared. Between the Linux distributions, the differences are expected to be almost entirely in packaging and dependency names, not in the code.

**Containers for building, VMs for the checklist.** Babawus is built and its unit tests run in a container for each supported distribution, locally and in GitHub Actions on every pull request. Containers can't test sticks or a desktop, so each distribution also has a VM. A smoke test installs the package there and tries it on an emulated stick, and the release checklist ([ReleaseChecklist.md](ReleaseChecklist.md)) is run there before each release, with real sticks where a step needs hands.

### Release 1.2.1

1.2.1 fixes the two known issues of 1.2.0: the underscores hidden in the log window on Ubuntu (T-148, issue 178), and the lock for one Babawus per computer, which can't be taken on Fedora and moves to `/dev/shm` (T-150, issue 181). It also makes "Prepare a stick" fit the screen with the Ext4 message showing: the stick to prepare is selected on the start screen, which now also lists the sticks that can't be used yet, and the page no longer lists the drives (T-151, issue 183). Three more things come before its version is raised: the user guide says that Babawus may only show up in the menu after the next login (T-161, issue 187), Help begins with a notice on the messages' numbers (T-162, issue 188), and the log window's highlight and shading can be read on a dark theme (T-163, issue 189). Two faults found while those were checked are fixed too: the log window scrolls to the highlighted run (T-164, issue 190), and Babawus opens dark on Ubuntu when started while the dark style is on (T-165, issue 191). Before its checklists, the test VMs are automated, and the checklists run with the new release test suite (T-166 to T-173, issue 192). There is no interim version. It is released as 1.2.0 was: the smoke test and the whole release checklist on all three guests, the release date, then the tag and the release (T-152 to T-157, issue 182).

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
1. *More automation of the test VMs*. Ideas left out of 1.2.0 (issue 170): an automated test suite for the window, on the same driver; accessible names for the widgets, which such a suite would find them by and which also help screen-reader users; and building the guests from scripts, so that a VM can be made anew without installing by hand. All three are taken up before 1.2.1 (T-166 to T-173, issue 192).
