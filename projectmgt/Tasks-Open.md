# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-142.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), with a link to its pull request, and keeps its number. Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 1.1.1 is released (T-113). Next is release 1.2.0, other Linux distributions (see "Step 1" in [ReleasePlanning.md](ReleasePlanning.md)), in two interim versions. 1.2.0-alpha.1, which builds, tests and packages Babawus on every distribution, is done (T-118 to T-123, T-130, T-131; issue 169). The test VMs of 1.2.0 are done too (T-124 to T-129, T-132, T-133; issue 170): three guests, a smoke test, and CI keeping the packages a release is made from. 1.2.0-alpha.2, GNOME, is done too (T-135 to T-140; issue 174). T-141, AppStream metadata, is done too (issue 175). Next is the release of 1.2.0 (issue 176), as "A release, step by step" in [TestSetup.md](../TestSetup.md) says: the version 1.2.0 (T-142, done) and T-147, found on the Mint checklist (done), then the smoke test and the checklist on every distribution, on the packages GitHub built for that commit, and the release.

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-143 | Run the smoke test on every guest, and the whole release checklist on Linux Mint 22.3, on the 1.2.0 packages GitHub built for main once T-147 is merged (the first run, on T-142's commit, found issue 177), recorded in an issue "Release 1.2.0 checklist on Linux Mint 22.3" | Todo | Issue 176 |
| T-144 | Run the whole release checklist on Ubuntu 26.04, on the same packages, recorded in an issue "Release 1.2.0 checklist on Ubuntu 26.04" | Todo | Issue 176 |
| T-145 | Run the whole release checklist on Fedora 45, on the same packages, recorded in an issue "Release 1.2.0 checklist on Fedora 45" | Todo | Issue 176 |
| T-146 | Release 1.2.0: the annotated tag `v1.2.0` on the commit whose packages were tested, and a GitHub release with that commit's `.deb`, `.rpm` and `SHA256SUMS` and the notes with the fixed parts, reviewed by you; ReleasePlanning.md names 1.2.0 as the latest release | Todo | Issue 176 |

## Needs a decision

Nothing is waiting for a decision.

## Waiting

Problems seen once and not since. Each is closed if it can't be made to happen again.

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-043 | Find out why the buttons under "You on this computer" were once drawn squeezed under the line below them, after the setup had been read again in another state. Seen once on the VM while checking T-040 ([#55](https://github.com/ziggymax/babawus/pull/55)), and not again in four tries; main may have it too. If it can't be made to happen again, close it | Todo | [#55](https://github.com/ziggymax/babawus/pull/55) |

## Later versions

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-027 | A "check this medium" function, for example using stored checksums | Todo | Requirements: "Not in Babawus yet" |
| T-028 | Detect moved files, to avoid storing them twice. An APT repository, so that updates arrive through the Update Manager, could come at the same time | Todo | Requirements: "Not in Babawus yet", "Packaging" |
