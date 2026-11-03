# Babawus Requirements

This document contains the requirements for Babawus. The introductory part provides and overview and provides some basic definitions.

## Introduction

The requirements are generally separated into two different parts:

- A *user centric* part that defines requirements to parts and behaviors of Babawus that users can directly see and/or engage with.
- A *technical* part that defines requirements to parts and behaviors of Babawus that can users can not directly see or engage with, but which are important for clearly defining and removing ambiguities for low level file and folder operations.

While the primary aim of Babawus is to make it as simple as possible to perform simple periodic backup operations, some amount of configuration and administrative functions are needed to set up an usb media for use with Babawus. Thus the requirements for Babawus operates with two roles:

- A **User** only uses Babawus to do backup with media that has already been prepared for use.
- A **Poweruser** uses Babawus to (possibly) format sub media and configure these for use with Babawus.

There is no login or some sort of identification of Babawus users, so a user's role is simply defined by the features of Babawus the user wants to use. The two roles are mostly used to provide hints to the implementation of the user interface. A user should be able to use Babawus for doing simple plug-and-play backups without seeing any of the fancy technical stuff a poweruser may want/need to use.

## User stories

### Start Babawus and see status

As a user I want to be able to simply start Babawus by clicking on an icon in a menu or the desktop. Babawus should then start and show me a list with current status of attached media - which are they and are they Babawus-ready (i.e. prepared and ready for use with Babawus), yes or no. That way I can see if Babawus is ready to start a backup.

### Update Babawus media status

As a user I want Babawus to continuously update the list of attached usb media. That way I can add or detach usb media I want use or not-use and see the change reflected in Babawus.

### Start a backup on a specific media

As a user I want to be able to simply click on a Babawus-enabled media in the Babawus list of usb media to start a backup. Babawus should provide me with explicit feedback indicating that the backup has started. That way I cna confidently start a backup.

### Provide feedback during a backup operation

As a user I want Babawus to provide me with continuously information showing me what is happening. That way I can be sure that things are progressing as expected. The info I want to see is:

- After starting the backup, but before any files are transferred, I want to see an overview of the the number of files that will be copied and the sum total of the size of the files (in bytes) to be backed up.
- During the backup process I want Babawus to update counters of remaining files and bytes to backup each time a file has been copied.

### Provide feedback after a backup operation

As a user I want Babawus to show me a summary when a backup process completes. Number of files and bytes successfully copied, a list of files that for some reason could not be copied and any other errors. That way I can know for sure if my backup has run to successful completion.

### Offer to "eject" media after a successful backup operation

As a user I want Babawus to offer to eject a media (unmount it) that has just been used for a successful backup, so that I can immediately detach it. That way I can quickly conclude my backup and store the usb media away.

### Format usb media

As a poweruser I want Babawus to provide me with the means to format an usb media in one of the 2 supported formats, exFAT or Ext4. Babawus should provide ample warning that this will erase all existing data on the media, provide me opportunity to cancel the operation before anything is changed on the sub media, and only actually format the media after my explicit "yes-I-really-want-to-do-this" acknowledgment. That way I can prepare a new media for use with Babawus, based on my requirements for platform interoperability of the media, wishes for file name and path length compatibility, or resilience against file system corruption.

### Configure usb media

As a poweruser I want Babawus to provide me with the means of easily creating and/or editing the Babawus configuration file on an usb media. Specifically:

- Use a "file explorer" of some sort to browse existing files and folders on the PC and select those I want to have backed up.
- Add wildcards to specifically exclude certain files or folder
- Continuously update a sum of files/bytes that are included as I update the configuration.
- Write (with (re-)calculated checksum) the resulting configuration when I am done.

That way I can easily create or update a configuration for an usb media.

### Maintain a log

As a poweruser I want Babawus to maintain a log of backup sessions performed on a media. The log is just plain text and should have at least two entries for each backup session:

- A date and time stamp (in local time) for the start of the session
- An entry for each anomaly (error) seen during the session. If more than 20 errors are seen, a single entry with "Multiple errors ...." is written and no more errors are logged (only counted)
- A date and time stamp (in local time) for the end of the session, with summaries of number of files/bytes successfully copied and number of errors

That way I have some good data to help me find the cause of errors.

### Get a clear verdict

As a user I want every backup run to end with a clear, unmistakable verdict: "All good", "Some files were skipped", "Backup not finished" or "Backup failed". The verdict should also tell me how long it has been since the last successful backup on this medium. If the verdict is not "All good", it should tell me a simple next step I can take without fixing anything myself, typically "Contact your poweruser", with a short text I can read out or forward. That way I am never in doubt about whether my files are safe.

### See when I last made a backup

As a user I want the start screen to show, for each Babawus-ready medium, how long it has been since the last successful backup (for example "Last backup: 47 days ago"). That way I am reminded to make a backup when it has been a while.

### View the history of a medium

As a user I want to be able to view the history of backups made on a medium, in plain words: when each backup ran and what its verdict was. That way I can see that my backups have been running as expected. (The history is read from the log on the medium, see "Maintain a log".)

### Add a computer or user account to a medium

As a poweruser I want to add a computer or a user account to a medium, choosing which folders to back up for it, without losing any backups already on the medium. That way several computers and accounts can share a medium, and I can continue backups on a replacement computer after a loss.

### Rename a computer or user account on a medium

As a poweruser I want to rename a computer or user account on a medium, for example after a computer has been renamed or reinstalled with a new hostname. Babawus renames the folder under `backup` and the entry in the configuration, and keeps all existing backups and older versions. That way backups continue where they left off, instead of starting over under a new name.

### Back up from a computer or account that has not been added

As a user, if I start a backup from a computer or user account that has not been added to the medium, I want Babawus to copy nothing and tell me plainly that the medium is not set up for this computer or user, with "Contact your poweruser" as the next step. That way I am not left wondering why nothing happened.

### Recover from an interrupted backup

As a user I want to be told when a backup starts that I should not remove the medium until the verdict appears. If a backup is interrupted anyway (the medium is pulled out, or the computer is closed or shut down), I want the next backup to simply continue and catch up, without anything having to be cleaned up by hand. That way an accident does not leave me with a broken backup.

### Handle a full medium

As a user, if the medium runs out of space during a backup, I want Babawus to stop cleanly and tell me so in plain words in the verdict, with "Contact your poweruser" as the next step. That way I know my backup is incomplete and what to do about it.

### Use an existing medium as it is

As a poweruser, if a medium is already formatted as exFAT or Ext4, I want the option to prepare it for Babawus without formatting it, keeping any files already on it. That way I do not have to erase a medium that is already suitable.

### Set the number of older versions to keep

As a poweruser I want to set, in the configuration of a medium, how many older versions of each file Babawus keeps (N). That way I can balance history against space on the medium.

### Change the language

As a user I want to change the language of Babawus from inside the application. Initially English and German are available. That way I can use Babawus in the language I understand best.

## Technical requirements

### Layout of a Babawus medium

An usb medium prepared for use with Babawus is formatted in either exFAT or Ext4. In the root of the filesystem there is a folder called `.babawus`. This is where all internal Babawus data is stored (log and configuration). Another top-level folder is called `backup` and this is where all backup files are stored. The backup files are stored in a `<hostname>/<username>` hierarchy.

This figure illustrates the folder layout:

    /media/root-folder
    ├── .babawus
    │   ├── babawus_config          -- The Babawus config file
    │   └── babawus_log.txt         -- Logfile for this media
    └── backup                      
        ├── hostname1
        │   ├── userA/*             -- backup files for userA@hostname1
        │   └── userB/*             -- backup files for userB@hostname1
        └── hostname2
            └── userC/*             -- backup files for userC@hostname2

### Source to target path conversion

Only files and folders inside the user's home folder can be backed up. The home folder is the one the system defines for the user account (the `HOME` setting), which is usually, but not always, `/home/<username>`.

The path of a source file or folder is truncated at the home folder when it is written to the backup medium, and the rest of the path is placed under `backup/<hostname>/<username>`. In the examples below, `<medium>` is the top folder of the mounted medium:

- File belonging to userA@hostname1: `/home/userA/myfile` -> `<medium>/backup/hostname1/userA/myfile`
- File belonging to userB@hostname1: `/home/userB/Documents/hello.odt` -> `<medium>/backup/hostname1/userB/Documents/hello.odt`
- Folder belonging to userC@hostname2: `/home/userC/project/hooray` -> `<medium>/backup/hostname2/userC/project/hooray`

`<hostname>` and `<username>` are the computer's hostname and the account's username at the time of the backup. If either changes, the computer or account is treated as not added to the medium, until the poweruser either adds it or renames the existing entry (see "Rename a computer or user account on a medium").

### Handling of file versioning

Babawus never overwrites the newest copy of a user's file, and never deletes it. Older versions are kept, up to a limit N set by the poweruser in the configuration. If a newer version of a file needs to be written, this algorithm is used (assume the file is named `userfile.ext`):

1. It is checked if a subfolder called `userfile[]ext` exists in the folder where `userfile.ext` resides - if not it is created.
1. The existing version of `userfile.ext` is moved to `userfile[]ext/userfile[<timestamp>].ext`, where `<timestamp>` is the last modification time of that existing version, in the form `YYYY-MM-DD_HHMMSS` (local time). This makes older versions sort by age and recognisable by date in an ordinary file manager.
1. The new version of `userfile.ext` is written.
1. If `userfile[]ext` now holds more than N older versions, the oldest ones are removed until N remain.

Here is an example, with N = 3. Before any overwrites:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- File about to be overwritten

A new version of `project_brief.md` needs to be written. Step 1: create subfolder for versions:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- File to be moved
                    └── project_brief[]md <-- Folder to keep older versions

Step 2: the old version is moved:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief[]md
                        └── project_brief[2026-10-12_093015].md <-- File moved here

Step 3: the new version is written:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- New version of file
                    └── project_brief[]md
                        └── project_brief[2026-10-12_093015].md

Step 4: nothing to remove, since only 1 older version exists (N = 3).

How it looks after a number of versions have been written. Once a fourth older version is moved in, the oldest is removed:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- Newest version of file
                    └── project_brief[]md
                        └── project_brief[2026-10-20_141002].md <-- Oldest version kept
                        └── project_brief[2026-10-27_080544].md
                        └── project_brief[2026-11-02_191230].md <-- Newest older version

### Not in the first version

- **Automatic detection of moved files.** Detecting files that were only moved or renamed, to avoid storing them twice, is deferred to a later version. In the first version, a moved or renamed file is simply backed up again at its new location, and the copy at its old location stays on the medium.
