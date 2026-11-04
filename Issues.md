# Babawus – Open issues before design

*Survey of README.md, Concept.md (revision 5) and Requirements.md, 2026-11-04. Items are ordered by priority within each section. Items 1–6 were settled first, because each one changes what gets built; their decisions are recorded below and in Concept.md and Requirements.md.*

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
8. **Exclusion wildcards.** The syntax isn't defined: is `**` supported, and does a pattern match the name or the path relative to home? It's also unclear whether exclusions apply per account or to the whole medium, and whether some are built in, such as `.cache`, Trash, or cloud folders mounted inside home.
9. **The version limit N.** It's unclear whether N is set per medium or per account, what the default and allowed range are, and whether N = 0 is allowed. If N is lowered, it's also unclear when the extra versions are removed.
10. **Verdict rules.** Each of the four verdicts needs exact conditions. Unclear cases include a full medium, an account that isn't added, an invalid config and a medium removed during the scan. The "short text to read out or forward" has no content yet, and "forward" has no mechanism (clipboard?).
11. **"Last backup N days ago."** Unclear whether this counts per medium or per user and computer, given that a medium can be shared. It's also unclear whether "Some files were skipped" counts as successful, and what is shown before the first backup.
12. **Interrupted runs.** If the computer shuts down mid-run, the "Backup not finished" verdict can only be worked out on the next start, from a start entry in the log with no end entry. That rule should be written down.
13. **Log content.** The concept lists files scanned and data used ([Concept.md](Concept.md), "Includes" 2), but the requirements leave them out. The history view needs a format it can parse. The log should store language-neutral codes so the history can be shown in either language. It also needs a safe way to trim it below 1 MB.
14. **Names that can't be stored.** The concept leaves these "to the design": characters exFAT forbids, names that aren't valid UTF-8, and length limits. The requirements should still fix the behaviour, which is to skip and report.
15. **Type and case changes.** Undefined cases:
    - A file becomes a folder with the same name, or the other way round.
    - On exFAT, a file renamed only in case (`report.pdf` → `Report.pdf`).
    - Files that change during copying, or can't be read.
    - Whether copies are read back to check them.
16. **Mixed Ext4 permissions.** The root folder of a newly formatted Ext4 medium and `lost+found` belong to root. An existing Ext4 medium may not be writable by the user at all.
17. **Formatting details.** Unclear whether Babawus writes a new partition table or only formats the partition, and what volume label it sets. Reserved blocks on Ext4 (5% by default) should be set to 0. Ejecting should be defined as unmount only, or unmount plus power-off.
18. **The media list.** How is a medium identified: label, vendor, size, or a name the poweruser gives it, which would help when rotating media? Unclear cases:
    - Sticks with several partitions.
    - Read-only media.
    - Media that aren't mounted.
    - A valid medium where this account hasn't been added.
19. **Several Babawus instances.** Two users on one computer, or two Babawus windows, could back up to the same medium at once. That probably needs a single running instance and a lock file on the medium.
20. **Language setting.** Unclear where the choice is stored and what the default is (the system language?).
21. **Gaps in scope:**
    - Renaming a computer or account appears in the requirements but not in the concept's includes or first-version criteria.
    - There's no story for removing a computer or account from a medium.
    - The concept says folders are selected; the requirements say "files and folders" ([Requirements.md](Requirements.md), "Configure USB media").

## C. Technical decisions to make at the start of design

22. **Language and UI toolkit.** For example GTK4 with Python, or Qt. This affects dependencies, tests and packaging.
23. **Packaging.** A `.deb` package or something else. Flatpak's sandbox clashes with formatting, udisks and full access to home.
24. **Formatting and eject.** Use udisks2/polkit or pkexec. Decide whether xxHash64 comes from a library or is written natively.
25. **Platforms.** Which Mint version and desktop to target, and whether to support Wayland, X11 or both.
26. **Testing exFAT and Ext4 behaviour.** Loop devices need root; fake file systems may not behave like the real ones. Pick an approach.
27. **Speed.** Scanning, the live file and byte totals in the config editor, and comparing files must not freeze the UI on a large home folder.

## D. Document housekeeping

28. **Pick one source of truth.** Concept and Requirements repeat each other, which has already caused small differences (items 2, 13 and 21).
29. **Add a glossary.** For example "run" vs "session", "Babawus-ready", "newest copy" and "older version".
30. **Small wording fix.** The concept's Includes say "allow the *user* to" format and set up media ([Concept.md](Concept.md), "Includes"); those are poweruser functions.
