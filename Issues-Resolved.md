# Babawus – Resolved issues

The questions about Babawus that have been decided, with why. A question starts in [Issues-Unsettled.md](Issues-Unsettled.md), where it gets its number, and moves here word for word once it is decided, keeping that number. Elsewhere it is referred to as just "issue 132", so references don't go stale when it moves. GitHub issues and pull requests are "#63".

The first issues came from the survey below; later ones were added as they came up, under headings saying where.

*Survey of README.md, Concept.md (revision 5) and Requirements.md, 2026-11-04. Items are ordered by priority within each section. Issues 1–6 were settled first, because each one changes what gets built, followed by issues 7–30. Their decisions are recorded below and in Concept.md (revision 8) and Requirements.md.*

## A. Contradictions and risks to the design

1. **Two computers can end up sharing one folder.** Many Mint installs keep a default hostname such as `mint`, and many accounts have the same username. Two such computers would write into the same `backup/mint/user` folder and mix their files without anyone noticing. That breaks the promise that backups are "always kept apart". One fix is to also store a machine ID in the config. ([Requirements.md](Requirements.md), "Source to target path conversion")
    - **Decided (2026-11-04):** Store a computer ID per computer (HMAC-SHA256 of a fixed Babawus app ID, keyed with `/etc/machine-id`, 128 bits), mapped in the config to a unique, lower-case folder name (`mint`, `mint-2`, …). The hostname is for display only. A reinstalled or replacement computer is linked to its existing entry. See Concept.md rev. 6 and Requirements.md.
2. **The config can't choose where backups go, but the concept says it does.** The concept says the config sets "where to put the backups on the medium" ([Concept.md](Concept.md), "Finalizing a USB medium"). The requirements fix the location to `backup/<host>/<user>/<path from home>`.
    - **Decided (2026-11-04):** Keep the fixed layout `backup/<computer folder>/<user>/<path relative to home>`; the concept is corrected. See Concept.md rev. 6 and Requirements.md.
3. **"Ext4 guarantees no problem with names or path lengths" isn't true.** Ext4 limits each name to 255 bytes, and a version name adds about 20 characters (`[YYYY-MM-DD_HHMMSS-n]`). So a long name can fail on Ext4 too. ([Concept.md](Concept.md), "Initial preparation")
    - **Decided (2026-11-04):** If a version name (or version folder name) would be too long, the file is not updated and is listed as skipped. Users are educated to keep names shorter. The Ext4 wording in the concept is corrected. See Concept.md rev. 6 and Requirements.md.
4. **A full medium or an interruption can leave a broken newest copy.** Step 2 of the versioning moves the current copy away before step 3 writes the new one. If the medium fills up or is pulled out in between, the newest copy is half-written or missing. Writes need to go to a temporary file that is then renamed, and the requirements should say how that temporary file is named. ([Requirements.md](Requirements.md), "Handling of file versioning")
    - **Decided (2026-11-04):** Write to a temporary file in `.babawus/tmp/`, `fsync`, move the old copy into the version folder, then rename. `.babawus/tmp/` is emptied at the start of each run. Free space is checked before each file. See Concept.md rev. 6 and Requirements.md.
5. **"Oldest version" isn't defined, so pruning can delete the wrong file.** A version's timestamp is the file's modification time, not when it was backed up. If a user restores an old file, the next backup can turn that copy into the "oldest" version and delete it first. Sorting is also unclear: the `-2` suffix sorts before the version without a suffix.
    - **Decided (2026-11-04):** Name versions by backup time. Each copy keeps its modification time. Prune by parsed timestamp and suffix, not by raw name order. See Concept.md rev. 6 and Requirements.md.
6. **Only the right user on the right computer can edit their entry.** Folders for an account can only be changed while running on that computer, as that user ([Requirements.md](Requirements.md), "Configure USB media"). So the poweruser has to sit at each computer, logged in as each user. That should be stated openly. It's also unclear whether "Add a computer or account" has the same limit.
    - **Decided (2026-11-04):** Adding, linking, renaming and changing an entry is done only on that computer, as that user. Other entries are read-only. See Concept.md rev. 6 and Requirements.md.

## B. Specification gaps

7. **Config file format.** The syntax, encoding, line endings, header text and a format version field are not defined. Without a version field, an older Babawus can't safely read a newer file. It's also unclear whether the "validation marker" and the checksum are the same thing.
    - **Decided (2026-11-04):** TOML, UTF-8, LF line endings, a marker line and header comments, `format = 1` (a newer format makes the medium "Needs attention" and it is never written), the checksum as the last comment line, which is the validation marker.
8. **Exclusion wildcards.** The syntax isn't defined: is `**` supported, and does a pattern match the name or the path relative to home? It's also unclear whether exclusions apply per account or to the whole medium, and whether some are built in, such as `.cache`, Trash, or cloud folders mounted inside home.
    - **Decided (2026-11-04):** Name patterns (`*`, `?`, `[…]`, matched against names at any depth) plus specific folders picked in the file explorer, per account. A visible, editable pre-filled list; no hidden built-in exclusions. No crossing into other file systems below a selected folder. Case-sensitive.
9. **The version limit N.** It's unclear whether N is set per medium or per account, what the default and allowed range are, and whether N = 0 is allowed. If N is lowered, it's also unclear when the extra versions are removed.
    - **Decided (2026-11-04):** One N per medium, range 1–99, default 10. Every run prunes each visited version folder down to N.
10. **Verdict rules.** Each of the four verdicts needs exact conditions. Unclear cases include a full medium, an account that isn't added, an invalid config and a medium removed during the scan. The "short text to read out or forward" has no content yet, and "forward" has no mechanism (clipboard?).
    - **Decided (2026-11-04):** Verdicts are defined by the next step (a full medium or an account that isn't added = Backup failed; interrupted or cancelled = Backup not finished). The worst verdict wins. Every bad verdict has a reason line, and a "Details for your poweruser" box with a Copy button (clipboard). A Cancel button with confirmation ends the run after the current file.
11. **"Last backup N days ago."** Unclear whether this counts per medium or per user and computer, given that a medium can be shared. It's also unclear whether "Some files were skipped" counts as successful, and what is shown before the first backup.
    - **Decided (2026-11-04):** Per account on this computer. "All good" and "Skipped" both count (with a warning mark for skipped files). "No backup yet" before the first backup. Log trimming keeps each account's latest counting session.
12. **Interrupted runs.** If the computer shuts down mid-run, the "Backup not finished" verdict can only be worked out on the next start, from a start entry in the log with no end entry. That rule should be written down.
    - **Decided (2026-11-04):** A session without an end entry is read as "Not finished", and closed with an end entry at the start of the next run (unless its lock is live). The start entry is flushed; damaged lines are ignored.
13. **Log content.** The concept lists files scanned and data used ([Concept.md](Concept.md), "Includes" 2), but the requirements leave them out. The history view needs a format it can parse. The log should store language-neutral codes so the history can be shown in either language. It also needs a safe way to trim it below 1 MB.
    - **Decided (2026-11-04):** Tab-separated fields (timestamp, session ID, type, `key=value` pairs, optional English text), language-neutral codes, an extended end entry, trimming via `.babawus/tmp/` and rename.
14. **Names that can't be stored.** The concept leaves these "to the design": characters exFAT forbids, names that aren't valid UTF-8, and length limits. The requirements should still fix the behaviour, which is to skip and report.
    - **Decided (2026-11-04):** Checked before copying, with precise reason codes (`NAME_INVALID_CHARS`, `NAME_TRAILING_DOT`, `NAME_NOT_UTF8`, `PATH_TOO_LONG`) and a catch-all for other write errors. A skipped folder counts as one entry; reasons are grouped in the verdict.
15. **Type and case changes.** Undefined cases:
    - A file becomes a folder with the same name, or the other way round.
    - On exFAT, a file renamed only in case (`report.pdf` → `Report.pdf`).
    - Files that change during copying, or can't be read.
    - Whether copies are read back to check them.
    - **Decided (2026-11-04):** File → folder versions the old file; folder → file is skipped (`TYPE_CONFLICT`). Case-only renames on exFAT count as the same file. Files changing during copy are retried once, then skipped. Unreadable files are skipped. No read-back verification in the first version.
16. **Mixed Ext4 permissions.** The root folder of a newly formatted Ext4 medium and `lost+found` belong to root. An existing Ext4 medium may not be writable by the user at all.
    - **Decided (2026-11-04):** The "use as is" option is removed for both file systems: Babawus always formats. Ext4 permissions are handled while formatting (`tune2fs -o acl`, root folder owned by the formatting user with mode `0777`, default ACL). `lost+found` is left alone; unwritable folders give `NOT_WRITABLE` skips.
17. **Formatting details.** Unclear whether Babawus writes a new partition table or only formats the partition, and what volume label it sets. Reserved blocks on Ext4 (5% by default) should be set to 0. Ejecting should be defined as unmount only, or unmount plus power-off.
    - **Decided (2026-11-04):** Whole device, MBR (GPT above 2 TiB), one partition. Ext4 with `-m 0` and full initialisation; exFAT with defaults. A label of up to 11 characters. Eject = unmount + power-off, falling back to unmount only.
18. **The media list.** How is a medium identified: label, vendor, size, or a name the poweruser gives it, which would help when rotating media? Unclear cases:
    - Sticks with several partitions.
    - Read-only media.
    - Media that aren't mounted.
    - A valid medium where this account hasn't been added.
    - **Decided (2026-11-04):** One row per mounted file system on a removable device. Label in large type, maker/model/size below it. A status per row, and only "Ready" can be clicked; "Not set up for you" rows can't be clicked.
19. **Several Babawus instances.** Two users on one computer, or two Babawus windows, could back up to the same medium at once. That probably needs a single running instance and a lock file on the medium.
    - **Decided (2026-11-04):** One Babawus window per computer (lock in `/run/lock/`). One lock per medium (`.babawus/lock`) covering every action that writes to the medium; locks that are out of date are removed automatically.
20. **Language setting.** Unclear where the choice is stored and what the default is (the system language?).
    - **Decided (2026-11-04):** Stored per user in `~/.config/babawus/settings.toml`. Defaults to the system language, otherwise English. Files on the medium stay in English. Switching takes effect at once.
21. **Gaps in scope:**
    - Renaming a computer or account appears in the requirements but not in the concept's includes or first-version criteria.
    - There's no story for removing a computer or account from a medium.
    - The concept says folders are selected; the requirements say "files and folders" ([Requirements.md](Requirements.md), "Configure USB media").
    - **Decided (2026-11-04):** "Link" is in the first version. Removing an entry keeps its backups and works from any computer. Both files and folders can be selected.

## C. Technical decisions to make at the start of design

22. **Language and UI toolkit.** For example GTK4 with Python, or Qt. This affects dependencies, tests and packaging.
    - **Decided (2026-11-05):** Rust with plain GTK 4 through gtk4-rs, without libadwaita, for the first version; Python with GTK 3 is the fallback. All logic lives in a library with no UI code, with a thin UI on top.
23. **Packaging.** A `.deb` package or something else. Flatpak's sandbox clashes with formatting, udisks and full access to home.
    - **Decided (2026-11-05):** A `.deb` package built with `cargo-deb`. Our own APT repository is a possible later addition.
24. **Formatting and eject.** Use udisks2/polkit or pkexec. Decide whether xxHash64 comes from a library or is written natively.
    - **Decided (2026-11-05):** udisks2 over D-Bus for everything, with no root code in Babawus; a single `pkexec` call is the fallback for anything udisks2 can't do (removed on 2026-11-07, because nothing needed it: see issue 39). The config checksum is SHA-256, first 16 hex digits, replacing xxHash64. Rust crates count under doctrine 3.
25. **Platforms.** Which Mint version and desktop to target, and whether to support Wayland, X11 or both.
    - **Decided (2026-11-05):** Supported: Mint 22.x Cinnamon on X11. Everything else is best effort. The target is reviewed when a new major Mint version comes out.
26. **Testing exFAT and Ext4 behaviour.** Loop devices need root; fake file systems may not behave like the real ones. Pick an approach.
    - **Decided (2026-11-05):** Three layers: unit tests with an in-memory fake and fault injection, integration tests on real file systems in loop-mounted image files, and a manual checklist on real sticks before each release. Automated tests can only ever format a loop device they created themselves.
27. **Speed.** Scanning, the live file and byte totals in the config editor, and comparing files must not freeze the UI on a large home folder.
    - **Decided (2026-11-05):** Plain worker threads with a message channel, one scan per backup, background live totals with "Counting…", screen updates limited to about 10 per second, and the byte counter moves during large files. The per-file `fsync` stays (see "Follow-ups").

## D. Document housekeeping

28. **Pick one source of truth.** Concept and Requirements repeat each other, which has already caused small differences (issues 2, 13 and 21).
    - **Decided (2026-11-05):** Concept.md explains what and why; Requirements.md is authoritative and wins any disagreement. Details are trimmed from Concept.md.
29. **Add a glossary.** For example "run" vs "session", "Babawus-ready", "newest copy" and "older version".
    - **Decided (2026-11-05):** A glossary at the start of Requirements.md with all relevant terms. "Run" is the single term, replacing "session"; "Ready" replaces "Babawus-ready".
30. **Small wording fix.** The concept's Includes say "allow the *user* to" format and set up media ([Concept.md](Concept.md), "Includes"); those are poweruser functions.
    - **Decided (2026-11-05):** The concept's Includes are split into one list for the user and one for the poweruser.

## Follow-ups

These decisions left three things to check on real hardware. The measurements are in [DevEnvSetup.md](DevEnvSetup.md).

- **udisks2 and `mkfs` options:** verify that Mint 22's udisks2 passes on `-m 0` and the initialisation options to `mkfs.ext4`.
    - **Checked (2026-11-05):** It does, through the `mkfs-args` option of `Format`.
- **ACL default mount option:** verify whether `mkfs.ext4` already sets `acl` as a default mount option. If it doesn't, `tune2fs -o acl` needs the `pkexec` fallback.
    - **Checked (2026-11-05):** It does, so neither `tune2fs` nor `pkexec` is needed on Mint 22.
- **`fsync` per file:** measure backups of many small files on a real USB stick. Only if that's too slow, look at syncing in batches.
    - **Measured (2026-11-05):** About 60 small files per second on both file systems. One sync at the end is 1.3 times faster on Ext4 and 3.4 times faster on exFAT.
    - **Decided (2026-11-05):** Keep `fsync` per file. Safety matters more than speed: batching would risk losing more recent files if a stick is pulled, and the first backup is the only one that copies everything.

## Findings from testing on real hardware

Testing on the development VM and a real USB stick turned up these issues. All are now covered in Requirements.md, and Concept.md needed no changes. The measurements are in [DevEnvSetup.md](DevEnvSetup.md).

31. **The verdict could appear before the data was on the medium.** After every file had its own `fsync`, Ext4 still needed 8.6 seconds to finish flushing.
    - **Decided (2026-11-05):** A run ends by flushing the whole medium, then writing and flushing the log's end entry, and only then showing the verdict.
32. **The byte counter would not show real progress.** Writes to a 512 MiB file finished in 0.6 seconds, and `fsync` then took 20.1 seconds.
    - **Decided (2026-11-05):** Files are copied in flushed chunks, and only flushed bytes count. Cancel is also checked between chunks, and a file still being copied to the temporary folder is simply discarded.
33. **Device names are not stable.** After a reboot, the stick and the system disk swapped `/dev/sda` and `/dev/sdb`.
    - **Decided (2026-11-05):** Drives are identified by their udisks2 drive ID, and checked again right before formatting.
34. **The format steps were in the wrong order.** Setting permissions needs the medium mounted, and nothing said to unmount it before formatting.
    - **Decided (2026-11-05):** Unmount, partition, format, mount, set permissions, then create the folders and configuration.
35. **"Removable" was not precise.** A USB stick has `MediaRemovable: false`, and a CD drive also counts as removable to the kernel.
    - **Decided (2026-11-05):** A USB medium is a drive with `Removable` set and `ConnectionBus` set to `usb`, in both the media list and the format view.
36. **Formatting can outlast the D-Bus timeout.** Ext4 took up to a minute on 16 GB, and the default timeout is 25 seconds.
    - **Decided (2026-11-05):** Formatting runs in the background with no short timeout, shows "Formatting…", and can't be cancelled once started.
37. **Partition types were not specified.**
    - **Decided (2026-11-05):** `0x07` for exFAT and `0x83` for Ext4 on MBR, with the matching GPT types.
    - **Checked (2026-11-07):** The integration tests write GPT on loop devices, and udisks2 sets both GPT types. That a device over 2 TiB gets GPT is a unit test.
38. **Formatting may need no password.** It ran without one from an SSH session, using an administrator's account.
    - **Decided (2026-11-05):** Accepted. udisks2 may or may not ask, and the explicit confirmation is the safeguard. The manual checklist covers formatting from an account without administrator rights.
39. **Powering off has not been tested.** The stick reports `CanPowerOff: true`, but that is all that is known.
    - **Checked (2026-11-07):** Babawus powered off the Kingston stick through udisks2's `PowerOff`, without a password, after unmounting it. udisks2 flushed the stick's cache, stopped it, and removed it from the USB bus. Partitioning, formatting, mounting and unmounting had already worked through udisks2, so nothing needs the `pkexec` fallback, and it is removed from Requirements.md.
40. **Files dated outside exFAT's range looked changed on every run.** exFAT moves times before 1980 and after 2107 to the nearest it can store, so each run copied such a file again, and pushed out a real older version.
    - **Decided (2026-11-05):** On exFAT, Babawus compares such a time as the time exFAT would store.
41. **The chunk size was not measured.**
    - **Decided (2026-11-05):** 4 MiB. Flushing in chunks made no measurable difference to the speed, and the byte counter never waited more than about 1.4 seconds.
42. **Control characters.** Only those below `0x20` are refused by exFAT, so "a control character" would have skipped files that can be stored.
    - **Decided (2026-11-06):** The rule now says below `0x20`.
43. **exFAT's idea of case is not Unicode's.** Simple upper-casing missed 27 pairs of Greek letters that exFAT treats as the same, so two files could have ended up as one.
    - **Decided (2026-11-06):** Babawus covers every pair in the table `mkfs.exfat` writes, and a test checks all 65,536 characters against it. A few harmless extra pairs remain, such as `ı` and `I`.
44. **Pruning after a rename in case.** On exFAT, a version folder keeps its old case, so versions in the old case were never pruned.
    - **Decided (2026-11-06):** On exFAT, versions are compared without regard to case.
45. **Gaps in what is copied.** The requirements didn't cover selections that no longer exist, selections inside excluded folders, overlapping selections, paths with `..`, or empty folders.
    - **Decided (2026-11-06):** See "What is copied" in Requirements.md. This also covers reporting a missing selection as skipped on every run, and accepting rare false clashes between names such as `ı.txt` and `I.txt`.
46. **A source file that disappears during a run**, between the scan and the copy.
    - **Decided (2026-11-06):** A new code, `DISAPPEARED`, says what happened more clearly than `UNREADABLE`. It covers a file or folder that vanishes at any point during a run.

## Findings before building the configuration and log

Comparing the requirements with what the configuration and log have to do turned up these gaps. All are now covered in Requirements.md.

47. **File names don't fit the file formats.** Linux names are bytes, but the configuration and log are UTF-8, and a name can hold a tab or a line ending, which would break a log line.
    - **Decided (2026-11-06):** Log values are escaped (`\t`, `\n`, `\\`, `\xNN`). The file explorer refuses to select anything whose path isn't valid UTF-8.
48. **The computer ID wasn't fully defined.** `/etc/machine-id` ends in a line ending, and the application ID was never given. Changing either later would disconnect every computer from its media.
    - **Decided (2026-11-06):** The key is the 32 hexadecimal digits as text, without the line ending. The message is `io.github.ziggymax.Babawus`.
49. **A half-written last line could swallow the next entry.** Appending straight after it would glue the new entry onto the garbage, and the whole line would then be ignored.
    - **Decided (2026-11-06):** A line ending is written first if the log doesn't end in one. A last line with no line ending never counts.
50. **Renaming an account lost "Last backup".** Runs are matched on `username@computer folder`, so a linked account would show "No backup yet".
    - **Decided (2026-11-06):** Linking an account also renames it in the log.
51. **A run that isn't set up has no computer folder** for its log entry or lock.
    - **Decided (2026-11-06):** The hostname is used in its place.
52. **"Below 1 MB" wasn't precise,** and a run adds entries after the log has been trimmed.
    - **Decided (2026-11-06):** The limit is 1,000,000 bytes. A log over 900,000 bytes at the start of a run is trimmed to that.
53. **The configuration's edge cases.** Saving wasn't required to be safe, and invalid files had no defined status.
    - **Decided (2026-11-06):** Saved through `.babawus/tmp/` and renamed. Any file at `.babawus/babawus_config` makes a Babawus medium, and one that isn't in order needs attention. `format` is checked first. A file that can't be parsed can only be replaced by a new configuration, which leaves the backups alone.
54. **The log's keys and codes weren't defined,** nor was the run ID, or what a failed prune means for the verdict.
    - **Decided (2026-11-06):** Listed in "Format of the log". A run ID is 16 random hexadecimal digits. A failed prune is an error that doesn't change the verdict.
55. **Older versions were named by local time, which can go backwards.** When the clocks change in autumn, or a computer moves west, a newer version could get a name that sorts as older, and pruning could remove the wrong one.
    - **Decided (2026-11-06):** Names use UTC, so they can differ from the local time by some hours. That matters little, because people rarely remember the exact time of a version and tend to try a few anyway, and each file keeps its own modification time, which a file manager shows in local time. The log keeps local time, because its entries carry the UTC offset. A computer whose clock is simply wrong can still put names out of order.
56. **A running process didn't prove that a lock was live.** If the medium is pulled during a backup and put back, the lock has the process ID of the Babawus that is still open. After a crash, another program can get the lock's process ID. Either would leave the medium locked until the next restart.
    - **Decided (2026-11-06):** The process must be Babawus, and a lock with Babawus's own process ID must belong to an action that is still going.
57. **What a run does when things are not as the media list showed.** The requirements didn't say what happens if the configuration is no longer usable or the lock has become live when a backup starts, or if the end entry or an error entry can't be written.
    - **Decided (2026-11-06):** A configuration that isn't usable gives "Backup failed", and the medium isn't touched. The reason is `NOT_ADDED` if there is no `.babawus` folder or no configuration file, and `CONFIG_DAMAGED` if there is a configuration file that can't be used, including one that needs a newer Babawus. The verdict says "the configuration is damaged or outdated". If the medium's root folder has gone, the reason is `MEDIUM_REMOVED`. A live lock gives no verdict, and the list shows "Backup in progress". An end entry that can't be written counts as `FLUSH_FAILED`. An error entry that can't be written doesn't stop the run.
58. **What if the lock in `/run/lock/` can't be used,** and how does a second start by the same user find the first?
    - **Decided (2026-11-06):** Babawus opens anyway, because each medium has a lock of its own. A second start by the same user goes through the session bus, which GTK does by itself. Without a session bus, it gets the message that Babawus is already open. So that the medium's lock doesn't depend on this lock, a new medium lock is read back, and only counts if it still holds its own run ID.
59. **The details of the language choice.** Unclear what happens with several system languages, a settings file that can't be used, or a choice that can't be saved.
    - **Decided (2026-11-06):** The first system language that is available counts. A settings file that is missing or can't be understood counts as no choice, and never stops Babawus. A choice that can't be saved still applies until Babawus is closed. Choosing a language in the menu is what makes it a choice: until then, nothing is saved, and Babawus follows the system.

## Findings while building formatting

60. **"In use" wasn't defined.** The requirements say a disk with a partition in use other than the medium itself is never offered for formatting, but not what "in use" means.
    - **Decided (2026-11-07):** In use means active swap, another device on top (encryption, LVM or RAID, from the kernel's `holders`), or mounted anywhere other than below `/media/` or `/run/media/`. A disk with `HintSystem` is never offered either. A file system the desktop mounted is the medium itself, and is unmounted first.
61. **The desktop mounted the old file system while formatting.** The new partition starts where the old one did, so it showed the old exFAT file system, and Cinnamon mounted it while udisks2 was wiping it. Formatting then failed with "Device or resource busy". This happened in about one test run in three, and could happen on a real stick.
    - **Decided (2026-11-07):** The signatures of every old partition are wiped before the new partition table is written. In the test runs since, and when reformatting the real stick, nothing mounted it.
62. **Which letters a label may use, and which label is suggested.**
    - **Decided (2026-11-07):** `A`–`Z`, `a`–`z`, digits and `-`, because Ext4 and exFAT store other characters differently, and some systems show them badly. Babawus suggests the first free label of `BACKUP-A` to `BACKUP-Z`, then `BACKUP-1` and so on, ignoring case.
63. **Ejecting a medium that is in use.** The requirements didn't say what happens if a program still has a file open on the medium, or a file manager window shows it.
    - **Decided (2026-11-07):** udisks2 refuses to unmount it, and Babawus leaves it mounted and asks the user to close that program and try again. If powering off fails after unmounting, the medium can still be pulled out safely, and Babawus says so. Only USB media are ever ejected.

## Decisions on the start screen

64. **One click started a backup.** The requirements had a click on a Ready medium start the backup, which left no room for anything else to do with a medium, and let a stray click start a run.
    - **Decided (2026-11-07):** The window has the media list on the left and the details of the selected medium on the right, with its actions below. A click only selects. The backup starts with "Start backup". If exactly one medium is Ready, it is selected already, so the usual case still takes one press.
65. **Two marks were not enough for six statuses.** A tick and a cross blur "needs your poweruser" with "can never be used".
    - **Decided (2026-11-07):** A green tick for Ready, an amber "!" for Not set up, Backup in progress and Needs attention, and a grey cross for Write-protected and Not a backup medium. The shapes differ as well as the colours. The stick itself is in colour for a Babawus medium and grey for anything else. A row shows only the label, the stick and the mark. The maker, model, size, free space and last backup are in the details, with a small logo in their upper right corner.
66. **Actions that don't apply.** Hiding them would leave the user wondering where "Start backup" went.
    - **Decided (2026-11-07):** "Start backup" is always shown, and greyed out with a tooltip giving the reason when the medium isn't Ready. Other actions only appear when they apply.
67. **How Advanced works, and whether it lasts.** If it were saved, the next user could meet "Prepare…" by chance.
    - **Decided (2026-11-07):** Advanced is a menu item that adds the poweruser's actions next to the user's. It is off at every start and never saved, and the title bar shows a clear marker while it is on. Eject is for everyone, on any medium in the list.
68. **How texts are translated.** The language must change at once without a restart, which gettext handles badly.
    - **Decided (2026-11-07):** A small catalogue of Babawus's own in the library, with a test that every text exists in every language. The UI sets its texts again when the language changes. No new dependency is needed.

## Decisions on the backup screen

69. **What the rest of the window does during a backup.** Leaving the list in view would let the user start a second backup or eject the medium mid-run.
    - **Decided (2026-11-07):** The backup screen fills the window until the run has ended and the user goes back.
70. **Closing the window during a backup.** Quitting there and then would end the run without its end entry, so the next run would report it as interrupted.
    - **Decided (2026-11-07):** Closing asks "Stop the backup?", like Cancel. On yes, the run is cancelled and the window closes once it has ended properly.

## Decisions on the verdict screen

71. **Which language the details for the poweruser are in.** The user's language is easiest to read, but the poweruser may not share it.
    - **Decided (2026-11-07):** The user's language, with the fixed codes from the log next to the verdict, the reason and each problem, so that the poweruser can look them up whatever the language.
72. **A mark for "Backup failed".** The start screen's grey cross means "can't be used", which is too mild for a run that failed.
    - **Decided (2026-11-07):** A red cross. "Some files were skipped" and "Backup not finished" get the amber "!", "All good" the green tick.

## Decisions on the history and "What's backed up"

73. **How the history shows when a run happened.** Month names would need translating, and "3 days ago" says little about a run months back.
    - **Decided (2026-11-09):** The start time, as "Today, 14:03", "Yesterday, 09:12" or "2026-11-03, 17:54". Today and yesterday are calendar days in local time, so a run at 23:50 is yesterday's even a few minutes after midnight.
74. **Whose runs the history shows on a medium shared by several computers or accounts.**
    - **Decided (2026-11-09):** Every run on the medium, newest first, each showing whose it was ("By anna on laptop"). A run with no end entry whose user holds the lock shows "Backup in progress" rather than "Backup not finished".
75. **What each row of the history shows besides the time and the verdict.**
    - **Decided (2026-11-09):** The reason in plain words, the files and bytes copied, and how many files were skipped.
76. **How the history and "What's backed up" open.**
    - **Decided (2026-11-09):** Each fills the window, like the backup screen, with "Back" to return to the start screen.
77. **What "What's backed up" lists for my account.**
    - **Decided (2026-11-09):** The selected files and folders, the excluded folders and the excluded name patterns. Each selection is looked up on this computer when the screen opens, and one that is missing gets an amber "!" and says so. So does one that backs up nothing for another reason: excluded, outside the home folder, a link, or unreadable.

## Decisions on the format view

78. **How the poweruser reaches the format view.** The media list only shows mounted file systems, so a blank or unmounted stick never appears in it. But the format view must list drives whether they are mounted or not.
    - **Decided (2026-11-09):** "Prepare…" opens a full-window view that lists every drive Babawus may format, mounted or not. The selected medium's drive is preselected if it may be formatted. While Advanced is on, "Prepare…" is shown whenever at least one drive may be formatted. That includes the empty page, so a blank stick can be reached even when the list has nothing to select.
79. **What the explicit "yes, I really want to do this" looks like.**
    - **Decided (2026-11-09):** The warning says what will be erased: the drive's name and size, every partition on it, and any Babawus backups on it. A checkbox, "I understand that everything on this stick will be erased", enables a red "Erase and format" button.
80. **How exFAT and Ext4 are offered.**
    - **Decided (2026-11-09):** Two choices, each with a line in plain words: exFAT also works on Windows and Mac, and Ext4 works on Linux only, but allows longer names and copes better with being pulled out. exFAT is preselected.
81. **What follows a successful format.** Adding this computer and account belongs to the configuration editor (T-018), which isn't built yet.
    - **Decided (2026-11-09):** A done screen says the stick is ready, and that it must now be set up for each computer and account. It has "Add this computer and account…", greyed out as "Not yet available" until T-018, and "Back", which returns to the start screen with the new stick selected.

## Decisions on the configuration editor

82. **What kind of file explorer chooses what is backed up and which folders are left out.**
    - **Decided (2026-11-09):** A tree of the home folder that expands folder by folder. Each row has a tick box for backing it up, and a folder also has a "Leave out" choice. What is selected or left out is marked in place, and the totals update as the poweruser ticks.
83. **How "Change setup…" is laid out.**
    - **Decided (2026-11-09):** A full-window overview with the number of older versions to keep (N) and every computer and account on the stick. This computer's account can be changed, and the others are read-only. Each entry can be removed. Adding or linking this computer and account is done here too. "Choose what to back up…" opens a second full-window screen with the tree, the name patterns and the live totals.
84. **When changes are written to the stick.**
    - **Decided (2026-11-09):** Changes collect in the editor until "Save". Saving takes the medium's lock, writes the file safely and releases the lock. "Cancel" discards the changes, and leaving with unsaved changes asks first. Removing an entry and linking an account each ask for their own confirmation.
85. **How T-018 is delivered.**
    - **Decided (2026-11-09):** In two pull requests. T-018 covers the overview, N, adding, linking and removing entries, Save and Cancel, a damaged configuration, starting a new configuration, and the format view's "Add this computer and account…". T-031 covers the tree, the exclusions and the live totals.

## Decisions on choosing what to back up

86. **What unticking a folder inside a ticked folder does.** Its tick is inherited from the folder above.
    - **Decided (2026-11-09):** Unticking it leaves it out: it becomes an excluded folder, and is shown crossed out. Ticking it again removes the exclusion. So one tick box per row does both, in place of the separate "Leave out" choice in issue 82. A file inside a ticked folder can't be unticked on its own, because only folders and name patterns can be excluded.
87. **Whether the tree shows hidden files and folders**, whose names start with a dot.
    - **Decided (2026-11-09):** Not at first. A "Show hidden files" switch above the tree shows them. A hidden item that is already selected or left out is always shown, so that nothing chosen is out of sight.
88. **How the name patterns are edited.**
    - **Decided (2026-11-09):** As a list, each pattern on its own row with "Remove", and a text field with "Add" below. An empty pattern, or one containing `/`, is refused with the reason.
89. **What the live totals count.**
    - **Decided (2026-11-09):** The files and bytes chosen, next to the free space on the stick, for example "Chosen: 12,345 files (4.2 GB) · 15.5 GB free on the stick". They are counted in the background with a spinner, and counting starts over on each change, so the window never freezes. When the total is more than the free space, a warning says so.

## Decisions on the German texts

90. **Whether German addresses the user as "Sie" or "du".**
    - **Decided (2026-11-09):** "du", for example "Steck deinen Backup-Stick ein".
91. **What "your poweruser" is in German.**
    - **Decided (2026-11-09):** "dein Betreuer", for example "Wende dich an deinen Betreuer". The Advanced marker is "Erweitert".
92. **How German dates look in the history.**
    - **Decided (2026-11-09):** The same as in English: "Heute, 14:03", "Gestern, 09:12" and "2026-11-03, 17:54". Only the words change.

## Decisions on "Last backup"

93. **Whether "Last backup" shows the time.** Several backups can be made in one day, and "today" doesn't say which was last.
    - **Decided (2026-11-09):** It shows when the last backup finished, after the words: "Last backup: today at 14:03", "Last backup: yesterday at 09:12", "Last backup: 47 days ago (2026-09-23 at 17:54)". The start screen and the verdict screen show it alike. Days are calendar days in local time, as in the history, so that a backup at 23:00 is never "today" the next morning. A time in the future, from a wrong clock, counts as today.

## Decisions on the release checklist

94. **How each release's run of the manual checklist is recorded.**
    - **Decided (2026-11-09):** The steps live in `ReleaseChecklist.md`. Before a release, they are pasted into a GitHub issue ("Release 0.1.0 checklist"), whose tick boxes are ticked as each step passes, with notes on anything odd. The repository keeps the steps, and GitHub keeps each run.
95. **How detailed each step is.**
    - **Decided (2026-11-09):** Each step says what to do and what should be seen, so that anyone can run it, not only whoever wrote Babawus.
96. **How the checklist relates to T-022, T-024 and T-025.**
    - **Decided (2026-11-09):** The checklist has steps for pulling out a stick mid-copy, formatting without administrator rights, and installing, updating and uninstalling the package. T-021 writes the checklist only. The other three tasks stay open as running their part for the first time and fixing what it finds, because each needs someone at the computer.

## Decisions on installing (T-025)

97. **Which icon the menu entry uses.** The package shipped a placeholder icon of a stick with an arrow, but the checklist expects the Babawus elephant.
    - **Decided (2026-11-10):** The elephant logo, `assets/BabawusLogo-plain.svg`, the same one the app shows inside its window. The placeholder goes.
98. **Where the uninstall notes go.**
    - **Decided (2026-11-10):** In an "Uninstalling" section of `README.md`. They are not shipped in the package.
99. **How the package is installed in the checklist's first run.**
    - **Decided (2026-11-10):** By double-clicking the `.deb` at the computer and entering the password, as the checklist says, so that the path a poweruser takes is the one tested.
100. **How the first run of the Installing, Updating and Uninstalling steps is recorded.** Issue 94 calls for one issue per release, but this run is not a release.
    - **Decided (2026-11-10):** In the description of the pull request for T-025. The "Release 0.1.0 checklist" issue is kept for the release itself.
101. **Why the menu entry showed a gear instead of the elephant.** Installed by double-clicking, with Mint's package installer Captain, the "Backup" entry in Cinnamon's menu showed a generic gear until the next login or a restart of Cinnamon. It happened on both installs that way, but on none of five installs with `apt` over SSH. The cause wasn't found: Cinnamon finds the icon by name afterwards, but its menu keeps the first miss for the rest of the session.
    - **Decided (2026-11-10):** The desktop entry names the icon file by its full path, `/usr/share/icons/hicolor/scalable/apps/io.github.ziggymax.Babawus.svg`, instead of by the icon name, so that the menu loads the file without asking the icon theme. Tested through Captain against a package that used the icon name: that one showed the gear, and the full path showed the elephant. The cost, that an icon theme can't replace the icon, doesn't matter for Babawus.

## Decisions on pulling out a stick (T-022)

102. **Which test files the first run uses.** `~/BabawusTest` on the VM lacked three of the checklist's files.
    - **Decided (2026-11-10):** All three are created (`Photos/big.bin` of 2 GB, `meeting: notes.txt`, and an unreadable `secret.txt`), so that the folder matches the checklist for later runs. A backup after the pull therefore says "Some files were skipped" for the last two.
103. **Whether the stick is also pulled out during "Finishing…".** The checklist pulls it while it says "Backing up…", but the final flush is a separate path in the code.
    - **Decided (2026-11-10):** Yes, once at each moment. If "Finishing…" is too short to hit, the run notes that. As for T-025 (issue 100), the run is recorded in the description of the pull request.
    - **Found (2026-11-10):** "Finishing…" can't be hit by hand. Copying the 2 GB `big.bin` to the Kingston stick took about 80 seconds, but "Finishing…" didn't show in screenshots taken every third of a second, because each file is already flushed as soon as it is copied. So the checklist has no step for it. The automated tests cover it: `check_pulling_at_every_step` pulls the medium at every step of a run, the final flush included, and a medium that can't be flushed gives "Backup not finished" with `FLUSH_FAILED`, as "Finishing a run" in Requirements.md says.

## Decisions on formatting without administrator rights (T-024)

104. **What the second account formats, with only one disposable stick.** The checklist formats a second stick as Ext4 from that account.
    - **Decided (2026-11-10):** The second account formats stick A as Ext4, which also tests the owner and default ACL that Babawus sets on an Ext4 medium (issue 16). Afterwards stick A is formatted back to exFAT from the administrator's account, so that later runs find it as the checklist expects.
    - **Done (2026-11-10):** Stick A was formatted back to exFAT as `BACKUP-A`, from the administrator's account, after the test in issue 106.
105. **How the second account is created.**
    - **Decided (2026-11-10):** As the checklist says: by the user, in Users and Groups, as a "Standard" account.
106. **Whether the first account also uses the stick that the second account formatted.**
    - **Decided (2026-11-10):** Yes. Before reformatting, the administrator's account adds itself to the stick and backs up once, to check that an Ext4 medium formatted by one account can be used by another (issue 16).
    - **Changed (2026-11-10):** After issue 108, only an administrator can format Ext4, so the test is turned round: the administrator's account formats stick A as Ext4, and the second account adds itself and backs up once. Stick A is then formatted back to exFAT, as issue 104 says.
107. **How this first run is recorded.**
    - **Decided (2026-11-10):** In the description of the pull request, as for T-025 and T-022 (issues 100 and 103).
108. **Ext4 from an account without administrator rights.** In the first run, the second account's Ext4 format replaced the partition table without a password, and then needed an administrator's password to create the file system, so the stick was left erased but not formatted. udisks2 asks for `modify-device-system` both for extra `mkfs` arguments (Babawus passes `-m 0` and the `lazy_*_init` options) and for `take-ownership`, and Babawus needs one of the two so that the root folder doesn't belong to root. exFAT needs neither, and worked without a password.
    - **Decided (2026-11-10):** In "Prepare a stick", Ext4 can only be chosen by an administrator, meaning an account in the `sudo` or `admin` group, the groups that polkit asks for on Mint. For anyone else it is greyed out, with a line saying why, so that formatting never starts what it can't finish.
109. **Whether an administrator is told about the password.** polkit asks administrators for their password too (`auth_admin_keep`), except over SSH.
    - **Decided (2026-11-10):** Yes: when an administrator chooses Ext4, a short line says that it asks for their password while formatting.
110. **A stick that is still open for another account.** A mount stays after its account logs out. In the first run, stick A was mounted at `/media/steen/BACKUP-A` when the second account logged in, and `/media/steen` can only be opened by `steen`. Babawus took the "Permission denied" to mean that the stick isn't a Babawus medium, and offered only "Prepare…", which would erase it.
    - **Decided (2026-11-10):** A stick whose root folder this account can't open gets its own status, "Open for another account", with an amber "!" and the advice to pull out the stick and plug it back in, which mounts it for this account. Babawus doesn't guess what is on it, and offers neither a backup nor "Prepare…" for it. The fix goes into the pull request for T-024.

## Decisions on the package description (T-033)

111. **What the installer shows about Babawus.** Mint's installer Captain showed only the summary, title-cased by Python's `str.title()` as "Basic Backup With Usb Storage", with nothing below it.
    - **Decided (2026-11-10):** The summary stays "Basic backup with USB storage", which spells out the name, even though Captain shows "Usb". Below it comes a longer description for whoever installs Babawus: what it does, in plain words, and that the person who looks after the computer prepares the stick under Advanced. It is in English only, because a `.deb` package has one description.

## Decisions on the review of the look (T-032)

112. **How the user interface is reviewed.**
    - **Decided (2026-11-10):** In four steps, each a pull request of its own:
        1. `ui-review/UiReview.md` numbers every screen, state, dialog and menu a user can see.
        2. Babawus is run on the VM, and each item is screenshotted as `ui-review/<number>-<name>.png`. Claude drives the window, and the user does what needs hands: the stick, and the second account.
        3. The items are gone through one by one, and each one's comments and decision are written next to it in `UiReview.md`.
        4. The user interface is changed accordingly, in one or more pull requests, with new screenshots to compare.
    - **Done (2026-11-10):** Steps 1 to 3 are done. Step 4 is carried by tasks T-035 to T-041, and the review's outcome is summed up at the top of `UiReview.md`.
113. **Where the review keeps its decisions.**
    - **Decided (2026-11-10):** In `ui-review/UiReview.md`, next to each item, rather than one by one in this file. This item points there. Changes still to make become tasks in Tasks.md.
114. **Whether the screenshots are committed.**
    - **Decided (2026-11-10):** No. `UiReview.md` is committed, and the screenshots stay on the VM (`ui-review/*.png` is in `.gitignore`). The document refers to each screenshot by its number, and says enough in words that a decision can be understood without the picture.
115. **Which languages are screenshotted.**
    - **Decided (2026-11-10):** Every item in English. German too for the screens with the longest texts, where cut-off text or wrapping is likeliest.
116. **Which window size the screenshots use.**
    - **Decided (2026-11-10):** The size Babawus opens with. A few key screens are also taken with the window as small as it goes, to check the 1366 × 768 limit.
117. **Whether users copy diagnostic details for the poweruser.** The verdict screen had "Details for your poweruser" with a Copy button, which largely repeats what the log on the stick records. Found while reviewing the look (T-032, `ui-review/UiReview.md` item V04).
    - **Decided (2026-11-10):** No. The details and Copy are removed from the verdict screen. The log is the single place for diagnosing: it is always there, and doesn't depend on a user doing anything. So the log must be enough on its own: each run's start also records Babawus's version and the computer's hostname, and every skipped file and error is logged, instead of the first 20 and then only a count. (Issue 124 changed this to the first 100.) The log is already trimmed by size, so it can't grow without limit. The README says where the log is and that it is plain text.
118. **The window's size.** Requirements.md said the window fits on a 1366 × 768 screen with room to spare. At the size it opened with, 848 × 635, "Prepare a stick" had to be scrolled to reach the warning and the button. Found while reviewing the look (T-032, `ui-review/UiReview.md` items F01 and G03).
    - **Decided (2026-11-10):** Babawus opens at 1024 × 768, assuming that no screen smaller than Full HD (1920 × 1080) is used. Having to scroll to find buttons or read text is avoided at that size. The window can still be made smaller, down to what each screen needs.
119. **What the screen calls an entry.** Names of entries such as "laptop" can be read as ordinary words: "Link this computer to laptop?" reads as "link this computer to this computer" for someone at a laptop. Found while reviewing the look (T-032, `ui-review/UiReview.md` item C10).
    - **Decided (2026-11-10):** The screen calls an entry a **backup profile**, and always marks its name as a name: "Link this computer to the backup profile called “laptop”?". The term becomes official where the screen starts using it, in T-040: the glossary in Requirements.md gets "Backup profile" next to "Entry", and Concept.md says what it is, so that the code, the documents and the texts use the same words. T-040 also settles whether both computer and account entries are called that on screen, and uses the same words on the setup page as in its questions.

## Decisions on the start screen (T-038)

120. **How large the list's stick pictures, their labels and the logo are.** The review asked for larger pictures, a smaller label taking the same width, and a slightly larger logo (`ui-review/UiReview.md` G01 and S06).
    - **Decided (2026-11-10):** The stick picture grows from 32 to 48 pixels. The label shrinks from 15 pt to the normal text size, still bold. The logo grows from 64 to 80 pixels.
121. **What the empty page says while a stick is plugged in but not listed.** `ui-review/UiReview.md` S02 decided on "The stick isn't opened. Pull it out and plug it back in." for a stick that may be formatted but isn't opened. A blank stick, with no file system on it, isn't listed either, and plugging it back in doesn't help.
    - **Decided (2026-11-10):** A stick that has a file system that isn't opened gets the S02 text. A stick with no file system Babawus can open gets its own text: "The stick that is plugged in can't be used yet. Your poweruser can prepare it." Both show whether Advanced is on or off; "Prepare…" still shows only with Advanced on. On the screen, the first sentence is the large heading and the second, what to do, sits below it in the normal text style, so that a word isn't left alone on a line of the large text.

## Decisions on the setup texts (T-040)

122. **What a backup profile is, and how the setup page names things.** Issue 119 introduced the term, and left open whether computers and accounts are both called that.
    - **Decided (2026-11-10):** A **backup profile** is the settings and backups of one account on one computer: an account's entry in the configuration, inside its computer's entry. A computer is not a backup profile; it holds them. In German, "Sicherungsprofil". On screen:
        - Names of computers and backup profiles are quoted in sentences, questions and buttons: "the computer called “laptop”", "the backup profile for “anna”", "Link to “laptop”…". The list rows stay unquoted, because there a name stands on its own.
        - The setup page's list is headed "Backup profiles on this stick".
        - The button that adds this computer says "Add this computer and your account", and the line above it gives the folder name it will be added as.
        - Linking a computer asks "Link this computer to the computer called “laptop”?"; linking an account asks "Link your account to the backup profile for “anna”?". Removing asks "Remove the computer called “laptop” and its backup profiles from the setup?" or "Remove the backup profile for “anna” on “laptop” from the setup?".
        - The glossary in Requirements.md gets "Backup profile", and Concept.md says what it is.

## Decisions on choosing what to back up (T-041)

123. **When a folder's tick box is partly ticked.** `ui-review/UiReview.md` T02 decided that a ticked folder with something left out inside it shows a partly ticked box.
    - **Decided (2026-11-10):** A folder's box is partly ticked when the folder is backed up but a file or folder inside it has been unticked by hand, or when it isn't backed up but a file or folder inside it has been ticked. What a name pattern leaves out doesn't count, or almost every folder would look partly ticked. Clicking the box does what it did before: it ticks or unticks the folder.

## Decisions on the log (T-036)

124. **How many errors a run logs one by one.** Issue 117 said every skipped file and error is logged, relying on trimming to bound the log. But trimming happens at the next run's start and always keeps each account's last backup, so one run with, say, 100,000 unreadable files would leave a log of several megabytes that never shrinks.
    - **Decided (2026-11-10):** The first 100 skipped files and errors of a run are logged one by one, instead of the first 20; then one `MULTIPLE_ERRORS` entry, and the rest are only counted. Many errors are most likely caused by the same problem, and if they have different causes, the poweruser fixes what caused the first 100 and runs the backup again. Each start entry also records Babawus's version and the computer's hostname, as issue 117 decided.

## Decisions on the start screen's buttons (T-045)

125. **How the user's buttons fit and wrap.** In German, the four buttons need about 722 pixels, and the details pane had about 712 at 1024 × 768. So Eject wrapped onto a line of its own, and GTK's wrapping row lined the other three up in wide columns.
    - **Decided (2026-11-10):** Both: the stick list narrows from 260 to 240 pixels, so that all four fit on one line at 1024 × 768 in German too, and the buttons wrap like words in a narrower window, each at its own width and close to the one before, instead of in columns.

## Decisions after trying the new look (T-046 to T-048)

The user tried the package built from pull request 59 on the VM: formatting as Ext4, setting up, several backups, and ejecting and plugging back in. Everything worked; these came up.

126. **The title of the name patterns' section.** "Leave out by name" doesn't say what is left out.
    - **Decided (2026-11-11):** "Files to leave out by name", in German "Dateien nach Namen auslassen" (T-046).
127. **What the setup page says after saving.** "Saved." on its own is easy to miss the meaning of.
    - **Decided (2026-11-11):** "Stick setup saved.", in German "Einrichtung des Sticks gespeichert.", using the same word as the page's heading, "Setup of this stick", rather than "configuration" (T-047).
128. **How the start screen tells how ejecting went.** After Eject, the empty page said "Plug in your backup stick" with "You can pull out BACKUP-A now." in smaller type below it, which read confusingly.
    - **Decided (2026-11-11):** Every outcome of Eject is a pop-up with an OK button: "You can pull out BACKUP-A now.", or that it is still in use by a program, was already pulled out, or couldn't be ejected. "Ejecting BACKUP-A…" stays in the explanation while it goes on. Once the pop-up is closed, the start screen shows what is plugged in, for example "Plug in your backup stick". The verdict screen's "Eject BACKUP-A" uses the same pop-up, and after a successful eject Babawus goes back to the start screen; after a failed one, the verdict screen stays, so that Eject can be tried again (T-048).

## Decisions from the 0.1.0 checklist run ([#63](https://github.com/ziggymax/babawus/issues/63))

129. **What keeps a stick busy.** The checklist's "still in use" step opened stick A in the file manager and pressed Eject. On Linux Mint 22 the stick ejected anyway: Nemo watches a folder without keeping anything open on the stick, and closes its window when the stick goes. A text editor showing the stick's log didn't stop it either, because it reads the whole file and closes it. So Babawus was right to eject: nothing was using the stick in a way that could lose data. But the step and the message both name a file manager window as an example.
    - **Decided (2026-11-11):** The checklist makes the stick busy with a terminal whose current folder is on the stick (`cd` into it), which reliably keeps it in use. The message no longer names a file manager: "BACKUP-A is still in use by a program, for example a terminal or a file being copied. Close it, or wait until it has finished, and try again.", in English and German, and Requirements.md says the same (T-051).
130. **When and how 0.1.0 is released.** The checklist run ([#63](https://github.com/ziggymax/babawus/issues/63)) passed, apart from the "still in use" step of issue 129.
    - **Decided (2026-11-11):** T-051 is fixed first, so that 0.1.0 has the right wording; the rest of the run doesn't need repeating for it. 0.1.0 is then published as a git tag `v0.1.0` on main and a GitHub release with the `.deb` attached and short notes, linking [#63](https://github.com/ziggymax/babawus/issues/63) (T-052).

## Decisions on the logos (T-053)

131. **Which logo goes where.** The user made a second logo, the elephant without the name below it: `assets/BabawusLogo-NoText.svg`, with `assets/BabawusLogo-NoText-plain.svg` as the plain SVG that Babawus uses.
    - **Decided (2026-11-12):** The logo with the name is only shown in the upper right corner of the start screen, which includes the page shown when no stick is plugged in. Everywhere else shows the logo without the name: the menu icon the package installs (and so the window's icon in the panel), the About pop-up, and the "already open" message. Both SVG files are kept in git, as for the logo with the name.

## Decisions on About (T-054)

132. **How About explains the name.** The user asked for the comment in About to show the letters that spell Babawus in bold. The German comment, "Einfache Sicherung auf USB-Sticks", doesn't spell it.
    - **Decided (2026-11-12):** The English long form, in title case, with the name's letters in bold: "**Ba**sic **Ba**ckup **w**ith **U**SB **S**torage". It is shown in every language, because it is where the name comes from. In any language other than English, that language's own comment follows on the line below, so that the name is still explained; in German "Einfache Sicherung auf USB-Sticks".

## Decisions on the stick's details (T-055)

133. **Where the file system is shown.** The user asked for the start screen's details to name the stick's file system.
    - **Decided (2026-11-12):** It is the last item of the details line, for every user, not only under Advanced: "Kingston DataTraveler 3.0 · 15.5 GB · 15.5 GB free · exFAT". It is named as people know it, the same in every language: exFAT, Ext4 (as on the format page), FAT, NTFS, and Ext2 or Ext3; any other file system as udisks2 names it, and nothing if udisks2 reports none.

## Decisions on the start screen's new layout (T-056)

134. **Start backup and Eject in the stick list.** The user drew two mock-ups of the start screen after trying 0.1.0, one for a user and one with Advanced on. Every stick in the list has its own round Eject button, and the selected Babawus stick has a large "Start backup" button right below its row. The details on the right keep their texts, and the remaining buttons come right after the texts, below a thin line.
    - **Decided (2026-11-12):**
        - The status mark in each row (the tick, "!" or cross) is dropped: Eject takes its place. The selected stick's status and mark are in the details, as before. A spinner still shows in a row while its stick is being checked.
        - "Start backup" shows only under the selected stick, and only once it is known to be a Babawus stick. If no backup can start, for example while another account is backing up to it or when it isn't set up for this account, the button is greyed out, and its tooltip says why, as before.
        - The poweruser's buttons, "Prepare…" and "Change setup…", go on the same row as the user's, after them, wrapping like words in a narrow window. They no longer have a row and a line of their own; this replaces part of `ui-review/UiReview.md`'s start screen decision (T-038).
        - The Eject button of a stick that isn't selected ejects that stick at once, and the selection stays where it was. While a stick is being ejected, every Eject button is greyed out.
        - If the selected stick has none of the remaining buttons, the line below the texts isn't shown either.

## Decisions on the log window (T-057)

135. **A log window for the poweruser.** The user asked for a "pretty listing" of a stick's log, opened from a run in the history, with that run's lines highlighted, in a window of its own that isn't modal and follows changes to the log.
    - **Decided (2026-11-12):**
        - Right-clicking a run in the history offers it only while Advanced is on; otherwise right-clicking does nothing. The offer is a small menu at the pointer with one item, "Show in the log".
        - The window shows the log one row per line, as close to the file as possible, in a fixed-width font: the timestamp as written, the first 8 digits of the run ID, the entry type, the `key=value` pairs as "key value", and any closing English text, dimmed. `ERROR` is red, and `END` takes the colour of its verdict's mark. Each run's lines are shaded alike, alternating from run to run, and the clicked run's lines are highlighted, with the window scrolled to them. Garbled or incomplete lines are shown dimmed, as they are in the file, so that nothing is hidden.
        - The window follows the log: when the file changes, it is read again, the highlight stays on the same run, and the view keeps its place.
        - When the stick is ejected or pulled out, its log window closes by itself.
    - **Decided by Claude, as the obvious defaults (2026-11-12):** there is at most one log window per stick; choosing "Show in the log" for another run of that stick moves the highlight and brings the window to the front. The window is titled "Log of BACKUP-A", opens at a size of its own, and closes when Babawus does. Its texts are in the language chosen when it opened. Long lines don't wrap: the window scrolls sideways too.

## For the next version: the configuration and a reminder at login (T-058, T-059)

The user asked on 2026-11-13, after trying the build with T-053 to T-057, for two more things before the next release. Their sample of the new configuration:

    # Babawus configuration
    # This file is written by Babawus. Change it with the Advanced functions in Babawus.
    # If it is edited by hand, the medium needs attention until it has been saved in Babawus.

    [[babawus]]
    config_format = 1
    config_app_version = 0.1.1
    config_update_local = "Fri Nov 13 12:43:40 PM CET 2026"
    config_update_utc = "Fri Nov 13 11:43:40 AM UTC 2026"

    [[computer]]
    id = "69cc641079d85ec2d1b909e0bc3d3a2a"
    folder = "steen-virtualbox"
    hostname = "steen-VirtualBox"

    [[computer.account]]
    username = "steen"
    older_versions = 20
    run_check_days = 7
    selected = ["Documents"]
    exclude_patterns = [".cache", "*.tmp", "*~"]
    exclude_folders = [".local/share/Trash", "Documents/Backups"]

    # checksum: 2cbbee54a52d7feb

136. **The new configuration format (T-058).** A `babawus` table with the format, the version of Babawus that last wrote the file, and when, in local time and UTC; and per account the number of older versions to keep (instead of one N for the whole medium) and `run_check_days`, the most days that may pass before the reminder of T-059.
    - **Open:**
        - The sample needs two small changes to be valid TOML that reads as meant: `[babawus]` rather than `[[babawus]]`, which would be a list of tables; and the version in quotes, `"0.1.1"`.
        - The times: as in the sample, or in the log's form (`2026-11-13 12:43:40 +0100`, and `2026-11-13 11:43:40 UTC`), which sorts and reads the same in every language.
        - Whether the file also keeps when it was first created, or only when it was last written.
        - The format number: the new layout is format 2, so that 0.1.0 shows a stick with it as "Needs attention" (a newer Babawus is needed) instead of misreading it. Today the format is found before the first table; with `[babawus]` first, reading it needs to change.
        - Sticks set up with 0.1.0 (format 1): read as they are, and written in the new format the next time they are saved, or upgraded at once. Each account then starts with the old N, and with a default for `run_check_days`.
        - The range and default of `run_check_days`, and whether 0 means "never remind".
        - Older versions per account: each account's versions are kept apart already (under `backup/<computer folder>/<username>`), so pruning follows the account's own N. "Change setup…" sets N per account instead of once for the stick, and Requirements.md's user story about N changes with it.
    - **Decided (2026-11-13):**
        - The table is `[babawus]`, first in the file, with `config_format = 1`, `config_app_version` (in quotes, for example `"0.1.0"`), and `config_update_local` and `config_update_utc`, the time the file was last written, in the log's form: `"2026-11-13 12:43:40 +0100"` and `"2026-11-13 11:43:40 UTC"`. Only the last write is kept, as in the user's sample. Every write sets all three.
        - The format number stays 1. There are no real users of Babawus yet, only test sticks, so a file in the 0.1.0 layout isn't read or upgraded: it is damaged, like a file edited by hand, and the poweruser starts a new configuration on that stick. That only replaces the file: the backups stay. Babawus carries no code for older layouts until it has real users.
        - Each account has `older_versions` (1 to 99, 10 for a new account) and `run_check_days` (0 to 365, 7 for a new account; 0 means never remind). Pruning uses the account's own number.
        - "Change setup…" shows the two numbers for every backup profile in "Backup profiles on this stick", each with fields to change them, not only for the account that is logged in. The single field for the whole stick goes.
137. **A reminder at login when the last backup is too long ago (T-059).** Babawus started with `--checklastbackup` looks at when this account last made a backup. If that is more than `run_check_days` ago, it shows a small window with the logo, "X days have passed since your last backup. Do you want to back up now?", with Yes, which opens the normal window, and No, which closes Babawus. Otherwise, or if no backup has been noted yet, it closes without showing anything. Each successful backup notes its time in a file under `~/.local/state`.
    - **Open:**
        - The file: `$XDG_STATE_HOME/babawus/last_backup` (by default `~/.local/state/babawus/last_backup`), the standard place for state like this.
        - The stick is usually not plugged in at login, so `run_check_days` can't be read from it then. The state file would also note the account's `run_check_days` from the stick at each backup, and with several sticks, the one from the latest backup counts.
        - Whether "Some files were skipped" counts as a backup here, as it does for "Last backup" (Requirements.md, "See when I last made a backup").
        - How Babawus gets started at login: the package could install an entry in `/etc/xdg/autostart` for every account, the poweruser could add it by hand in "Startup Applications", or "Change setup…" could offer it per account.
        - What happens if Babawus is already open, for this account or another (only one runs per computer), and whether the reminder counts calendar days, as "Last backup" does.
    - **Decided (2026-11-13):**
        - Babawus doesn't start itself at login. The poweruser adds `babawus --checklastbackup` in "Startup Applications" for each account that wants the reminder; the README says how.
        - A backup counts if it finished, even with skipped files ("All good" or "Some files were skipped"), as for "Last backup". Days are calendar days in local time, as for "Last backup": with `run_check_days = 7`, the reminder comes from the 8th day after the backup on. `run_check_days = 0` never reminds.
        - The reminder shows the logo without the name, "Your last backup was 9 days ago." in bold, "Do you want to back up now?", and the buttons "Back up now" and "Not now". "Back up now" opens the normal window, as a normal start would; "Not now", or closing the reminder, closes Babawus.
    - **Decided by Claude, as the obvious defaults (2026-11-13):** the file is `$XDG_STATE_HOME/babawus/last_backup` (`~/.local/state/babawus/last_backup` by default), a small TOML file with the time the last counted backup finished, in the log's form, and the account's `run_check_days` from the stick it went to, so that the reminder works without the stick. It is written after each counted backup, by whichever stick it went to; a file that can't be read counts as no file. Whether to remind is decided before anything is shown, so that Babawus closes without a trace when no reminder is due. If Babawus is already open, `--checklastbackup` brings its window forward like any other start.

138. **Square logos (T-062).** The menu and panel icon sometimes looked stretched in height, because the elephant without the name was 87.4 × 60.6 mm, and the desktop fits icons into a square. The user redrew all four logos as 90 × 90 mm squares, with a white rounded background, and added PNG exports of both plain logos (340 × 340 pixels).
    - **Decided (2026-11-13):** Babawus keeps using the plain SVGs, `assets/BabawusLogo-plain.svg` and `assets/BabawusLogo-NoText-plain.svg`, under the same names, so the code and the package don't change: an SVG stays sharp at every size the desktop picks. The PNGs are kept in git as exports for other uses, such as the website, and aren't installed. Checked on the VM: the panel icon and About show the square elephant on its white tile, and the start screen shows the logo with the name in its corner.

139. **The menu icon is larger than the other apps' icons (T-063).** With the square logos of 0.2.0, the white tile fills the whole icon. Mint-Y's app icons leave a margin: their tile covers 42 of 48 pixels, 7/8 of the icon, at the size the panel uses. So the Babawus icon looked larger than the others in the menu and the panel.
    - **Decided by Claude, as the obvious fix (2026-11-14):** the package installs its own copy of `assets/BabawusLogo-NoText-plain.svg`, `crates/babawus/assets/io.github.ziggymax.Babawus.svg`, with the same drawing and only a larger page: 102.857 mm instead of 90, with the logo in the middle. That is a margin of 1/16 of the size on each side, as Mint-Y has. The logos inside the window don't change. A unit test checks that the copy is the logo with only the page changed, so a new logo needs a new copy too. Checked on the VM: in the panel and the menu the tile is now the same size as Firefox's and Timeshift's.

140. **How to release 0.2.1 (T-064).** Since 0.2.0, only the icon in the menu and the panel changed (T-063).
    - **Decided (2026-11-14):** Only the installing steps of the release checklist are run: 0.2.1 is installed over the released 0.2.0 as an update, the menu and panel show the new icon, and Babawus starts with its sticks and history as before. There is no checklist issue; the release notes say what was checked. One pull request raises the version and records the release in Tasks.md; the tag `v0.2.1` goes on its merge commit, and the GitHub release follows straight after.

## Decisions on 1.0.0-alpha.1, the document cleanup (T-065 to T-071)

141. **How the document cleanup of 1.0.0-alpha.1 is done.** ReleasePlanning.md lists Concept, Requirements, Issues, Tasks, DevEnvSetup and ReleaseChecklist for cleaning up, with Issues and Tasks each split in two.
    - **Decided (2026-11-15):**
        - One task and one pull request per document, in this order: Issues (T-065), Tasks (T-066), Concept (T-067), Requirements (T-068), DevEnvSetup (T-069) and ReleaseChecklist (T-070).
        - Our issues are referred to as "issue 132", and tasks as "T-063", without a file name. GitHub issues and pull requests are always "#63", linked where possible, so the two never clash; the existing "issue 63" and "issue 75" become "#63" and "#75".
        - "Clean up and consolidate" means: describe Babawus as it is in 0.2.1, fix what is out of date, drop the "first version" and step-by-step history, and remove repetition between the documents. The poweruser wording stays until 1.0.0-alpha.3. Claude proposes the changes, and the user reviews each pull request.
        - A resolved issue moves to Issues-Resolved.md word for word, including any old "Open:" options, so its history stays complete. Only references are updated.
        - A last task (T-071) raises the version to 1.0.0-alpha.1 and updates ReleasePlanning.md. After it is merged, Claude tags `v1.0.0-alpha.1` on main, with no GitHub release and no `.deb`.

## Decisions on splitting the task list (T-066)

142. **How Tasks-Open.md is organised.** Tasks.md had sections for the first version's parts (logic, udisks2, user interface, release) and the next version, all done or nearly empty, with T-061 and T-043, two problems seen once, left in them.
    - **Decided (2026-11-15):** By version: "Now", the version being worked on (1.0.0-alpha.1), "Needs a decision", "Waiting" for problems seen once and not since (T-061 and T-043), and "Later versions" (T-027 and T-028). The empty sections go. Tasks-Done.md keeps the done tasks word for word, in the order they were done, with "pull request 61" written as #61, as issue 141 decided.

## Decisions on cleaning up Concept.md (T-067)

143. **How far Concept.md is trimmed.** It repeated much of the exact behaviour that Requirements.md also states, such as the configuration file's name and checksum, how computer folders are named, the formatting warning, the rules for versions and the verdicts' wording.
    - **Decided (2026-11-15):** Keep the why, cut the how. Every section and its reasoning stays: who Babawus is for, what it protects against, why exFAT or Ext4, why there is no restore function. Exact details that Requirements.md owns are cut, and the text points there instead. The revision history and the "First version" section go, and what had changed since is brought up to date: the login reminder, the log in place of a text to copy, the start screen's list, and N per backup profile.

## Decisions on cleaning up Requirements.md (T-068)

144. **Whether Requirements.md's repetition is merged away.** It was mostly up to date, but its Introduction explained the two roles again, as Concept.md and the glossary do, and two user stories, "Provide feedback after a backup operation" and "Recover from an interrupted backup", said less than "Get a clear verdict" and "Interrupted runs in the log".
    - **Decided (2026-11-15):** Merge them. The Introduction points at Concept.md and the glossary, and the two thin stories are folded into "Get a clear verdict"; no behaviour is lost. Besides that, only what is out of date or history is fixed: `config_format` in the glossary, the Python fallback, "Not in the first version" (now "Not in Babawus yet"), the issues for the German wording, and the claim that a backup tells the user at its start not to pull out the stick, which only "Finishing…" does.

## Decisions on cleaning up DevEnvSetup.md (T-069)

145. **How the guide signs in to GitHub.** It said to use a fine-grained token limited to this repository, but the VM is signed in through `gh auth login` in the browser, with an OAuth token whose scopes include `repo` and `workflow`, and which reaches every repository of the account.
    - **Decided (2026-11-15):** The guide describes the browser sign-in, as used: GitHub.com, HTTPS, web browser, and approving the request in the browser, since a request that isn't approved ends with `access_denied`. The security notes say that the token reaches every repository, and how to revoke it. Besides that, the guide is brought up to date with the VM as it is: a 32 GB disk, two test sticks, `automount-open` turned off, and the size of `target/`; and dated history is dropped from the discoveries, which are kept.

## Decisions on cleaning up ReleaseChecklist.md (T-070)

146. **Whether the release checklist says how to pick a partial run.** It always assumes a full run from a clean install, but for 0.2.1 only the installing steps were run, as an update (issue 140).
    - **Decided (2026-11-15):** No. The checklist stays the full run. Whether a small release runs only part of it is decided for each release and recorded as an issue, as for 0.2.1. Otherwise only history is removed: the task numbers in the headings, and the versions 0.1.0 and 0.1.1 in the examples.

## Decisions on 1.0.0-alpha.2, language files (T-072 to T-075)

147. **How 1.0.0-alpha.2 is set up.** ReleasePlanning.md says the texts move into language files loaded at runtime, with English as one of them, and that a broken or incomplete file is rejected, with the reason "written to the log". It leaves open the file format, where the files are kept, which log is meant, and how numbers are written in each language. Today the 262 texts, in English and German, are compiled into `text.rs`, and the separators in numbers are in the code for those two languages only.
    - **Decided (2026-11-15):**
        - Four tasks: T-072 compares the formats in issue 148, and the user chooses; T-073 loads the language files at runtime, moves English and German into them and installs them with the `.deb`, in one pull request, since it can't land halfway; T-074 writes a short guide to adding a language; T-075 raises the version to 1.0.0-alpha.2, after which Claude tags it.
        - The language files are kept in one place only, `/usr/share/babawus/languages/`, installed by the `.deb`. Adding a language means copying a file there with `sudo`, a poweruser job. An upgrade keeps the files it didn't install.
        - Babawus's log is on the stick, but the language files are read at start, without a stick. So the reason a file is rejected is written to standard error, which the desktop puts in `~/.xsession-errors`. A poweruser sees it by running `babawus` in a terminal, as the guide explains.
        - Each language file states the language's code, its name for the Language menu, and its decimal and thousands separators, so that a new language needs no code.

148. **Which format the language files have (T-072).** The texts move into one file per language, loaded at runtime (issue 147). The format must allow switching the language at once, without a restart; placeholders such as `{files}`, checked against the English ones; and a broken or incomplete file must be found and rejected. It should be easy to edit by hand and to have translated by machine or AI, and, by doctrine 3, add as few dependencies as possible. The 262 texts already have names (`StartBackup`, `CantStart`), and plurals are done as now with a text for one and a text for many (`OneFile`, `Files`), which fits English and German but not every language: Polish, for example, has a third form for 2 to 4.
    - **TOML:** one file per language, for example `de.toml`, with a `[language]` table (`code`, `name`, `decimal`, `thousands`) and a `[texts]` table of `StartBackup = "Sicherung starten"`. Babawus reads TOML already, through the `toml` crate, so there is no new dependency, and a syntax error comes with its line. Checking for missing texts and placeholders is a few lines. Any text editor, and any AI, handles it. Few translation tools read it, and plurals stay one form and many.
    - **gettext (`.po`):** the long-standing format of Linux translations, with the best tool support: Poedit, Weblate and Lokalize, and `msgfmt` checks `{name}` placeholders. But Babawus can't use the system's gettext, which can't switch language without a restart, so it would read the files itself, either with a reader of its own for `.po` (a hundred lines or so, for the escapes and lines split in two) or with the `gettext` crate, which reads only compiled `.mo` files and brings 9 crates. In gettext, English is the source text rather than a language file like the others, which goes against issue 147, unless each text's name is the key. Real plural forms need the file's `Plural-Forms` formula to be worked out, which is more code.
    - **Fluent (`.ftl`):** Mozilla's format, made for switching at runtime, with real plural forms for every language and placeholders written `{ $files }`. Through the `fluent-bundle` crate it brings 24 crates. Weblate and Pontoon read it. It is the most capable of the three, and the most to depend on, while Babawus's texts are mostly plain sentences.
    - **Recommendation:** TOML. It meets every need with no new dependency, it keeps the texts' names, and English is a file like the others. If a language with more plural forms is added later, plurals can be revisited then.
    - **Decided (2026-11-15):** TOML, as recommended. Asked whether a long text, such as a built-in help of a few hundred lines, also works well: it does, as a multi-line string, which keeps its lines and blank lines and needs no escaping beyond a backslash or three quotes in a row, while gettext would make it one entry that goes out of date with any change to the English, and Fluent wants every line indented. Whether the help sits in the language file or in a file of its own is decided in 1.0.0-alpha.3.
