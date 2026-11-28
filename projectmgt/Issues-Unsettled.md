# Babawus – Unsettled issues

The questions about Babawus that are still open. Once decided, an issue moves word for word to [Issues-Resolved.md](Issues-Resolved.md), with the decision and its date.

## How this list works

- **Numbers never change or get reused.** A new issue takes the next free number here, and keeps it when it moves. The next free number is the highest one here and in Issues-Resolved.md, plus one; it is not written down anywhere (issue 185).
- **References** elsewhere say just "issue 132", without a file name, so they never go stale. GitHub issues and pull requests are "#63".
- **Each issue** states the question, the options with what speaks for each, and which tasks wait for it. A task in [Tasks-Open.md](Tasks-Open.md) points at every open issue that needs settling.
- **A decision** is written as "**Decided (date):**" under the issue as soon as it is given, and the issue then moves to Issues-Resolved.md.

## Open issues

190. **The log window may not scroll to the highlighted run.** Found on Fedora 45 (2026-11-28), also with main's own build before T-163: "Show in the log" for the second newest run of a log of 33 lines, in a window showing 27, highlighted its lines 26 to 29 but left the window at the top, so that only the first two lines of the run showed, at the bottom edge. Requirements.md says the window scrolls to the run if it isn't in view (issue 135). On Linux Mint and Ubuntu, where the window showed the whole log, there was nothing to scroll.
    - **Options:** fix it in 1.2.1, since it is a small fault in what 1.2.0 promises; or leave it for a later version, since the run is highlighted and one scroll shows it.
    - **Waits:** T-164.

191. **On Ubuntu 26.04, Babawus doesn't follow GNOME's dark style.** Found on Ubuntu 26.04 (2026-11-28): with "Dark" chosen in Settings, which sets `color-scheme` to `prefer-dark`, and the settings portal reporting it, Babawus stayed light; with a dark GTK theme, such as Yaru-dark, it was dark. On Fedora 45 it follows the dark style. Ubuntu has GTK 4.22, Fedora 4.24. Babawus doesn't use libadwaita, which is what follows the style in most GNOME apps.
    - **Options:** fix it in 1.2.1, which needs finding out why GTK 4.22 doesn't apply the style and a check on every guest; or leave it for a later version, since a dark GTK theme works and the light theme can be read.
    - **Waits:** T-165.
