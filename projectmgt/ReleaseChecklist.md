# Babawus release checklist

Run this before each release, in full, on each supported distribution: Linux Mint 22.3, Ubuntu 26.04 and Fedora 45, each in its test VM (issue 174). It covers what the automated tests can't: the GUI, formatting, pulling out a stick, ejecting, and installing the package (see "Testing" in [Requirements.md](Requirements.md), and "Testing" in [TestSetup.md](../TestSetup.md)). Claude drives the window and checks the rest over SSH; you do what needs hands, and type passwords.

**To record a run**, open a GitHub issue for each distribution, titled "Release <version> checklist on <distribution>", for example "Release 1.2.0 checklist on Fedora 45", and paste everything from "Before you start" down into it. Tick each box when its step shows what it should. Note anything odd under the step, even if the step passed. A failed step gets a task in [Tasks-Open.md](Tasks-Open.md) before the release goes ahead (see issues 94–96).

Each step says what to do, then **→** what should be seen. Texts in quotes are what Babawus shows in English. Run the whole list in English, except where a step says otherwise.

## What differs per distribution

The steps below are the same on every distribution, except where they point to this table.

| | Linux Mint 22.3 | Ubuntu 26.04 | Fedora 45 |
| --- | --- | --- | --- |
| Test VM (`vms/driver.py`) | `mint-22.3` | `ubuntu-26.04` | `fedora-45` |
| Desktop, and what `echo $XDG_SESSION_TYPE` prints | Cinnamon, `x11` | GNOME, `wayland` | GNOME, `wayland` |
| Package | `babawus_<version>-1_amd64.deb` | `babawus_<version>-1_amd64.deb` | `babawus-<version>-1.x86_64.rpm` |
| Is it installed? | `dpkg -s babawus` | `dpkg -s babawus` | `rpm -q babawus` |
| **Install** it as a user would | Double-click the `.deb` in the file manager: Captain opens. Press "Install Package" | Double-click the `.deb` in the file manager: App Center opens, with "Potentially unsafe" for a package from outside it. Press "Install" | In a terminal: `sudo dnf install ./babawus-<version>-1.x86_64.rpm`. (GNOME Software can't open a local `.rpm`; see issue 174) |
| **Update** to a newer version | As Install | In a terminal: `sudo apt install ./babawus_<version>-1_amd64.deb`. (App Center only shows "Installed" for a package that is installed already; see issue 179) | As Install |
| **Find** Babawus | The menu in the lower left: search for "Backup" | Press the Super key, and type "Backup" | Press the Super key, and type "Backup" |
| Make a **second account** | "Users and Groups", as a "Standard" account | Settings → System → Users → "Unlock…" → "Add User", as a "Standard" account | As on Ubuntu |
| **Switch** to another account without logging out | The menu's "Log out" button, then "Switch User" | The system menu in the top right, the power button, then "Switch User…" | As on Ubuntu |
| Where a stick is **mounted** | `/media/$USER/<name>` | `/run/media/$USER/<name>` | `/run/media/$USER/<name>` |
| **Uninstall** | `sudo apt remove babawus` | `sudo apt remove babawus` | `sudo dnf remove babawus` |

## Before you start

- [ ] The distribution's test VM, reverted to its `clean` snapshot: `vms/driver.py revert <VM>`. Nothing from a smoke test may be left in it, such as the polkit rule that lets the tests use udisks2 without a password. → `echo $XDG_SESSION_TYPE` prints what the table says.
- [ ] Two USB sticks whose contents may be lost, called **stick A** and **stick B** below. In a test VM, they are emulated sticks, image files on the host made with `vms/driver.py stick <file> 16G` and plugged in and pulled out with `vms/driver.py plug` and `unplug`; a real stick can be passed through instead with `vms/driver.py pass` (see "Sticks" in [TestSetup.md](../TestSetup.md)). → Both show up in the file manager when plugged in.
- [ ] Two accounts on the computer: yours, with administrator rights, and a **second account** without them (see the table).
- [ ] A folder `~/BabawusTest` in your account, holding:
  - `Letters/anna.txt` and `Letters/old/draft.txt`, any small text;
  - `Letters/notes.tmp`;
  - `meeting: notes.txt`, whose name exFAT can't store;
  - `Photos/big.bin`, about 2 GB, so that a copy takes long enough to pull out a stick: `head -c 2G /dev/urandom > ~/BabawusTest/Photos/big.bin`;
  - `secret.txt` that can't be read: `chmod 000 ~/BabawusTest/secret.txt`.
- [ ] The package for this release, built by CI for main's commit, and fetched with `vms/fetch-packages.sh` on the host. → It prints the folder, `target/github-packages/<commit>/`, after "babawus_<version>-1_amd64.deb: OK" and "babawus-<version>-1.x86_64.rpm: OK". Copy the distribution's package into the VM's Downloads folder: `vms/driver.py copy <VM> <file> Downloads/`. This very file is the one released (TestSetup.md, "A release, step by step").
- [ ] Babawus is not installed yet (see the table). → Not installed. (If it is, uninstall it first.)

## Installing

- [ ] Install the package from Downloads, as the table says. → On Mint, Captain shows "Basic Backup With Usb Storage" (it capitalises every word), and on Ubuntu, App Center shows "babawus" and "Basic backup with USB storage"; below it, three short paragraphs about Babawus. (App Center also shows "License: unknown" and a blank icon, as for every package from outside it; issue 175.) It installs after asking for your password. The dependencies (`udisks2`, `exfatprogs`, `e2fsprogs`, `acl`) are already there or come with it.
- [ ] Find Babawus, as the table says. → An entry "Backup" with the Babawus elephant icon is there, straight away, without logging out first (see issue 101). Right after installing, the search may need a few seconds to find it.
- [ ] Start it from the menu. → A window titled "Backup" opens. With no stick plugged in, it says "Plug in your backup stick".
- [ ] Start it again from the menu while it is open. → No second window: the open one comes to the front.
- [ ] In a terminal, run `ls /usr/share/babawus/languages`. → `da.toml  de.toml  en.toml`.
- [ ] Fedora only: open Software, choose "Installed", then "Backup". → The page shows the elephant, "Steen Kroeyer", the README's screenshot, "Back up files to USB media" (the menu entry's comment) above the three paragraphs about Babawus, the version, "released under the MIT license", the project website and "Report an Issue" (issue 175).

## Language

- [ ] Open Menu → Language. → Dansk, Deutsch and English, in that order, with English chosen.
- [ ] Choose Menu → Language → Deutsch. → Every text changes at once, for example "Steck deinen Sicherungs-Stick ein". Nothing needs a restart.
- [ ] Close Babawus and start it again. → It starts in German.
- [ ] Look through the screens below once in German too, as far as you like, and at least the Restore window and "Choose what to back up" with a single file left out, whose German texts are new in 1.1.0. → No English left, no text cut off, and wording that reads well. Note in the issue any wording to change.
- [ ] Choose Menu → Language → Dansk. → Every text changes at once, for example "Sæt dit backup-stik i". Look through the screens below once in Danish too, as far as you like, and at least the same two as in German. → No English or German left, no text cut off, and wording that reads well. Note in the issue any wording to change.
- [ ] Choose Menu → Language → English. → Back to English, and it stays English on the next start.
- [ ] Rest the mouse on the logo in the upper right. → After a short pause, the tooltip "Babawus never forgets". In German it is "Babawus vergisst nie", in Danish "Babawus glemmer aldrig".

## Adding a language by hand

These steps follow [AddingALanguage.md](../AddingALanguage.md), with a made-up French file that has only one text translated.

- [ ] Close Babawus. Copy the English file and make it French: `cp /usr/share/babawus/languages/en.toml ~/fr.toml`, and in `~/fr.toml` change `code = "en"` to `code = "fr"`, `name = "English"` to `name = "Français"`, and `PlugIn = "Plug in your backup stick"` to `PlugIn = "Branchez votre clé de sauvegarde"`.
- [ ] Install it: `sudo cp ~/fr.toml /usr/share/babawus/languages/`. → It asks for your password, and copies it.
- [ ] Start Babawus from the terminal with `babawus`. → The terminal shows nothing about the language files. Menu → Language lists Dansk, Deutsch, English and Français.
- [ ] With no stick plugged in, choose Français. → The window says "Branchez votre clé de sauvegarde"; the menu and every other text are still English, as in the file. Choose English again, and close Babawus.
- [ ] Break the file: `sudo sed -i '/^PlugIn/d' /usr/share/babawus/languages/fr.toml`. Start `babawus` from the terminal again. → The terminal says "Babawus: /usr/share/babawus/languages/fr.toml: not offered, because these texts are missing: PlugIn", and Français is not in the Language menu. Babawus otherwise works as before.
- [ ] Leave the broken `fr.toml` where it is: "Updating" and "Uninstalling" check what happens to it.

## Formatting (Advanced)

- [ ] Plug in stick A. → It appears in the list, with a grey stick and its own round Eject button, if it isn't a Babawus medium yet. Select it. → No "Start backup" below it.
- [ ] Choose Menu → Advanced. → "Turn on the advanced functions?" asks first, with "Cancel" and "Continue". Press Esc. → Advanced stays off. Choose Menu → Advanced again and press "Continue". → The title bar shows an orange "Advanced" marker, and "Prepare…" and, for a Babawus medium, "Change setup…" appear after the everyday buttons. At the start size of 700 wide they wrap onto a second line, and each line is centred.
- [ ] With stick A selected, press "Prepare…". → "Prepare a stick" names stick A in a highlighted frame at the top, with its maker, size and what is on it; there is no list of drives to choose from. A name such as "BACKUP-A" is filled in, beside "Name for the stick:", and exFAT is chosen. The warning names the stick and what will be erased. "Erase and format" is greyed out. Nothing on the page needs scrolling.
- [ ] Type a name with a space in it. → "Use 1 to 11 letters, digits and dashes, with no spaces." shows, and the button stays greyed out. Put the suggested name back.
- [ ] Tick "I understand that everything on this stick will be erased". → "Erase and format" turns red, and can be pressed.
- [ ] Press "Erase and format". → "Formatting BACKUP-A…", with no way back and the window can't be closed. After a few seconds: "BACKUP-A is ready", with a green tick.
- [ ] Press "Back". → The start screen, with stick A selected, showing "Not set up for you" and an amber "!", and a greyed-out green "Start backup" below stick A's row, whose tooltip says why.
- [ ] Format stick B the same way, but choose Ext4. → "Ext4 asks for your password while formatting." shows below Ext4. It asks for your password, takes longer (about a minute for 16 GB), then "… is ready".
- [ ] One Babawus per computer: with Babawus open in your account, switch to the second account without logging out (see the table), and start Babawus there. → No second window, but a message with Babawus's logo: "Babawus is already open for <your user name> on this computer." and "Ask them to close it, or switch to their account. #4", with an OK button. Press OK. → It closes. In a terminal, `ls -l /dev/shm/babawus.lock`. → `-rw-rw-rw-`, owned by your account (issue 181). Switch back to your account.
- [ ] Without administrator rights: close Babawus, because only one runs per computer, log out, leaving both sticks plugged in, log in to the second account, and start Babawus. → Both sticks show "Open for another account", with an amber "!", a grey stick, and only their Eject buttons: no "Start backup" and no buttons in the details, also with Advanced on.
- [ ] Pull out both sticks and plug them back in. → Both show "Not set up for you".
- [ ] Turn Advanced on, select stick A, and press "Prepare…". → Ext4 is greyed out, with "Only an administrator can choose Ext4, …" below it, and "I understand that everything on this stick will be erased" shows without scrolling, with both sticks plugged in.
- [ ] Format stick A with exFAT. → No password is asked for, and "BACKUP-A is ready".
- [ ] Select stick B, which your account formatted with Ext4. Press "Change setup…", "Add this computer and your account", and "Choose what to back up…", tick a folder with a file in it, then "Done" and "Save", and "Start backup". → "All good". Note in the issue any password that was asked for, and whose.
- [ ] Close Babawus, and log back in to your own account. Pull out both sticks and plug them back in.
- [ ] Turn Advanced off in the menu. → It asks nothing, and the marker and the advanced buttons go away. Close and start Babawus. → Advanced is off.
- [ ] Turn Advanced on, press "Prepare…", and turn Advanced off in the menu. → The start page, Advanced off. Turn it on again, press "Change setup…", press the + of "Older versions to keep", and turn Advanced off. → "Leave without saving?" Choose "Keep editing". → The setup is still showing with the change, and Advanced is still ticked in the menu. Turn Advanced off again and choose "Leave without saving". → The start page, Advanced off, and the setup unchanged.

## Setting up

- [ ] Turn Advanced on, select stick A and press "Change setup…". → "Setup of this stick", with "This computer (…) isn't set up on this stick yet."
- [ ] Press "Add this computer and your account". → Your backup profile shows as set up, the entry is listed with "(this computer)" and "(you)", with "Older versions to keep: 10" and "Remind after days without a backup: 7", and "Save" can be pressed.
- [ ] Press "Back". → "Leave without saving?" Choose "Keep editing".
- [ ] Press "Choose what to back up…". → A tree with "Your home folder" open. Hidden folders don't show, except those holding something chosen or left out, such as `.local`, which holds `.local/share/Trash`, left out by default.
- [ ] Tick `BabawusTest` and open it. → Everything in it is ticked. `notes.tmp` is crossed out, with "Left out by the name pattern *.tmp". The totals count, then show the files and size, with the free space on the stick.
- [ ] Open `Letters` and untick `old`. → It is crossed out, "Left out", and the totals go down. `Letters` and `BabawusTest` now show partly ticked boxes.
- [ ] Untick `anna.txt` in `Letters`. → It is crossed out, "Left out", and the totals go down by one file. Tick it again. → Ticked again, and the totals go back up. Untick it once more, so that it stays left out.
- [ ] Press "Done", then "Save". → "Stick setup saved." The start screen shows stick A as "Ready", "No backup yet".
- [ ] A stick set up with Babawus 0.1.0, if you have one: plug it in. → "Needs attention", and "Change setup…" offers only "Start a new setup…" (issue 136). Its backups stay on it.
- [ ] Damage the setup by hand: open `.babawus/babawus_config` on stick A in a text editor, change `older_versions = 10` to `11` in your backup profile, and save. Replug the stick. → "Needs attention". "Change setup…" warns that the setup "was changed outside Babawus, or is damaged", and shows 11 in your backup profile. Set it back to 10, press the + of "Remind after days without a backup" once, and press "Save". → "Ready" again. The file now says `older_versions = 10` and `run_check_days = 8` in your profile, and its `[babawus]` table has `config_format = 2`, this version of Babawus and the time it was saved. Your profile has `exclude_files = ["BabawusTest/Letters/anna.txt"]`.

## Backing up

- [ ] Select stick A, press "Start backup". → The backup screen fills the window: "Finding the files to back up…", then "Backing up…" with counters and a bar, then "Finishing… Don't pull out the stick yet."
- [ ] Wait for the verdict. → "Some files were skipped", amber "!", with the skipped files grouped by reason ("Name has characters this stick can't store: 1 file #25", "Couldn't be read: 1 file #19"), "Look up its number in Help to see what to do.", what was copied, and "Last backup: today at" the time it finished.
- [ ] Open `.babawus/babawus_log.txt` on stick A in a text editor. → The run's start entry, with `account`, `version` and `hostname`; an `ERROR` entry for each skipped file, with codes such as `NAME_INVALID_CHARS` and `UNREADABLE`; and the end entry with `verdict=SKIPPED`. Paste the run's lines into the issue.
- [ ] Press "Back", then "Start backup" again. → "Some files were skipped" again, for the same two files, and it is quick, because nothing else changed.
- [ ] Press "Show history". → Both runs, newest first, each with the time, the verdict, the reason, what was copied, how many were skipped, and whose run it was.
- [ ] Right-click the older run. → Nothing happens. Turn on Advanced, and right-click it again. → A menu with "Show in the log". Choose it. → A window "Log of BACKUP-A" shows the log in a fixed-width font, one row per line, with the older run's lines highlighted and its `ERROR` lines in red. Every underscore shows, as in `NAME_INVALID_CHARS`, on shaded and highlighted lines too (issue 178).
- [ ] Right-click the newer run and choose "Show in the log". → The same window comes to the front, with the newer run highlighted instead.
- [ ] Leave the log window open, press "Back" and "Start backup". → When it has finished, the log window shows the new run's lines too, and the highlight hasn't moved.
- [ ] Close Babawus. → The log window closes with it. Advanced is off again when Babawus next starts.
- [ ] Press "Back", then "What's backed up". → `BabawusTest`, then under "Left out:" the left-out folders, `BabawusTest/Letters/anna.txt`, and the name patterns.
- [ ] Look in stick A's backup of `BabawusTest/Letters`. → No `anna.txt`, and no `notes.tmp`.
- [ ] Set up stick B as for stick A, with `BabawusTest` ticked, and back up to it once. → `meeting: notes.txt` is backed up there, because Ext4 can store the name. Only `secret.txt` is skipped.

## Restoring

- [ ] Select stick B and press "Restore files…". → A window "Restore from BACKUP-B", headed "Restore files for" you, with "Your home folder" open, `BabawusTest` in it, "Versions" on the right with "Select a file to see its versions.", and "Put them back in the folders they came from" chosen. "Restore" is greyed out, and its tooltip says to tick first.
- [ ] Open `BabawusTest` and `Photos`, and select `big.bin`. → Its versions on the right: "Newest copy, changed" with a date and "2.1 GB", and older versions, if any, each "Replaced" with a date.
- [ ] Tick `Letters`, and untick `anna.txt` in it. → `anna.txt` is crossed out, "Left out", and `Letters`, `BabawusTest` and "Your home folder" show partly ticked boxes.
- [ ] Choose "Put them in another folder:", which shows a new folder such as `Restored from BACKUP-B` with the date and time, and press "Restore". → "1 file restored." and "Open folder". The ticks are cleared, the folders stay open, and another folder is suggested.
- [ ] Press "Open folder". → The file manager shows the folder, with `BabawusTest/Letters/old/draft.txt` in it, with its old modification date. Close the file manager and delete the folder.
- [ ] Tick `meeting: notes.txt`, choose "Put them back in the folders they came from", and press "Restore". → "One of the files is already there", with "Replace them", "Keep both", "Skip them" and "Cancel". Press "Keep both". → "1 file restored." `BabawusTest` now also has `meeting: notes (` with a date and time `).txt`. Delete it.
- [ ] Empty the computer's file cache first, or the 2 GB are read from it in a few seconds, too fast to press "Cancel": `sync; echo 3 | sudo tee /proc/sys/vm/drop_caches`. Then tick `big.bin`, choose "Put them in another folder:", press "Restore", and while it shows "Restoring… 0 of 1 files" and the bar, press "Cancel". → "The restore was stopped. Nothing was restored." The folder has no `big.bin` and no other file. Delete the folder.
- [ ] Empty the file cache again, start the same restore, and close the window while it runs. → "Stop restoring?" Choose "Stop". → The window closes. Delete the folder.
- [ ] Close the Restore window, and press "Restore files…" again. → The whole window shows again, as the first time, not an empty frame (T-097).
- [ ] While the Restore window is open, eject stick B in the main window. → The Restore window closes.

## Help

- [ ] Choose Menu → Help. → A window "Help" opens, with a search field, "Search the Help", that has the focus, and sections from "What Babawus does" to "Messages". It isn't modal: the main window can still be used while it is open.
- [ ] Click the main window, and press F1. → No second Help window: the open one comes to the front.
- [ ] Type `#25` in the search field. → The entry "#25 Name has characters this stick can't store." shows, with "#25" in orange, and next to the field "1 of 1". `#2` alone doesn't find "#25".
- [ ] Clear the field, and type `stick`. → Every "stick" is yellow, ignoring upper and lower case, the first is orange, and it says "1 of" the number found.
- [ ] Press Enter, then Shift+Enter. → "2 of …", then "1 of …" again, the orange match moving with it.
- [ ] Hover over the ↓ button next to the counter. → "Next match (Enter)". Over ↑: "Previous match (Shift+Enter)". Press ↑. → It goes round to the last match. Press ↓. → Back to "1 of …".
- [ ] Type `zzqx`. → "Not found", and ↑ and ↓ are greyed out.
- [ ] Type `#14`, and with the Help open, choose Menu → Language → Dansk in the main window. → The Help window's title becomes "Hjælp", its text is Danish, and the search is kept: "#14 USB-stikket er fuldt." shows, with "2 af 2", as the entry is the second match. Choose English again. → The Help is English again, on "#14 The stick is full.".
- [ ] Close Babawus. → The Help window closes with it.

## The reminder at login

- [ ] In a terminal, run `cat ~/.local/state/babawus/last_backup`. → The time of the last backup above, to stick B, and `run_check_days = 7`, from stick B's setup: the note follows the stick backed up last.
- [ ] Run `babawus --checklastbackup`. → Nothing shows, and the command ends at once.
- [ ] Make the note 9 days old: `sed -i "s/^last_backup = .*/last_backup = \"$(date -d '9 days ago' '+%Y-%m-%d %H:%M:%S %z')\"/" ~/.local/state/babawus/last_backup`. Run `babawus --checklastbackup`. → A small window with the elephant: "Your last backup was 9 days ago.", "Do you want to back up now?", "Not now" and "Back up now". Press "Not now". → Babawus closes.
- [ ] Run it again, and press "Back up now". → The normal window opens. Back up to stick A. → The note has today's time again, and `run_check_days = 8`, from stick A's setup.
- [ ] Start Babawus, open the menu, and look for "Remind me at login". → It isn't there. Turn on Advanced. → "Remind me at login" is in the menu, between Advanced and Help, not ticked.
- [ ] Tick it, and close Babawus. → `ls ~/.config/autostart` shows `io.github.ziggymax.Babawus-reminder.desktop`.
- [ ] Make the note 9 days old again, and log out and in (in a test VM: `sudo systemctl reboot`, as it logs in by itself). → The reminder shows after login. On Fedora, GNOME opens the overview first, with the reminder as a window in it: press Esc, and it is on the desktop. Press "Not now".
- [ ] Start Babawus, turn on Advanced, and untick "Remind me at login". → The entry is gone from `~/.config/autostart`.
- [ ] **Linux Mint only:** add "Babawus reminder" in "Startup Applications" as [UserGuide.md](../UserGuide.md) says. Start Babawus and turn on Advanced. → "Remind me at login" is ticked. Untick it. → In "Startup Applications", "Babawus reminder" is still listed, switched off. Remove it there.

## Cancelling and closing

- [ ] Start a backup to stick A after changing `big.bin` (`touch ~/BabawusTest/Photos/big.bin`), and press "Cancel" while it says "Backing up…". → "Stop the backup?" Choose "Keep going". → It carries on.
- [ ] Press "Cancel" again, and choose "Stop backup". → It stops soon: "Backup not finished", "You stopped the backup."
- [ ] Start again, and close the window while it copies. → The same question. Choose "Stop backup". → The window closes once the run has ended. Start Babawus: "Show history" shows the stopped run as "Backup not finished".

## Pulling out a stick

- [ ] Change `big.bin` again, start a backup to stick A, and pull the stick out while it says "Backing up…". → "Backup not finished", "The stick was removed. #10", and "Run the backup again. If it happens again, look up its number in Help." Files skipped before the pull may be listed, such as `meeting: notes.txt`, but nothing that the pull caused: no "Couldn't be written". Note in the issue what was listed.
- [ ] Plug stick A back in, and start a backup. → It finishes, catching up on `big.bin`. The history shows the interrupted run as "Backup not finished", "The backup was interrupted. #11", and this one after it.

There is no step for pulling out a stick during "Finishing…": it lasted less than a third of a second, too short to hit by hand. The automated tests cover it (see issue 103).

## Ejecting

- [ ] After a backup that counts ("All good" or "Some files were skipped"), press "Eject BACKUP-A". → "Ejecting BACKUP-A…", then a pop-up: "You can pull out BACKUP-A now.", with OK. The stick's light goes out, and it leaves the file manager. Press OK. → The start screen, without the stick.
- [ ] Plug it in, open a terminal and `cd` into the stick (for example `cd /media/$USER/BACKUP-A`, or where the table says). Then press stick A's Eject button in the list. → A pop-up: "BACKUP-A is still in use by a program, for example a terminal or a file being copied. Close it, or wait until it has finished, and try again. #33" Press OK. → It stays in the list. (A file manager window showing the stick doesn't keep it in use: it just closes when the stick is ejected. See issue 129.)
- [ ] In the terminal, leave the stick (`cd ~`), and press "Eject" again. → A pop-up: "You can pull out BACKUP-A now." Press OK. → The start screen, without stick A.
- [ ] With both sticks plugged in and stick A selected, press stick B's Eject button. → A pop-up: "You can pull out BACKUP-B now." Press OK. → Stick B has left the list, and stick A is still selected, with "Start backup" below it.

## Updating

- [ ] Build the packages again on the host with the version raised (for example from `1.2.0` to `1.2.1` in `Cargo.toml`, and for `babawus` and `babawus-core` in `Cargo.lock`, which the build won't change by itself; not committed), with `containers/package.sh`, copy the distribution's package into the VM, and update to it as the table says. → It installs over the old one, as an update.
- [ ] Start Babawus. → It runs, keeps the chosen language, and stick A and its history are as before.
- [ ] Run `ls /usr/share/babawus/languages`. → `da.toml  de.toml  en.toml  fr.toml`: the update kept the file added by hand.

## Uninstalling

- [ ] Uninstall, as the table says. → The "Backup" entry leaves the menu.
- [ ] Run `ls /usr/share/babawus/languages`. → Only `fr.toml`, the file added by hand, is left. Remove it and its folder with `sudo rm -r /usr/share/babawus`, as [UserGuide.md](../UserGuide.md) says.
- [ ] Look at stick A. → Nothing changed on it: `.babawus` and `backup` are as before.
- [ ] Look for the settings file. → `~/.config/babawus/settings.toml` is still there, if a language was ever chosen, as the uninstall notes in [UserGuide.md](../UserGuide.md) say, and so is `~/.local/state/babawus/last_backup`.

## Afterwards

- [ ] Every box is ticked, or has a task in Tasks-Open.md for what it found. → Close the issue.
- [ ] Once the checklist has run on every distribution: the README's "Supported platforms" on the tested commit names exactly the distributions it ran on, and the release message has "Supported platforms", "How these files were tested", with a link to each distribution's checklist issue, and "Checking the download", as "A release, step by step" in [TestSetup.md](../TestSetup.md) says (issue 174). → Release.
