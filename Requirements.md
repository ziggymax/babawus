# Babawus Requirements

This document contains the requirements for Babawus. The introductory part provides an overview and some basic definitions.

## Introduction

The requirements are generally separated into two different parts:

- A *user-centric* part that defines requirements for parts and behaviors of Babawus that users can directly see and/or engage with.
- A *technical* part that defines requirements for parts and behaviors of Babawus that users cannot directly see or engage with, but which are important for clearly defining and removing ambiguities for low-level file and folder operations.

While the primary aim of Babawus is to make it as simple as possible to perform simple periodic backup operations, some amount of configuration and administrative functions are needed to set up a USB medium for use with Babawus. Thus the requirements for Babawus operate with two roles:

- A **User** only uses Babawus to do backups with media that have already been prepared for use.
- A **Poweruser** uses Babawus to (possibly) format USB media and configure them for use with Babawus.

There is no login or other identification of Babawus users, so a user's role is simply defined by the features of Babawus the user wants to use. The two roles are mostly used to provide hints to the implementation of the user interface. A user should be able to use Babawus for doing simple plug-and-play backups without seeing any of the fancy technical stuff a poweruser may want/need to use.

## User stories

### Start Babawus and see status

As a user I want to be able to simply start Babawus by clicking on an icon in a menu or the desktop. Babawus should then start and show me a list with the current status of attached media: which media they are, and whether they are Babawus-ready (i.e. prepared and ready for use with Babawus), yes or no. That way I can see if Babawus is ready to start a backup.

### Update Babawus media status

As a user I want Babawus to continuously update the list of attached USB media. That way I can attach or detach USB media I want to use or not use and see the change reflected in Babawus.

### Start a backup on a specific medium

As a user I want to be able to simply click on a Babawus-ready medium in the Babawus list of USB media to start a backup. Babawus should provide me with explicit feedback indicating that the backup has started. That way I can confidently start a backup.

### Provide feedback during a backup operation

As a user I want Babawus to continuously provide me with information showing me what is happening. That way I can be sure that things are progressing as expected. The info I want to see is:

- After starting the backup, but before any files are transferred, I want to see an overview of the number of files that will be copied and the sum total of the size of the files (in bytes) to be backed up.
- During the backup process I want Babawus to update counters of remaining files and bytes to back up each time a file has been copied.

### Provide feedback after a backup operation

As a user I want Babawus to show me a summary when a backup process completes. Number of files and bytes successfully copied, a list of files that for some reason could not be copied and any other errors. That way I can know for sure if my backup has run to successful completion.

### Offer to "eject" media after a successful backup operation

As a user I want Babawus to offer to eject a medium (unmount it) that has just been used for a successful backup, so that I can immediately detach it. That way I can quickly conclude my backup and store the USB medium away.

### Format USB media

As a poweruser I want Babawus to provide me with the means to format a USB medium in one of the two supported formats, exFAT or Ext4. Babawus should provide ample warning that this will erase all existing data on the medium, give me the opportunity to cancel the operation before anything is changed on the USB medium, and only actually format the medium after my explicit "yes-I-really-want-to-do-this" acknowledgment. That way I can prepare a new medium for use with Babawus, based on my requirements for platform interoperability of the medium, wishes for file name and path length compatibility, or resilience against file system corruption.

### Configure USB media

As a poweruser I want Babawus to provide me with the means of easily creating and/or editing the Babawus configuration file on a USB medium. Specifically:

- Use a "file explorer" of some sort to browse existing files and folders on the PC and select those I want to have backed up.
- Add wildcards to specifically exclude certain files or folders.
- Continuously update a sum of files/bytes that are included as I update the configuration.
- Write (with (re-)calculated checksum) the resulting configuration when I am done.
- The entry for a computer and account (adding it, linking it, renaming it, and changing its folders and exclusions) can only be changed while running Babawus on that computer, as that user. Entries for other computers and accounts are shown, but read-only. In practice I visit each computer and log in as each user, or have the user log in.

That way I can easily create or update a configuration for a USB medium.

### Maintain a log

As a poweruser I want Babawus to maintain a log of backup sessions performed on a medium, in `.babawus/babawus_log.txt`. The log is plain text. All timestamps are in local time with the UTC offset, for example `2026-11-03 17:54:00 +0100`, so that entries from computers in different time zones can be compared. Each backup session has these entries:

- A start entry with the timestamp and the `username@computer folder` the backup runs for.
- An entry for each anomaly (error) seen during the session. The first 20 errors are logged individually. If there are more, a single entry "Multiple errors, further errors are only counted" is written, and the rest are only counted.
- An end entry with the timestamp, the verdict, and the number of files/bytes successfully copied and the number of errors.

The log is kept below 1 MB. When it would grow beyond that, the oldest whole sessions are removed from it.

That way I have some good data to help me find the cause of errors.

### Get a clear verdict

As a user I want every backup run to end with a clear, unmistakable verdict: "All good", "Some files were skipped", "Backup not finished" or "Backup failed". The verdict should also tell me how long it has been since the last successful backup on this medium. If the verdict is not "All good", it should tell me a simple next step I can take without fixing anything myself, typically "Contact your poweruser", with a short text I can read out or forward. That way I am never in doubt about whether my files are safe.

### See which folders a medium covers

As a user I want to be able to see, for a Babawus-ready medium, in plain words which folders on this computer are backed up to it for my account. This is shown on request, not on the start screen, which only shows whether each medium is Babawus-ready and, as the one exception, when it was last used for a backup (see "See when I last made a backup"). That way I can check that my important files are covered, and see differences between media.

### See when I last made a backup

As a user I want the start screen to show, for each Babawus-ready medium, how long it has been since the last successful backup (for example "Last backup: 47 days ago"). That way I am reminded to make a backup when it has been a while.

### View the history of a medium

As a user I want to be able to view the history of backups made on a medium, in plain words: when each backup ran and what its verdict was. That way I can see that my backups have been running as expected. (The history is read from the log on the medium, see "Maintain a log".)

### Add a computer or user account to a medium

As a poweruser I want to add a computer or a user account to a medium, choosing which folders to back up for it, without losing any backups already on the medium. I do this on that computer, logged in as that user. Babawus records the computer's ID (see "Identifying a computer") and suggests a folder name for it that is not yet used on the medium. That way several computers and accounts can share a medium.

### Link a computer or user account to an existing entry

As a poweruser I want to link a computer to an existing computer entry on a medium, for example a replacement computer after a loss, or a computer that has been reinstalled and therefore has a new computer ID. Likewise, I want to link a user account to an existing account entry after the account's user name has changed. I do this on that computer, logged in as that user. Babawus updates the computer ID, or renames the user folder under `backup` and the entry in the configuration, and keeps all existing backups and older versions. That way backups continue where they left off, instead of starting over under a new name.

A change of hostname alone needs no action, because computers are recognised by their ID, not their hostname.

### Back up from a computer or account that has not been added

As a user, if I start a backup from a computer or user account that has not been added to the medium, I want Babawus to copy nothing and tell me plainly that the medium is not set up for this computer or user, with "Contact your poweruser" as the next step. That way I am not left wondering why nothing happened.

### Recover from an interrupted backup

As a user I want to be told when a backup starts that I should not remove the medium until the verdict appears. If a backup is interrupted anyway (the medium is pulled out, or the computer is closed or shut down), I want the next backup to simply continue and catch up, without anything having to be cleaned up by hand. That way an accident does not leave me with a broken backup.

### Handle a full medium

As a user, if the medium runs out of space during a backup, or a file to be copied does not fit in the space left (see "Writing a new copy safely"), I want Babawus to stop cleanly, without damaging any copy already on the medium, and tell me so in plain words in the verdict, with "Contact your poweruser" as the next step. That way I know my backup is incomplete and what to do about it.

### Use an existing medium as it is

As a poweruser, if a medium is already formatted as exFAT or Ext4, I want the option to prepare it for Babawus without formatting it, keeping any files already on it. That way I do not have to erase a medium that is already suitable.

### Set the number of older versions to keep

As a poweruser I want to set, in the configuration of a medium, how many older versions of each file Babawus keeps (N). That way I can balance history against space on the medium.

### Change the language

As a user I want to change the language of Babawus from inside the application. Initially English and German are available. That way I can use Babawus in the language I understand best.

## Technical requirements

### Layout of a Babawus medium

A USB medium prepared for use with Babawus is formatted in either exFAT or Ext4. In the root of the filesystem there is a folder called `.babawus`. This is where all internal Babawus data is stored (log, configuration and temporary files). All names that Babawus itself chooses on the medium (`.babawus`, `babawus_config`, `babawus_log.txt`, `tmp`, `backup`, and the computer and user folders) are lower case, and no two of them differ only in case. Another top-level folder is called `backup` and this is where all backup files are stored. The backup files are stored in a `<computer folder>/<username>` hierarchy. The layout is fixed and is not part of the configuration.

This figure illustrates the folder layout:

    /media/root-folder
    ├── .babawus
    │   ├── babawus_config          -- The Babawus config file
    │   ├── babawus_log.txt         -- Logfile for this media
    │   └── tmp/                    -- Temporary files during a backup
    └── backup
        ├── mint
        │   ├── usera/*             -- backup files for usera on the computer with folder "mint"
        │   └── userb/*             -- backup files for userb on the computer with folder "mint"
        └── mint-2
            └── userc/*             -- backup files for userc on the computer with folder "mint-2"

### Source to target path conversion

Only files and folders inside the user's home folder can be backed up. The home folder is the one the system defines for the user account (`$HOME`), which is usually, but not always, `/home/<username>`. The user name is the login name of the account Babawus runs as (the same value that `$USER` normally shows in a terminal).

The path of a source file or folder is truncated at the home folder when it is written to the backup medium, and the rest of the path is placed under `backup/<computer folder>/<username>`. In the examples below, `<medium>` is the top folder of the mounted medium, and the computers have the folders `mint` and `mint-2`:

- File belonging to usera on `mint`: `/home/usera/myfile` -> `<medium>/backup/mint/usera/myfile`
- File belonging to userb on `mint`: `/home/userb/Documents/hello.odt` -> `<medium>/backup/mint/userb/Documents/hello.odt`
- Folder belonging to userc on `mint-2`: `/home/userc/project/hooray` -> `<medium>/backup/mint-2/userc/project/hooray`

`<computer folder>` is the folder name stored in the configuration for the computer's ID (see "Identifying a computer"). `<username>` is the account's user name at the time of the backup, converted to lower case. exFAT does not distinguish upper and lower case in names, so converting to lower case makes both file systems behave the same. If the computer's ID is not in the configuration, or the user name has changed, the computer or account is treated as not added to the medium, until the poweruser either adds it or links it to the existing entry (see "Link a computer or user account to an existing entry").

### Identifying a computer

A computer is identified by a computer ID, not by its hostname, because many computers keep a default hostname (such as `mint`), and two computers with the same hostname must never share a folder on a medium.

The computer ID is computed as HMAC-SHA256, with the contents of `/etc/machine-id` as the key and a fixed application ID defined by Babawus as the message. The first 128 bits are written as 32 lower-case hexadecimal digits. This follows the method systemd recommends for application-specific IDs: the raw machine ID never appears on the medium, and the computer ID cannot be matched with IDs that other applications derive from the same machine ID. A reinstalled computer gets a new machine ID, and therefore a new computer ID.

For each computer added to a medium, the configuration holds:

- The computer ID.
- The name of the computer's folder under `backup`. When a computer is added, Babawus suggests its hostname converted to lower case, and if that name is already used on the medium, the hostname followed by `-2`, `-3` and so on. Folder names are unique on the medium, compared without regard to case.
- The hostname last seen, for display only.
- The user accounts added for that computer, with their folders and exclusions.

### Handling of file versioning

Babawus never overwrites the newest copy of a user's file, and never deletes it. Older versions are kept, up to a limit N set by the poweruser in the configuration.

A source file needs to be backed up again when its size or its last modification time differs from that of its newest copy on the medium. "Differs" is used rather than "is newer", so a file that has been replaced by an older copy is also backed up. To make this work, every copy written to the medium keeps the last modification time of its source file. Modification times are compared with a tolerance of 2 seconds, because exFAT may store them with less precision than the source file system.

The name of a file is split into a base name and an extension at its *last* dot. A name with no dot, or with only a leading dot, has no extension. In the version folder and version names, `[]` and `[<timestamp>]` go where the last dot would be, or at the end of the name if there is no extension:

| Source file | Version folder | Older version |
|---|---|---|
| `report.pdf` | `report[]pdf` | `report[<timestamp>].pdf` |
| `archive.tar.gz` | `archive.tar[]gz` | `archive.tar[<timestamp>].gz` |
| `Makefile` | `Makefile[]` | `Makefile[<timestamp>]` |
| `.bashrc` | `.bashrc[]` | `.bashrc[<timestamp>]` |

Two kinds of name clashes are handled like this:

- A source file or folder whose name has the form of a version folder (contains `[]`) is not backed up, and is listed as skipped in the verdict.
- If an older version with the same timestamp already exists in the version folder, a suffix is added to the timestamp: `report[2026-11-03_175400-2].pdf`, then `-3` and so on.

The timestamp in an older version's name is the time the version was replaced on the medium (the backup time), in the form `YYYY-MM-DD_HHMMSS` (local time). It is not the file's modification time, so the order of the names always matches the order in which versions were added. The file itself keeps its own modification time, which an ordinary file manager shows as usual. So the name says when a version was replaced, and the file says when it was last edited.

When a newer version of a file needs to be written, this algorithm is used (assume the file is named `userfile.ext`):

1. The source file is copied to a temporary file in `.babawus/tmp/` with a unique name, given the source file's modification time, and flushed to the medium (`fsync`). See "Writing a new copy safely".
1. It is checked if a subfolder called `userfile[]ext` exists in the folder where `userfile.ext` resides - if not it is created.
1. The existing version of `userfile.ext` is moved to `userfile[]ext/userfile[<timestamp>].ext`, where `<timestamp>` is the current backup time.
1. The temporary file is renamed to `userfile.ext`.
1. If `userfile[]ext` now holds more than N older versions, the oldest ones are removed until N remain. "Oldest" is decided by the timestamp and suffix read from the names (a version without a suffix is older than one with `-2`, which is older than one with `-3`), not by sorting the names as text.

If the name of the version folder or of the older version would be longer than the medium allows for a single name (255 bytes on Ext4, 255 characters on exFAT), the file is not updated: the existing newest copy stays as it is, the temporary file is removed, and the file is listed as skipped in the verdict. Users are expected to keep file and folder names reasonably short.

Here is an example, with N = 3. Before any overwrites:

    backup/
    └── mint
        └── userb
            └── Projects
                └── tadah
                    └── project_brief.md  <-- File about to be overwritten

A new version of `project_brief.md` needs to be written. Step 1 (copying the new version to `.babawus/tmp/`) is not shown. Step 2: create subfolder for versions:

    backup/
    └── mint
        └── userb
            └── Projects
                └── tadah
                    └── project_brief.md  <-- File to be moved
                    └── project_brief[]md <-- Folder to keep older versions

Step 3: the old version is moved, named with the backup time:

    backup/
    └── mint
        └── userb
            └── Projects
                └── tadah
                    └── project_brief[]md
                        └── project_brief[2026-10-12_093015].md <-- File moved here

Step 4: the temporary file is renamed to the new version:

    backup/
    └── mint
        └── userb
            └── Projects
                └── tadah
                    └── project_brief.md  <-- New version of file
                    └── project_brief[]md
                        └── project_brief[2026-10-12_093015].md

Step 5: nothing to remove, since only 1 older version exists (N = 3).

How it looks after a number of versions have been written. Once a fourth older version is moved in, the oldest is removed:

    backup/
    └── mint
        └── userb
            └── Projects
                └── tadah
                    └── project_brief.md  <-- Newest version of file
                    └── project_brief[]md
                        └── project_brief[2026-10-20_141002].md <-- Oldest version kept
                        └── project_brief[2026-10-27_080544].md
                        └── project_brief[2026-11-02_191230].md <-- Newest older version

### Writing a new copy safely

A copy on the medium is never written in place. Every file is first copied to a temporary file in `.babawus/tmp/` and only then renamed to its final name (see the algorithm in "Handling of file versioning"; a file backed up for the first time skips the version steps). This guarantees:

- **A full medium never damages a copy.** Running out of space can only happen while the temporary file is written, before the existing newest copy has been touched.
- **An interruption leaves no half-written copy.** The only gap is between moving the old copy into the version folder and renaming the temporary file. If a run is interrupted there, the next run finds no newest copy and simply copies the file again.
- **No clutter in the backup folders.** Temporary files never appear among the user's backups.

Before each file is copied, Babawus checks that it fits in the free space left on the medium. If it does not, Babawus stops cleanly and the verdict reports a full medium (see "Handle a full medium").

At the start of every run, Babawus empties `.babawus/tmp/`. These are Babawus's own files, so this does not conflict with the rule that Babawus never deletes backups.

### What is copied

Only regular files and folders are backed up. Symbolic links are neither followed nor copied, and special files (sockets, pipes, device files) are skipped. Neither counts as an error or appears in the verdict. This avoids loops and duplicates, and keeps the behaviour the same on exFAT, which cannot store symbolic links.

Exclusion wildcards are case-sensitive, as file names are on Linux: `*.tmp` excludes `file.tmp` but not `file.TMP`.

On an exFAT medium, which does not distinguish upper and lower case in names, two source files or folders in the same folder whose names differ only in case (for example `Report.pdf` and `report.pdf`) cannot both be stored. Babawus then backs up the one whose name comes first when the names are sorted by their bytes (here `Report.pdf`), and lists the others as skipped in the verdict. On an Ext4 medium all of them are backed up.

### Permissions on Ext4 media

A medium must be readable by any user on any computer. On an Ext4 medium, everything Babawus creates is therefore made readable and writable by everyone: folders get mode `0777` and files get mode `0666`, whatever their permissions in the source. (exFAT does not store owners or permissions, so nothing is needed there.)

### Media that can be formatted

Babawus only offers to format USB devices that the system reports as removable. It never offers internal disks, the system disk, or any disk with a partition that is currently in use other than the medium itself.

### One backup at a time

Only one backup runs at a time. While a backup is running, no other medium in the list can be started. When it has finished, the user can start a backup on another medium.

### Checksum of the configuration file

The last line of `.babawus/babawus_config` holds a checksum of all lines above it, computed with xxHash64 and written as 16 lower-case hexadecimal digits. When the checksum is checked, upper and lower case hexadecimal digits are treated as equal. Babawus computes the checksum when it writes the file and checks it every time it reads the file. If the checksum does not match, the medium is treated as not Babawus-ready until the poweruser has checked and saved the configuration again in Babawus. The checksum detects damage and accidental edits; it is not meant to protect against deliberate changes.

### Not in the first version

- **Automatic detection of moved files.** Detecting files that were only moved or renamed, to avoid storing them twice, is deferred to a later version. In the first version, a moved or renamed file is simply backed up again at its new location, and the copy at its old location stays on the medium.
