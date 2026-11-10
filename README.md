# Babawus

**BA**sic **BA**ckup **W**ith **U**sb **S**torage

Babawus is an application designed to make it easy to maintain backups of important files on detachable usb media (usb sticks).
 
## Development

Babawus is a Cargo workspace: `crates/babawus-core` holds all logic and no UI code, and `crates/babawus` is the thin GTK 4 UI on top. See [Requirements.md](Requirements.md) for what it must do. [Tasks.md](Tasks.md) lists all outstanding work, so add anything that is left for later there.

To build on Linux Mint 22.x, install Rust, `cargo-deb`, and `libgtk-4-dev`, then use the commands below. [DevEnvSetup.md](DevEnvSetup.md) explains how to rebuild the full development VM.

| Task | Command |
| --- | --- |
| Run the application | `cargo run -p babawus` |
| Run the unit tests | `cargo test` |
| Also run the tests on real file systems (needs passwordless `sudo`) | `cargo test -- --include-ignored` |
| List the USB media as Babawus sees them, and follow changes | `cargo run -p babawus-core --example media -- --follow` |
| List the drives Babawus may format, or format one (this erases it) | `cargo run -p babawus-core --example format` |
| List the USB media, or eject one | `cargo run -p babawus-core --example eject` |
| Add this computer and account to a mounted Babawus medium, with what to back up, without the configuration editor | `cargo run -p babawus-core --example add_account -- <medium root> Documents` |
| The manual checklist to run on real sticks before each release | [ReleaseChecklist.md](ReleaseChecklist.md) |
| Check formatting and lints | `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` |
| Build the `.deb` package | `cargo deb -p babawus` |

## Finding out what went wrong

Each stick keeps a log of every backup made to it, in `.babawus/babawus_log.txt` on the stick. It is plain text, one entry per line: when each run started, for whom, on which computer and with which version of Babawus, every file it had to skip and why, and how it ended. Open it with any text editor; the folder `.babawus` is hidden, so show hidden files in the file manager first. The format is described under "Format of the log" in [Requirements.md](Requirements.md).

## Installing and uninstalling

Install Babawus by double-clicking the `.deb` package and choosing "Install Package". Install a newer version the same way: it replaces the old one, and keeps the chosen language and everything on the sticks.

To uninstall Babawus, run `sudo apt remove babawus`, or remove it in the Software Manager. This removes the program and its "Backup" entry in the menu, and nothing else:

- **Nothing on any stick is changed.** Backups, their history and the stick's configuration stay as they are, and a later install of Babawus finds them again.
- **The settings file stays behind.** Babawus keeps the chosen language in `~/.config/babawus/settings.toml`, in the home folder of each user who chose one. Uninstalling can't reach into home folders, so delete that file by hand if it is no longer wanted: `rm -r ~/.config/babawus`.
