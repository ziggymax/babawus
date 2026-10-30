# Babawus (BAsic BAckup With Usb Storage)

*Concept, revision 3 (2026-10-30). Revision 2 added the outcome of a conceptual review: purpose and limits, roles, the life of a medium after setup, and the scope of a first version. Revision 3 added keeping older versions of files. Revision 4 makes media computer and user agnostic and limits them to the exFAT and Ext4 file systems.*

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
- **Anything outside the chosen folders.** Babawus backs up the folders that were chosen for the medium, nothing more. Email, browser bookmarks and application data often live in hidden places or on a server and are only covered if their folders are chosen explicitly.
- **Unauthorized access to the medium.** Files on the medium are stored in plain, unencrypted form and are readable by anyone who holds the medium, whatever its file system, so they can be restored by simply copying them. The medium must be looked after like the papers it holds.

### Keeping the medium safe

To protect against hardware loss, the medium is kept apart from the computer outside backup sessions. Users are urged to use more than one medium for the same computer, and to use them in turn, so that a lost or failed medium never leaves them without a backup.

## Users and roles

Babawus has two roles:

- **The end user** is a non-technical, possibly almost computer-illiterate person. The end user only ever runs backups, views the history and views the status of media. The end user never has to fix anything.
- **The helper** is a more technical person (a relative, a friend, "the computer person"). The helper installs, updates and uninstalls Babawus, formats, prepares and changes media, adds computers and user accounts to a medium, and restores files after a loss.

## General use case and concept

### Use case - The KISS of backup systems

The primary purpose of Babawus is to make it very easy for a non-technical person to maintain a backup of important files and folders:

1. The user inserts a USB stick in their computer.
2. The user clicks on the Babawus icon.
3. Babawus starts, recognizes that a Babawus-enabled USB storage medium is present and copies relevant files and folders to the medium.
4. Babawus presents the user with a clear verdict.

That's it.

### The verdict

Every run ends with a clear, unmistakable verdict that leaves the user in no doubt about the outcome, for example "all good", "some files were skipped", "backup not finished" or "backup failed". The verdict also says how long it has been since the last successful backup.

A bad verdict always ends with a simple next step that the end user can take without fixing anything themselves, typically "Contact your helper", ideally with a short text they can read out or forward.

### Backups are manual

Babawus runs only when the user starts it. There is no background process and no reminder while Babawus is closed. Instead, the start screen and the verdict show how long it has been since the last backup ("Last backup: 47 days ago"). Making regular backups a habit is something the helper sets up with the user, outside the application.

### Interrupted runs

The user is told at the start of a run not to remove the medium until the verdict appears. If a run is interrupted anyway (the medium is pulled out, the computer is closed or shut down), the run gets its own verdict ("Backup not finished"). The next run simply continues and catches up. Nothing has to be cleaned up by hand.

### The medium is user and computer agnostic

A medium is prepared in such a way that it can be mounted on any computer and read by any user.

This means two things:

- The backups on a medium can be read and restored on any computer, by any user.
- Several computers and user accounts can back up to the same medium. Their backups are always kept apart on the medium, never mixed, so that two computers that each have a "Documents" folder never overwrite each other's files. The verdict and the status show whose backups a medium holds.

A computer or user account must be added to a medium by the helper before it can back up to it. If the end user starts a backup on a computer or account that has not been added, Babawus copies nothing and says plainly that the medium is not set up for this computer or user, with "Contact your helper" as the next step.

### Initial preparation of a USB medium for use with Babawus

Babawus only works with media formatted with either an exFAT or Ext4 file system. When the helper instructs Babawus to use a medium that has not been used with Babawus before, Babawus offers to format the medium in either exFAT or Ext4 format.

- **exFAT is the recommended default.** It works across almost all platforms, but may cause problems with some file/folder names and path lengths. Such problems are reported in the verdict.
- **Ext4** offers better resilience against file system corruption (for example when a medium is pulled out during a run) and guarantees that no problem with file/folder names or path lengths will occur. But it can only be read on Linux. Plugged into a Windows PC or a Mac, an Ext4 medium looks empty or broken, and Windows may even offer to format it, which would wipe the backup. Ext4 is therefore meant for helpers who know the medium will only ever be used with Linux.

If the new medium is already in one of these two formats, the helper also has the option to use the medium as it is.

Formatting is a helper-only function, reached through "Advanced". Because formatting erases everything on the medium, Babawus first shows an unmistakable warning that names the medium and says how much data it holds, and it formats only after explicit confirmation. Formatting may require the administrator password; this is acceptable because only the helper formats media.

### Finalizing a USB medium for use with Babawus

A USB stick (or other USB storage medium) is prepared for use with Babawus by placing a text file named `babawus_drive.cfg` in its top folder. The file contains:

- A header with a marker that identifies the medium as a Babawus medium.
- Configuration: which folders to back up, and where to put the backups on the medium.
- A validation marker that shows the file has been checked for syntax and correctness.

The helper uses Babawus to prepare the medium. Without going into low-level details, here are some thoughts on the configuration file:

1. The file is always called `babawus_drive.cfg` and it is the only file Babawus needs to look for.
2. The first part of the file (header) holds fixed boilerplate text that explains what the file is and warns against deletion.
3. The second part of the file holds the configuration, such as:
    - For each computer and user account added to the medium: which source folders to back up/copy and where to put them on the USB medium.
    - Exclude patterns/wildcards that specify files and folders NOT to be copied.
    - How many older versions of each file to keep.
4. Finally there is a verifiable marker/checksum at the end of the file indicating that the file has been generated by Babawus and is valid.

The validation marker exists to detect damage (for example a corrupted or half-written file) and accidental edits. It is not a lock. Babawus is the supported way to change the file. Editing the file by hand is not forbidden, but it makes the medium invalid until Babawus has checked it again.

### Changing a medium after setup

The helper can change the configuration of a prepared medium at any time, for example to add a folder or an exclusion, or to fix a mistake. Changing the configuration never deletes anything already on the medium.

When several media are used for the same computer, each has its own configuration. Keeping them in line is the helper's responsibility; Babawus does not synchronize them. To make differences visible, Babawus shows in plain words which folders each medium covers.

### After a computer is lost

Restoring is a helper task, done by hand: the helper mounts the medium and copies the wanted files, folders or older versions to wherever they are needed.

The helper can then add the new computer (and user account) to the medium, so backups can continue without losing the backups already on the medium.

### Copying with versions

Babawus copies and keeps older versions; it does not mirror.

- Files in the source folders are copied to the medium. When a newer version of a file is backed up, the newest copy on the medium is replaced, and the previous version is kept as an older version.
- Files deleted from a source folder are never deleted from the medium.
- The helper sets a limit on how many older versions are kept (for example, the last N versions). Babawus removes older versions beyond that limit, so that the medium does not fill up forever.
- Babawus never removes the newest copy of any file.
- The newest copy of a file stays where it would be without versioning. Older versions are recognisable by date when browsing the medium in an ordinary file manager, without Babawus.

### Handling of the medium

Using a medium with Babawus is the same as using it without Babawus. The medium is mounted automatically by the system when it is inserted, and the user ejects it explicitly before physically removing it. During backups, Babawus neither mounts nor ejects media. The only exception is formatting, a helper function that necessarily takes the medium offline while it runs.

Limits that come from the medium itself (such as file names that exFAT cannot store, or a full medium) are left to the design. Whatever they turn out to be, the verdict reports skipped files and a full medium in plain language.

## General requirements and doctrines

1. The application must present a very simple and easy-to-use graphical interface. This is not an objective requirement, but the point is that the application should be easy to use for someone who is almost computer-illiterate but can manage to start it by inserting a USB stick and clicking an icon titled "Backup". That means a user interface with few buttons, fly-by help on all visible elements, no hidden state or context-dependent UI elements, liberal use of good icons and graphics, and solid, friendly feedback on all relevant user actions.
2. The application should be kept as simple as possible.
3. The application should have as few dependencies as possible. Don't pull in external libraries or frameworks for functionality that can be coded natively in a handful of lines. But using some sort of UI framework is obviously necessary.
4. All non-GUI (logic) parts of the application must be covered by tests.
5. Testability and robustness are more important than architectural finesse.
6. Linux Mint is the supported target platform. Other modern Linux distributions are best effort: Babawus should work there, but it is not guaranteed.
7. The application must be easy to install, update and uninstall, for example through custom scripts or platform-relevant package formats. These are helper tasks.
8. The user can change the language of the user interface from inside the application. Initially English and German are available. All user-facing text is written so it can be translated into further languages.

## Specific includes and excludes

### Includes

The application must allow the user to:

1. Start a backup to a mounted USB medium that contains a valid `babawus_drive.cfg` file, ending with a clear verdict.
2. View the history of Babawus usage for a medium. Each run logs basic data to a file on the medium (date and time of start and completion, outcome, number of source files scanned and copied, files skipped due to errors, data consumption, etc.). The application must make it easy to view this log.
3. View the status of mounted USB media with respect to Babawus, i.e. whether each is set up for use with Babawus, which folders it covers, and when it was last used for a backup.
4. Format, set up or change a USB medium for use with Babawus. Setting up and changing means creating or changing a properly formatted `babawus_drive.cfg` file.
5. Add a computer or user account to a medium, for example a new computer after a loss, without losing the backups already on it.
6. Change the language of the user interface.

Include 1 is the most important use case. The initial UI (the one shown right after start) should only offer starting a backup, viewing the history, seeing the status and changing the language (includes 1, 2, 3 and 6). Includes 4 and 5 are helper functions, reached through a separate "Advanced" menu item or button. This conflicts somewhat with the "no context-dependent UI elements" doctrine, but it makes it harder for a non-technical user to wander into functions that may cause problems.

### Excludes

- **Restore.** Babawus provides no restore function. To restore, the helper simply mounts the medium and copies the wanted files, folders or older versions from it by hand, to wherever they are wanted.
- **Deletion on the medium.** Babawus never deletes files from the medium because they were deleted from a source folder, or because the configuration was changed. The only thing Babawus removes is older versions beyond the limit set by the helper.
- **Mounting and ejecting.** These are left to the system and the user, except while the helper formats a medium.
- **Other file systems.** Babawus only works with exFAT and Ext4 media.
- **Scheduling and reminders.** Babawus runs only when the user starts it.
- **Encryption.** Files on the medium are stored unencrypted.
- **Synchronizing media.** Several media for the same computer are configured independently.

## First version

The first version is done when:

- An end user can complete the four steps of the main use case unaided and gets a correct verdict.
- Older versions of files are kept within the limit set by the helper, and can be found by date in an ordinary file manager.
- A helper can format a medium, set it up, change it, and add a computer or user account to it.
- Backups from different computers and user accounts are kept apart on a shared medium.

Everything else waits for later versions.
