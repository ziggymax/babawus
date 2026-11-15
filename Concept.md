# Babawus (Basic Backup with USB Storage)

This document explains what Babawus is and why. [Requirements.md](Requirements.md) describes exactly how it behaves and defines the terms used. If the two ever disagree, Requirements.md wins.

Babawus is a simple application for backing up files and folders from a computer to a USB storage device, typically a USB stick. It is made for the Linux desktop.

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
- **Anything outside the chosen files and folders.** Babawus backs up the files and folders that were chosen for the medium, nothing more. Email, browser bookmarks and application data often live in hidden places or on a server, and are only covered if their folders are chosen explicitly.
- **Unauthorized access to the medium.** Files on the medium are stored in plain, unencrypted form and are readable by anyone who holds the medium, whatever its file system, so they can be restored by simply copying them. The medium must be looked after like the papers it holds.

### Keeping the medium safe

To protect against hardware loss, the medium is kept apart from the computer outside backups. Users are urged to use more than one medium for the same computer, and to use them in turn, so that a lost or failed medium never leaves them without a backup.

## Who Babawus is for

Babawus is made for **the user**: a non-technical, possibly almost computer-illiterate person, who mostly runs backups, views the history and views the status of media. These are Babawus's **everyday functions**.

Setting up media takes more care, and a mistake can cost backups: formatting and preparing a medium, choosing what is backed up, and adding, linking or removing computers and user accounts. These are Babawus's **advanced functions**. They are only shown after choosing "Advanced" in the menu and confirming a warning. The user may do them too, or have someone more technical do them, such as a relative or a friend. Babawus never says who that should be, or where to get help: when something needs fixing, it says what is wrong and where under Advanced it is fixed.

Installing, updating and uninstalling Babawus, and restoring files after a loss, are done outside Babawus.

## How Babawus is used

### Use case: the KISS of backup systems

The primary purpose of Babawus is to make it very easy for a non-technical person to keep a backup of important files and folders:

1. The user plugs a USB stick into their computer.
2. The user clicks the Babawus icon, "Backup".
3. Babawus shows the USB media that are plugged in. Selecting one shows in plain words whether it is ready for this user.
4. The user presses "Start backup", and Babawus copies the chosen files and folders to the medium.
5. Babawus gives the user a clear verdict.

That's it.

### The verdict

Every run ends with a clear, unmistakable verdict that leaves the user in no doubt about the outcome. The verdicts are defined by what the user should do next, and each leads to a single, simple next step, so the user is never left wondering what to do. Each message about a problem carries a number, which leads to its explanation in the built-in Help. Whatever is needed to find out what went wrong is in the log on the medium, so nobody has to copy details down.

### Backups are started by the user

Babawus runs only when the user starts it: it never copies anything in the background. To help make backups a habit, Babawus shows how long it has been since this user's last backup on this computer, and it can be set, under Advanced, to remind the user at login when the last backup is too long ago.

### Interrupted runs

A run can be cancelled, and the medium can be pulled out or the computer shut down during a run. The next run simply continues and catches up. Nothing has to be cleaned up by hand, and a half-written file never replaces a good copy.

## The medium

### One medium, many computers and accounts

A medium can be read on any computer, by any user, so its backups can be restored anywhere.

Several computers and user accounts can back up to the same medium. Their backups are always kept apart, never mixed, so that two computers that each have a "Documents" folder never overwrite each other's files. A computer is recognised by an ID derived from its installation, not by its hostname, because many computers keep a default hostname such as `mint`.

What is set up for one user account on one computer is its **backup profile**: what to back up, what to leave out, how many older versions to keep, and when to remind. Each computer and account must be added to the medium, under Advanced, before it can back up to it; until then, the medium says so, and where that is done.

### Preparing a medium

Babawus only works with media formatted with exFAT or Ext4, and it always formats a medium itself before using it for the first time, so that every Babawus medium is set up in the same known way.

- **exFAT is the recommended default.** It works on almost every platform, but cannot store some file and folder names. Such files are reported in the verdict.
- **Ext4** stands up better to a medium pulled out during a run, and stores any name Linux allows. But it can only be read on Linux: plugged into a Windows PC or a Mac, it looks empty or broken, and Windows may even offer to format it, which would wipe the backup. Ext4 is therefore meant for those who know the medium will only ever be used with Linux.

Formatting erases everything on the medium, so it is an advanced function, and Babawus asks for explicit confirmation after an unmistakable warning.

Preparing a medium places a configuration file on it, which marks it as a Babawus medium and holds its backup profiles. Where the backups go on the medium is fixed, so that it is never a choice to get wrong. Babawus is the supported way to change the file; a checksum detects damage and hand edits.

### Changing a medium after setup

A medium's setup can be changed at any time, under Advanced, for example to add a folder or fix a mistake. Changing the setup never deletes anything already on the medium.

A backup profile is added, linked or changed only on its own computer, logged in as its own user, because Babawus needs that computer's ID and that user's home folder. In practice, whoever sets up the medium visits each computer and logs in as each user, or has the user log in. A backup profile can be removed from any computer, for example for a retired or lost computer: that only removes it from the setup, and its backups stay on the medium.

While Babawus changes a medium, the medium is locked, so that nothing else changes it at the same time. Only one Babawus runs on a computer at a time.

When several media are used for the same computer, each has its own setup. Keeping them in line is up to whoever sets them up; Babawus does not synchronize them. To make differences visible, Babawus shows in plain words which files and folders each medium covers.

### After a computer is lost

Restoring is done by hand, outside Babawus: open the medium in a file manager, and copy the wanted files, folders or older versions to wherever they are needed.

The new computer is then either added to the medium as a new computer, or linked to the lost computer's backup profiles, so that backups continue in the same folders. Either way, the backups already on the medium are kept. Linking is also how a computer continues after it has been reinstalled, since a new installation gets a new ID.

### Copying with versions

Babawus copies and keeps older versions; it does not mirror.

- When a newer version of a file is backed up, the copy on the medium is replaced, and the previous version is kept as an older version.
- Files deleted on the computer are never deleted from the medium.
- Only a limited number of older versions of each file is kept, so that the medium does not fill up forever. The newest copy of a file is never removed.
- The newest copy of a file sits where it would without versioning, and older versions can be told apart by date, so a medium can be browsed and restored from in an ordinary file manager, without Babawus.

### Handling the medium

Using a medium with Babawus is the same as using it without Babawus. The system opens the medium when it is plugged in, and Babawus never does. After a successful backup, Babawus offers to eject the medium, like "Safely remove" in the file manager, so the user can pull it out right away.

Limits that come from the medium itself, such as names that exFAT cannot store, or a full medium, are reported in the verdict in plain words. Babawus never renames a file to make it fit; a file that can't be stored is skipped.

## Doctrines

1. The application must present a very simple and easy-to-use graphical interface. This is not an objective requirement, but the point is that the application should be easy to use for someone who is almost computer-illiterate but can manage to start it by plugging in a USB stick and clicking an icon titled "Backup". That means a user interface with few buttons, a tooltip on every visible element, no hidden state or context-dependent elements, good icons and pictures, and solid, friendly feedback on everything the user does.
2. The application should be kept as simple as possible.
3. The application should have as few dependencies as possible. Don't pull in a library for something that can be written in a handful of lines. A UI framework is obviously needed. Libraries compiled into the application count as dependencies too: each must be well maintained and replace more than a handful of lines, and the list should stay short.
4. All non-GUI (logic) parts of the application must be covered by tests.
5. Testability and robustness are more important than architectural finesse.
6. Linux Mint 22.x with the Cinnamon desktop is the supported platform. Elsewhere Babawus should work, but it is not guaranteed.
7. Babawus is installed, updated and uninstalled through a `.deb` package. They need an administrator's password.
8. The user can change the language from inside Babawus. All text the user sees is written so it can be translated.

## What Babawus does and doesn't do

### Includes

The everyday functions let the user:

1. Start a backup to a Babawus medium, ending with a clear verdict.
2. View the history of backups on a medium.
3. See the status of the media that are plugged in: whether each is ready, which files and folders it covers, and when it was last used for a backup.
4. Change the language.

The advanced functions also let the user:

5. Format a medium, and set it up or change its setup.
6. Add a computer or user account to a medium, link a new or reinstalled computer to existing backup profiles, or remove a backup profile, without losing the backups already on the medium.

Include 1 is the most important use case. Babawus starts with only the everyday functions. The advanced functions appear only after choosing "Advanced" in the menu and confirming a warning that they can erase a stick and cost backups, as extra actions next to the everyday ones, and disappear again when Babawus is closed. This bends the "no context-dependent elements" doctrine, but it makes it harder for a non-technical user to wander into functions that may cause problems, and the warning lets them turn back.

### Excludes

- **Restore.** Babawus has no restore function. To restore, the wanted files, folders or older versions are copied from the medium by hand.
- **Deletion on the medium.** Babawus never deletes backups because files were deleted on the computer, or because the setup was changed. The only backups it removes are older versions beyond the limit.
- **Mounting.** Opening media is left to the system.
- **Other file systems.** Babawus only works with exFAT and Ext4 media.
- **Scheduled backups.** Babawus never starts a backup by itself; at most, it reminds the user at login.
- **Encryption.** Files on the medium are stored unencrypted.
- **Synchronizing media.** Several media for the same computer are set up independently.
