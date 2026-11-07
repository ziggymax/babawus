# Babawus tasks

This is the central list of outstanding work. If something still needs doing, it is here.

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **`Requirements.md` says what Babawus does, `Issues.md` records questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them. An open question stays in `Issues.md`, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-030.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves to "Done", with a link to its pull request.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen. Within a section, tasks are in the order to do them.

## Now

T-014 is waiting for review. T-015, the verdict screen, is next.

## First version: logic

Everything here is done.

## First version: udisks2

Everything here is done.

## First version: user interface

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-015 | Verdict screen: reason, skipped files by reason, details with Copy, offer to eject. It replaces the stand-in end of the backup screen, which only shows the verdict and Back. Ejecting uses `Udisks::eject`, and says what to do if the medium is busy or could only be unmounted | Todo | Requirements: "Get a clear verdict" |
| T-016 | History and "what does this medium cover" views | Todo | Requirements: "View the history of a medium" |
| T-017 | Advanced: the format view. It uses `Udisks::format`, says when nothing was erased, and leads on to adding this computer and account | Todo | Requirements: "Format USB media" |
| T-018 | Advanced: the configuration editor, with the file explorer, live totals, read-only entries for others, and adding, linking and removing entries. It also offers checking a damaged configuration, and starting a new one | Todo | Requirements: "Configure USB media" |
| T-020 | Translations: the German texts in the catalogue, including codes from the log | Todo | Requirements: "Change the language" |

## First version: release

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-021 | Write the manual release checklist | Todo | Requirements: "Testing" |
| T-022 | Pull a real stick during a copy, and check that its errors count as "medium gone" | Todo | Pull request 14 |
| T-024 | Test formatting from an account without administrator rights | Todo | Issues: 38 |
| T-025 | Install, update and uninstall the `.deb` for real, and write the uninstall notes | Todo | Requirements: "Packaging" |
| T-029 | `real_processes_are_told_apart` failed once in a full `cargo test`: it may check the copy of the test program before `exec` has replaced its name, which until then is the test thread's. Wait until the copy is running before checking | Todo | Pull request 31 |

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
