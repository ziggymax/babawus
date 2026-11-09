# Babawus tasks

This is the central list of outstanding work. If something still needs doing, it is here.

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **`Requirements.md` says what Babawus does, `Issues.md` records questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them. An open question stays in `Issues.md`, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-032.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves to "Done", with a link to its pull request.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen. Within a section, tasks are in the order to do them.

## Now

T-020 and T-030 are waiting for review. The release tasks, from T-021, are next.

## First version: logic

Everything here is done.

## First version: udisks2

Everything here is done.

## First version: user interface

Everything here is done.

## First version: release

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-021 | Write the manual release checklist | Todo | Requirements: "Testing" |
| T-022 | Pull a real stick during a copy, and check that its errors count as "medium gone" | Todo | Pull request 14 |
| T-024 | Test formatting from an account without administrator rights | Todo | Issues: 38 |
| T-025 | Install, update and uninstall the `.deb` for real, and write the uninstall notes | Todo | Requirements: "Packaging" |

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
