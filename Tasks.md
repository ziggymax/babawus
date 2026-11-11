# Babawus tasks

This is the central list of outstanding work. If something still needs doing, it is here.

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **`Requirements.md` says what Babawus does, `Issues.md` records questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them. An open question stays in `Issues.md`, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-051.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves to "Done", with a link to its pull request.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen. Within a section, tasks are in the order to do them.

## Now

T-044, the test that failed now and then, is waiting for review. Next is a full run of the release checklist for 0.1.0, now with a second stick. T-043 waits for its problem to show again.

## First version: logic

Everything here is done.

## First version: udisks2

Everything here is done.

## First version: user interface

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-043 | Find out why the buttons under "You on this computer" were once drawn squeezed under the line below them, after the setup had been read again in another state. Seen once on the VM while checking T-040 (pull request 55), and not again in four tries; main may have it too. If it can't be made to happen again, close it | Todo | Pull request 55 |

## First version: release

Everything here is done.

## Needs a decision

Nothing is waiting for a decision.

## Later versions

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-027 | A "check this medium" function, for example using stored checksums | Todo | Requirements: "Not in the first version" |
| T-028 | Detect moved files, to avoid storing them twice. An APT repository, so that updates arrive through the Update Manager, could come at the same time | Todo | Requirements: "Not in the first version", "Packaging" |

## Done

| ID | Task | Pull request |
| --- | --- | --- |
| T-001 | Configuration: file format, checksum, reading, safe writing, adding, linking and removing entries | [#19](https://github.com/ziggymax/babawus/pull/19) |
| T-002 | Computer ID from `/etc/machine-id` | [#19](https://github.com/ziggymax/babawus/pull/19) |
| T-003 | Verdicts and reason codes | [#20](https://github.com/ziggymax/babawus/pull/20) |
| T-004 | Log: writing, trimming, closing interrupted runs, reading, history and last backup, and renaming an account in it when linking | [#20](https://github.com/ziggymax/babawus/pull/20) |
| T-026 | Name older versions by UTC, so that the clock going back can't make pruning remove the wrong one | [#21](https://github.com/ziggymax/babawus/pull/21) |
| T-005 | Medium lock: create, check whether it is live, remove stale locks | [#22](https://github.com/ziggymax/babawus/pull/22) |
| T-006 | Run orchestration: from taking the lock to the verdict, on a worker thread, with progress messages and Cancel | [#23](https://github.com/ziggymax/babawus/pull/23) |
| T-007 | Keep "hostname last seen" up to date in the configuration | [#24](https://github.com/ziggymax/babawus/pull/24) |
| T-008 | One Babawus per computer: the lock in `/run/lock/`, and bringing the existing window to the front | [#25](https://github.com/ziggymax/babawus/pull/25) |
| T-009 | Settings file and the choice of language | [#26](https://github.com/ziggymax/babawus/pull/26) |
| T-010 | Find USB media, work out their file system and status, and follow changes | [#27](https://github.com/ziggymax/babawus/pull/27) |
| T-011 | Format a medium, including the check of the drive ID, permissions and the first configuration | [#29](https://github.com/ziggymax/babawus/pull/29) |
| T-023 | Test the GPT partition types on a medium over 2 TiB, or with a simulated one | [#29](https://github.com/ziggymax/babawus/pull/29) |
| T-012 | Eject a medium | [#30](https://github.com/ziggymax/babawus/pull/30) |
| T-019 | Test powering off a real stick, and remove the `pkexec` fallback if nothing needs it | [#30](https://github.com/ziggymax/babawus/pull/30) |
| T-013 | Start screen: media list, details, actions, the menu with Language, Advanced and About, Eject, and the text catalogue | [#31](https://github.com/ziggymax/babawus/pull/31) |
| T-014 | Backup screens: overview, progress, Cancel, "Finishing…", and asking before closing during a backup | [#32](https://github.com/ziggymax/babawus/pull/32) |
| T-015 | Verdict screen: reason, skipped files by reason, details with Copy, offer to eject | [#33](https://github.com/ziggymax/babawus/pull/33) |
| T-029 | Wait until the copy of the test program runs before checking it, so that `real_processes_are_told_apart` no longer fails now and then | [#33](https://github.com/ziggymax/babawus/pull/33) |
| T-016 | History and "What's backed up" views | [#34](https://github.com/ziggymax/babawus/pull/34) |
| T-017 | Advanced: the format view, which says when nothing was erased, and leads on to adding this computer and account | [#35](https://github.com/ziggymax/babawus/pull/35) |
| T-018 | Advanced: the configuration editor's overview: older versions, every entry, adding, linking and removing entries, Save and Cancel, a damaged configuration, and starting a new one | [#36](https://github.com/ziggymax/babawus/pull/36) |
| T-031 | Advanced: "Choose what to back up…": the home folder tree with tick boxes, the name patterns, and live totals | [#37](https://github.com/ziggymax/babawus/pull/37) |
| T-020 | Translations: the German texts in the catalogue | [#38](https://github.com/ziggymax/babawus/pull/38) |
| T-030 | "Last backup" shows when the last backup finished, and counts calendar days | [#38](https://github.com/ziggymax/babawus/pull/38) |
| T-021 | Write the manual release checklist | [#43](https://github.com/ziggymax/babawus/pull/43) |
| T-025 | Install, update and uninstall the `.deb` for real, for the first time, and write the uninstall notes. The menu icon now names its file by full path | [#44](https://github.com/ziggymax/babawus/pull/44) |
| T-022 | Pull a real stick during a copy, and check that its errors count as "medium gone": the checklist's "Pulling out a stick", for the first time | [#45](https://github.com/ziggymax/babawus/pull/45) |
| T-034 | Wait until udisks2 has read the label before checking it, so that `exfat_is_reported` no longer fails now and then | [#45](https://github.com/ziggymax/babawus/pull/45) |
| T-024 | Test formatting from an account without administrator rights, for the first time. Ext4 can now only be chosen by an administrator, and a stick open for another account says so | [#46](https://github.com/ziggymax/babawus/pull/46) |
| T-033 | Give the package a longer description, for the installer to show | [#47](https://github.com/ziggymax/babawus/pull/47) |
| T-032 | Review the look of the user interface as a whole: every screen listed, screenshotted and decided in `ui-review/UiReview.md`. The changes are tasks T-035 to T-041 | [#50](https://github.com/ziggymax/babawus/pull/50) |
| T-037 | Open the window at 1024 × 768, and start every page that scrolls at the top whenever it opens or changes part | [#51](https://github.com/ziggymax/babawus/pull/51) |
| T-038 | The start screen as the review decided: a larger stick picture and a bold label in the list; in the details the label, the stick's details, a thin line, then the status and every message in one style; a larger logo; buttons at their own width, Eject apart, the poweruser's on a row of their own; and texts for a stick that is plugged in but not listed (Issues.md items 120 and 121) | [#52](https://github.com/ziggymax/babawus/pull/52) |
| T-042 | Wait until udisks2 has read the file system before checking a real medium's status, so that `a_real_babawus_medium_is_ready` no longer fails now and then, as T-034 did for `exfat_is_reported` | [#53](https://github.com/ziggymax/babawus/pull/53) |
| T-039 | The verdict screen as the review decided: three text styles, no reason line repeating "Some files were skipped", and "Run the backup again when it suits you." after the user stopped a backup. Also brings back two text styles that T-038 had removed while other pages still used them | [#54](https://github.com/ziggymax/babawus/pull/54) |
| T-040 | Setup of this stick: "Add this computer and your account", with the folder name in the line above; the unreadable-setup text in the normal style; and "backup profile" for one account on one computer, with names quoted in sentences, made official in Requirements.md and Concept.md (Issues.md items 119 and 122) | [#55](https://github.com/ziggymax/babawus/pull/55) |
| T-041 | Choose what to back up: the heading "Choose what to back up for …", which now also follows the language; only the tree scrolls, with the name patterns in a section "Leave out by name" that starts closed; open folders stay open when the language changes; a partly ticked box (Issues.md item 123); and a pattern's message cleared when the field changes. Also About without the name selected, and "already open" as a proper message | [#56](https://github.com/ziggymax/babawus/pull/56) |
| T-035 | Show "Ext4 asks for your password while formatting." as soon as Ext4 is chosen: the format page is now shown again when Ext4 is toggled too, not only exFAT | [#57](https://github.com/ziggymax/babawus/pull/57) |
| T-036 | Remove "Details for your poweruser" and Copy from the verdict screen, and make the log enough for diagnosing on its own: Babawus's version and the hostname in each start entry, and the first 100 skipped files and errors of a run, instead of 20 (Issues.md items 117 and 124). Requirements.md, the checklist's Copy step and the README, which now says where the log is, are updated | [#58](https://github.com/ziggymax/babawus/pull/58) |
| T-045 | The start screen's buttons: the stick list narrows to 240 pixels, so that the user's four buttons fit on one line at 1024 × 768 in German too, and a layout of Babawus's own wraps them like words in a narrower window, instead of GTK's `FlowBox`, which spread them out in columns (Issues.md item 125) | [#59](https://github.com/ziggymax/babawus/pull/59) |
| T-046 | Name the name patterns' section "Files to leave out by name" | [#60](https://github.com/ziggymax/babawus/pull/60) |
| T-047 | After saving the setup, say "Stick setup saved." instead of "Saved." | [#60](https://github.com/ziggymax/babawus/pull/60) |
| T-048 | Say how ejecting went in a pop-up with OK, on the start screen and the verdict screen, instead of a notice on the page; after a successful eject from the verdict screen, go back to the start screen | [#60](https://github.com/ziggymax/babawus/pull/60) |
| T-049 | Keep the window's height after leaving "Choose what to back up…": since T-041 that page doesn't scroll as a whole, and while hidden it still set the window's least height, so Cancel made the window grow from 797 to 942 pixels. Each screen now asks only for its own height | [#60](https://github.com/ziggymax/babawus/pull/60) |
| T-050 | The name under the elephant in the logo is black with a thinner outline, instead of green with a black one. A change by the user to `assets/BabawusLogo.svg` and `assets/BabawusLogo-plain.svg` | [#61](https://github.com/ziggymax/babawus/pull/61) |
| T-044 | Find the test that fails now and then. On CI for pull request 61, `exfat_is_reported` timed out: udisks2 never reported the test stick's label and file system. The test sticks are formatted outside udisks2, which relies on udev to look at a device again once it has been written; on a busy machine that look can come too early or be lost. The test helper now asks udev to look again after formatting, and waits for it, and a wait that gives up says what udisks2 reported instead | [#62](https://github.com/ziggymax/babawus/pull/62) |
