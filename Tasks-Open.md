# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-130.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), with a link to its pull request, and keeps its number. Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 1.1.1 is released (T-113). Next is release 1.2.0, other Linux distributions (see "Step 1" in [ReleasePlanning.md](ReleasePlanning.md)), in two interim versions. The host is set up for development (T-118), each distribution has a container (T-119), and Fedora's `wheel` group counts as administrator (T-120). These tasks make 1.2.0-alpha.1, which builds, tests and packages Babawus on every distribution (issue 169).

| ID | Task | Status | Source |
| --- | --- | --- | --- |
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

T-125 to T-129 and T-124 make the test VMs of 1.2.0, between its two interim versions (issue 170, and "The VMs" in [ReleasePlanning.md](ReleasePlanning.md)).

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-125 | Describe Babawus's development, test and release set-up in a document of its own in the project, from the research paper and the decisions in issue 170: the containers, the VMs under libvirt, the driver and its three channels, emulated and real sticks, the smoke test, and how a release's packages are built and tested. It is kept up to date as T-126 to T-129 and T-124 show what works | Todo | Issue 170 |
| T-126 | Install libvirt on the host, in the user's session, and prove the driver on one guest, Ubuntu 26.04 with GNOME on Wayland: a small command-line driver that reverts the guest to a clean snapshot, takes a screenshot, clicks, types, runs a command over SSH, and plugs in and unplugs an emulated stick. Find out whether VirtualBox and KVM can run at the same time, and what passing a real stick through needs in the user session. The user does the steps that need sudo | Todo | Issue 170 |
| T-127 | Move the Linux Mint VM from VirtualBox to libvirt, by converting its disk or installing it anew, and drive it with the same driver. VirtualBox is removed when nothing needs it any more | Todo | Issue 170 |
| T-124 | Make a test VM each for Ubuntu 24.04, Debian stable and the newest Fedora in libvirt, as the Ubuntu 26.04 one of T-126: the desktop each ships, automatic login, SSH from the host, and a clean snapshot. Describe them in DevEnvSetup.md. Needed before the release checklists of 1.2.0 run | Todo | Issues 169, 170 |
| T-128 | A smoke test that runs on every guest without Claude: revert to the clean snapshot, install the package as a user would, start Babawus, plug in an emulated stick and check that it shows up, and run the tests on real file systems, built in that distribution's container | Todo | Issue 170 |
| T-129 | Let GitHub build the `.deb` and the `.rpm` of a release, so that the checklists run on the very files that are released | Todo | Issue 170 |
| T-027 | A "check this medium" function, for example using stored checksums | Todo | Requirements: "Not in Babawus yet" |
| T-028 | Detect moved files, to avoid storing them twice. An APT repository, so that updates arrive through the Update Manager, could come at the same time | Todo | Requirements: "Not in Babawus yet", "Packaging" |
