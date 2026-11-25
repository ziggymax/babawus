# Babawus user guide

This guide holds the practical details behind the [README](README.md): how to set up the reminder at login, where to find the log, and what is left behind when Babawus is uninstalled.

## Setting up the reminder at login

Babawus can remind a user at login when their last backup is too long ago, and offer to start one. For each account that wants this, open "Startup Applications" in that account, press "+", choose "Custom command", and enter `babawus --checklastbackup` as the command, with "Babawus reminder" as the name.

At login Babawus then shows nothing, unless the last backup is more days ago than "Remind after days without a backup" in that account's backup profile on the stick (7 by default; 0 turns it off). See "Remind me at login when my last backup is too old" in [Requirements.md](projectmgt/Requirements.md).

## Finding out what went wrong

Each stick keeps a log of every backup made to it, in `.babawus/babawus_log.txt` on the stick. It is plain text, one entry per line: when each run started, for whom, on which computer and with which version of Babawus, every file it had to skip and why, and how it ended.

There are two ways to read it:

- **In Babawus.** Turn on Advanced in the menu, right-click a run in the history, and choose "Show in the log". The whole log opens in a window of its own, laid out clearly, with that run highlighted.
- **In a text editor.** Open the file with any text editor. The folder `.babawus` is hidden, so show hidden files in the file manager first.

The format is described under "Format of the log" in [Requirements.md](projectmgt/Requirements.md).

## Uninstalling

To uninstall Babawus, run `sudo apt remove babawus`, or remove it in the Software Manager. This removes the program and its "Backup" entry in the menu, and nothing else:

- **Language files added by hand stay behind,** in `/usr/share/babawus/languages/`: `sudo rm -r /usr/share/babawus`.
- **Nothing on any stick is changed.** Backups, their history and the stick's configuration stay as they are, and a later install of Babawus finds them again.
- **The settings file stays behind.** Babawus keeps the chosen language in `~/.config/babawus/settings.toml`, in the home folder of each user who chose one. Uninstalling can't reach into home folders, so delete that file by hand if it is no longer wanted: `rm -r ~/.config/babawus`.
- **So does the note of the last backup,** `~/.local/state/babawus/last_backup`, for the reminder at login: `rm -r ~/.local/state/babawus`. Remove "Babawus reminder" from "Startup Applications" too, if it was added.
