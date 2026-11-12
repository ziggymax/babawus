# Babawus Requirements

This document contains the requirements for Babawus. The introductory part provides an overview and some basic definitions.

This document is the authoritative description of how Babawus behaves. `Concept.md` explains what Babawus is and why. If the two ever disagree, this document wins.

## Introduction

The requirements are generally separated into two different parts:

- A *user-centric* part that defines requirements for parts and behaviors of Babawus that users can directly see and/or engage with.
- A *technical* part that defines requirements for parts and behaviors of Babawus that users cannot directly see or engage with, but which are important for clearly defining and removing ambiguities for low-level file and folder operations.

While the primary aim of Babawus is to make it as simple as possible to perform simple periodic backup operations, some amount of configuration and administrative functions are needed to set up a USB medium for use with Babawus. Thus the requirements for Babawus operate with two roles:

- A **User** only uses Babawus to do backups with media that have already been prepared for use.
- A **Poweruser** uses Babawus to format USB media and configure them for use with Babawus.

There is no login or other identification of Babawus users, so a user's role is simply defined by the features of Babawus the user wants to use. The two roles are mostly used to provide hints to the implementation of the user interface. A user should be able to use Babawus for doing simple plug-and-play backups without seeing any of the fancy technical stuff a poweruser may want/need to use.

## Glossary

These terms have a fixed meaning in this document and in `Concept.md`.

| Term | Meaning |
|---|---|
| Account | A user account on a computer, identified by its user name. Also called a user account. |
| Advanced | The menu item that shows the poweruser's actions until Babawus is closed (see "Start Babawus and see status"). |
| Backup | A file or folder copied to a medium, or the act of making those copies (see *run*). |
| Backup profile | What is set up for one account on one computer: its files and folders, its exclusions and its folder on the medium. It is an account's entry, inside its computer's entry. A computer is not a backup profile. The screen uses this term, never "entry" (Issues.md items 119 and 122). |
| Babawus medium | A medium that Babawus has formatted and set up, recognised by its configuration file. |
| Checksum | The last line of the configuration file, used to detect damage and accidental edits. It is also the file's validation marker. |
| Computer | One installation of Linux, identified by its computer ID. |
| Computer folder | A computer's folder under `backup` on a medium, for example `mint-2`. |
| Computer ID | The hashed ID that identifies a computer (see "Identifying a computer"). |
| Configuration | The contents of `.babawus/babawus_config`. |
| Eject | Unmounting a medium and powering off the device. |
| Entry | A computer or account in the configuration. On screen, an account's entry is a *backup profile*, and a computer's entry is the computer. |
| Exclusion | A name pattern or a specific folder that is left out of a backup. |
| Format version | The value of the `format` field, which states the version of the configuration file format. |
| History | The list of past runs on a medium, read from the log. |
| Home folder | The folder the system defines for an account (`$HOME`). |
| Label | The short name a medium gets when it is formatted, for example `BACKUP-A`. |
| Link | Connecting a computer or account to an existing entry, so that backups continue in the same folder. |
| Lock | Either the medium's lock (`.babawus/lock`), held by any action that writes to the medium, or the lock that allows only one Babawus per computer. |
| Log | The file `.babawus/babawus_log.txt`, which records every run on a medium. |
| Medium (plural media) | A USB storage device, typically a USB stick, whether it has been prepared by Babawus or not. |
| N | The limit on how many older versions of each file are kept on a medium. |
| Newest copy | The copy of a file at its normal place on the medium. |
| Older version | A replaced copy of a file, kept in a version folder. |
| Poweruser | The more technical person, who formats and configures media (see "Introduction"). |
| Pruning | Removing older versions beyond the limit N. |
| Ready | The status of a medium this user can back up to on this computer. |
| Reason | The plain-words explanation that comes with a verdict or a skipped file. In the log it is written as a fixed reason code, for example `MEDIUM_FULL`. |
| Run | One backup from start to verdict. Each run has a run ID. |
| Selected files and folders | The files and folders chosen for an account in the configuration. |
| Skipped file | A file that should have been backed up but could not be, and is listed in the verdict. Excluded files, symbolic links and special files are not skipped files. |
| Source file | A file on the computer that is to be backed up. |
| Status | The plain-words state of a medium in the media list. |
| Temporary file | A file in `.babawus/tmp/`, used while writing a new copy or trimming the log. |
| User | The non-technical person who runs backups (see "Introduction"). |
| Verdict | One of the four outcomes of a run: All good, Some files were skipped, Backup not finished, Backup failed. |
| Version folder | The folder that holds a file's older versions, for example `report[]pdf`. |

## User stories

### Start Babawus and see status

As a user I want to be able to simply start Babawus by clicking on an icon in a menu or the desktop. Babawus should then start and show me a list with the current status of attached media: which media they are, and whether they are ready for me to back up to. That way I can see if Babawus is ready to start a backup.

The window is titled "Backup". The list of media is on the left. The details of the selected medium are on the right, with the actions for it below them. A menu button in the title bar holds Language, Advanced and About. About shows the logo without the name, the version, and where the name comes from: "**Ba**sic **Ba**ckup **w**ith **U**SB **S**torage", with the letters that spell Babawus in bold, in every language; in any language other than English, that language's own comment follows on the line below, for example "Einfache Sicherung auf USB-Sticks" (Issues.md item 132). The window is uncluttered. It opens at 1024 × 768, title bar included, which assumes a screen of at least Full HD (1920 × 1080); at that size no page needs scrolling to reach its buttons or read its text (Issues.md item 118). It can be made smaller, down to what each screen needs, and a page that scrolls then starts at the top whenever it opens or changes part. Every control has a tooltip.

The list has one row for each mounted file system on a USB medium (as defined in "Formatting a medium"). Media that are not mounted are not shown, because Babawus never mounts media. Each row shows only a large picture of a stick, the medium's label in bold (for example **BACKUP-A**), and a mark (Issues.md item 120). Everything else is in the details. The stick is in colour for a Babawus medium and grey for anything else. The mark shows at a glance whether a backup can be started. Its shape differs as well as its colour, so it can be read without telling colours apart:

| Status | When | Mark |
|---|---|---|
| Ready | A Babawus medium, and this computer and account have been added to it | Green tick |
| Not set up for you | A Babawus medium, but this computer or account has not been added | Amber "!" |
| Backup in progress (by `<user>`) | Another Babawus holds the lock on the medium (see "One Babawus at a time") | Amber "!" |
| Needs attention | The configuration is damaged (see "Reading the configuration file") or has a newer format than this Babawus understands | Amber "!" |
| Write-protected | A Babawus medium that is mounted read-only | Grey cross |
| Open for another account | Mounted for another account, so that this account can't reach it, for example at `/media/anna/BACKUP-A` after Anna logged out. What is on it is unknown, so the stick is grey, and neither "Prepare…" nor "Change setup…" is offered. The details say "This stick is still open for another account on this computer, so Babawus can't look at it. If nobody else is using it, pull it out and plug it back in." (Issues.md item 110) | Amber "!" |
| Not a backup medium | Anything else | Grey cross |

Working out a status reads the medium, so it happens off the main thread. Until it is known, the row shows a small spinner in place of the mark, and the details say "Checking…".

**Selecting.** Clicking a row selects it, and never starts anything. If nothing is selected and exactly one medium is Ready, that medium is selected, so the usual case needs only a press on "Start backup". The selection follows the medium while the list changes. When the selected medium is removed, nothing is selected.

**The details** show a small Babawus logo, with the name below the elephant, in their upper right corner, out of the way of everything else. From the top they show the label as their heading; the maker and model, the size, the free space and the file system (for example "Kingston DataTraveler 3.0 · 15.5 GB · 9.8 GB free · exFAT", for every user; Issues.md item 133); a thin line; and the explanation. Sizes are in GB, as on the packaging. The explanation starts with the status and its mark, for example "✓ Ready", on a line of its own. Below it, a Ready medium shows the last backup (see "See when I last made a backup"), and any other status says in plain words what it means and what to do, for example "This stick hasn't been set up for you on this computer. Contact your poweruser." Every other message about the medium goes there too, such as "Ejecting BACKUP-A…". The explanation has one text style throughout, and the mark makes the status stand out (`ui-review/UiReview.md` item G01).

**The empty page.** With nothing selected, the details only say "Plug in your backup stick", or "Choose a stick in the list" if the list isn't empty. When a USB stick is plugged in but not listed, they say instead "The stick isn't opened", with "Pull it out and plug it back in." below it, for a stick whose file system isn't mounted; or "The stick that is plugged in can't be used yet", with "Your poweruser can prepare it.", for a stick with no file system Babawus can open (Issues.md item 121).

**The actions** for the selected medium:

| Action | Shown for | Role |
|---|---|---|
| Start backup | Every medium. It only works on a Ready medium: otherwise it is greyed out, and its tooltip says why | User |
| Show history | A Babawus medium whose log has at least one run (see "View the history of a medium") | User |
| What's backed up | A Ready medium (see "See which files and folders a medium covers") | User |
| Eject | Every medium (see "Offer to eject media after a successful backup operation") | User |
| Prepare… | A medium that may be formatted (see "Format USB media") | Poweruser |
| Change setup… | A Babawus medium (see "Configure USB media") | Poweruser |

The buttons keep their own width. At 1024 × 768 the user's four fit on one line, in German too; when there isn't room, they wrap like words, each close to the one before (Issues.md item 125). Eject comes last among the user's buttons, after a small gap. The poweruser's buttons sit on a row of their own, below a thin line.

**Advanced.** The poweruser's actions only appear after choosing Advanced in the menu. Advanced stays on until it is chosen again or Babawus is closed. It is always off when Babawus starts, and is never saved, so the next user never meets the poweruser's actions by chance. While it is on, the title bar shows a clear marker, so that nobody forgets it is on.


### Update Babawus media status

As a user I want Babawus to continuously update the list of attached USB media. That way I can attach or detach USB media I want to use or not use and see the change reflected in Babawus.

### Start a backup on a specific medium

As a user I want to be able to simply select a Ready medium in the Babawus list of USB media and press "Start backup" to start a backup. Babawus should provide me with explicit feedback indicating that the backup has started. That way I can confidently start a backup.

### Provide feedback during a backup operation

As a user I want Babawus to continuously provide me with information showing me what is happening. That way I can be sure that things are progressing as expected. The info I want to see is:

- After starting the backup, but before any files are transferred, I want to see an overview of the number of files that will be copied and the sum total of the size of the files (in bytes) to be backed up.
- During the backup process I want Babawus to update counters of remaining files and bytes to back up each time a file has been copied. The byte counter also moves while a large file is being copied, so that a large file on a slow medium does not look as if Babawus has hung. It counts bytes that have reached the medium, not bytes that are still waiting in the computer's memory (see "Writing a new copy safely").

While a backup runs, the backup screen fills the window, so that nothing else can be started or ejected meanwhile. It shows the medium's label, what is going on ("Finding the files to back up…", "Backing up…" or "Finishing… Don't pull out the stick yet."), the overview, the counters, and a bar that shows how far the copying has got by bytes.

I also want a Cancel button, in case I regret starting a long backup. Babawus asks me to confirm. If it is still copying a file to the temporary folder, it stops right away and discards the temporary file, so that a very large file does not delay the cancel. Otherwise it first finishes the steps for the file it is working on (so it never stops between moving the old copy and renaming the new one, see "Handling of file versioning"). It then ends with the verdict "Backup not finished". Cancel can't be pressed while the run is finishing, because by then it is quicker to let it end. If the run ends while Babawus is asking, the question goes away. Closing the window during a backup asks the same question. If I agree to stop, the window closes once the run has ended properly, with its end entry in the log; otherwise the backup goes on.

### Provide feedback after a backup operation

As a user I want Babawus to show me a summary when a backup process completes. Number of files and bytes successfully copied, a list of files that for some reason could not be copied and any other errors. That way I can know for sure if my backup has run to successful completion.

### Offer to "eject" media after a successful backup operation

As a user I want Babawus to offer to eject a medium that has just been used for a successful backup, so that I can immediately detach it. Ejecting works like "Safely remove" in the file manager: Babawus unmounts the medium and then powers off the device, so its light goes out and the system no longer shows it. If powering off is not possible, Babawus only unmounts it, and says that the medium can still be pulled out safely. If a program is still using the medium, for example a terminal whose current folder is on it, or a file being copied, Babawus leaves it mounted and asks me to close that program, or wait until it has finished, and try again. A file manager window showing the medium doesn't keep it in use: it closes when the medium is ejected (Issues.md item 129). Only USB media are ever ejected. That way I can quickly conclude my backup and store the USB medium away. Eject is also one of the actions on the start screen, for any medium in the list, so that I can remove a medium safely without a backup too. How ejecting went is said in a pop-up with an OK button, on whichever screen Eject was pressed: "You can pull out BACKUP-A now.", or why it could not be ejected. While it goes on, the screen says "Ejecting BACKUP-A…". After a successful eject from the verdict screen, closing the pop-up returns to the start screen; after a failed one, the verdict screen stays, so that Eject can be tried again (Issues.md item 128).

### Format USB media

As a poweruser I want Babawus to provide me with the means to format a USB medium in one of the two supported formats, exFAT or Ext4. Babawus should provide ample warning that this will erase all existing data on the medium, give me the opportunity to cancel the operation before anything is changed on the USB medium, and only actually format the medium after my explicit "yes-I-really-want-to-do-this" acknowledgment. That way I can prepare a new medium for use with Babawus, based on my requirements for platform interoperability of the medium, wishes for file name and path length compatibility, or resilience against file system corruption.

Babawus always formats a medium before using it for the first time; a medium is never prepared "as it is". Formatting covers the whole device, and the warning says that any other partitions on it are erased too, and that formatting cannot be cancelled once it has started. While formatting, I give the medium a label (see "Formatting a medium").

"Prepare…" opens the format view, which fills the window. It lists every drive Babawus may format, mounted or not, each with its maker, model and size, and what is on it now ("BACKUP-A (exFAT), with Babawus backups", "DATA (NTFS), not mounted", or "No file system"). The selected medium's drive is chosen already, if it may be formatted. While Advanced is on, "Prepare…" is shown whenever a drive may be formatted, also on the empty page, so that a blank or unmounted stick can be reached. Below the list come the label, with the suggested one filled in and its rule shown when it is broken, and the file system: exFAT ("Also works on Windows and Mac", chosen already) or Ext4 ("Works on Linux only, but allows longer names and copes better with being pulled out"). Formatting Ext4 needs an administrator's password (see "Formatting a medium"), so only an administrator, an account in the `sudo` or `admin` group, can choose it. For anyone else it is greyed out, with "Only an administrator can choose Ext4, because formatting it needs an administrator's password. Choose exFAT, or ask an administrator to prepare the stick." When an administrator chooses it, "Ext4 asks for your password while formatting." shows below it (Issues.md items 108 and 109). The warning names the drive and what will be erased, including every partition, and says that formatting can't be stopped once started. "Erase and format" is red, and only works once I have ticked "I understand that everything on this stick will be erased". Choosing another drive clears the tick.

While formatting, the view shows "Formatting BACKUP-B…" and that it may take a few minutes, with no way back, and the window can't be closed. Then it says "BACKUP-B is ready", and that the stick must now be set up for each computer and account, with "Add this computer and account…" (see "Configure USB media") and "Back", which returns to the start screen with the new medium selected. If formatting fails, it says why, whether the stick was left unchanged or must be formatted again, and what the system said, with "Try again" and "Back".

### Configure USB media

As a poweruser I want Babawus to provide me with the means of easily creating and/or editing the Babawus configuration file on a USB medium. Specifically:

- Use a "file explorer" of some sort to browse existing files and folders on the PC and select those I want to have backed up. Both files and folders can be selected: a selected folder is backed up with everything in it, a selected file on its own.
- Exclude files and folders in two ways: name patterns (see "Exclusions"), and specific folders that I pick in the same file explorer.
- When an account is added, the exclusions start with a visible, editable list of suggestions: the name patterns `.cache`, `*.tmp` and `*~`, and the excluded folder `.local/share/Trash`. There are no hidden built-in exclusions.
- A file or folder whose path is not valid UTF-8 cannot be selected or excluded by folder, because the configuration file is UTF-8. The file explorer shows it, but refuses it and says why. It can still be backed up as part of a selected folder above it, on Ext4.
- Continuously update a sum of files/bytes that are included as I update the configuration.
- Set the number of older versions to keep (see "Set the number of older versions to keep").
- Write (with (re-)calculated checksum) the resulting configuration when I am done.
- The entry for a computer and account (adding it, linking it, and changing its files, folders and exclusions) can only be changed while running Babawus on that computer, as that user. Entries for other computers and accounts are shown, but read-only. In practice I visit each computer and log in as each user, or have the user log in. The one exception is removing an entry (see "Remove a computer or user account from a medium").

That way I can easily create or update a configuration for a USB medium.

"Change setup…" opens the overview, which fills the window. It shows the number of older versions to keep, this computer and account, and every backup profile on the medium, under "Backup profiles on this stick", grouped by computer, each with how many files or folders it has chosen. In sentences, questions and buttons, names are quoted and say what they name: "the computer called “laptop”", "the backup profile for “anna”", "Link to “laptop”…" (Issues.md item 122). If this computer isn't in the configuration, the overview offers "Add this computer and your account", and says the folder it will be added as. If other computers are there, it also offers to link it to one of them. If the computer is there but this account isn't, it offers to add the account, or to link it to another account on this computer whose user name it had before. Every entry, this one or another, can be removed. "Choose what to back up…" opens a full-window screen for this account, described below. Changes wait for "Save", which takes the medium's lock, writes the file and releases the lock, and says "Stick setup saved." (Issues.md item 127) Leaving or closing the window with unsaved changes asks first. Removing an entry and linking a computer ask for confirmation, and wait for "Save". Linking an account also asks, and saves at once, because it renames the account's folder and its name in the log. If the medium's lock is live, saving says whose backup is running and changes nothing.

A damaged configuration that can be parsed opens with a warning to check it and save it, and "Save" works without a change. One that can't be parsed shows "Start a new setup…" instead, which asks first, and then opens an empty configuration that replaces the old file when saved. A configuration that needs a newer Babawus can't be changed. The format view's "Add this computer and your account…" opens the overview of the new medium.

"Choose what to back up…" is headed "Choose what to back up for `<user>`", and shows a tree of the home folder, with the home folder itself at the top, open. The tree fills the free height and is the only part that scrolls, so that at the window's opening size nothing else needs scrolling (`ui-review/UiReview.md` T01). Each folder opens when clicked, and each row has a tick box. Ticking a file or folder selects it. A ticked folder ticks everything in it. A folder's box is partly ticked when it is ticked but something inside it has been unticked, or it isn't ticked but something inside it has been ticked; what a name pattern leaves out doesn't count (Issues.md item 123). Unticking a folder inside a ticked one leaves it out (an excluded folder), and its name is crossed out; ticking it again takes the exclusion back. A file inside a ticked folder can't be unticked on its own. Something left out by a name pattern, or inside a left-out folder, is crossed out with the reason, and can't be ticked. A folder on another drive inside a ticked folder is not ticked, and says it is only backed up if ticked on its own. Links can't be ticked, and neither can a name that isn't valid text. Hidden entries only show with "Show hidden files", unless something in them is chosen. Below the tree come the totals ("Chosen: 3 files (500.0 kB) · 15.5 GB free on the stick"), counted in the background and started over on each change, with a warning when they are more than the free space. Then a section "Files to leave out by name" (Issues.md item 126), closed at first, with the name patterns, each with "Remove", and a field with "Add". A message about a pattern that can't be added goes as soon as the field changes. Changing the language or showing hidden files keeps the open folders open. "Done" returns to the overview, where the changes wait for "Save"; "Cancel" returns without them.

### Maintain a log

As a poweruser I want Babawus to maintain a log of the runs performed on a medium, in `.babawus/babawus_log.txt`. The log is plain text (see "Format of the log"). All timestamps are in local time with the UTC offset, for example `2026-11-03 17:54:00 +0100`, so that entries from computers in different time zones can be compared. Each run has these entries:

- A start entry with the timestamp, the `username@computer folder` the backup runs for, the version of Babawus, and the computer's hostname.
- An entry for each anomaly (error) seen during the run, including each skipped file, with an error code and the path of the file relative to the home folder. The first 100 errors are logged individually (Issues.md item 124). If there are more, a single entry "Multiple errors, further errors are only counted" is written, and the rest are only counted.
- An end entry with the timestamp, the verdict, the reason, the number of files scanned, the number of files and bytes successfully copied, the number of files skipped, the number of errors, the number of older versions removed, and the free space left on the medium.

The log is kept below 1 MB (1,000,000 bytes). When it has grown beyond 900,000 bytes at the start of a run, the oldest whole runs are removed from it until it is back within 900,000 bytes, which leaves room for the run that follows. The most recent run of each account that counts as a backup (see "See when I last made a backup") is always kept.

That way I have some good data to help me find the cause of errors.

### Get a clear verdict

As a user I want every backup run to end with a clear, unmistakable verdict, with a single next step I can take without fixing anything myself. That way I am never in doubt about whether my files are safe. The verdicts are defined by what I should do next:

| Verdict | When | Next step |
|---|---|---|
| All good | Everything that needed copying was copied | None |
| Some files were skipped | The run finished, but files were skipped (for example unreadable, name cannot be stored, case clash, version name too long) | Contact your poweruser |
| Backup not finished | The run was interrupted: the medium was removed, the computer was shut down, or I cancelled | Run the backup again. If it happens again, contact your poweruser. If I stopped it myself: run the backup again when it suits me |
| Backup failed | The run cannot succeed without the poweruser: the medium is full, the computer or account has not been added, the configuration is damaged or outdated, or the medium cannot be written to | Contact your poweruser |

If several of these happen in one run, the worst verdict wins: Backup failed, then Backup not finished, then Some files were skipped, then All good.

Every verdict other than "All good" gives the reason in plain words (for example "the medium is full" or "the medium was removed"). Skipped files are grouped by reason (for example "3 files have characters in their names that this medium can't store"). The verdict also tells me how long it has been since my last backup (see "See when I last made a backup").

The log is the one place for diagnosing a run. Users don't copy details for the poweruser: the log is always there, whether or not anyone did anything, and holds what the poweruser needs, in fixed codes that read the same whatever the language (Issues.md item 117).

The verdict screen takes the place of the backup screen when the run ends. It shows a mark next to the verdict: a green tick for "All good", an amber "!" for "Some files were skipped" and "Backup not finished", and a red cross for "Backup failed". Below it come the reason, the skipped files by reason, the next step, what was copied ("Copied: 12 files (3.4 MB)", or "Everything was already up to date."), and the last backup. "Some files were skipped" needs no reason below it, because it says it already. The screen has three text styles: the label, the verdict with its mark, and one style for everything below, in which only the next step is bold (`ui-review/UiReview.md` items G02, V03 and V05). When the run counts as a backup, an "Eject" button offers to eject the medium (see "Offer to eject media after a successful backup operation"). "Back" returns to the start screen.

### See which files and folders a medium covers

As a user I want to be able to see, for a Ready medium, in plain words which files and folders on this computer are backed up to it for my account. This is shown on request, through "What's backed up", not in the details on the start screen, which only show the medium itself, its status and, for a Ready medium, when it was last used for a backup (see "See when I last made a backup"). That way I can check that my important files are covered, and see differences between media.

"What's backed up" fills the window with the files and folders selected for my account, in the order of the configuration, each with a folder or file picture. Babawus looks each one up on this computer. One from which nothing would be backed up is marked with an amber "!" and says why: it doesn't exist any more, it is left out by an exclusion, it is outside my home folder, it is a link, or it can't be read. Below come the exclusions, under "Left out": the excluded folders, then the name patterns in one line ("Anything named .cache, *.tmp or *~, in any folder"). If nothing is selected, it says so and to contact the poweruser. "Back" returns to the start screen.

### See when I last made a backup

As a user I want the details of a Ready medium on the start screen to show how long it has been since my last backup on this computer to that medium, and when it finished (for example "Last backup: today at 14:03", "Last backup: yesterday at 09:12" or "Last backup: 47 days ago (2026-09-23 at 17:54)"). That way I am reminded to make a backup when it has been a while, and can tell which of several backups on one day was the last.

- Days are calendar days in local time, so a backup at 23:50 reads "yesterday" just after midnight. A time in the future, from a wrong clock, counts as today (see Issues.md item 93).

- Only my own account on this computer counts (`username@computer folder`), not backups by other computers or accounts on a shared medium.
- Both "All good" and "Some files were skipped" count as a backup, because the run finished. If the last backup skipped files, a small warning mark is shown next to the date.
- Before my first backup to the medium, "No backup yet" is shown.
- The date is read from the log on the medium.

### View the history of a medium

As a user I want to be able to view the history of backups made on a medium, in plain words: when each backup ran and what its verdict was. That way I can see that my backups have been running as expected. (The history is read from the log on the medium, see "Maintain a log".)

"Show history" fills the window with the runs on the medium, newest first. Each run shows the verdict with its mark (as on the verdict screen), when it started ("Today, 14:03", "Yesterday, 09:12" or "2026-11-03, 17:54", counting calendar days in local time), the reason for any verdict but "All good", what was copied (files and bytes), how many files were skipped, and whose run it was ("By anna on laptop"). Every run on the medium is listed, whichever computer or account made it. A run that is still going, by whoever holds the medium's lock, shows "Backup in progress". "Back" returns to the start screen.

### Add a computer or user account to a medium

As a poweruser I want to add a computer or a user account to a medium, choosing which files and folders to back up for it, without losing any backups already on the medium. I do this on that computer, logged in as that user. Babawus records the computer's ID (see "Identifying a computer") and suggests a folder name for it that is not yet used on the medium. That way several computers and accounts can share a medium.

### Link a computer or user account to an existing entry

As a poweruser I want to link a computer to an existing computer entry on a medium, for example a replacement computer after a loss, or a computer that has been reinstalled and therefore has a new computer ID. Likewise, I want to link a user account to an existing account entry after the account's user name has changed. I do this on that computer, logged in as that user. Babawus updates the computer ID, or renames the user folder under `backup`, the entry in the configuration and the account's name in the log, and keeps all existing backups and older versions. That way backups continue where they left off, instead of starting over under a new name, and "Last backup" and the history still show the account's earlier runs.

Linking an account refuses to overwrite: if the new user folder already exists on the medium and the old one does too, Babawus stops and changes nothing, and the poweruser resolves it by hand. The folder and the log are changed first and the configuration last, so an interrupted link can simply be repeated.

A change of hostname alone needs no action, because computers are recognised by their ID, not their hostname.

### Remove a computer or user account from a medium

As a poweruser I want to remove a computer or user account from a medium, for example a retired or lost computer. Babawus removes the entry from the configuration only. The backups of that computer or account stay on the medium, and I can delete them by hand if I want the space back. Unlike other changes to an entry, removing it can be done from any computer, because the computer being removed may no longer exist, and nothing is deleted. That way the configuration stays tidy without risking any backups.

### Back up from a computer or account that has not been added

As a user, if the computer or user account I am using has not been added to a medium, I want the media list to show this plainly ("Not set up for you: contact your poweruser"), and not let me start a backup on that medium. That way I see why before I try, and know what to do. (If the configuration changes between showing the list and starting the backup, the run copies nothing and ends with "Backup failed", with the reason that the medium is not set up for this computer or user. The same goes for a medium that has lost its `.babawus` folder or configuration file by then. A configuration file that can't be used, because it is damaged or needs a newer Babawus, ends the same way, with the reason that the configuration is damaged or outdated. Either way, such a medium may mean anything to this Babawus, so the run leaves it untouched: no lock and no log entry. If the medium's root folder has gone, the medium was removed, and the verdict is "Backup not finished".)

### Recover from an interrupted backup

As a user I want to be told when a backup starts that I should not remove the medium until the verdict appears. If a backup is interrupted anyway (the medium is pulled out, or the computer is closed or shut down), I want the next backup to simply continue and catch up, without anything having to be cleaned up by hand. That way an accident does not leave me with a broken backup.

The history shows an interrupted run as "Backup not finished" (see "Interrupted runs in the log").

### Handle a full medium

As a user, if the medium runs out of space during a backup, or a file to be copied does not fit in the space left (see "Writing a new copy safely"), I want Babawus to stop cleanly, without damaging any copy already on the medium, and tell me so in plain words in the verdict, with "Contact your poweruser" as the next step. That way I know my backup is incomplete and what to do about it.

### Set the number of older versions to keep

As a poweruser I want to set, in the configuration of a medium, how many older versions of each file Babawus keeps (N). There is one N for the whole medium, shared by all computers and accounts, because it is about space on the medium. N is between 1 and 99, and 10 by default. N = 0 is not allowed, because it would remove the protection against accidentally overwritten files. When I lower N, the extra versions are removed on the next backup (see "Handling of file versioning"). That way I can balance history against space on the medium.

### Change the language

As a user I want to change the language of Babawus from inside the application. Initially English and German are available. The change takes effect at once, without a restart. Babawus remembers my choice for my account on this computer, in `~/.config/babawus/settings.toml` (following `$XDG_CONFIG_HOME` if set). Until I have made a choice, Babawus uses the system language if it is available, otherwise English. If the system lists several languages, the first one that is available counts. The file holds `language = "en"` or `language = "de"`. A settings file that is missing or can't be understood counts as no choice, and if the choice can't be saved, it still applies until Babawus is closed. The settings file is never written in place. Files on the medium (the configuration header and the log) are always in English; Babawus translates what it shows from them. The language is chosen in the menu in the title bar. German addresses the user as "du", and calls the poweruser "Betreuer" (see Issues.md items 90-92). Where texts are longer, as in German, the action buttons wrap onto another line, so that the window needn't grow. That way I can use Babawus in the language I understand best.

## Technical requirements

### Layout of a Babawus medium

A USB medium prepared for use with Babawus is formatted in either exFAT or Ext4. In the root of the filesystem there is a folder called `.babawus`. This is where all internal Babawus data is stored (log, configuration, lock and temporary files). All names that Babawus itself chooses on the medium (`.babawus`, `babawus_config`, `babawus_log.txt`, `lock`, `tmp`, `backup`, and the computer and user folders) are lower case, and no two of them differ only in case. Another top-level folder is called `backup` and this is where all backup files are stored. The backup files are stored in a `<computer folder>/<username>` hierarchy. The layout is fixed and is not part of the configuration.

This figure illustrates the folder layout:

    /media/root-folder
    ├── .babawus
    │   ├── babawus_config          -- The Babawus config file
    │   ├── babawus_log.txt         -- Logfile for this media
    │   ├── lock                    -- Present only while Babawus changes the medium
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

The computer ID is computed as HMAC-SHA256. The key is the 32 hexadecimal digits in `/etc/machine-id`, as text and without the line ending. The message is the fixed application ID `io.github.ziggymax.Babawus`, as text. The first 128 bits of the result are written as 32 lower-case hexadecimal digits. Neither the key's form nor the application ID may ever change, because that would disconnect every computer from every medium. If `/etc/machine-id` does not hold 32 hexadecimal digits, the computer has no computer ID, and it is treated as not added to any medium. This follows the method systemd recommends for application-specific IDs: the raw machine ID never appears on the medium, and the computer ID cannot be matched with IDs that other applications derive from the same machine ID. A reinstalled computer gets a new machine ID, and therefore a new computer ID.

For each computer added to a medium, the configuration holds:

- The computer ID.
- The name of the computer's folder under `backup`. When a computer is added, Babawus suggests its hostname converted to lower case (or `computer`, if the hostname would not make a valid folder name), and if that name is already used on the medium, the name followed by `-2`, `-3` and so on. A name counts as used if it is in the configuration *or* a folder of that name already exists under `backup` (for example from a removed entry), so a new computer never ends up in an old one's folder. Folder names are unique on the medium, compared without regard to case.
- The hostname last seen, for display only. A backup saves it if it has changed, for a computer that has been added. If that fails, the backup goes on, its verdict stays the same, and the next backup tries again.
- The user accounts added for that computer, with their selected files and folders and their exclusions.

### Handling of file versioning

Babawus never overwrites the newest copy of a user's file, and never deletes it. Older versions are kept, up to a limit N set by the poweruser in the configuration.

A source file needs to be backed up again when its size or its last modification time differs from that of its newest copy on the medium. "Differs" is used rather than "is newer", so a file that has been replaced by an older copy is also backed up. To make this work, every copy written to the medium keeps the last modification time of its source file. Modification times are compared with a tolerance of 2 seconds, because exFAT may store them with less precision than the source file system. exFAT also can't store times before 1980 or after 2107, and moves them to the nearest time it can store. On exFAT, Babawus therefore compares such a time as that nearest time. Otherwise the file would look changed on every run, and each new copy would push out a real older version.

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

The timestamp in an older version's name is the time the version was replaced on the medium (the backup time), in the form `YYYY-MM-DD_HHMMSS`, in UTC. It is not the file's modification time, so the order of the names matches the order in which versions were added, as long as the computer's clock is right. UTC is used because local time can go backwards, when the clocks change in autumn or when a computer moves west, and pruning would then remove the wrong version. This means that a name can differ from the local time by some hours. The file itself keeps its own modification time, which an ordinary file manager shows as usual, in local time. So the name says when a version was replaced, and the file says when it was last edited.

When a newer version of a file needs to be written, this algorithm is used (assume the file is named `userfile.ext`):

1. The source file is copied to a temporary file in `.babawus/tmp/` with a unique name, given the source file's modification time, and flushed to the medium (`fsync`) as it is copied. See "Writing a new copy safely".
1. It is checked if a subfolder called `userfile[]ext` exists in the folder where `userfile.ext` resides - if not it is created.
1. The existing version of `userfile.ext` is moved to `userfile[]ext/userfile[<timestamp>].ext`, where `<timestamp>` is the current backup time.
1. The temporary file is renamed to `userfile.ext`.
1. If `userfile[]ext` now holds more than N older versions, the oldest ones are removed until N remain. "Oldest" is decided by the timestamp and suffix read from the names (a version without a suffix is older than one with `-2`, which is older than one with `-3`), not by sorting the names as text.

An older version that cannot be removed is tried again on the next run. It is logged and counted as an error with the code `PRUNE_FAILED`, but it does not change the verdict: every file is safe, and the only cost is some space.

If the name of the version folder or of the older version would be longer than the medium allows for a single name (255 bytes on Ext4, 255 characters on exFAT), the file is not updated: the existing newest copy stays as it is, the temporary file is removed, and the file is listed as skipped in the verdict. Users are expected to keep file and folder names reasonably short.

Pruning (step 5) is not limited to files that get a new version. In every run, whenever Babawus visits a source file that has a version folder on the medium, it prunes that folder down to N, even if the file itself has not changed. So when the poweruser lowers N, the space comes back on the next run. Version folders of files that were deleted from the source are not visited, so they keep their versions, in line with the rule that deleted files are never removed from the medium.

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

A file is copied in chunks of 4 MiB, and each chunk is flushed to the medium (`fsync`) before the next one is written. Without this, writes only reach the computer's memory: a 512 MiB file was "written" in under a second, and then took 20 seconds to reach a real stick. Only flushed bytes count towards the byte counter. Flushing in chunks made no measurable difference to the speed.

Before each file is copied, Babawus checks that it fits in the free space left on the medium. If it does not, Babawus stops cleanly and the verdict reports a full medium (see "Handle a full medium").

At the start of every run, Babawus empties `.babawus/tmp/`. These are Babawus's own files, so this does not conflict with the rule that Babawus never deletes backups.

**Finishing a run.** Flushing each file does not cover everything: renames and folder changes can still be waiting in the computer's memory. So at the end of a run, Babawus first flushes the whole medium (`syncfs`), then writes the end entry to the log and flushes it, and only then shows the verdict. This can take several seconds, during which the screen shows "Finishing…". Once the verdict appears, everything is on the medium, and pulling it out cannot lose anything. If the medium cannot be flushed, or the end entry cannot be written, the verdict is "Backup not finished", with the reason `FLUSH_FAILED`. Without an end entry, a later run reports this run the same way.

### What is copied

Only regular files and folders are backed up, including empty folders. Symbolic links are neither followed nor copied, and special files (sockets, pipes, device files) are skipped. Neither counts as an error or appears in the verdict. This avoids loops and duplicates, and keeps the behaviour the same on exFAT, which cannot store symbolic links.

Below a selected folder, Babawus does not enter folders that belong to another file system (for example cloud folders, network shares or other disks mounted inside the home folder). A selected file or folder may itself be on any file system.

- **A selection that does not exist any more** is skipped with the code `SELECTION_MISSING`, so that the poweruser learns that something is no longer being backed up.
- **A selection inside something excluded** stays excluded, because an excluded folder is skipped with everything in it.
- **Overlapping selections** (a folder, and a file in it) are backed up once.
- **A selection with `..` in its path** is skipped with the code `OUTSIDE_HOME`, so that a damaged configuration can never write outside the account's folder on the medium.

On an exFAT medium, which does not distinguish upper and lower case in names, two source files or folders in the same folder whose names differ only in case (for example `Report.pdf` and `report.pdf`) cannot both be stored. Babawus then backs up the one whose name comes first when the names are sorted by their bytes (here `Report.pdf`), and lists the others as skipped in the verdict, with the code `CASE_CLASH`. On an Ext4 medium all of them are backed up. exFAT compares names with its own table of upper-case letters, which is not the same as Unicode's. Babawus uses a rule that covers every pair in that table and a few more, so it may very rarely report a clash between names that exFAT could have stored (`ı.txt` and `I.txt`), but never misses one.

### Exclusions

Each account has its own exclusions, stored with the account's entry in the configuration. There are two kinds:

- **Name patterns.** A pattern may use `*` (any characters), `?` (one character) and `[…]` (one of the listed characters), and is matched against a single file or folder *name*, at any depth. For example, `*.tmp` excludes every file ending in `.tmp`, and `node_modules` excludes every folder of that name. Patterns cannot contain `/`. An excluded folder is skipped with everything in it.
- **Excluded folders.** Specific folders, picked in the file explorer, and stored as exact paths relative to the home folder (for example `Documents/old`). An excluded folder is skipped with everything in it.

Exclusions are case-sensitive, as file names are on Linux: `*.tmp` excludes `file.tmp` but not `file.TMP`. Excluded files and folders do not count as skipped and do not appear in the verdict.

### Names that cannot be stored

Before a file or folder is copied, Babawus checks whether its name can be stored on the medium, using a fixed set of rules for each file system. Names that fail are skipped, with one of these reason codes:

| Code | Rule | File system |
|---|---|---|
| `NAME_INVALID_CHARS` | The name contains one of `"` `*` `:` `<` `>` `?` `\` `\|` or a control character below `0x20` | exFAT |
| `NAME_TRAILING_DOT` | The name ends with a dot (the Linux exFAT driver silently drops it, so two different names could end up the same) | exFAT |
| `NAME_NOT_UTF8` | The name is not valid UTF-8 and cannot be converted to the UTF-16 names exFAT uses | exFAT |
| `PATH_TOO_LONG` | The full path on the medium would exceed the system's path limit (4096 bytes) | Both |
| `CASE_CLASH` | The name differs only in case from one that comes first (see "What is copied") | exFAT |
| `VERSION_FOLDER_FORM` | The name has the form of a version folder (see "Handling of file versioning") | Both |

Any other error that the file system reports while writing is also caught, and the file is skipped with a general code. If a *folder* is skipped, everything in it is skipped too, and it is reported as one entry with the number of files in it. In the verdict, skipped files are grouped by reason in plain words.

### Changes in the source

- **A file becomes a folder** with the same name: the old file on the medium is moved into its version folder, as an ordinary older version, and then the folder is created and backed up.
- **A folder becomes a file** with the same name: the file is skipped with the code `TYPE_CONFLICT`, and the poweruser resolves it by hand. Versioning a whole folder is not done, because pruning it later would delete backups of files that may no longer exist in the source.
- **A file renamed only in case** (for example `report.pdf` → `Report.pdf`): on exFAT, Babawus looks up names on the medium without regard to case, and treats the renamed file as the same file. The next time the file changes, the new copy takes the source's current case; until then the old case stays on the medium. On Ext4, a case change is an ordinary rename: the file is backed up under its new name, and the old copy stays on the medium.
- **A file that changes during copying:** Babawus compares the file's size and modification time before and after copying. If they differ, it discards the temporary file and tries once more. If the file changes again, it is skipped with the code `CHANGED_DURING_COPY` ("the file was in use").
- **A file that cannot be read** (no permission, or a read error): it is skipped with the code `UNREADABLE`.
- **A file or folder that disappears during a run** (it was found, and is gone when Babawus gets to it): it is skipped with the code `DISAPPEARED`. Any copy on the medium stays as it is.
- **A folder on the medium that cannot be written:** the affected files are skipped with the code `NOT_WRITABLE`. If `.babawus` itself cannot be written, the run ends with "Backup failed", with the reason that the medium cannot be written to.

### Permissions on Ext4 media

A medium must be readable by any user on any computer. On an Ext4 medium, everything Babawus creates is therefore made readable and writable by everyone: folders get mode `0777` and files get mode `0666`, whatever their permissions in the source. (exFAT does not store owners or permissions, so nothing is needed there.)

The root folder of a new Ext4 file system belongs to `root`. This is handled once, while formatting (see "Formatting a medium"). `lost+found` is left alone, and is never part of a backup.

### Formatting a medium

A **USB medium** is a drive that udisks2 reports with `Removable` set and `ConnectionBus` set to `usb`. `MediaRemovable` is not used, because it is false for a USB stick, whose medium cannot be taken out of the drive.

Babawus only offers to format USB media. It never offers internal disks, the system disk, or any disk with a partition that is currently in use other than the medium itself. A partition is in use if it is active swap, if another device sits on it (such as encryption, LVM or RAID), or if it is mounted anywhere other than below `/media/` or `/run/media/`, where the desktop mounts removable media. A disk that udisks2 marks as a system disk (`HintSystem`) is never offered either. The format view lists devices whether they are mounted or not.

Babawus always identifies a drive by its udisks2 drive ID, never by a device name such as `/dev/sda`: those names can change between boots, and the system disk can then get the name a USB stick had before. Right after the poweruser has confirmed, and before anything is changed, Babawus checks that the drive with that ID is still present and still meets the rules above. If not, it stops and changes nothing.

Babawus always formats a medium before using it for the first time; there is no way to prepare a medium "as it is". Formatting goes through the udisks2 service (see "Technology"), and works like this:

1. Every mounted file system on the device is unmounted.
2. The signatures of the old partitions are wiped, so that the new partition, which starts where an old one did, does not show the old file system: the desktop would mount it while it is being formatted. Then a new partition table is written to the whole device: MBR, or GPT if the device is larger than 2 TiB. It holds one partition that spans the whole device, with the type that matches the file system:

    | File system | MBR | GPT |
    |---|---|---|
    | exFAT | `0x07` | Microsoft basic data |
    | Ext4 | `0x83` | Linux file system |

3. The file system is created on that partition, with the label the poweruser has entered. The label has 1 to 11 characters (the exFAT limit), using the letters `A`–`Z` and `a`–`z`, digits and `-`. Babawus suggests the first of `BACKUP-A` to `BACKUP-Z`, then `BACKUP-1`, `BACKUP-2` and so on, that no attached medium has, ignoring case.
    - **exFAT:** the defaults of `mkfs.exfat`.
    - **Ext4:** no reserved blocks (`-m 0`), and full initialisation during formatting (`-E lazy_itable_init=0,lazy_journal_init=0`), so that the medium is finished when formatting is, and Ext4 does not keep initialising in the background after the first mount. These options are passed on through the `mkfs-args` option of udisks2. The formatting user is made the owner of the root folder (the `take-ownership` option of udisks2), so that Babawus can do step 5 as an ordinary user. udisks2 asks for an administrator's password for both options, which is why only an administrator can choose Ext4. exFAT needs neither, and on Mint the user of the active desktop session can format it without a password.
4. The medium is mounted through udisks2.
5. On Ext4 only:
    - The root folder gets mode `0777`.
    - A default ACL is set on the root folder (the equivalent of `setfacl -m d:u::rwx,d:g::rwx,d:o::rwx`), so that files that other programs create on the medium are also readable and writable by everyone. Babawus still sets modes explicitly on everything it creates, because a computer may mount the medium without ACL support. This relies on ACL support being a default mount option, which `mkfs.ext4` sets on Mint 22.
6. Babawus creates `.babawus` and `backup` (mode `0777` on Ext4) and writes a new configuration, with no computers yet. The medium then shows as "Not set up for you" until the poweruser adds this computer and account.

Formatting cannot be cancelled once step 1 has started. Ext4 takes about a minute on a 16 GB stick, and longer on larger media, while exFAT is almost instant.

### One Babawus at a time

**One Babawus per computer.** Only one Babawus window runs on a computer at a time, whichever user started it. Babawus holds a lock on the file `/run/lock/babawus.lock`, which the system releases automatically when the process ends, even after a crash. All users share this file, which holds the user name of the Babawus that has the lock. If the same user starts Babawus again, the existing window is brought to the front. If another user starts it (for example with fast user switching), Babawus shows a message with its logo (without the name), "Babawus is already open for `<user>` on this computer.", "Ask them to close it, or switch to their account.", and an OK button that closes it, and does not open a second window (`ui-review/UiReview.md` W06). If the lock can't be used at all, Babawus opens anyway, because each medium has a lock of its own.

**One backup at a time.** While a backup is running, no other medium in the list can be started. When it has finished, the user can start a backup on another medium.

**One lock per medium.** Every action that writes to a medium (a backup, changing the configuration, adding, linking or removing an entry) first takes the medium's lock, `.babawus/lock`, and releases it when done. While the lock is held, no other Babawus can do anything that writes to that medium.

- The lock file is created in a single operation that fails if it already exists. After writing it, Babawus reads it back, and only goes on if it still holds its own run ID. Otherwise another Babawus got in between, and the medium counts as busy. It records the run ID, `username@computer folder`, the computer ID, the system's boot ID and the process ID, in TOML with the keys `run_id`, `account`, `computer_id`, `boot_id` and `process_id`.
- A medium can only be in one computer at a time. So a lock is only *live* if it has this computer's ID, the current boot ID, and a process that is still running. Any other lock is left over from an interrupted run, and Babawus removes it. An interruption therefore never leaves a medium locked.
- A running process only counts if it is Babawus, because the system can give a process ID to another program after a crash. A lock with Babawus's own process ID only counts while that action is still going, because a medium can be pulled and put back while Babawus stays open.
- A lock file that can't be understood isn't live.
- The lock is not flushed to the medium, when taken or when removed, because a lock that isn't live stops nobody.
- In the media list, a medium with a live lock shows "Backup in progress (by `<user>`)", and a backup can't be started on it. If the lock has become live since the list was shown, a backup that is started does nothing, gives no verdict, and the list shows the same.
- After a backup, the lock is removed after the end entry has been written to the log.

### Format of the configuration file

`.babawus/babawus_config` is a TOML file, encoded in UTF-8, with LF line endings and a line ending after the last line.

- The first line is the fixed marker comment `# Babawus configuration`, which identifies the file as a Babawus configuration. It is followed by the boilerplate header as comments, in English.
- The field `format = 1` states the version of the file format. If a file has a newer format than this Babawus understands, the medium is shown as "Needs attention" in the media list (with the hint that a newer Babawus is needed), and Babawus never writes to it.
- The configuration itself: N, and for each computer its computer ID, folder name and last seen hostname, and for each of its accounts the user name, the selected files and folders, and the exclusions.
- The last line is the checksum, written as a TOML comment, so the file stays valid TOML. The checksum is the validation marker; there is no separate one.

For example:

    # Babawus configuration
    # This file is written by Babawus. Change it with the Advanced functions in Babawus.
    # If it is edited by hand, the medium needs attention until it has been saved in Babawus.
    format = 1
    older_versions = 10

    [[computer]]
    id = "0123456789abcdef0123456789abcdef"
    folder = "mint"
    hostname = "mint"

    [[computer.account]]
    username = "usera"
    selected = ["Documents", "notes.txt"]
    exclude_patterns = [".cache", "*.tmp", "*~"]
    exclude_folders = [".local/share/Trash"]

    # checksum: 0123456789abcdef

### Reading the configuration file

A medium that has a file at `.babawus/babawus_config` is a Babawus medium, whatever the file holds. The file is read in this order:

1. If it starts with the marker and has a newer `format` than this Babawus understands, the medium needs attention, with the hint that a newer Babawus is needed. The rest of the file is not looked at, because a newer format may mean anything. `format` is found by looking at the lines before the first table, so this works even if the rest of the file can't be parsed.
2. The file is parsed and its values are checked (see below), and the checksum and the marker are checked.

If everything is in order, the configuration is used. Otherwise the configuration is damaged, and the medium needs attention:

- **If the file can be parsed and its values are valid**, the poweruser sees the configuration in Babawus, checks it and saves it, which writes a new checksum.
- **If not**, nothing in the file can be trusted, so the poweruser is offered to start a new configuration. That only replaces the file: all backups stay on the medium, and folders already under `backup` still count as used when new entries are added.

These values are checked. Anything else makes the file invalid, including a field that Babawus does not know:

- `format` is a whole number from 1 up, and N is from 1 to 99.
- A computer ID is 32 lower-case hexadecimal digits, and no two computers have the same one.
- A computer folder is a single, lower-case name that can be stored on exFAT (see "Names that cannot be stored"), not `.` or `..`, and no two differ only in case.
- A user name is a single name that can be stored on exFAT, and no two accounts on a computer differ only in case, because they would share a folder.
- A name pattern is not empty and does not contain `/`.
- An excluded folder is a relative path with no `.` or `..` in it.

Selected files and folders are not checked here: a selection with `..` in its path is dealt with during the run (see "What is copied"), so that the poweruser learns of it.

### Writing the configuration file

A configuration is never written in place. It is written to a temporary file in `.babawus/tmp/`, flushed, renamed over the old one, and the medium is then flushed. So an interruption leaves either the old configuration or the new one, never a damaged one. On Ext4, the file gets mode `0666`, like everything else Babawus creates.

### Checksum of the configuration file

The last line of `.babawus/babawus_config` holds a checksum of all lines above it, computed with SHA-256 (the same hash the computer ID uses, so no extra code is needed) and written as the first 16 lower-case hexadecimal digits, in a comment line. When the checksum is checked, upper and lower case hexadecimal digits are treated as equal. Babawus computes the checksum when it writes the file and checks it every time it reads the file. If the checksum does not match, the medium is shown as "Needs attention" until the poweruser has checked and saved the configuration again in Babawus. The checksum detects damage and accidental edits; it is not meant to protect against deliberate changes.

### Format of the log

`.babawus/babawus_log.txt` is plain text, encoded in UTF-8, with one entry per line. The fields of an entry are separated by tabs:

1. The timestamp (local time with the UTC offset).
2. The run ID, which is the same for all entries of one run.
3. The entry type: `START`, `ERROR` or `END`.
4. Any number of `key=value` pairs, for example `verdict=FAILED`, `reason=MEDIUM_FULL`, `copied_files=1234`.
5. Optionally, a short English text for anyone reading the file directly. It is a fixed text from Babawus, without `=`, which is how it is told apart from the pairs.

The run ID is 16 random lower-case hexadecimal digits.

Verdicts, reasons and error codes are written as fixed, language-neutral codes. The history view translates them into the user's language.

| Entry | Fields |
|---|---|
| `START` | `account`: `username@computer folder`. If the computer has not been added to the medium, there is no computer folder, so the hostname is used in its place. `version`: the version of Babawus, for example `0.1.0`. `hostname`: the computer's hostname, as it is now. A log written before these two were added has a start entry without them |
| `ERROR` | `code`, and `path` (relative to the home folder). `files` gives the number of files in a skipped folder, if it has any |
| `END` | `verdict` and `reason`. Except for an interrupted run, whose numbers are not known, also `scanned_files`, `copied_files`, `copied_bytes`, `skipped_files`, `errors`, `versions_removed` and `free_bytes` |

The codes are:

| Field | Codes |
|---|---|
| `verdict` | `ALL_GOOD`, `SKIPPED`, `NOT_FINISHED`, `FAILED` |
| `reason` | `NONE` and `FILES_SKIPPED`; `CANCELLED`, `MEDIUM_REMOVED`, `INTERRUPTED` (found by a later run) and `FLUSH_FAILED`; `MEDIUM_FULL`, `NOT_ADDED` and `CANNOT_WRITE`. `CONFIG_DAMAGED` is also a reason, but only in a verdict: a run with a configuration file it can't use writes no log |
| `code` | The codes in "Names that cannot be stored", `SELECTION_MISSING`, `OUTSIDE_HOME`, `TYPE_CONFLICT`, `CHANGED_DURING_COPY`, `UNREADABLE`, `DISAPPEARED`, `NOT_WRITABLE`, `VERSION_NAME_TOO_LONG`, `WRITE_ERROR` (the general code for other write errors), `PRUNE_FAILED`, and `MULTIPLE_ERRORS` for the entry that says further errors are only counted, which has no `path` |

File names on Linux are bytes, and can hold anything except `/`, including tabs and line endings. So values are escaped: a tab is written as `\t`, a line ending as `\n`, a backslash as `\\`, and any other control character and every byte that is not valid UTF-8 as `\xNN`, with two hexadecimal digits. The log therefore always stays valid UTF-8 with one entry per line, and the original name can always be read back.

Writing and trimming the log:

- The start entry is written and flushed to the medium (`fsync`) before any file is copied.
- Every entry is flushed to the medium as it is written. If an error entry can't be written, the run goes on, because a problem with the medium shows up at the next step, and a problem with the log shows up at the end entry.
- The log is trimmed at the start of a run, before the start entry, never in place: the trimmed log is written to `.babawus/tmp/`, flushed, and renamed over the old log (the same mechanism as in "Writing a new copy safely"). Garbled lines are dropped when this happens.
- When the log is read, incomplete or garbled lines (for example a half-written last line after the medium was pulled out) are ignored, instead of rejecting the whole log. A last line with no line ending is always incomplete, even if it looks whole.
- Before anything is added to a log whose last line has no line ending, a line ending is written, so that the new entry does not become part of the garbled line.
- On Ext4, a new log gets mode `0666`.

At the start of a run the order is: take the medium's lock, empty `.babawus/tmp/`, close interrupted runs (see "Interrupted runs in the log"), trim the log, write the start entry.

### Interrupted runs in the log

A run with a start entry but no end entry was interrupted. When the log is read, such a run counts as "Backup not finished", and it does not count as a backup for "See when I last made a backup".

At the start of each run, after taking the medium's lock, Babawus writes the missing end entry for every such run: verdict `NOT_FINISHED`, reason `INTERRUPTED`, and the time it was detected as the timestamp. The log then only ever holds complete runs. A run whose lock is still live (see "One Babawus at a time") is never closed this way.

### Technology

- **Language and toolkit.** Babawus is written in Rust, with plain GTK 4 through gtk4-rs, without libadwaita, so that it follows the system theme as far as possible. It builds against GTK 4.14, the version in Mint 22, and uses nothing newer. If this does not work out, the fallback is Python with GTK 3.
- **Structure.** All logic (versioning, configuration, log, name rules, verdicts) lives in a library with no UI code, and a thin UI sits on top.
- **Dependencies.** Rust crates count as dependencies: each must be well maintained and replace more than a handful of lines, and the list stays short.
- **Translations.** Every text the user sees comes from a small catalogue of Babawus's own in the library, with one entry per text and language, not from gettext, because the language must change at once without a restart. A test checks that every text exists in every language. When the language changes, the UI sets all its texts again.
- **Administrator rights.** Babawus contains no code that runs as root. Partitioning, formatting, mounting while formatting, unmounting and powering off go through the udisks2 service over D-Bus (using the `gio` crate). udisks2 may ask for the administrator password itself, depending on the rules of the system, but it may also ask for nothing. On Mint, it asks for Ext4, and for unmounting a stick that another account mounted, but not for exFAT, unmounting, ejecting or powering off the user's own stick. Babawus must work either way, so the safeguard against formatting by mistake is the explicit confirmation (see "Format USB media"), not a password. Every step works through udisks2, so Babawus never runs `pkexec` or `sudo`.

### Responsiveness

The user interface must never freeze.

- A backup, and a count of files and bytes in the configuration editor, run on a worker thread, and send progress messages to the UI. Cancel is a flag that the worker checks between files and between the chunks of a file.
- Formatting runs in the background and shows "Formatting…". The D-Bus call has no short timeout, because formatting can take much longer than the 25-second default.
- A backup scans the source once. That single pass finds what needs copying and gives the totals for the overview, and the copying works from that list.
- Live totals in the configuration editor show "Counting…" until they are done. A change to the selection cancels the running count and starts a new one.
- The screen is updated at most about 10 times per second. The counters themselves stay exact.

### Testing

All non-GUI logic is covered by tests, in three layers:

1. **Unit tests** run with `cargo test`. All logic runs against a small file system interface, using an in-memory fake that can also inject faults (for example "fail at step 3" or "disk full after N bytes"), so that interruptions at every step of the write algorithm are tested. Name rules, verdicts, the configuration, log parsing and pruning are plain functions.
2. **Integration tests** run real backups on real exFAT and Ext4 file systems, created in small image files and mounted through loop devices. The same tests run on both file systems. A tiny image gives a truly full medium. These tests need `sudo`, so a plain `cargo test` skips them, and `cargo test -- --include-ignored` runs them.
3. **A manual checklist** on real USB sticks before each release covers what cannot be automated: formatting (also from an account without administrator rights, noting whether a password is asked for), pulling out a stick, eject (including powering off), installing, updating and uninstalling the package, and the GUI. It is [ReleaseChecklist.md](ReleaseChecklist.md), and each run is recorded in a GitHub issue (see Issues.md item 94).

Automated tests must never be able to format anything except a loop device created by the test itself.

CI runs the formatting and lint checks and all automated tests, including the integration tests, on every pull request and on `main`.

### Packaging

Babawus is delivered as a `.deb` package, built with `cargo-deb`. The poweruser installs it by double-clicking it, updates by installing a newer package, and uninstalls through the normal system tools. An APT repository, so that updates arrive through the Update Manager, may be added later.

- The package adds a desktop entry titled "Backup", with the Babawus elephant as its icon, to the application menu. The application ID is `io.github.ziggymax.Babawus`. The desktop entry names the icon file by its full path, so that the menu shows it straight after installing (Issues.md item 101).
- Its summary is "Basic backup with USB storage", and a longer description below it says in plain words what Babawus does, and that whoever looks after the computer prepares the stick under Advanced (Issues.md item 111).
- It declares its dependencies: the GTK 4 libraries, `udisks2`, `exfatprogs`, `e2fsprogs` and `acl`.
- Uninstalling never touches any medium. The small settings file in each user's home folder stays behind, and the uninstall notes in `README.md` mention it.

### Supported platform

The supported platform is Linux Mint 22.x with the Cinnamon desktop on X11. Everything else (the MATE and Xfce editions of Mint, Wayland, other Linux distributions) is best effort: Babawus should work there, but this is not tested or guaranteed. The target is reviewed when a new major version of Mint comes out.

### Not in the first version

- **Checking copies.** Copies are not read back and compared after writing. Babawus relies on write errors and `fsync`. Reading back right away usually comes from the computer's memory cache rather than the medium, and on slow USB sticks it would roughly double the time a backup takes. A "check this medium" function, for example using stored checksums, may come in a later version.
- **Automatic detection of moved files.** Detecting files that were only moved or renamed, to avoid storing them twice, is deferred to a later version. In the first version, a moved or renamed file is simply backed up again at its new location, and the copy at its old location stays on the medium.
