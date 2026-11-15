# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-076.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), with a link to its pull request, and keeps its number. Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 0.2.1 is released (T-064), and 1.0.0-alpha.1, the document cleanup, is done (T-065 to T-071). Being worked on is 1.0.0-alpha.2, the second of three versions leading to 1.0.0 ([ReleasePlanning.md](ReleasePlanning.md)): language files loaded at runtime (issue 147).

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-073 | Load the TOML language files from `/usr/share/babawus/languages/` at runtime, move English and German into them, and install them with the `.deb`. A broken or incomplete file is not offered, and the reason goes to standard error; without a usable English file, Babawus shows an error and stops. Update Requirements.md | Todo | ReleasePlanning.md, issues 147 and 148 |
| T-074 | Write a short guide to adding a language: where the file goes, what it holds, how to check it, and how to see why a file is rejected | Todo | Issue 147 |
| T-075 | Raise the version to 1.0.0-alpha.2 and update ReleasePlanning.md. After it is merged, tag `v1.0.0-alpha.2` on main | Todo | Issue 147 |

## Needs a decision

Nothing is waiting for a decision.

## Waiting

Problems seen once and not since. Each is closed if it can't be made to happen again.

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-061 | Find out why `computer_lock::tests::a_released_lock_can_be_taken_and_records_only_the_new_user` failed once on CI ([#74](https://github.com/ziggymax/babawus/pull/74)), reading back the first user's name instead of the second's. Not reproduced locally in 20 tries. If it can't be made to happen again, close it | Todo | [#74](https://github.com/ziggymax/babawus/pull/74) |
| T-043 | Find out why the buttons under "You on this computer" were once drawn squeezed under the line below them, after the setup had been read again in another state. Seen once on the VM while checking T-040 ([#55](https://github.com/ziggymax/babawus/pull/55)), and not again in four tries; main may have it too. If it can't be made to happen again, close it | Todo | [#55](https://github.com/ziggymax/babawus/pull/55) |

## Later versions

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-027 | A "check this medium" function, for example using stored checksums | Todo | Requirements: "Not in Babawus yet" |
| T-028 | Detect moved files, to avoid storing them twice. An APT repository, so that updates arrive through the Update Manager, could come at the same time | Todo | Requirements: "Not in Babawus yet", "Packaging" |
