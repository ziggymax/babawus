# Babawus

**BA**sic **BA**ckup **W**ith **U**sb **S**torage

Babawus is an application designed to make it easy to maintain backups of important files on detachable usb media (usb sticks).
 
## Development

Babawus is a Cargo workspace: `crates/babawus-core` holds all logic and no UI code, and `crates/babawus` is the thin GTK 4 UI on top. See [Requirements.md](Requirements.md) for what it must do.

To build on Linux Mint 22.x, install Rust, `cargo-deb`, and `libgtk-4-dev`, then use the commands below. [DevEnvSetup.md](DevEnvSetup.md) explains how to rebuild the full development VM.

| Task | Command |
| --- | --- |
| Run the application | `cargo run -p babawus` |
| Run the unit tests | `cargo test` |
| Also run the tests on real file systems (needs passwordless `sudo`) | `cargo test -- --include-ignored` |
| Check formatting and lints | `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` |
| Build the `.deb` package | `cargo deb -p babawus` |
