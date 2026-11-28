# Babawus user guide

This guide holds the practical details behind the [README](README.md): how to start Babawus on each desktop, how to set up the reminder at login, how your files are kept on the stick, where to find the log, how to update Babawus, and what is left behind when it is uninstalled.

## Starting Babawus

Babawus is called "Backup" in the desktop's menus, with the green elephant as its icon. Search for "Backup", or for "Babawus". Right after installing, the desktop may need a few seconds before its search finds it. On some desktops, such as KDE Plasma on Kubuntu, Babawus only shows up in the menu once you have logged out and in again.

- **Linux Mint (Cinnamon).** Open the menu in the lower left and search for "Backup". To put it on the desktop, so that starting it is one double-click, right-click "Backup" in the menu and choose "Add to desktop". "Add to panel" puts it in the panel at the bottom instead.
- **Ubuntu and Fedora (GNOME).** GNOME shows no icons on the desktop. Press the Super key (the one with the Windows logo), or click the small rounded button in the upper left corner, and type "Backup". To keep it at hand, right-click "Backup" there and choose "Pin to Dock" on Ubuntu, or "Pin to Dash" on Fedora: it then stays in the row of programs, and one click starts it.

## Setting up the reminder at login

Babawus can remind a user at login when their last backup is too long ago, and offer to start one. For each account that wants this, log in to that account, start Babawus, turn on "Advanced" in the menu, and tick "Remind me at login", just below it. This works on every desktop. Untick it to turn the reminder off again.

At login Babawus then shows nothing, unless the last backup is more days ago than "Remind after days without a backup" in that account's backup profile on the stick (7 by default; 0 turns it off). Then it asks "Do you want to back up now?". On Fedora, GNOME opens the overview after login, the screen that shows all open windows, so the reminder first appears there: click it, or press Esc to leave the overview. See "Remind me at login when my last backup is too old" in [Requirements.md](projectmgt/Requirements.md).

On Linux Mint the reminder can also be set up in the desktop's own "Startup Applications": press "+", choose "Custom command", and enter `babawus --checklastbackup` as the command, with "Babawus reminder" as the name. Babawus's tick and "Startup Applications" see each other's entry, so either one can turn it off, and there is never more than one reminder. GNOME has no "Startup Applications", so on Ubuntu and Fedora use the tick in Babawus.

## How your files are kept on the stick

Babawus keeps the backups on the stick as ordinary files in ordinary folders, so they can always be read without Babawus too. Everything is in the folder `backup`, with a folder for each computer, and in it a folder for each account on that computer:

    backup
    ├── mint                    -- the computer "mint"
    │   ├── anna                -- Anna's files from "mint"
    │   └── ben                 -- Ben's files from "mint"
    └── laptop                  -- the computer "laptop"
        └── anna                -- Anna's files from "laptop"

Inside an account's folder, the files sit in the same folders as in the home folder: `/home/anna/Documents/letter.odt` on "mint" is kept as `backup/mint/anna/Documents/letter.odt`. The older versions of a file are in a folder next to it, as the Help explains. Babawus's own files, its setup and the log, are in the hidden folder `.babawus`.

**How Babawus knows a computer.** Babawus recognises a computer by an ID of its own, made from the computer's machine ID, which each Linux installation gets when it is installed, and not by the computer's name. The computer's folder is named after the computer's name when it is added to the stick, and if that name is taken already, Babawus adds `-2`, `-3` and so on: a second computer called "mint" gets `mint-2`. Renaming the computer later changes nothing. An account is known by its user name.

### What this makes possible

- **One stick for several computers.** Add each computer to the stick, under "Advanced" with "Change setup…" and "Add this computer and your account", on that computer. Each one backs up into a folder of its own, and none of them touches the others' files.
- **One stick for a computer that starts more than one system.** A computer that can start, say, Linux Mint or Ubuntu, is two computers to Babawus, because each installation has its own machine ID. Add each one while it is running, and both back up to the same stick, each in its own folder, even if both have the same name and user name: the second gets `mint-2`. Once Babawus runs on Windows too, a computer that also starts Windows will work the same way.
- **One stick for several people.** Each person who logs in to the computer with an account of their own can be added, for example everyone in a family that shares one computer. Each person backs up, and gets files back, as themselves: the Restore window only shows your own files.

### What to keep in mind

- **A reinstalled or new computer is a stranger at first.** Reinstalling Linux, or moving to a new computer, gives a new machine ID, so the stick shows "Not set up for you". To carry on where the old one left off, with all its backups and older versions, link it to the old computer's entry under "Advanced", with "Change setup…" and "Link to …", rather than adding it anew.
- **So is a renamed account.** If your user name changes, link your account to the backup profile of your earlier name the same way.
- **Nothing is shared between the folders.** A file kept on two computers is backed up twice, once in each computer's folder, and all computers and accounts share the stick's space. The Restore window only shows the files of the account that opens it, on that computer. To get a file from another computer, or another person, copy it from their folder on the stick with the file manager.
- **Everyone with the stick can read everything on it.** The files are not encrypted, and Babawus makes them readable by everyone, so that any computer can read the stick. On a stick shared by several people, each of them can read the others' backups. Keep the stick somewhere safe.
- **Only the home folder is backed up.** Files elsewhere, such as in `/etc` or on another disk, are not, unless they are inside your home folder.

## Finding out what went wrong

Each stick keeps a log of every backup made to it, in `.babawus/babawus_log.txt` on the stick. It is plain text, one entry per line: when each run started, for whom, on which computer and with which version of Babawus, every file it had to skip and why, and how it ended.

There are two ways to read it:

- **In Babawus.** Turn on Advanced in the menu, right-click a run in the history, and choose "Show in the log". The whole log opens in a window of its own, laid out clearly, with that run highlighted.
- **In a text editor.** Open the file with any text editor. The folder `.babawus` is hidden, so show hidden files in the file manager first.

The format is described under "Format of the log" in [Requirements.md](projectmgt/Requirements.md).

## Updating

To update Babawus, install the newer package as the [README](README.md) says. On Linux Mint, double-click the `.deb`. On Ubuntu, App Center can't update a package that is installed already: it only shows a greyed-out "Installed". Instead, in a terminal, in the folder with the package, run `sudo apt install ./babawus_<version>-1_amd64.deb`. On Fedora, run `sudo dnf install ./babawus-<version>-1.x86_64.rpm`, as when installing.

The newer version replaces the old one. The chosen language, the reminder at login and everything on the sticks stay as they were.

## Uninstalling

To uninstall Babawus, run `sudo apt remove babawus` on Linux Mint and Ubuntu, or `sudo dnf remove babawus` on Fedora. On Linux Mint, "Uninstall" in the right-click menu of "Backup" in the menu does the same. This removes the program and its "Backup" entry in the menu, and nothing else:

- **Language files added by hand stay behind,** in `/usr/share/babawus/languages/`: `sudo rm -r /usr/share/babawus`.
- **Nothing on any stick is changed.** Backups, their history and the stick's configuration stay as they are, and a later install of Babawus finds them again.
- **The settings file stays behind.** Babawus keeps the chosen language in `~/.config/babawus/settings.toml`, in the home folder of each user who chose one. Uninstalling can't reach into home folders, so delete that file by hand if it is no longer wanted: `rm -r ~/.config/babawus`.
- **So does the note of the last backup,** `~/.local/state/babawus/last_backup`, for the reminder at login: `rm -r ~/.local/state/babawus`.
- **And so does the reminder at login,** if it was turned on. Untick "Remind me at login" before uninstalling, or afterwards delete its entry: `rm ~/.config/autostart/io.github.ziggymax.Babawus-reminder.desktop`. On Linux Mint, also remove "Babawus reminder" from "Startup Applications", if it was added there.
