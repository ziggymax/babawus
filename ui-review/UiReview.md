# Review of the look (T-032)

This document lists everything a user of Babawus can see, so that each item can be looked at on its own and get a decision. How the review runs is in [Issues.md](../Issues.md) items 112–116:

1. This list of items.
2. A screenshot of each item, taken on the VM as `ui-review/<number>-<name>.png`, for example `S08-not-set-up.png`. Items marked **DE** are also taken in German, as `S08-not-set-up-de.png`. The screenshots are not committed (item 114), so each item says in words what it shows. *(Done on 2026-11-10: 79 screenshots of 60 items. Six items couldn't be reached, and each says why. Under **Noticed**, each item has what was seen while taking it.)*
3. The items are gone through one by one, and each gets its comments and a decision below. Decisions that apply across several screens are items of group G, and the items they apply to point to them. *(Done on 2026-11-10.)*
4. The user interface is changed, and the changes become tasks in [Tasks.md](../Tasks.md).

## Outcome

Every item has a decision. Most items need no change. What changes, as tasks in [Tasks.md](../Tasks.md):

| Task | What | Items |
| --- | --- | --- |
| T-037 | The window opens at 1024 × 768, and pages that scroll start at the top (Issues.md item 118). *Done in PR #51: every screen fits at that size, except "Choose what to back up…", which T-041 changes.* | G03, F01, F06, F07 |
| T-038 | The start screen's layout: larger stick pictures, one explanation area for the status and every message, fewer text styles, buttons at their natural width | G01, S02, S06, S14–S17 |
| T-039 | The verdict screen: fewer text styles, no repeated reason, a fitting text after a stopped backup | G02, V03, V05 |
| T-040 | Setup of this stick: clearer adding and linking texts | C01, C06, C10 |
| T-041 | Choose what to back up, plus About and "already open" | T01, T02, T04, W05, W06 |
| T-035 | Bug: the Ext4 note only appears after another change | F03 |
| T-036 | No copied details: the log is the one place for diagnosing (Issues.md item 117) | V04 |

**Numbers.** A letter for the screen, and a number for the state: G for general decisions that apply to several screens, W for the window and its menu, S for the start screen, B for a running backup, V for the verdict, H for history and "What's backed up", F for preparing a stick, C for "Setup of this stick", T for "Choose what to back up…". Numbers never change once screenshots exist. A new item gets the next free number in its group.

**Who.** *Claude* drives the window over SSH. *You* means the user has to do something by hand: pull out or plug in the stick, or log in to the second account.

**Size.** Screenshots are taken at the size Babawus opens with, 848 × 635. Items marked **small** are also taken with the window as small as it goes (item 116).

## Index

| No. | Item | Who | DE | Small |
| --- | --- | --- | --- | --- |
| G01 | The start screen as a whole | | | |
| G02 | The verdict screen as a whole | | | |
| G03 | The window's size | | | |
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

## General decisions

### G01 The start screen as a whole
- **Applies to:** S01–S18, and the stick list on every screen that shows it.
- **Comments (2026-11-10):**
    - The stick pictures should be larger.
    - The label in the list should be smaller: the details already have it as their heading. The larger picture and the smaller label together should take up the same width as now.
    - Four text sizes and colours are used for the heading, the status, the stick's details and the explanation: that's a lot in a small space.
    - The order should be: label, then the stick's details (maker, size, free space), then the explanation.
    - The status, mark and text, is just the first line of the explanation, on a line of its own, in the same font, size and colour as the rest of it. The mark makes it stand out enough.
    - A thin line between the explanation and what is above it might look nice.
    - Added while going through group S: all extra information goes in the explanation area: errors, statuses, suggestions and the time of the last backup, for example "You can pull out the stick now", "The stick is in use by another program" and "This stick isn't set up for you".
- **Decision:**
    - **The list:** a larger stick picture, and a smaller label, together as wide as now.
    - **The details, from the top:** the label as the heading; the maker, size and free space; a thin horizontal line; then the explanation. Its first line is the status, with its mark, for example "✓ Ready" or "⚠ Not set up for you", and below it "Last backup: …" or what the status means and what to do.
    - **Three text styles in the details:** the heading, the stick's details, and the explanation. The status loses its larger size, and becomes explanation text with its mark.
    - **Every message about the stick goes in the explanation area**, below the status: "Last backup: …", what a status means and what to do, and the notices that now appear elsewhere, such as "Ejecting BACKUP-A…" and "BACKUP-A is still in use by a program…" (S15, S17), which now sit below the buttons.
    - After an eject the stick has left the list, so there is no explanation area. Then "You can pull out BACKUP-A now." goes on the empty page, right below "Plug in your backup stick", in the explanation's style (S16).
    - The logo stays where it is, a little larger (S06).

### G02 The verdict screen as a whole
- **Applies to:** V01–V07.
- **Comments (2026-11-10):** Agreed with the suggestion. Copying details for the poweruser is dropped altogether: see V04.
- **Decision:** At most three text styles, as in G01: the label, the verdict with its mark, and one style for everything below it. Only the advice ("Contact your poweruser.") stays in bold.

### G03 The window's size
- **Applies to:** every screen; found under F01, F06 and F07.
- **Comments (2026-11-10):** Having to scroll to find buttons or read text is to be avoided, at least at the window's normal size. It is fair to assume that no screen smaller than Full HD (1920 × 1080) is used, so the window can open at 1024 × 768. That should remove the need to scroll at the default size, while keeping the format page's layout and wording.
- **Decision:** Babawus opens at 1024 × 768 (Issues.md item 118). Each screen is checked at that size for scrolling. A page that scrolls still starts at the top whenever it is opened or changes part.

## The window and its menu

### W01 "Backup" in the desktop menu, and in the panel
- **How:** search the Cinnamon menu for "Backup", then start it, and look at the panel.
- **Shows:** the entry "Backup", "Back up files to USB media", with the elephant; the elephant in the panel while Babawus runs.
- **Screenshots:** `W01-desktop-menu.png`.
- **Comments:** None.
- **Decision:** No change.

### W02 The window's menu
- **How:** press the menu button in the title bar.
- **Shows:** Language ▸, Advanced (a tick box), About Babawus.
- **Screenshots:** `W02-menu.png`.
- **Comments:** None.
- **Decision:** No change.

### W03 Menu → Language
- **How:** open the menu, then Language.
- **Shows:** "Language" with a back arrow, English and Deutsch as choices.
- **Screenshots:** `W03-language.png`.
- **Comments:** None.
- **Decision:** No change.

### W04 Advanced turned on: the marker in the title bar
- **How:** Menu → Advanced.
- **Shows:** an orange "Advanced" marker next to "Backup" in the title bar.
- **Screenshots:** `W04-advanced.png`.
- **Comments:** None.
- **Decision:** No change.

### W05 About Babawus
- **How:** Menu → About Babawus.
- **Shows:** GTK's About dialog: name, version, logo, comments, licence.
- **Screenshots:** `W05-about.png`.
- **Noticed:** The name "Babawus" is selected, highlighted, when the dialog opens.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** Don't select the name when the dialog opens.

### W06 Babawus is already open for another account
- **How:** You start Babawus in the second account and switch user without closing it; Claude then starts it in yours.
- **Shows:** "Babawus is already open for testuser on this computer", and no window.
- **Screenshots:** `W06-already-open.png`.
- **Noticed:** A small separate window with minimise and maximise buttons, and no button to close it; the sentence has no full stop.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** A proper message dialog: the logo, "Babawus is already open for <user> on this computer.", what to do ("Ask them to close it, or switch to their account."), and an OK button that closes Babawus. In English and German.

### W07 udisks2 isn't running
- **How:** Claude stops udisks2 for a moment, and starts it again afterwards.
- **Shows:** the empty page with the text for a missing udisks2.
- **Screenshots:** `W07-no-udisks.png`.
- **Comments:** None.
- **Decision:** No change. The text is clear, and this shouldn't happen on Mint.

## The start screen

### S01 No stick: "Plug in your backup stick"
- **How:** You pull out the stick.
- **Shows:** an empty list on the left, the logo, and "Plug in your backup stick" in the middle.
- **Screenshots:** `S01-no-stick-de.png`, `S01-no-stick-small.png`, `S01-no-stick.png`.
- **Noticed:** At the smallest size the window keeps its full height (564 × 635).
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change beyond G01. Keeping the full height at the smallest size is fine.

### S02 No stick listed, Advanced on: "Prepare…" on the empty page
- **How:** Claude unmounts the stick, so that it can still be formatted but isn't listed; Advanced on.
- **Shows:** "Plug in your backup stick", with "Prepare…" below it.
- **Screenshots:** `S02-empty-prepare.png`.
- **Noticed:** A stick is plugged in, but the page says "Plug in your backup stick".
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** When a stick that may be formatted is plugged in but not opened, say "The stick isn't opened. Pull it out and plug it back in." instead of "Plug in your backup stick". "Prepare…" stays.

### S03 A stick listed, but none chosen
- **How:** a stick that isn't Ready, with nothing selected.
- **Shows:** "Choose a stick in the list".
- **Screenshots:** none: Babawus always selects a lone stick, so this needs two sticks plugged in.
- **Comments:** None.
- **Decision:** No change. Not reachable with one stick.

### S04 Checking…
- **How:** right after a stick is plugged in, if it can be caught.
- **Shows:** a spinner in place of the mark, and "Checking…".
- **Screenshots:** none: a stick went from absent to Ready between two frames a quarter of a second apart.
- **Noticed:** The very first frame after plugging in drew the row with the stick picture and the mark, but no label.
- **Comments:** None.
- **Decision:** No change. Too quick to see on the VM.

### S05 Ready, no backup yet
- **How:** a stick set up for this account that has never been backed up to.
- **Shows:** green tick, "Ready", maker, size and free space, "No backup yet", and Start backup, Eject.
- **Screenshots:** `S05-ready-no-backup.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S06 Ready, last backup all good
- **Shows:** green tick, "Ready", "Last backup: today at 14:03", Start backup, Show history, What's backed up, Eject.
- **Known:** the stick pictures and the logo should be larger (T-032).
- **Screenshots:** `S06-ready-all-good-small.png`, `S06-ready-all-good.png`.
- **Noticed:** The line with maker, size and free space wraps, leaving "free" alone on a line. At the smallest size (658 × 544) the buttons stack, full width.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01, which also stops "free" wrapping onto a line of its own. The logo gets a little larger, in line with the larger stick pictures.

### S07 Ready, last backup skipped files
- **Shows:** as S06, with an amber "!" after the last backup.
- **Screenshots:** `S07-ready-skipped.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S08 Not set up for you
- **How:** a Babawus stick without this account.
- **Shows:** amber "!", "Not set up for you", "This stick hasn't been set up for you on this computer. Contact your poweruser."
- **Screenshots:** `S08-not-set-up-de.png`, `S08-not-set-up.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S09 Backup in progress (by another account)
- **How:** Claude holds the stick's lock from a second process.
- **Shows:** amber "!", "Backup in progress (by …)", and what to do.
- **Screenshots:** `S09-in-progress.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S10 Needs attention
- **How:** Claude changes the stick's configuration by hand, so that its checksum no longer fits.
- **Shows:** amber "!", "Needs attention", and "The setup on this stick is damaged, or needs a newer Babawus. Contact your poweruser."
- **Screenshots:** `S10-needs-attention.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S11 Write-protected
- **How:** Claude mounts the stick read-only.
- **Shows:** grey cross, "Write-protected", and the advice about the lock switch.
- **Screenshots:** `S11-write-protected.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S12 Open for another account
- **How:** You log out and in to the other account with the stick still plugged in.
- **Shows:** amber "!", grey stick, "Open for another account", the advice to replug, and only Start backup (greyed out) and Eject.
- **Screenshots:** `S12-other-account.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S13 Not a backup medium
- **How:** Claude renames `.babawus` on the stick for a moment.
- **Shows:** grey stick, grey cross, "Not a backup medium", and "Your poweruser can prepare it."
- **Screenshots:** `S13-not-backup-medium.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G01.

### S14 Ready, with Advanced on: all six buttons
- **Shows:** Start backup, Show history, What's backed up, Eject, Prepare…, Change setup….
- **Known:** the buttons stretch to fill their columns when they wrap (T-032).
- **Screenshots:** `S14-advanced-buttons-small.png`, `S14-advanced-buttons.png`.
- **Noticed:** "Eject" sits alone on a second row, stretched. At the smallest size (678 × 574) the first four buttons stack full width, while Prepare… and Change setup… stay side by side.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** Buttons keep their natural width and wrap like words, instead of stretching to fill a column. The poweruser's two buttons stay on a row of their own, below a thin line.

### S15 Ejecting…
- **Shows:** "Ejecting BACKUP-A…", if it can be caught.
- **Screenshots:** `S15-ejecting.png`.
- **Comments:** Agreed (2026-11-10), with G01.
- **Decision:** "Ejecting BACKUP-A…" goes in the explanation area (G01), not below the buttons.

### S16 Ejected
- **Shows:** the stick leaves the list, and "You can pull out BACKUP-A now."
- **Screenshots:** `S16-ejected-notice-stays.png`, `S16-ejected.png`.
- **Noticed:** After the stick was plugged back in, "You can pull out BACKUP-A now." was still shown below it (`S16-ejected-notice-stays.png`). With Advanced off, Eject sits where Start backup is with Advanced on, which made it easy to press by mistake.
- **Comments:** Agreed with the suggestion (2026-11-10), with G01.
- **Decision:** "You can pull out BACKUP-A now." goes right below "Plug in your backup stick" (G01), and disappears as soon as a stick is plugged in. Eject goes last among the user's buttons, after a small gap, so that it doesn't take the place of another button when Advanced changes.

### S17 Eject refused: the stick is still in use
- **How:** a file manager window open on the stick, then Eject.
- **Shows:** "BACKUP-A is still in use by a program, for example an open file or a file manager window. Close it and try again."
- **Screenshots:** `S17-eject-busy.png`.
- **Comments:** Agreed (2026-11-10), with G01.
- **Decision:** "BACKUP-A is still in use by a program…" goes in the explanation area (G01), not below the buttons.

### S18 A tooltip: why "Start backup" can't be pressed
- **How:** the pointer on the greyed-out "Start backup" of a stick that isn't Ready.
- **Shows:** "No backup can start: …".
- **Screenshots:** `S18-tooltip-cant-start.png`.
- **Comments:** None.
- **Decision:** No change.

## A running backup

### B01 Finding the files to back up…
- **Shows:** the backup screen filling the window, "Finding the files to back up…", and Cancel.
- **Screenshots:** none: finding the files ended before the first frame, even for 2 GB.
- **Comments:** None.
- **Decision:** No change. Too quick to see on the VM.

### B02 Backing up…, with counters and the bar
- **How:** a backup with the 2 GB `big.bin` to copy.
- **Shows:** "Backing up…", "To back up: 2 files (2.1 GB)", the bar, files and bytes left, and Cancel.
- **Screenshots:** `B02-backing-up-de.png`, `B02-backing-up-small.png`, `B02-backing-up.png`.
- **Noticed:** At the smallest size the window gets much narrower than elsewhere (304 × 495).
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change. The narrow smallest size is fine for a bar and two lines.

### B03 "Stop the backup?"
- **How:** Cancel during B02.
- **Shows:** the question, with "Keep going" and "Stop backup".
- **Screenshots:** `B03-stop-question.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### B04 Stopping…
- **Shows:** "Stopping…", if it can be caught.
- **Screenshots:** `B04-stopping.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### B05 Finishing…
- **Shows:** "Finishing… Don't pull out the stick yet." It lasted under a third of a second in T-022, so it may not be caught.
- **Screenshots:** none: "Finishing…" lasts under a third of a second (as in T-022).
- **Comments:** None.
- **Decision:** No change. Too quick to see.

### B06 Closing the window during a backup
- **How:** the window's close button during B02.
- **Shows:** the same question as B03, before the window closes.
- **Screenshots:** `B06-close-during-backup.png`.
- **Noticed:** The same question as B03.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change. The same question for Cancel and for closing the window is right.

## The verdict

### V01 All good, files copied
- **Shows:** green tick, "All good", what was copied, "Last backup: today at …", the details closed, "Eject BACKUP-A" and Back.
- **Screenshots:** `V01-all-good.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G02.

### V02 All good, nothing needed copying
- **Shows:** "All good", "Everything was already up to date."
- **Screenshots:** `V02-nothing-to-copy.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** See G02.

### V03 Some files were skipped
- **How:** a backup of `~/BabawusTest`, with `meeting: notes.txt` and the unreadable `secret.txt`.
- **Shows:** amber "!", "Some files were skipped", the skipped files grouped by reason, "Contact your poweruser.", what was copied.
- **Screenshots:** `V03-skipped-de.png`, `V03-skipped-small.png`, `V03-skipped.png`.
- **Noticed:** The heading "Some files were skipped" and the line below it, "Some files could not be backed up.", say the same thing.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** Leave out the reason line "Some files could not be backed up." when the heading already says "Some files were skipped". The skipped groups follow the heading. See G02.

### V04 Details for your poweruser, opened, after Copy
- **Shows:** the details in a fixed-width font, with codes such as `NAME_INVALID_CHARS`, and Copy.
- **Screenshots:** `V04-details-copied.png`.
- **Comments:** Users shouldn't copy and paste diagnostic details: the log overlaps with this, a single place for diagnosing is easier, and it doesn't depend on a user doing anything (2026-11-10).
- **Decision:** "Details for your poweruser" and Copy are removed. The log must be enough for diagnosing on its own. Issues.md item 117, task T-036.

### V05 Backup not finished: you stopped it
- **Shows:** "Backup not finished", "You stopped the backup.", and what to do.
- **Screenshots:** `V05-stopped.png`.
- **Noticed:** After the user stopped the backup on purpose, it still says "If it happens again, contact your poweruser." It also lists a skipped file, from before the stop.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** Replace "Run the backup again. If it happens again, contact your poweruser." with "Run the backup again when it suits you.", in English and German. Skipped files from before the stop stay listed. See G02.

### V06 Backup not finished: the stick was removed
- **How:** You pull out the stick during B02.
- **Shows:** "Backup not finished", "The stick was removed.", "Run the backup again. If it happens again, contact your poweruser."
- **Screenshots:** `V06-stick-removed.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change beyond G02.

### V07 Backup failed: the stick is full
- **How:** Claude fills the stick with one large file first.
- **Shows:** "Backup failed", and that the stick is full.
- **Screenshots:** `V07-full.png`.
- **Noticed:** Shown with a red cross, "Backup failed", "The stick is full.", and the earlier skipped file.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change beyond G02.

## History and "What's backed up"

### H01 History with several runs
- **Shows:** each run, newest first, with its mark, verdict, time, reason, what was copied and skipped, and whose run it was.
- **Screenshots:** `H01-history-de.png`, `H01-history.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### H02 History with no runs
- **How:** a freshly prepared stick (the button only shows once there is a run, so this may not be reachable).
- **Screenshots:** none: "Show history" only shows once there is a run, so an empty history can't be opened.
- **Comments:** None.
- **Decision:** No change. It can't be reached.

### H03 What's backed up
- **Shows:** "These are backed up from this computer for steen:", the chosen folders, and "Left out:" with folders and name patterns.
- **Screenshots:** `H03-whats-backed-up.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### H04 What's backed up, with a chosen folder that's gone
- **How:** Claude renames a chosen folder for a moment.
- **Shows:** the folder marked as missing.
- **Screenshots:** `H04-folder-gone.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

## Preparing a stick

### F01 Prepare a stick: choosing
- **How:** Advanced on, "Prepare…".
- **Shows:** the stick with maker, size and contents, the label, exFAT and Ext4 with a line each, the warning, the tick box, "Erase and format" greyed out.
- **Screenshots:** `F01-prepare-de.png`, `F01-prepare-scrolled.png`, `F01-prepare-small.png`, `F01-prepare.png`.
- **Noticed:** At the opening size the page doesn't fit: Ext4's line is cut off, and the warning, the tick box and the button are below the fold (`F01-prepare-scrolled.png`). The stick is described as "with Babawus backups" although it had none yet. When the page is opened again, it keeps the old scroll position, so the heading is out of sight. Smallest size: 584 × 478.
- **Comments:** Avoid having to scroll; keep the layout, size and wording of the format page, and open the window at 1024 × 768 instead (2026-11-10).
- **Decision:** See G03: the window opens at 1024 × 768, so the whole page fits. The page always opens at the top. The stick's description stays as it is.

### F02 The label breaks the rule
- **How:** a space typed into the label.
- **Shows:** "Use 1 to 11 letters, digits and dashes, with no spaces."
- **Screenshots:** `F02-label-rule.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### F03 Ext4 chosen by an administrator, ticked, ready to erase
- **Shows:** "Ext4 asks for your password while formatting.", the tick, and a red "Erase and format".
- **Screenshots:** `F03-ext4-note-missing.png`, `F03-ext4-ticked.png`.
- **Noticed:** **Bug from PR #46:** choosing Ext4 doesn't show "Ext4 asks for your password while formatting." (`F03-ext4-note-missing.png`). It only appears after something else changes, such as ticking the box. The page refreshes when exFAT is toggled, and at that moment Ext4 isn't active yet. Fix: refresh on Ext4's toggle too.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** Fix the bug, so that the line shows as soon as Ext4 is chosen (T-035).

### F04 Ext4 greyed out for a Standard account
- **How:** You, in the second account.
- **Shows:** Ext4 greyed out, with "Only an administrator can choose Ext4, …".
- **Screenshots:** `F04-ext4-greyed.png`.
- **Noticed:** Taken by the user in the second account; Claude may not reach another session's screen.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### F05 No stick can be formatted
- **How:** You pull out the stick while F01 is open.
- **Shows:** the text for no drives, and no list.
- **Screenshots:** `F05-no-drives.png`.
- **Noticed:** The label and the file system choices stay on screen with nothing to format.
- **Comments:** Keep the format page's layout (2026-11-10).
- **Decision:** No change: the name and file system choices stay.

### F06 Formatting…
- **Shows:** a spinner, "Formatting BACKUP-A…", "This can take a few minutes. Don't pull out the stick.", and no way back.
- **Screenshots:** `F06-formatting.png`.
- **Noticed:** The page keeps the scroll position from choosing, so "Formatting BACKUP-A…" is cut off at the top.
- **Comments:** Agreed (2026-11-10), with G03.
- **Decision:** Starts at the top (G03).

### F07 BACKUP-A is ready
- **Shows:** green tick, "BACKUP-A is ready", what comes next, "Add this computer and account…" and Back.
- **Screenshots:** `F07-ready.png`.
- **Noticed:** As F06: the heading is out of sight, and "BACKUP-A is ready" is cut off at the top.
- **Comments:** Agreed (2026-11-10), with G03.
- **Decision:** Starts at the top (G03).

### F08 Formatting failed, nothing changed
- **How:** You, in the second account, with the stick still mounted for the first, cancel the password dialog.
- **Shows:** "Formatting failed", why, and "Nothing on the stick was changed."
- **Screenshots:** none: since item 110, "Prepare…" isn't offered for a stick open for another account, which was the way to reach it. It can't be reached in normal use.
- **Comments:** None.
- **Decision:** No change. It can't be reached in normal use.

## Setup of this stick

### C01 This computer isn't set up yet
- **How:** Advanced on, "Change setup…" on a freshly prepared stick.
- **Shows:** "Setup of this stick", "Older versions to keep of each file: 10", "This computer (…) isn't set up on this stick yet.", "Add this computer as …".
- **Known:** "Add this computer as steen-virtualbox" also adds the account running Babawus, but doesn't say so (T-024).
- **Screenshots:** `C01-not-set-up-de.png`, `C01-not-set-up-small.png`, `C01-not-set-up.png`.
- **Noticed:** The smallest size here is 561 × 478, different from F01.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** The button becomes "Add this computer and your account", and the line above it gives the folder name: "This computer (steen-VirtualBox) isn't set up on this stick yet. It will be added as steen-virtualbox."

### C02 This computer and account added, Save possible
- **Shows:** the entry with "(this computer)" and "(you)", "Choose what to back up…", and Save.
- **Screenshots:** `C02-added.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C03 Saved
- **Shows:** "Saved."
- **Screenshots:** `C03-saved.png`.
- **Noticed:** "Saved." shows above Save and Back, not near the top.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C04 "Leave without saving?"
- **How:** Back with unsaved changes.
- **Shows:** the question, with "Keep editing" and the button that discards.
- **Screenshots:** `C04-leave-unsaved.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C05 The setup was changed outside Babawus
- **How:** Claude edits `older_versions` in the configuration by hand.
- **Shows:** the warning that the setup "was changed outside Babawus, or is damaged".
- **Screenshots:** `C05-changed-outside.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C06 The setup can't be read at all
- **How:** Claude replaces the configuration with something that isn't one, for a moment.
- **Shows:** that it can't be read, and "Start a new setup…".
- **Screenshots:** `C06-unreadable.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** The paragraph uses the normal text style, not a larger one.

### C07 "Start a new setup?"
- **Shows:** the question, and that the backups are kept.
- **Screenshots:** `C07-start-new.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C08 Several computers and accounts
- **How:** Claude adds a second computer and account with Babawus's `add_account` example.
- **Shows:** every entry, with what each backs up, and the remove buttons.
- **Screenshots:** `C08-several.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C09 "Remove …?"
- **Shows:** the question, and what happens to that entry's backups.
- **Screenshots:** `C09-remove.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### C10 "Link to …?"
- **Shows:** the question for linking this computer or account to an existing entry.
- **Screenshots:** `C10-link.png`.
- **Noticed:** The suggested name was "steen-virtualbox-2", because the stick already had a `steen-virtualbox` folder from earlier backups. Correct, but it looks odd.
- **Comments:** The wording can be hard to parse: "Link this computer to laptop?" reads as "link this computer to this computer" for someone sitting at a laptop. Something like "Link this computer to the backup profile called 'laptop'?" is better, if more verbose (2026-11-10).
- **Decision:** Names of entries are always marked as names: "Link this computer to the backup profile called “laptop”?", and the button "Link to “laptop”…". The same for accounts: "Link your account to the backup profile called “anna”?". "steen-virtualbox-2" stays. The setup page and these questions use the same words for an entry, so if "backup profile" is used here, the page uses it too. The term becomes official in Requirements.md and Concept.md with the change (Issues.md item 119, task T-040).

## Choose what to back up

### T01 The home folder
- **How:** "Choose what to back up…" in the setup.
- **Shows:** the tree with "Your home folder" open, the totals, the name patterns, Done and Cancel.
- **Known:** the tree scrolls inside a page that scrolls too (T-032).
- **Screenshots:** `T01-home-de.png`, `T01-home-small.png`, `T01-home.png`.
- **Noticed:** The heading says "What's backed up for steen", while the button that opens it says "Choose what to back up…". `.local` shows although hidden files are off, because it holds the left-out Trash. Switching the language closes the open folders. Smallest size: 467 × 478.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** (a) The heading becomes "Choose what to back up for steen", like the button. (b) At 1024 × 768 the page doesn't scroll: the tree fills the free height and is the only part that scrolls, and the name patterns go in a section "Leave out by name…" that starts closed. (c) Switching the language keeps the open folders open.

### T02 A folder ticked, with what is left out
- **How:** `BabawusTest` ticked and opened, `Letters/old` unticked.
- **Shows:** `notes.tmp` crossed out with "Left out by the name pattern *.tmp", `old` crossed out, and the totals.
- **Screenshots:** `T02-ticked-left-out.png`.
- **Noticed:** `Letters` keeps a full tick although `old` inside it is left out. `meeting: notes.txt`, which exFAT can't store, and the unreadable `secret.txt` show nothing special.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** A folder with something left out inside it shows a partly ticked box, not a full tick.

### T03 Hidden folders shown
- **How:** "Show hidden folders" ticked.
- **Screenshots:** `T03-hidden-shown.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### T04 Name patterns, with a broken one
- **How:** a pattern with a slash, or one that is there already.
- **Shows:** the message under the pattern.
- **Screenshots:** `T04-pattern-slash.png`.
- **Noticed:** The message only appears after Add, below the fold, and it stays after the field is emptied.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** The message clears as soon as the field is changed or emptied. With T01 (b) it's no longer below the fold.

### T05 Nothing chosen
- **Shows:** the totals saying that nothing is chosen.
- **Screenshots:** `T05-nothing-chosen.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.

### T06 Chosen more than fits on the stick
- **How:** a large folder ticked, or the stick nearly full.
- **Shows:** the warning that it doesn't fit.
- **Screenshots:** `T06-too-big.png`.
- **Comments:** Agreed with the suggestion (2026-11-10).
- **Decision:** No change.
