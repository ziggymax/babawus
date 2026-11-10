# Review of the look (T-032)

This document lists everything a user of Babawus can see, so that each item can be looked at on its own and get a decision. How the review runs is in [Issues.md](../Issues.md) items 112–116:

1. This list of items. *(This step.)*
2. A screenshot of each item, taken on the VM as `ui-review/<number>-<name>.png`, for example `S08-not-set-up.png`. Items marked **DE** are also taken in German, as `S08-not-set-up-de.png`. The screenshots are not committed (item 114), so each item says in words what it shows.
3. The items are gone through one by one, and each gets its comments and a decision below.
4. The user interface is changed, and the changes become tasks in [Tasks.md](../Tasks.md).

**Numbers.** A letter for the screen, and a number for the state: W for the window and its menu, S for the start screen, B for a running backup, V for the verdict, H for history and "What's backed up", F for preparing a stick, C for "Setup of this stick", T for "Choose what to back up…". Numbers never change once screenshots exist. A new item gets the next free number in its group.

**Who.** *Claude* drives the window over SSH. *You* means the user has to do something by hand: pull out or plug in the stick, or log in to the second account.

**Size.** Screenshots are taken at the size Babawus opens with. Items marked **small** are also taken with the window as small as it goes (item 116).

## Index

| No. | Item | Who | DE | Small |
| --- | --- | --- | --- | --- |
| W01 | "Backup" in the desktop menu, and in the panel | Claude | | |
| W02 | The window's menu | Claude | | |
| W03 | Menu → Language | Claude | | |
| W04 | Advanced turned on: the marker in the title bar | Claude | | |
| W05 | About Babawus | Claude | | |
| W06 | Babawus is already open for another account | You | | |
| W07 | udisks2 isn't running | Claude | | |
| S01 | No stick: "Plug in your backup stick" | You | DE | small |
| S02 | No stick listed, Advanced on: "Prepare…" on the empty page | Claude | | |
| S03 | A stick listed, but none chosen: "Choose a stick in the list" | Claude | | |
| S04 | Checking… | Claude | | |
| S05 | Ready, no backup yet | Claude | | |
| S06 | Ready, last backup all good | Claude | | small |
| S07 | Ready, last backup skipped files | Claude | | |
| S08 | Not set up for you | Claude | DE | |
| S09 | Backup in progress (by another account) | Claude | | |
| S10 | Needs attention | Claude | | |
| S11 | Write-protected | Claude | | |
| S12 | Open for another account | You | | |
| S13 | Not a backup medium | Claude | | |
| S14 | Ready, with Advanced on: all six buttons | Claude | | small |
| S15 | Ejecting… | Claude | | |
| S16 | Ejected: "You can pull out BACKUP-A now." | Claude | | |
| S17 | Eject refused: the stick is still in use | Claude | | |
| S18 | A tooltip: why "Start backup" can't be pressed | Claude | | |
| B01 | Finding the files to back up… | Claude | | |
| B02 | Backing up…, with counters and the bar | Claude | DE | small |
| B03 | "Stop the backup?" | Claude | | |
| B04 | Stopping… | Claude | | |
| B05 | Finishing… | Claude | | |
| B06 | Closing the window during a backup | Claude | | |
| V01 | All good, files copied | Claude | | |
| V02 | All good, nothing needed copying | Claude | | |
| V03 | Some files were skipped | Claude | DE | small |
| V04 | Details for your poweruser, opened, after Copy | Claude | | |
| V05 | Backup not finished: you stopped it | Claude | | |
| V06 | Backup not finished: the stick was removed | You | | |
| V07 | Backup failed: the stick is full | Claude | | |
| H01 | History with several runs | Claude | DE | |
| H02 | History with no runs | Claude | | |
| H03 | What's backed up | Claude | | |
| H04 | What's backed up, with a chosen folder that's gone | Claude | | |
| F01 | Prepare a stick: choosing | Claude | DE | small |
| F02 | The label breaks the rule | Claude | | |
| F03 | Ext4 chosen by an administrator, ticked, ready to erase | Claude | | |
| F04 | Ext4 greyed out for a Standard account | You | | |
| F05 | No stick can be formatted | You | | |
| F06 | Formatting… | Claude | | |
| F07 | BACKUP-A is ready | Claude | | |
| F08 | Formatting failed, nothing changed | You | | |
| C01 | Setup of this stick: this computer isn't set up yet | Claude | DE | small |
| C02 | This computer and account added, Save possible | Claude | | |
| C03 | Saved | Claude | | |
| C04 | "Leave without saving?" | Claude | | |
| C05 | The setup was changed outside Babawus | Claude | | |
| C06 | The setup can't be read at all | Claude | | |
| C07 | "Start a new setup?" | Claude | | |
| C08 | Several computers and accounts | Claude | | |
| C09 | "Remove …?" | Claude | | |
| C10 | "Link to …?" | Claude | | |
| T01 | Choose what to back up: the home folder | Claude | DE | small |
| T02 | A folder ticked, with what is left out | Claude | | |
| T03 | Hidden folders shown | Claude | | |
| T04 | Name patterns, with a broken one | Claude | | |
| T05 | Nothing chosen | Claude | | |
| T06 | Chosen more than fits on the stick | Claude | | |

## The window and its menu

### W01 "Backup" in the desktop menu, and in the panel
- **How:** search the Cinnamon menu for "Backup", then start it, and look at the panel.
- **Shows:** the entry "Backup", "Back up files to USB media", with the elephant; the elephant in the panel while Babawus runs.
- **Comments:**
- **Decision:**

### W02 The window's menu
- **How:** press the menu button in the title bar.
- **Shows:** Language ▸, Advanced (a tick box), About Babawus.
- **Comments:**
- **Decision:**

### W03 Menu → Language
- **How:** open the menu, then Language.
- **Shows:** "Language" with a back arrow, English and Deutsch as choices.
- **Comments:**
- **Decision:**

### W04 Advanced turned on: the marker in the title bar
- **How:** Menu → Advanced.
- **Shows:** an orange "Advanced" marker next to "Backup" in the title bar.
- **Comments:**
- **Decision:**

### W05 About Babawus
- **How:** Menu → About Babawus.
- **Shows:** GTK's About dialog: name, version, logo, comments, licence.
- **Comments:**
- **Decision:**

### W06 Babawus is already open for another account
- **How:** You start Babawus in the second account and switch user without closing it; Claude then starts it in yours.
- **Shows:** "Babawus is already open for testuser on this computer", and no window.
- **Comments:**
- **Decision:**

### W07 udisks2 isn't running
- **How:** Claude stops udisks2 for a moment, and starts it again afterwards.
- **Shows:** the empty page with the text for a missing udisks2.
- **Comments:**
- **Decision:**

## The start screen

### S01 No stick: "Plug in your backup stick"
- **How:** You pull out the stick.
- **Shows:** an empty list on the left, the logo, and "Plug in your backup stick" in the middle.
- **Comments:**
- **Decision:**

### S02 No stick listed, Advanced on: "Prepare…" on the empty page
- **How:** Claude unmounts the stick, so that it can still be formatted but isn't listed; Advanced on.
- **Shows:** "Plug in your backup stick", with "Prepare…" below it.
- **Comments:**
- **Decision:**

### S03 A stick listed, but none chosen
- **How:** a stick that isn't Ready, with nothing selected.
- **Shows:** "Choose a stick in the list".
- **Comments:**
- **Decision:**

### S04 Checking…
- **How:** right after a stick is plugged in, if it can be caught.
- **Shows:** a spinner in place of the mark, and "Checking…".
- **Comments:**
- **Decision:**

### S05 Ready, no backup yet
- **How:** a stick set up for this account that has never been backed up to.
- **Shows:** green tick, "Ready", maker, size and free space, "No backup yet", and Start backup, Eject.
- **Comments:**
- **Decision:**

### S06 Ready, last backup all good
- **Shows:** green tick, "Ready", "Last backup: today at 14:03", Start backup, Show history, What's backed up, Eject.
- **Known:** the stick pictures and the logo should be larger (T-032).
- **Comments:**
- **Decision:**

### S07 Ready, last backup skipped files
- **Shows:** as S06, with an amber "!" after the last backup.
- **Comments:**
- **Decision:**

### S08 Not set up for you
- **How:** a Babawus stick without this account.
- **Shows:** amber "!", "Not set up for you", "This stick hasn't been set up for you on this computer. Contact your poweruser."
- **Comments:**
- **Decision:**

### S09 Backup in progress (by another account)
- **How:** Claude holds the stick's lock from a second process.
- **Shows:** amber "!", "Backup in progress (by …)", and what to do.
- **Comments:**
- **Decision:**

### S10 Needs attention
- **How:** Claude changes the stick's configuration by hand, so that its checksum no longer fits.
- **Shows:** amber "!", "Needs attention", and "The setup on this stick is damaged, or needs a newer Babawus. Contact your poweruser."
- **Comments:**
- **Decision:**

### S11 Write-protected
- **How:** Claude mounts the stick read-only.
- **Shows:** grey cross, "Write-protected", and the advice about the lock switch.
- **Comments:**
- **Decision:**

### S12 Open for another account
- **How:** You log out and in to the other account with the stick still plugged in.
- **Shows:** amber "!", grey stick, "Open for another account", the advice to replug, and only Start backup (greyed out) and Eject.
- **Comments:**
- **Decision:**

### S13 Not a backup medium
- **How:** Claude renames `.babawus` on the stick for a moment.
- **Shows:** grey stick, grey cross, "Not a backup medium", and "Your poweruser can prepare it."
- **Comments:**
- **Decision:**

### S14 Ready, with Advanced on: all six buttons
- **Shows:** Start backup, Show history, What's backed up, Eject, Prepare…, Change setup….
- **Known:** the buttons stretch to fill their columns when they wrap (T-032).
- **Comments:**
- **Decision:**

### S15 Ejecting…
- **Shows:** "Ejecting BACKUP-A…", if it can be caught.
- **Comments:**
- **Decision:**

### S16 Ejected
- **Shows:** the stick leaves the list, and "You can pull out BACKUP-A now."
- **Comments:**
- **Decision:**

### S17 Eject refused: the stick is still in use
- **How:** a file manager window open on the stick, then Eject.
- **Shows:** "BACKUP-A is still in use by a program, for example an open file or a file manager window. Close it and try again."
- **Comments:**
- **Decision:**

### S18 A tooltip: why "Start backup" can't be pressed
- **How:** the pointer on the greyed-out "Start backup" of a stick that isn't Ready.
- **Shows:** "No backup can start: …".
- **Comments:**
- **Decision:**

## A running backup

### B01 Finding the files to back up…
- **Shows:** the backup screen filling the window, "Finding the files to back up…", and Cancel.
- **Comments:**
- **Decision:**

### B02 Backing up…, with counters and the bar
- **How:** a backup with the 2 GB `big.bin` to copy.
- **Shows:** "Backing up…", "To back up: 2 files (2.1 GB)", the bar, files and bytes left, and Cancel.
- **Comments:**
- **Decision:**

### B03 "Stop the backup?"
- **How:** Cancel during B02.
- **Shows:** the question, with "Keep going" and "Stop backup".
- **Comments:**
- **Decision:**

### B04 Stopping…
- **Shows:** "Stopping…", if it can be caught.
- **Comments:**
- **Decision:**

### B05 Finishing…
- **Shows:** "Finishing… Don't pull out the stick yet." It lasted under a third of a second in T-022, so it may not be caught.
- **Comments:**
- **Decision:**

### B06 Closing the window during a backup
- **How:** the window's close button during B02.
- **Shows:** the same question as B03, before the window closes.
- **Comments:**
- **Decision:**

## The verdict

### V01 All good, files copied
- **Shows:** green tick, "All good", what was copied, "Last backup: today at …", the details closed, "Eject BACKUP-A" and Back.
- **Comments:**
- **Decision:**

### V02 All good, nothing needed copying
- **Shows:** "All good", "Everything was already up to date."
- **Comments:**
- **Decision:**

### V03 Some files were skipped
- **How:** a backup of `~/BabawusTest`, with `meeting: notes.txt` and the unreadable `secret.txt`.
- **Shows:** amber "!", "Some files were skipped", the skipped files grouped by reason, "Contact your poweruser.", what was copied.
- **Comments:**
- **Decision:**

### V04 Details for your poweruser, opened, after Copy
- **Shows:** the details in a fixed-width font, with codes such as `NAME_INVALID_CHARS`, and Copy.
- **Comments:**
- **Decision:**

### V05 Backup not finished: you stopped it
- **Shows:** "Backup not finished", "You stopped the backup.", and what to do.
- **Comments:**
- **Decision:**

### V06 Backup not finished: the stick was removed
- **How:** You pull out the stick during B02.
- **Shows:** "Backup not finished", "The stick was removed.", "Run the backup again. If it happens again, contact your poweruser."
- **Comments:**
- **Decision:**

### V07 Backup failed: the stick is full
- **How:** Claude fills the stick with one large file first.
- **Shows:** "Backup failed", and that the stick is full.
- **Comments:**
- **Decision:**

## History and "What's backed up"

### H01 History with several runs
- **Shows:** each run, newest first, with its mark, verdict, time, reason, what was copied and skipped, and whose run it was.
- **Comments:**
- **Decision:**

### H02 History with no runs
- **How:** a freshly prepared stick (the button only shows once there is a run, so this may not be reachable).
- **Comments:**
- **Decision:**

### H03 What's backed up
- **Shows:** "These are backed up from this computer for steen:", the chosen folders, and "Left out:" with folders and name patterns.
- **Comments:**
- **Decision:**

### H04 What's backed up, with a chosen folder that's gone
- **How:** Claude renames a chosen folder for a moment.
- **Shows:** the folder marked as missing.
- **Comments:**
- **Decision:**

## Preparing a stick

### F01 Prepare a stick: choosing
- **How:** Advanced on, "Prepare…".
- **Shows:** the stick with maker, size and contents, the label, exFAT and Ext4 with a line each, the warning, the tick box, "Erase and format" greyed out.
- **Comments:**
- **Decision:**

### F02 The label breaks the rule
- **How:** a space typed into the label.
- **Shows:** "Use 1 to 11 letters, digits and dashes, with no spaces."
- **Comments:**
- **Decision:**

### F03 Ext4 chosen by an administrator, ticked, ready to erase
- **Shows:** "Ext4 asks for your password while formatting.", the tick, and a red "Erase and format".
- **Comments:**
- **Decision:**

### F04 Ext4 greyed out for a Standard account
- **How:** You, in the second account.
- **Shows:** Ext4 greyed out, with "Only an administrator can choose Ext4, …".
- **Comments:**
- **Decision:**

### F05 No stick can be formatted
- **How:** You pull out the stick while F01 is open.
- **Shows:** the text for no drives, and no list.
- **Comments:**
- **Decision:**

### F06 Formatting…
- **Shows:** a spinner, "Formatting BACKUP-A…", "This can take a few minutes. Don't pull out the stick.", and no way back.
- **Comments:**
- **Decision:**

### F07 BACKUP-A is ready
- **Shows:** green tick, "BACKUP-A is ready", what comes next, "Add this computer and account…" and Back.
- **Comments:**
- **Decision:**

### F08 Formatting failed, nothing changed
- **How:** You, in the second account, with the stick still mounted for the first, cancel the password dialog.
- **Shows:** "Formatting failed", why, and "Nothing on the stick was changed."
- **Comments:**
- **Decision:**

## Setup of this stick

### C01 This computer isn't set up yet
- **How:** Advanced on, "Change setup…" on a freshly prepared stick.
- **Shows:** "Setup of this stick", "Older versions to keep of each file: 10", "This computer (…) isn't set up on this stick yet.", "Add this computer as …".
- **Known:** "Add this computer as steen-virtualbox" also adds the account running Babawus, but doesn't say so (T-024).
- **Comments:**
- **Decision:**

### C02 This computer and account added, Save possible
- **Shows:** the entry with "(this computer)" and "(you)", "Choose what to back up…", and Save.
- **Comments:**
- **Decision:**

### C03 Saved
- **Shows:** "Saved."
- **Comments:**
- **Decision:**

### C04 "Leave without saving?"
- **How:** Back with unsaved changes.
- **Shows:** the question, with "Keep editing" and the button that discards.
- **Comments:**
- **Decision:**

### C05 The setup was changed outside Babawus
- **How:** Claude edits `older_versions` in the configuration by hand.
- **Shows:** the warning that the setup "was changed outside Babawus, or is damaged".
- **Comments:**
- **Decision:**

### C06 The setup can't be read at all
- **How:** Claude replaces the configuration with something that isn't one, for a moment.
- **Shows:** that it can't be read, and "Start a new setup…".
- **Comments:**
- **Decision:**

### C07 "Start a new setup?"
- **Shows:** the question, and that the backups are kept.
- **Comments:**
- **Decision:**

### C08 Several computers and accounts
- **How:** Claude adds a second computer and account with Babawus's `add_account` example.
- **Shows:** every entry, with what each backs up, and the remove buttons.
- **Comments:**
- **Decision:**

### C09 "Remove …?"
- **Shows:** the question, and what happens to that entry's backups.
- **Comments:**
- **Decision:**

### C10 "Link to …?"
- **Shows:** the question for linking this computer or account to an existing entry.
- **Comments:**
- **Decision:**

## Choose what to back up

### T01 The home folder
- **How:** "Choose what to back up…" in the setup.
- **Shows:** the tree with "Your home folder" open, the totals, the name patterns, Done and Cancel.
- **Known:** the tree scrolls inside a page that scrolls too (T-032).
- **Comments:**
- **Decision:**

### T02 A folder ticked, with what is left out
- **How:** `BabawusTest` ticked and opened, `Letters/old` unticked.
- **Shows:** `notes.tmp` crossed out with "Left out by the name pattern *.tmp", `old` crossed out, and the totals.
- **Comments:**
- **Decision:**

### T03 Hidden folders shown
- **How:** "Show hidden folders" ticked.
- **Comments:**
- **Decision:**

### T04 Name patterns, with a broken one
- **How:** a pattern with a slash, or one that is there already.
- **Shows:** the message under the pattern.
- **Comments:**
- **Decision:**

### T05 Nothing chosen
- **Shows:** the totals saying that nothing is chosen.
- **Comments:**
- **Decision:**

### T06 Chosen more than fits on the stick
- **How:** a large folder ticked, or the stick nearly full.
- **Shows:** the warning that it doesn't fit.
- **Comments:**
- **Decision:**
