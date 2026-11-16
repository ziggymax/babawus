# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-088.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), with a link to its pull request, and keeps its number. Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 0.2.1 is released (T-064). The three versions leading to 1.0.0 ([ReleasePlanning.md](ReleasePlanning.md)) are done (T-065 to T-080), and Danish was added after them (T-081, T-082). Now 1.0.0 is being released (issue 157). T-087 came first, because from 1.0.0 the language file format is something others build on.

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-083 | Bring ReleaseChecklist.md up to date for 1.0.0: the Help, Danish, and adding a language file by hand with `sudo`, kept by an update and left behind by uninstalling | In progress | Issue 157 |
| T-084 | Raise the version to 1.0.0 | Todo | Issue 157 |
| T-085 | Run the release checklist on the 1.0.0 build, recorded in an issue "Release 1.0.0 checklist": Claude drives the GUI on the VM, the user does the steps that need hands or a password. A failed step gets a task before the release goes ahead | Todo | Issue 157 |
| T-086 | Release 1.0.0: the tag `v1.0.0` on main, and a GitHub release with the `.deb` and notes drafted by Claude and reviewed by the user. ReleasePlanning.md then names 1.0.0 as the latest release, and says the next isn't planned yet | Todo | Issue 157 |

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
