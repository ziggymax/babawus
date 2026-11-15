# Babawus (BAsic BAckup With Usb Storage)

*Concept, revision 10 (2026-11-10). Revision 2 added the outcome of a conceptual review: purpose and limits, roles, the life of a medium after setup, and the scope of a first version. Revision 3 added keeping older versions of files. Revision 4 makes media computer and user agnostic and limits them to the exFAT and Ext4 file systems. Revision 5 aligns the concept with the requirements: the configuration file is `.babawus/babawus_config`, the roles are called user and poweruser, the user starts a backup by clicking a medium in a list, and Babawus offers to eject the medium after a successful backup. Revision 6 records the decisions on issues 1–6: computers are recognised by a computer ID instead of their hostname, the layout on the medium is fixed, Ext4 also has name length limits, new copies are written safely, older versions are named by the time they were replaced, and entries are changed only on their own computer and account. Revision 7 records the decisions on issues 7–21: Babawus always formats a medium before use, verdicts are defined by the user's next step, a backup can be cancelled, "last backup" is shown per account, files as well as folders can be selected, entries can be removed, the language is stored per user, and only one Babawus runs per computer. Revision 8 records the decisions on issues 22–30: this document now explains the purpose and the reasons, and exact behaviour is left to `Requirements.md`; the supported platform, the packaging and the rule for dependencies are stated, and the includes are split by role. Revision 9 records the decisions on issues 64–68: the user selects a medium and then presses "Start backup", and Advanced shows the poweruser's actions next to the user's until Babawus is closed. Revision 10 records the decisions on issues 119 and 122: what is set up for one account on one computer is called a backup profile.*

This document explains what Babawus is and why. `Requirements.md` describes exactly how it behaves and defines the terms used. If the two ever disagree, `Requirements.md` wins.

Babawus is a (fairly) simple application for backing up folders from a hard disk to a USB storage device, typically a USB stick. It is, at least initially, intended only for a Linux desktop environment.

## Purpose and limits

### What Babawus protects against

Babawus protects against:

- **Hardware loss**: a dead disk, or a computer that is stolen, broken or destroyed.
- **Accidental deletion** of files.
- **Accidentally overwritten files**: a document saved over or emptied by mistake. The previous version is kept on the medium.
- **Corrupted files**: a damaged file is backed up as a new version, and the previous, good version is kept.

Ransomware is partly covered: encrypted files are backed up as new versions, but the older, good versions survive, as long as the medium was not plugged in during the attack. This is not promised.

### What Babawus does not protect against

The following are deliberately outside the promise:

- **Ransomware**, beyond the partial cover described above.
- **Unlimited history.** Only a limited number of older versions is kept (see "Copying with versions").
- **Anything outside the chosen files and folders.** Babawus backs up the files and folders that were chosen for the medium, nothing more. Email, browser bookmarks and application data often live in hidden places or on a server and are only covered if their folders are chosen explicitly.
- **Unauthorized access to the medium.** Files on the medium are stored in plain, unencrypted form and are readable by anyone who holds the medium, whatever its file system, so they can be restored by simply copying them. The medium must be looked after like the papers it holds.

### Keeping the medium safe

To protect against hardware loss, the medium is kept apart from the computer outside backups. Users are urged to use more than one medium for the same computer, and to use them in turn, so that a lost or failed medium never leaves them without a backup.

## Users and roles

Babawus has two roles:

- **The user** is a non-technical, possibly almost computer-illiterate person. The user only ever runs backups, views the history and views the status of media. The user never has to fix anything.
- **The poweruser** is a more technical person (a relative, a friend, "the computer person"). The poweruser installs, updates and uninstalls Babawus, formats, prepares and changes media, adds computers and user accounts to a medium, and restores files after a loss.

## General use case and concept

### Use case - The KISS of backup systems

The primary purpose of Babawus is to make it very easy for a non-technical person to maintain a backup of important files and folders:

1. The user inserts a USB stick in their computer.
2. The user clicks on the Babawus icon.
3. Babawus starts and shows a list of attached USB media, each with its name and a mark that shows whether it is ready for this user. Selecting one shows its details and status in plain words.
4. The user selects a ready medium in the list and presses "Start backup", and Babawus copies relevant files and folders to it. If only one medium is ready, it is already selected.
5. Babawus presents the user with a clear verdict.

That's it.

### The verdict

Every run ends with a clear, unmistakable verdict that leaves the user in no doubt about the outcome: "All good", "Some files were skipped", "Backup not finished" or "Backup failed". Each verdict leads to a single, clear next step, usually "Contact your poweruser", so the user never has to fix anything themselves. The verdict also says how long it has been since the last backup, and offers a short text the user can copy and send to the poweruser.

A running backup can be cancelled; it then ends with "Backup not finished".

### Backups are manual

Babawus runs only when the user starts it. There is no background process and no reminder while Babawus is closed. Instead, the start screen and the verdict show how long it has been since this user's last backup on this computer ("Last backup: 47 days ago"). Making regular backups a habit is something the poweruser sets up with the user, outside the application.

### Interrupted runs

The user is told at the start of a run not to remove the medium until the verdict appears. If a run is interrupted anyway (the medium is pulled out, the computer is closed or shut down), the run gets its own verdict ("Backup not finished"). The next run simply continues and catches up. Nothing has to be cleaned up by hand.

### The medium is user and computer agnostic

A medium is prepared in such a way that it can be mounted on any computer and read by any user.

This means two things:

- The backups on a medium can be read and restored on any computer, by any user.
- Several computers and user accounts can back up to the same medium. Their backups are always kept apart on the medium, never mixed, so that two computers that each have a "Documents" folder never overwrite each other's files. The verdict and the status show whose backups a medium holds.

A computer is recognised by a computer ID that Babawus derives from the computer's installation, not by its hostname, because many computers keep a default hostname such as `mint`. Each computer added to a medium gets its own folder on the medium, named after its hostname where possible (`mint`, `mint-2`, …), and the configuration maps each computer ID to its folder. User accounts are kept apart by their user name within the computer's folder.

A computer or user account must be added to a medium by the poweruser before it can back up to it. On a computer or account that has not been added, the medium is shown in the list as not set up for this user, with "Contact your poweruser" as the next step, and a backup cannot be started on it.

### Initial preparation of a USB medium for use with Babawus

Babawus only works with media formatted with either an exFAT or Ext4 file system, and it always formats a medium itself before using it for the first time, even if the medium already has one of these file systems. This way every Babawus medium is set up in exactly the same known way. When preparing a new medium, the poweruser chooses exFAT or Ext4 and gives the medium a short name, which Babawus and the file manager both show.

- **exFAT is the recommended default.** It works across almost all platforms, but cannot store some file/folder names (certain characters, and names that differ only in upper and lower case). Such problems are reported in the verdict.
- **Ext4** offers better resilience against file system corruption (for example when a medium is pulled out during a run) and stores any name that Linux allows. Like exFAT, it limits the length of a single name. But it can only be read on Linux. Plugged into a Windows PC or a Mac, an Ext4 medium looks empty or broken, and Windows may even offer to format it, which would wipe the backup. Ext4 is therefore meant for powerusers who know the medium will only ever be used with Linux.

Formatting is a poweruser-only function, reached through "Advanced". Formatting always covers the whole device. Because it erases everything on the medium, including any other partitions, Babawus first shows an unmistakable warning that names the medium and says how much data it holds, and it formats only after explicit confirmation. Formatting may require the administrator password; this is acceptable because only the poweruser formats media.

### Finalizing a USB medium for use with Babawus

The poweruser uses Babawus to prepare the medium, which places a plain text configuration file, `.babawus/babawus_config`, on it. This is the only file Babawus needs to look for. It identifies the medium as a Babawus medium, and holds which computers and user accounts have been added, which files and folders to back up for each of them, what to leave out, and how many older versions to keep. Where the backups go on the medium is fixed, so that is not configured.

What is set up for one user account on one computer is its **backup profile**: what to back up, what to leave out, and the folder its backups go in. A computer holds the backup profiles of its accounts, but is not one itself. Babawus uses this term on screen, because the names of computers and accounts, such as "laptop" or "anna", can read as ordinary words; so it speaks of "the backup profile for “anna”" and "the computer called “laptop”".

The file ends with a checksum, which exists to detect damage (for example a corrupted or half-written file) and accidental edits. It is not a lock. Babawus is the supported way to change the file. Editing the file by hand is not forbidden, but it makes the medium invalid until Babawus has checked it again.

### Changing a medium after setup

The poweruser can change the configuration of a prepared medium at any time, for example to add a folder or an exclusion, or to fix a mistake. Changing the configuration never deletes anything already on the medium.

Adding, linking or changing the entry for a computer and user account is done only while running Babawus on that computer, logged in as that user. Babawus needs that computer's ID, and needs to browse that user's home folder. In practice the poweruser visits each computer and logs in as each user, or has the user log in. Entries for other computers and accounts are shown, but cannot be changed.

The one exception is removing an entry, for example for a retired or lost computer. This can be done from any computer, because it only removes the entry from the configuration: the backups of that computer or account stay on the medium.

While Babawus changes a medium, whether by a backup or a change of configuration, the medium is locked, so that nothing else can change it at the same time. Only one Babawus runs on a computer at a time.

When several media are used for the same computer, each has its own configuration. Keeping them in line is the poweruser's responsibility; Babawus does not synchronize them. To make differences visible, Babawus shows in plain words which files and folders each medium covers.

### After a computer is lost

Restoring is a poweruser task, done by hand: the poweruser mounts the medium and copies the wanted files, folders or older versions to wherever they are needed.

The poweruser can then, on the new computer, either add it to the medium as a new computer, or link it to the lost computer's entry, so backups continue in the same folder. Either way, the backups already on the medium are kept. Linking is also how a computer continues after it has been reinstalled, since a new installation gets a new computer ID.

### Copying with versions

Babawus copies and keeps older versions; it does not mirror.

- The chosen files, and the files in the chosen folders, are copied to the medium. When a newer version of a file is backed up, the newest copy on the medium is replaced, and the previous version is kept as an older version.
- Files deleted from a source folder are never deleted from the medium.
- The poweruser sets a limit on how many older versions are kept, so that the medium does not fill up forever. Babawus removes older versions beyond that limit.
- Babawus never removes the newest copy of any file.
- The newest copy of a file stays where it would be without versioning. Older versions are recognisable by date when browsing the medium in an ordinary file manager, without Babawus.
- A full medium or an interrupted run never leaves a half-written newest copy.
- A file whose name is too long for its older versions to be stored is skipped. The poweruser is expected to help the user keep file and folder names reasonably short.

### Handling of the medium

Using a medium with Babawus is the same as using it without Babawus. The medium is mounted automatically by the system when it is inserted, and Babawus never mounts media. After a successful backup, Babawus offers to eject the medium so the user can remove it right away. Ejecting works like "Safely remove" in the file manager: the medium is unmounted and switched off. The user may also eject it the usual way. The only other time Babawus takes a medium offline is formatting, a poweruser function.

Limits that come from the medium itself (such as file names that exFAT cannot store, or a full medium) are reported in the verdict in plain language. Babawus never changes a file's name to make it fit; a file whose name cannot be stored is skipped.

## General requirements and doctrines

1. The application must present a very simple and easy-to-use graphical interface. This is not an objective requirement, but the point is that the application should be easy to use for someone who is almost computer-illiterate but can manage to start it by inserting a USB stick and clicking an icon titled "Backup". That means a user interface with few buttons, fly-by help on all visible elements, no hidden state or context-dependent UI elements, liberal use of good icons and graphics, and solid, friendly feedback on all relevant user actions.
2. The application should be kept as simple as possible.
3. The application should have as few dependencies as possible. Don't pull in external libraries or frameworks for functionality that can be coded natively in a handful of lines. But using some sort of UI framework is obviously necessary. Libraries that are compiled into the application count as dependencies too: each must be well maintained and replace more than a handful of lines, and the list should stay short.
4. All non-GUI (logic) parts of the application must be covered by tests.
5. Testability and robustness are more important than architectural finesse.
6. Linux Mint 22.x with the Cinnamon desktop on X11 is the supported target platform. Everything else (other Mint editions, Wayland, other modern Linux distributions) is best effort: Babawus should work there, but it is not guaranteed. The target is reviewed when a new major version of Mint comes out.
7. The application must be easy to install, update and uninstall, through a `.deb` package. These are poweruser tasks.
8. The user can change the language of the user interface from inside the application. Initially English and German are available. Babawus starts in the system language if it is available, and remembers each user's choice on the computer. All user-facing text is written so it can be translated into further languages.

## Specific includes and excludes

### Includes

The application must allow the user to:

1. Start a backup to a mounted USB medium that contains a valid `.babawus/babawus_config` file, ending with a clear verdict.
2. View the history of Babawus usage for a medium, which each run logs to a file on the medium.
3. View the status of mounted USB media with respect to Babawus, i.e. whether each is set up for use with Babawus, which files and folders it covers, and when it was last used for a backup.
4. Change the language of the user interface.

The application must allow the poweruser to:

5. Format, set up or change a USB medium for use with Babawus. Setting up and changing means creating or changing a properly formatted `.babawus/babawus_config` file.
6. Add a computer or user account to a medium, link a new or reinstalled computer to an existing entry, or remove an entry, without losing the backups already on it.

Include 1 is the most important use case. The initial UI (the one shown right after start) should only offer the user's functions (includes 1 to 4). The poweruser's functions (includes 5 and 6) only appear after choosing "Advanced" in the menu, as extra actions next to the user's, and disappear again when Babawus is closed. This conflicts somewhat with the "no context-dependent UI elements" doctrine, but it makes it harder for a non-technical user to wander into functions that may cause problems.

### Excludes

- **Restore.** Babawus provides no restore function. To restore, the poweruser simply mounts the medium and copies the wanted files, folders or older versions from it by hand, to wherever they are wanted.
- **Deletion on the medium.** Babawus never deletes files from the medium because they were deleted from a source folder, or because the configuration was changed. The only backups Babawus removes are older versions beyond the limit set by the poweruser. (Babawus also tidies up its own temporary files and locks in `.babawus`.)
- **Mounting.** This is left to the system, except while the poweruser formats a medium. (Ejecting is offered after a successful backup, see "Handling of the medium".)
- **Other file systems.** Babawus only works with exFAT and Ext4 media.
- **Scheduling and reminders.** Babawus runs only when the user starts it.
- **Encryption.** Files on the medium are stored unencrypted.
- **Synchronizing media.** Several media for the same computer are configured independently.

## First version

The first version is done when:

- A user can complete the five steps of the main use case unaided and gets a correct verdict.
- Older versions of files are kept within the limit set by the poweruser, and can be found by date in an ordinary file manager.
- A poweruser can format a medium, set it up, change it, add a computer or user account to it, link one to an existing entry, and remove an entry.
- Backups from different computers and user accounts are kept apart on a shared medium.

Everything else waits for later versions.
