# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-125.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), with a link to its pull request, and keeps its number. Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 1.1.1 is released (T-113). Next is release 1.2.0, other Linux distributions (see "Step 1" in [ReleasePlanning.md](ReleasePlanning.md)), in two interim versions. The host is set up for development (T-118). These tasks make 1.2.0-alpha.1, which builds, tests and packages Babawus on every distribution (issue 169).

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-119 | A container for each distribution (Ubuntu 24.04 and 26.04, Debian stable, the newest Fedora) that builds Babawus and runs clippy and its unit tests: a script for podman on the host, and a CI job for each on every pull request. The tests on real file systems, which need udisks2 and loop devices, stay in the existing job on the runner | Todo | Issue 169 |
| T-120 | Count Fedora's `wheel` group as administrator, as `sudo` and `admin` are on Mint and Ubuntu, checked against the polkit rules of Debian and Fedora (issue 108) | Todo | Issue 169 |
| T-121 | Build an `.rpm` for Fedora, with Fedora's package names for its dependencies (udisks2, exfatprogs, e2fsprogs, acl), and check in the Fedora container that it installs, with all its dependencies, and that `ldd` finds every library `/usr/bin/babawus` needs | Todo | Issue 169 |
| T-122 | Check in the containers that the `.deb` built on Ubuntu 24.04 installs on Ubuntu 26.04 and Debian stable, with all its dependencies, and that `ldd` finds every library `/usr/bin/babawus` needs | Todo | Issue 169 |
| T-123 | Raise the version to 1.2.0-alpha.1 and tag `v1.2.0-alpha.1` on main, without a GitHub release | Todo | Issue 169 |

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
| T-124 | Make a test VM each for Ubuntu 24.04 and 26.04, Debian stable and the newest Fedora, with the desktop each ships, SSH from the host and the USB sticks passed through, and describe them in DevEnvSetup.md. Needed before the release checklists of 1.2.0 run | Todo | Issue 169 |
| T-027 | A "check this medium" function, for example using stored checksums | Todo | Requirements: "Not in Babawus yet" |
| T-028 | Detect moved files, to avoid storing them twice. An APT repository, so that updates arrive through the Update Manager, could come at the same time | Todo | Requirements: "Not in Babawus yet", "Packaging" |
