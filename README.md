# Babawus

![Babawus logo](assets/BabawusLogo-NoText-plain.svg)

**BA**sic **BA**ckup **W**ith **U**sb **S**torage

## Contents

- [General concept](#general-concept)
- [Getting your files back](#getting-your-files-back)
- [A reminder at login](#a-reminder-at-login)
- [Adding a language](#adding-a-language)
- [Built-in help](#built-in-help)
- [Choice of file system](#choice-of-file-system)
- [Supported platforms](#supported-platforms)
- [Installing and uninstalling](#installing-and-uninstalling)
- [For developers](#for-developers)
- [Document map](#document-map)
- [Licence](#licence)

Babawus is an application designed to make it easy to maintain backups of important files on detachable USB media (USB sticks). Think of it as the backup solution you might set up for your Gramps. Once it has been set up, all Gramps has to do is (and the order of the first two steps does not matter, should Gramps get confused):

1. Insert their USB stick
2. Start Babawus (just put a link to Babawus on their desktop, or in their dock)
3. Click a button
4. Remove the stick when Babawus says the backup is done.

That's it. The green elephant logo? Because it's cute and elephants never forget.

![The Babawus main window](assets/screenshot-main.png)

For the thinking behind Babawus, who it is for and why it works the way it does, see [Concept.md](projectmgt/Concept.md).

## General concept

All configuration and data are stored on the stick. The aim is ease of use, not complete data security so saved files are not encrypted (yet - may come in a later release). So keep the backup USB media safe somewhere (good advice in any case). But Babawus does protect against:

- **Hardware loss**: a dead disk, or a computer that is stolen, broken or destroyed.
- **Accidental deletion** of files.
- **Accidentally overwritten files**: a document saved over or emptied by mistake. The previous version is kept on the medium.
- **Corrupted files**: a damaged file is backed up as a new version, and the previous, good version is kept.

Ransomware is partly covered: encrypted files are backed up as new versions, but the older, good versions survive, as long as the medium was not plugged in during the attack.

Babawus never deletes your files from the USB stick. The only thing it ever removes is the oldest versions beyond the number you have chosen to keep (10 by default, 1 to 99).

Each stick keeps a log of every backup made to it, so you can see what happened if something went wrong. It is kept below 1 MB, enough for several thousand entries, by removing the oldest runs; the last run of each account is always kept. Babawus has a built-in log viewer, and the log is plain text, so it can also be read in any text editor. [UserGuide.md](UserGuide.md) explains where to find it.

The simple day-to-day backup operations are by design very easy to carry out. The more tricky setup stuff that involves things like formatting USB sticks and tuning settings is kept under Advanced in the menu, behind a "You are entering a dangerous area, are you sure you want to do that?" warning.

## Getting your files back

Choose "Restore files…" for the stick, tick the files and folders you want back, and pick an older version of a file if you need one. You don't even need Babawus for it: the files on the stick are ordinary files in ordinary folders, so you can copy them back with any file manager, on any computer that can read the stick.

## A reminder at login

Babawus can remind a user at login when their last backup is too long ago, and offer to start one. Tick "Remind me at login" in the menu, under Advanced, and at each login Babawus checks whether the last backup is more days ago than configured (7 by default). If not, Babawus quietly closes, but if too many days have passed since the last backup, Babawus asks the user whether a backup should be run. It works on every desktop; [UserGuide.md](UserGuide.md) explains it.

## Adding a language

Babawus comes in English, German and Danish. Other languages can be added as language files, without a new build: [AddingALanguage.md](AddingALanguage.md) explains how.

## Built-in help

There is a built-in help system that provides general info and specifics on all error messages.

## Choice of file system

Babawus allows you to create USB backup media in two different file systems:

- **exFAT** Read- and writeable on almost any platform. Good for cross-platform usage.
- **Ext4** For Linux use only. Stands up better to a medium pulled out during a run (or other mishaps), and stores any name Linux allows.

## Supported platforms

From Babawus 1.2.0, these Linux distributions are supported, each with the desktop it comes with:

| Distribution | Desktop | Package |
| --- | --- | --- |
| Linux Mint 22.x | Cinnamon | `.deb` |
| Ubuntu 24.04 LTS and 26.04 LTS | GNOME | `.deb` |
| Fedora 45 | GNOME | `.rpm` |

Until 1.2.0 is released, the latest release, 1.1.1, runs on Linux Mint 22.x only. Windows is planned; macOS is not.

**How the packages are tested.**

- **Every change** is built, and its automated tests run, on each of these distributions, in GitHub's CI. The tests that format and fill real USB file systems run on Ubuntu 24.04, the base of Linux Mint 22. The `.deb` is installed in clean Ubuntu 24.04 and 26.04, and the `.rpm` in a clean Fedora 45, to check that everything it needs comes with it.
- **Every release** is tested on the very `.deb` and `.rpm` that are released, which GitHub builds. They are installed as a user would in a virtual machine for each distribution, Linux Mint 22.3 (Cinnamon, X11), Ubuntu 26.04 and Fedora 45 (both GNOME, Wayland), and tried there: first by an automated smoke test, which also runs the tests on real file systems, then by hand with the whole [release checklist](projectmgt/ReleaseChecklist.md), on USB sticks. Ubuntu 24.04 has no machine of its own: Linux Mint 22 is built on its packages, so Mint's tests cover them.

Each release's notes say which distributions its files were tested on. [TestSetup.md](TestSetup.md) describes the set-up.

## Installing and uninstalling

Download the package for your distribution from the release on GitHub, and install it:

- **Linux Mint:** double-click the `.deb`, and choose "Install Package".
- **Ubuntu:** double-click the `.deb`. App Center opens, warning that the package comes from outside it; choose "Install".
- **Fedora:** in a terminal, in the folder with the package, run `sudo dnf install ./babawus-<version>-1.x86_64.rpm`. (Fedora's Software can't install a package file that was downloaded.)

Install a newer version the same way: it replaces the old one, and keeps the chosen language and everything on the sticks. Then start Babawus as [UserGuide.md](UserGuide.md) says: it is called "Backup" in the desktop's menus.

To uninstall Babawus, run `sudo apt remove babawus` on Linux Mint and Ubuntu, or `sudo dnf remove babawus` on Fedora. Your backups on the sticks are not touched. A few small files stay behind on the computer; [UserGuide.md](UserGuide.md) lists them and how to remove them.

## For developers

[DevEnvSetup.md](DevEnvSetup.md) explains how to build, test and package Babawus, and how to set up the development environment. [TestSetup.md](TestSetup.md) explains how Babawus is built, tested and released on every distribution: the containers, CI, the VMs and how they are driven.

## Document map

Every document in the project, and who it is for, sorted by who it is for.

| Document | For | What it holds |
| --- | --- | --- |
| [README.md](README.md) | Everyone | This page: what Babawus is, what it protects against, and where to read on |
| [Concept.md](projectmgt/Concept.md) | Everyone | What Babawus is and why: its purpose and limits, who it is for, and its everyday and advanced functions |
| [LICENSE](LICENSE) | Everyone | Babawus's licence, MIT |
| [UserGuide.md](UserGuide.md) | Users | The practical details: starting Babawus on each desktop, setting up the reminder at login, how the files are kept on the stick, where to find a stick's log, and what is left behind after uninstalling |
| [AddingALanguage.md](AddingALanguage.md) | Translators | How to write a language file for a new language and put it in place, without a new build |
| [ReleaseChecklist.md](projectmgt/ReleaseChecklist.md) | Testers | The checklist run by hand on each distribution before each release |
| [DevEnvSetup.md](DevEnvSetup.md) | Developers | The steps for setting up the host and the test VMs, the commands for building and testing, and what was learned along the way |
| [TestSetup.md](TestSetup.md) | Developers | How Babawus is built, tested and released on every distribution: the containers, CI, the packages, the VMs, the driver and the smoke test, and why |
| [Requirements.md](projectmgt/Requirements.md) | Developers | The authoritative description of how Babawus behaves, with the glossary, the user stories and the technical requirements |
| [ReleasePlanning.md](projectmgt/ReleasePlanning.md) | Developers | The latest release, the plan for the next one with its interim versions, and ideas for later releases |
| [Tasks-Open.md](projectmgt/Tasks-Open.md) | Developers | The central list of outstanding work |
| [Tasks-Done.md](projectmgt/Tasks-Done.md) | Developers | The finished tasks, in order, each with the pull request that finished it |
| [Issues-Unsettled.md](projectmgt/Issues-Unsettled.md) | Developers | The design questions that are still open |
| [Issues-Resolved.md](projectmgt/Issues-Resolved.md) | Developers | The design questions that have been decided, with the decisions and the reasons for them |

## Licence

Babawus is under the MIT licence; see [LICENSE](LICENSE).
