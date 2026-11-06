# Babawus – Open issues before design

*Survey of README.md, Concept.md (revision 5) and Requirements.md, 2026-11-04. Items are ordered by priority within each section. Items 1–6 were settled first, because each one changes what gets built, followed by items 7–30. Their decisions are recorded below and in Concept.md (revision 8) and Requirements.md.*

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
    - **Decided (2026-11-05):** udisks2 over D-Bus for everything, with no root code in Babawus; a single `pkexec` call is the fallback for anything udisks2 can't do. The config checksum is SHA-256, first 16 hex digits, replacing xxHash64. Rust crates count under doctrine 3.
25. **Platforms.** Which Mint version and desktop to target, and whether to support Wayland, X11 or both.
    - **Decided (2026-11-05):** Supported: Mint 22.x Cinnamon on X11. Everything else is best effort. The target is reviewed when a new major Mint version comes out.
26. **Testing exFAT and Ext4 behaviour.** Loop devices need root; fake file systems may not behave like the real ones. Pick an approach.
    - **Decided (2026-11-05):** Three layers: unit tests with an in-memory fake and fault injection, integration tests on real file systems in loop-mounted image files, and a manual checklist on real sticks before each release. Automated tests can only ever format a loop device they created themselves.
27. **Speed.** Scanning, the live file and byte totals in the config editor, and comparing files must not freeze the UI on a large home folder.
    - **Decided (2026-11-05):** Plain worker threads with a message channel, one scan per backup, background live totals with "Counting…", screen updates limited to about 10 per second, and the byte counter moves during large files. The per-file `fsync` stays (see "Follow-ups").

## D. Document housekeeping

28. **Pick one source of truth.** Concept and Requirements repeat each other, which has already caused small differences (items 2, 13 and 21).
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
    - **Decided (2026-11-05):** `0x07` for exFAT and `0x83` for Ext4 on MBR, with the matching GPT types. The GPT types have not been tested.
38. **Formatting may need no password.** It ran without one from an SSH session, using an administrator's account.
    - **Decided (2026-11-05):** Accepted. udisks2 may or may not ask, and the explicit confirmation is the safeguard. The manual checklist covers formatting from an account without administrator rights.
39. **Powering off has not been tested.** The stick reports `CanPowerOff: true`, but that is all that is known.
    - **Open:** Test it on a real stick. The `pkexec` fallback stays in Requirements.md until then, and can go if nothing needs it. Tracked as T-019 in [Tasks.md](Tasks.md).
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
    - **Decided (2026-11-06):** A configuration that isn't usable gives "Backup failed", and the medium isn't touched. The reason is `NOT_ADDED` if there is no `.babawus` folder or no configuration file, and `CONFIG_DAMAGED` if there is a configuration file that can't be used, including one that needs a newer Babawus. A live lock gives no verdict, and the list shows "Backup in progress". An end entry that can't be written counts as `FLUSH_FAILED`. An error entry that can't be written doesn't stop the run.
