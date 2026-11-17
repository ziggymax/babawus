# Planning the next release

The purpose of this document is to have a place to capture ideas for the next release of Babawus and, in doing so, help in creating and maintaining a general plan for the future work on Babawus.

Some releases may represent so much work and/or complexity that a step-by-step sequence of interim non-released versions is needed to reach the stage where a new release can be made. These non-released versions are an important part of the release planning and hence also something that will be described in this document.

For the remainder of this document the word "version" is used for something that is not planned to be released, and the word "release" is used only for something that is intended for release.

Interim versions are numbered as semver pre-releases of the release they lead to, for example 1.0.0-alpha.1, 1.0.0-alpha.2, and so on. Each interim version is tagged on main, but gets no GitHub release.

When work on a version begins, its items become tasks (T-entries) in the task list.

## Latest release of Babawus

The latest/current release is: 1.0.0

The latest interim version is: 1.1.0-alpha.3 (tag `v1.1.0-alpha.3`)

## Next release of Babawus

This section always describes the next upcoming version/release of Babawus. Some of the ideas may be very specific, and some more speculative in nature. Task(s) created to deal with speculative ideas will be investigative rather than implementation-oriented.

### Administrative matters or other non-feature tasks

None for the next release.

### New features

These features are not listed by priority.

1. *Babawus fly-by text*. If the mouse cursor rests shortly on the Babawus logo on the main screen a fly-by text appears: "Babawus never forgets" (adjusted for each supported language).
1. *Single file selection add/skip in "Choose what to backup for user"*. The Advanced folder/file view where the user can select which folders to backup is extended with single file add/skip. Clicking a single file allows the user to add or skip a file. The specifics need to be decided (what happens if the file is already implicitly added via a selected folder etc.). How to handle a file marked to be skipped also needs to be decided: Add a specific pattern to "Files to leave out by name" (kind of the obvious choice) or add a list of specific individual files to skip.
1. *Restore file(s) window*. A user should be able to open a window showing files/folders (shown in much the same way that the what-backup pane does) and select one or more specific files to restore. When a file has earlier versions stored, the user should (also) be able to see and pick specific versions. The specifics need to be decided. The use of a separate window (which can be resized individually) makes it easier for the user to get an overview, plus it provides more screen space for a good combined folder/file view combined with extra info about versions for files.

### Versions needed for this release

Three versions will be made for the next release:

1. 1.1.0-alpha.1: Babawus logo fly-by text. Done in T-089 to T-090.
1. 1.1.0-alpha.2: Single file add/skip in "What to backup". Done in T-091 to T-093.
1. 1.1.0-alpha.3: Restore file(s) window. Done in T-094 to T-096.

## Future releases

This section captures ideas for some future, unspecified releases of Babawus. As time and versions progress some of these ideas will migrate to the "Next release of Babawus" section, some will be deleted and some may live here for a long time.

To the extent possible the new features are listed by priority, from highest to lowest.

1. *A runnable release script(?)*. Investigate the possibility of automating releases to the maximum by maintaining the release procedure as some kind of runnable script. Anything that can be done without human intervention should be done automatically, where human intervention is needed the script should prompt the person responsible for doing the release with step-by-step actions to perform (and, where the person must report some result back, provide simple fillable forms).
1. *Prepare for other platforms*. Future versions of Babawus should be able to work with the platforms listed below. For the next release the task is to investigate the need for code restructuring/refactoring to accommodate this overarching goal.
Any actual refactoring will be done in a later version - it will probably be a good idea to create some non-released interim versions to achieve these goals.
    - Ubuntu and other .deb using platforms (this is a short term goal). What differs from Mint is not known yet; finding out is part of the investigation.
    - Windows (a longer term goal)
