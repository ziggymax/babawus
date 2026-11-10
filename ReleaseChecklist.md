# Babawus release checklist

Run this on real USB sticks before each release. It covers what the automated tests can't: the GUI, formatting, pulling out a stick, ejecting, and installing the package (see "Testing" in [Requirements.md](Requirements.md)).

**To record a run**, open a GitHub issue titled "Release 0.1.0 checklist" (with the version being released), and paste everything from "Before you start" down into it. Tick each box when its step shows what it should. Note anything odd under the step, even if the step passed. A failed step gets a task in [Tasks.md](Tasks.md) before the release goes ahead (see Issues.md items 94–96).

Each step says what to do, then **→** what should be seen. Texts in quotes are what Babawus shows in English. Run the whole list in English, except where a step says otherwise.

## Before you start

- [ ] A computer with the supported platform: Linux Mint 22.x, Cinnamon, X11. → `echo $XDG_SESSION_TYPE` prints `x11`.
- [ ] Two USB sticks whose contents may be lost, called **stick A** and **stick B** below. → Both show up in the file manager when plugged in.
- [ ] Two accounts on the computer: yours, with administrator rights, and a **second account** without them (created in Users and Groups as a "Standard" account).
- [ ] A folder `~/BabawusTest` in your account, holding:
  - `Letters/anna.txt` and `Letters/old/draft.txt`, any small text;
  - `Letters/notes.tmp`;
  - `meeting: notes.txt`, whose name exFAT can't store;
  - `Photos/big.bin`, about 2 GB, so that a copy takes long enough to pull out a stick: `head -c 2G /dev/urandom > ~/BabawusTest/Photos/big.bin`;
  - `secret.txt` that can't be read: `chmod 000 ~/BabawusTest/secret.txt`.
- [ ] The package for this release, built with `cargo deb -p babawus`. → `target/debian/babawus_<version>-1_amd64.deb` exists.
- [ ] Babawus is not installed yet: `dpkg -s babawus` → "package 'babawus' is not installed". (If it is, uninstall it first with `sudo apt remove babawus`.)

## Installing (T-025)

- [ ] Double-click the `.deb` in the file manager. → The installer shows "Basic Backup With Usb Storage" (it capitalises every word), and below it three short paragraphs about Babawus. Install it. → It installs after asking for your password. The dependencies (`udisks2`, `exfatprogs`, `e2fsprogs`, `acl`) are already there or come with it.
- [ ] Open the application menu and search for "Backup". → An entry "Backup" with the Babawus elephant icon is there, straight away, without logging out first (see Issues.md item 101).
- [ ] Start it from the menu. → A window titled "Backup" opens. With no stick plugged in, it says "Plug in your backup stick".
- [ ] Start it again from the menu while it is open. → No second window: the open one comes to the front.

## Language

- [ ] Choose Menu → Language → Deutsch. → Every text changes at once, for example "Steck deinen Sicherungs-Stick ein". Nothing needs a restart.
- [ ] Close Babawus and start it again. → It starts in German.
- [ ] Look through the screens below once in German too, as far as you like. → No English left, and no text cut off.
- [ ] Choose Menu → Language → English. → Back to English, and it stays English on the next start.

## Formatting (Advanced)

- [ ] Plug in stick A. → It appears in the list, with a grey stick, if it isn't a Babawus medium yet.
- [ ] Choose Menu → Advanced. → The title bar shows an orange "Advanced" marker, and "Prepare…" and, for a Babawus medium, "Change setup…" appear.
- [ ] Press "Prepare…". → "Prepare a stick" lists stick A with its maker, size and what is on it, and no internal disks. Stick A is chosen, a name such as "BACKUP-A" is filled in, and exFAT is chosen. The warning names the stick and what will be erased. "Erase and format" is greyed out.
- [ ] Type a name with a space in it. → "Use 1 to 11 letters, digits and dashes, with no spaces." shows, and the button stays greyed out. Put the suggested name back.
- [ ] Tick "I understand that everything on this stick will be erased". → "Erase and format" turns red, and can be pressed.
- [ ] Press "Erase and format". → "Formatting BACKUP-A…", with no way back and the window can't be closed. After a few seconds: "BACKUP-A is ready", with a green tick.
- [ ] Press "Back". → The start screen, with stick A selected, showing "Not set up for you" and an amber "!".
- [ ] Format stick B the same way, but choose Ext4. → "Ext4 asks for your password while formatting." shows below Ext4. It asks for your password, takes longer (about a minute for 16 GB), then "… is ready". (On the development VM it doesn't ask: see "Polkit" in DevEnvSetup.md.)
- [ ] Without administrator rights (T-024): close Babawus, because only one runs per computer, log out, leaving both sticks plugged in, log in to the second account, and start Babawus. → Both sticks show "Open for another account", with an amber "!", a grey stick, and only "Start backup" (greyed out) and "Eject", also with Advanced on.
- [ ] Pull out both sticks and plug them back in. → Both show "Not set up for you".
- [ ] Turn Advanced on, and press "Prepare…". → Ext4 is greyed out, with "Only an administrator can choose Ext4, …" below it.
- [ ] Format stick A with exFAT. → No password is asked for, and "BACKUP-A is ready".
- [ ] Select stick B, which your account formatted with Ext4. Press "Change setup…", "Add this computer as …", and "Choose what to back up…", tick a folder with a file in it, then "Done" and "Save", and "Start backup". → "All good". Note in the issue any password that was asked for, and whose.
- [ ] Close Babawus, and log back in to your own account. Pull out both sticks and plug them back in.
- [ ] Turn Advanced off in the menu. → The marker and the poweruser's buttons go away. Close and start Babawus. → Advanced is off.

## Setting up

- [ ] Turn Advanced on, select stick A and press "Change setup…". → "Setup of this stick", with "Older versions to keep of each file: 10", and "This computer (…) isn't set up on this stick yet."
- [ ] Press "Add this computer as …". → Your account shows as set up, the entry is listed with "(this computer)" and "(you)", and "Save" can be pressed.
- [ ] Press "Back". → "Leave without saving?" Choose "Keep editing".
- [ ] Press "Choose what to back up…". → A tree with "Your home folder" open. Hidden folders don't show, except those holding something chosen.
- [ ] Tick `BabawusTest` and open it. → Everything in it is ticked. `notes.tmp` is crossed out, with "Left out by the name pattern *.tmp". The totals count, then show the files and size, with the free space on the stick.
- [ ] Open `Letters` and untick `old`. → It is crossed out, "Left out", and the totals go down.
- [ ] Press "Done", then "Save". → "Saved." The start screen shows stick A as "Ready", "No backup yet".
- [ ] Damage the setup by hand: open `.babawus/babawus_config` on stick A in a text editor, change `older_versions = 10` to `11`, and save. Replug the stick. → "Needs attention". "Change setup…" warns that the setup "was changed outside Babawus, or is damaged", and shows 11. Set it back to 10 and press "Save". → "Ready" again.

## Backing up

- [ ] Select stick A, press "Start backup". → The backup screen fills the window: "Finding the files to back up…", then "Backing up…" with counters and a bar, then "Finishing… Don't pull out the stick yet."
- [ ] Wait for the verdict. → "Some files were skipped", amber "!", with the skipped files grouped by reason ("Name has characters this stick can't store: 1 file", "Couldn't be read: 1 file"), "Contact your poweruser.", what was copied, and "Last backup: today at" the time it finished.
- [ ] Open "Details for your poweruser", and press "Copy". → The details, with codes such as `SKIPPED` and `NAME_INVALID_CHARS`. Paste them into the issue: they arrive whole.
- [ ] Press "Back", then "Start backup" again. → "Some files were skipped" again, for the same two files, and it is quick, because nothing else changed.
- [ ] Press "Show history". → Both runs, newest first, each with the time, the verdict, the reason, what was copied, how many were skipped, and whose run it was.
- [ ] Press "Back", then "What's backed up". → `BabawusTest`, then under "Left out:" the left-out folders and the name patterns.
- [ ] Set up stick B as for stick A, with `BabawusTest` ticked, and back up to it once. → `meeting: notes.txt` is backed up there, because Ext4 can store the name. Only `secret.txt` is skipped.

## Cancelling and closing

- [ ] Start a backup to stick A after changing `big.bin` (`touch ~/BabawusTest/Photos/big.bin`), and press "Cancel" while it says "Backing up…". → "Stop the backup?" Choose "Keep going". → It carries on.
- [ ] Press "Cancel" again, and choose "Stop backup". → It stops soon: "Backup not finished", "You stopped the backup."
- [ ] Start again, and close the window while it copies. → The same question. Choose "Stop backup". → The window closes once the run has ended. Start Babawus: "Show history" shows the stopped run as "Backup not finished".

## Pulling out a stick (T-022)

- [ ] Change `big.bin` again, start a backup to stick A, and pull the stick out while it says "Backing up…". → "Backup not finished", "The stick was removed.", and "Run the backup again. If it happens again, contact your poweruser." Files skipped before the pull may be listed, such as `meeting: notes.txt`, but nothing that the pull caused: no "Couldn't be written". Note in the issue what was listed.
- [ ] Plug stick A back in, and start a backup. → It finishes, catching up on `big.bin`. The history shows the interrupted run as "Backup not finished", "The backup was interrupted.", and this one after it.

There is no step for pulling out a stick during "Finishing…": it lasted less than a third of a second, too short to hit by hand. The automated tests cover it (see Issues.md item 103).

## Ejecting

- [ ] After a backup that counts ("All good" or "Some files were skipped"), press "Eject BACKUP-A". → "Ejecting BACKUP-A…", then "You can pull out BACKUP-A now." The stick's light goes out, and it leaves the list and the file manager.
- [ ] Plug it in, and open it in the file manager. Then press "Eject" on the start screen. → "BACKUP-A is still in use by a program, for example an open file or a file manager window. Close it and try again." It stays in the list.
- [ ] Close the file manager window and press "Eject" again. → It ejects.

## Updating (T-025)

- [ ] Build the package again with the version raised (for example `0.1.1` in `Cargo.toml`, not committed), and install it by double-clicking it. → It installs over the old one, as an update.
- [ ] Start Babawus. → It runs, keeps the chosen language, and stick A and its history are as before.

## Uninstalling (T-025)

- [ ] Uninstall with `sudo apt remove babawus`. → The "Backup" entry leaves the menu.
- [ ] Look at stick A. → Nothing changed on it: `.babawus` and `backup` are as before.
- [ ] Look for the settings file. → `~/.config/babawus/settings.toml` is still there, if a language was ever chosen, as the uninstall notes in [README.md](README.md) say.

## Afterwards

- [ ] Every box is ticked, or has a task in Tasks.md for what it found. → Close the issue, and release.
