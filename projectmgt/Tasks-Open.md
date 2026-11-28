# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is the highest one in this list and in Tasks-Done.md, plus one; it is not written down anywhere, so there is nothing to keep up to date (issue 185).
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), and keeps its number. The pull request's title names each task it finishes, so `git log --grep "T-120\b"` finds it (issue 185). Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 1.2.0 is released (T-146), for Linux Mint, Ubuntu and Fedora. Next is release 1.2.1, which fixes 1.2.0's two known issues and the "Prepare a stick" page (see "Release 1.2.1" in [ReleasePlanning.md](ReleasePlanning.md); issue 182). It is released as "A release, step by step" in [TestSetup.md](../TestSetup.md) says. Before it, the test VMs are automated (T-166 to T-173; issue 192): accounts with test passwords, guests built and updated by scripts, and a release test suite that drives the window through AT-SPI, which 1.2.1's checklists then run on.

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-166 | In DevEnvSetup.md, explain how the test VMs work underneath: how a guest is reset to its `clean` snapshot and what that snapshot holds, how an emulated stick is made and plugged in and pulled out through libvirt, and how a real stick is passed through | Todo | Issue 192 |
| T-167 | Give every guest the accounts `tester` (administrator) and `standard` (no administrator rights), with test passwords in `vms/accounts.toml`, through one post-install step shared by all guests (accounts, SSH, `sudo`, automatic login, the desktop's settings); let the driver log in as either account through the display manager's automatic login, and type an account's password from the file; apply it to the existing guests | Todo | Issue 192 |
| T-168 | Update a guest before release testing with one command: back to `clean`, upgrade the system, check that Babawus isn't installed, restart and log in, and save it as the new `clean`, keeping the one before as `clean-previous` and writing down the package versions | Todo | Issue 192 |
| T-169 | Build the Ubuntu 26.04 guest from scratch with `vms/build.py`: download and check the ISO, install with autoinstall, run T-167's step and T-168's update, make the stick images the tests start from, and take `clean` | Todo | Issue 192 |
| T-170 | Build the Fedora 45 guest from scratch the same way, with kickstart | Todo | Issue 192 |
| T-171 | Build the Linux Mint 22.3 guest from scratch the same way, with preseed, and the fewest clicks through the driver that its installer allows | Todo | Issue 192 |
| T-172 | Give Babawus's widgets accessible names, so that a test, and a screen reader, can find each of them in every language | Todo | Issue 192 |
| T-173 | Write the release test suite in `vms/`, which drives Babawus through AT-SPI in each guest and reports each step of the release checklist it covers; mark those steps in ReleaseChecklist.md, and change "A release, step by step" in TestSetup.md to run it | Todo | Issue 192 |
| T-152 | Raise the version to 1.2.1, with its entry in the metainfo's `<releases>` for the planned day; the update step of ReleaseChecklist.md then raises the version from `1.2.1` to `1.2.2` | Todo | Issue 182 |
| T-153 | Run the smoke test on every guest, and the whole release checklist on Linux Mint 22.3, on the 1.2.1 packages GitHub built for T-152's commit, recorded in an issue "Release 1.2.1 checklist on Linux Mint 22.3" | Todo | Issue 182 |
| T-154 | Run the whole release checklist on Ubuntu 26.04, on the same packages, recorded in an issue "Release 1.2.1 checklist on Ubuntu 26.04" | Todo | Issue 182 |
| T-155 | Run the whole release checklist on Fedora 45, on the same packages, recorded in an issue "Release 1.2.1 checklist on Fedora 45" | Todo | Issue 182 |
| T-156 | Once the checklists have passed, set the day 1.2.1 is released in the metainfo, and run the smoke test on every guest on that commit's packages | Todo | Issue 182 |
| T-157 | Release 1.2.1: the annotated tag `v1.2.1` on T-156's commit, and a GitHub release with that commit's `.deb`, `.rpm` and `SHA256SUMS` and the notes with the fixed parts, reviewed by you; ReleasePlanning.md names 1.2.1 as the latest release | Todo | Issue 182 |

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
