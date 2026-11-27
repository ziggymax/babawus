# Working on Babawus with Claude

How work on Babawus is done when Claude does it with the user. Claude Code reads this file at the start of every session. What Babawus is and does is in [README.md](README.md) and [Requirements.md](projectmgt/Requirements.md); the commands for building and testing are under "Building and testing" in [DevEnvSetup.md](DevEnvSetup.md); how it is built, tested and released is in [TestSetup.md](TestSetup.md).

## Where things are written down

- **Outstanding work** is in [Tasks-Open.md](projectmgt/Tasks-Open.md), and only there. Anything left "for later" gets a task at once, not a note in chat, in a pull request or in a comment. Its "Now" section says where the project stands: read it first.
- **Decisions** are in [Issues-Resolved.md](projectmgt/Issues-Resolved.md), as "issue 132". Before starting or restarting a task, search it for the task's decisions. When the user answers a design question, write the answer there as "**Decided (date):**" before building anything, so that it isn't lost with the session.
- **What Babawus does** is in [Requirements.md](projectmgt/Requirements.md), which changes in the same pull request as the behaviour.
- **To-dos in the code** name their task: `// TODO(T-012): …`.
- "Issue 132" is an issue in the Issues files; "#132" is a GitHub issue or pull request.
- **New tasks and issues take the next free number**, which is the highest in the files plus one, on a fresh `origin/main`. If another branch is still unmerged, count its numbers too.
    - Highest task: `grep -ohE 'T-[0-9]+' projectmgt/Tasks-*.md | sort -t- -k2 -n | tail -1`
    - Highest issue: `grep -ohE '^[0-9]+\.' projectmgt/Issues-*.md | sort -n | tail -1`
- **The pull request that finished a task** is found with `git log --grep "T-120\b"`.

## A task, step by step

1. Run `git fetch`, and branch from `origin/main` as `t-NNN-short-name`.
2. Ask the open design questions, at most four at a time, with the recommended option first. Record the answers as issues before building.
3. Build the logic in `babawus-core`, with unit tests, and the window in `babawus`. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.
4. Check the result in the real application on the test VMs (see below).
5. Update Requirements.md and the user's documents where the change shows. Move the task word for word from Tasks-Open.md to the end of [Tasks-Done.md](projectmgt/Tasks-Done.md).
6. Ask "Shall I commit, push and open the pull request?", and wait for a yes. Then commit, push and open it in one go. Its title names each task it finishes, written out one by one and not as a range, since that is how a task's pull request is found: "Count Fedora's wheel group as administrator (T-120)".
7. The user merges, and says when to carry on. Claude never merges.

If the user asks for a change while another is uncommitted, ask whether it goes into the same pull request or a new one.

## Pull requests and CI

- **Before pushing to an existing pull request**, check that it is still open: `gh pr view <n> --json state`. The user may have merged it meanwhile. If so, branch anew from `origin/main`, and ask before opening a new pull request.
- **Pull requests are squash-merged.** A branch stacked on one that was merged is carried over with `git rebase --onto origin/main <old base> <branch>`, never by resolving the conflicts on GitHub, which once dropped a module without CI noticing. Ask before a force-push.
- **Read CI** with `gh run list --branch <branch>` and `gh run view <id> --log-failed`. A new run can take a minute to show up.
- **Change a pull request's description** with `gh api -X PATCH repos/ziggymax/babawus/pulls/<n> -F body=@<file>`; `gh pr edit` fails here.

## Checking on the test VMs

- The guests are `mint-22.3` (Cinnamon, X11, `.deb`), `ubuntu-26.04` (GNOME, Wayland, `.deb`) and `fedora-45` (GNOME, Wayland, `.rpm`), driven with `vms/driver.py` (see "The driver" in TestSetup.md).
- **A change the user can see, or one that touches the desktop, is checked on all three**, each with the package built for it, installed as a user would. No guest stands in for another. Say in the summary which guests were checked.
- **Before calling something a bug**, rule out the driver: take a screenshot a second or more after an action, since guests draw late, and move the pointer away before a click, since a tooltip left open takes the click.
- **Formatting a real stick**, or changing one in any other way that can't be undone, needs a fresh yes from the user each time.
- The user types passwords in the guests, and does what needs hands.

## Reviews

When a review of a document or a plan raises many questions, ask them one at a time, as "Question n of N", with follow-ups as "nb", "nc". Change the files only when all are answered.

## Releases

A release follows "A release, step by step" in TestSetup.md and [ReleaseChecklist.md](projectmgt/ReleaseChecklist.md); its plan is in [ReleasePlanning.md](projectmgt/ReleasePlanning.md). The user reviews the release notes before the release is made.
