# Babawus open tasks

This is the central list of outstanding work. If something still needs doing, it is here. What is done is in [Tasks-Done.md](Tasks-Done.md).

## How this list works

- **One place.** Anything that comes up as "for later" gets a task here straight away, whether it is a feature, a test, a clean-up or a small to-do.
- **[Requirements.md](Requirements.md) says what Babawus does, [Issues-Unsettled.md](Issues-Unsettled.md) and [Issues-Resolved.md](Issues-Resolved.md) record questions and why they were decided, and this list says what is left to do.** A task links to those instead of repeating them, as "issue 132". An open question stays in Issues-Unsettled.md, with a task here to settle it.
- **IDs never change or get reused.** New tasks take the next free number, whatever section they go in. The next free number is T-115.
- **To-dos in the code** are welcome, but each one names its task, so that nothing lives only in a comment: `// TODO(T-012): Show the free space too.`
- **Pull requests** mention the tasks they finish, and update this list. A finished task moves word for word to [Tasks-Done.md](Tasks-Done.md), with a link to its pull request, and keeps its number. Elsewhere a task is referred to as just "T-063", so references don't go stale when it moves.

| Status | Meaning |
| --- | --- |
| Todo | Not started |
| In progress | Being worked on now |
| Blocked | Can't go on: the task says what it is waiting for |

Sections run from what comes next to what may never happen: the version being worked on, what needs a decision, what waits, and later versions. Within a section, tasks are in the order to do them.

## Now

Babawus 1.1.0 is released (T-101). Next is release 1.1.1, step 0 of bringing Babawus to more platforms: the Linux-specific code moves behind a platform layer, without changing what Babawus does (see "Step 0" in [ReleasePlanning.md](ReleasePlanning.md)). The tasks for 1.2.0, other Linux distributions, are created when work on it begins.

| ID | Task | Status | Source |
| --- | --- | --- | --- |
| T-108 | Move identity behind the platform layer: the computer's ID (`/etc/machine-id`) and name (`identity.rs`), and the boot ID (`lock.rs`) | Todo | ReleasePlanning.md: "Step 0" |
| T-109 | Move flushing and locking behind the platform layer: `syncfs` and `fsync` (`fs/real.rs`, `copy.rs`), the computer lock (`computer_lock.rs`), and how a lock checks whether its process is still running (`/proc` in `lock.rs`) | Todo | ReleasePlanning.md: "Step 0" |
| T-110 | Move paths behind the platform layer: the settings file (`settings.rs`), the note of the last backup (`reminder.rs`) and the installed language files (`language.rs`) | Todo | ReleasePlanning.md: "Step 0" |
| T-114 | Make CI check that `babawus-core` compiles for Windows, so that Linux-only code can't slip back into the shared parts: a placeholder Windows implementation of the platform layer that compiles but says Windows isn't supported yet, and a CI step `cargo check -p babawus-core --target x86_64-pc-windows-gnu`. Needs T-107 to T-110, so that `gio`, `libc` and `/proc` are only used on Linux | Todo | ReleasePlanning.md: "Step 0" |
| T-111 | Raise the version to 1.1.1, the build that the release checklist runs on (T-112) | Todo | ReleasePlanning.md: "Step 0" |
| T-112 | Run the whole release checklist on the 1.1.1 build on Linux Mint 22.x. Nothing a user sees should have changed | Todo | ReleasePlanning.md: "Step 0" |
| T-113 | Release 1.1.1: tag `v1.1.1` on main, and a GitHub release with the `.deb` and notes. ReleasePlanning.md names 1.1.1 as the latest release | Todo | ReleasePlanning.md: "Step 0" |

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
