# Babawus Requirements

This document contains the requirements for Babawus. The introductory part provides and overview and provides some basic definitions.

## Introduction

The requirements are generally separated into two different parts:

- A *user centric* part that defines requirements to parts and behaviors of Babawus that users can directly see and/or engage with.
- A *technical* part that defines requirements to parts and behaviors of Babawus that can users can not directly see or engage with, but which are important for clearly defining and removing ambiguities for low level file and folder operations.

While the primary aim of Babawus is to make it as simple as possible to perform simple periodic backup operations, some amount of configuration and administrative functions are needed to set up an usb media for use with Babawus. Thus the requirements for Babawus operates with two roles:

- A **User** only uses Babawus to do backup with media that has already been prepared for use.
- A **Superuser** uses Babawus to (possibly) format sub media and configure these for use with Babawus.

There is no login or some sort of identification of Babawus users, so a user's role is simply defined by the features of Babawus the user wants to use. The two roles are mostly used to provide hints to the implementation of the user interface. A user should be able to use Babawus for doing simple plug-and-play backups without seeing any of the fancy technical stuff a superuser may want/need to use.

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

As a superuser I want Babawus to provide me with the means to format an usb media in one of the 2 supported formats, exFAT or Ext4. Babawus should provide ample warning that this will erase all existing data on the media, provide me opportunity to cancel the operation before anything is changed on the sub media, and only actually format the media after my explicit "yes-I-really-want-to-do-this" acknowledgment. That way I can prepare a new media for use with Babawus, based on my requirements for platform interoperability of the media, wishes for file name and path length compatibility, or resilience against file system corruption.

### Configure usb media

As a superuser I want Babawus to provide me with the means of easily creating and/or editing the Babawus configuration file on an usb media. Specifically:

- Use a "file explorer" of some sort to browse existing files and folders on the PC and select those I want to have backed up.
- Add wildcards to specifically exclude certain files or folder
- Continuously update a sum of files/bytes that are included as I update the configuration.
- Write (with (re-)calculated checksum) the resulting configuration when I am done.

That way I can easily create or update a configuration for an usb media.

### Maintain a log

As a superuser I want Babawus to maintain a log of backup sessions performed on a media. The log is just plain text and should have at least two entries for each backup session:

- A date and time stamp (in local time) for the start of the session
- An entry for each anomaly (error) seen during the session. If more than 20 errors are seen, a single entry with "Multiple errors ...." is written and no more errors are logged (only counted)
- A date and time stamp (in local time) for the end of the session, with summaries of number of files/bytes successfully copied and number of errors

That way I have some good data to help me find the cause of errors.

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

The path of the source files/folders are truncated at the home folder level when they are written to the backup medium. Examples:

- File belonging to userA@hostname1 : `/home/userA/myfile -> /backup/hostname1/userA/myfile`
- File belonging to userB@hostname1 : `/home/userB/Documents/hello.odt -> /backup/hostname1/userB/Documents/hello.odt`
- Folder belonging to userC@hostnam2: `/home/userC/project/hooray -> /backup/hostname2/userC/project/hooray`

### Handling of file versioning

 Babawus never deletes or overwrites a users file. If a newer version of a files needs to be written, this algorithm is used (assume the file is named `userfile.ext`):

 1. It is checked if a subfolder called `userfile[]ext` exists in the folder where `userfile.ext` resides - if not it is created.
 1. It is counted how many copies of `userfile.ext` already exists in `userfile[]ext` (call this number version_count). Version_count may be 0 if no previous versions exists.
 1. The existing version of `userfile.ext` is moved to `userfile[]ext/userfile[version_count+1].ext`.
 1. The new version of `userfile.ext` is written
 
 Here is an example. Before any overwrites:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- File about to be overwritten

A new version of `project_brief.md` needs to be written. Step 1 : create subfolder for versions:

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
                        └── project_brief[1].md <-- File moved here

Step 3: the new version is written:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- New version of file                
                    └── project_brief[]md
                        └── project_brief[1].md

How it looks after a number of version have been written:

    backup/
    └── hostname1
        └── userB
            └── Projects
                └── tadah
                    └── project_brief.md  <-- Newest version of file                
                    └── project_brief[]md
                        └── project_brief[1].md <-- Oldest version of file
                        └── project_brief[2].md
                        └── project_brief[3].md
                        └── project_brief[4].md

### Automatic detection of moved files

Babawus tries to detect situations where files have just been moved around but otherwise not changed. It works like this:100%

1. Babawus maintains a persistent list of hash value for every file that has been backed up.
1. Whenever a new file (i.e. a file that does not already exist in the backup tree of files and folders) is written, a hash value for the file is computed, call it newhash.  
1. If no file with newhash already exists, the file is really new as in never seen before and it simply gets written and newhash is saved in the persistent table.
1. But if newhash already exists in the list of hash values, it means that an identical file already exists somewhere.

The diagram below shows the general flow.
![Flow diagram for new files](assets/Renaming%20flow.png "New file flow diagram")
