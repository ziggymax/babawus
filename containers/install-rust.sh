#!/bin/sh
# Installs Rust with rustup, stable, as on the host and in CI (issue 169), for every user of the
# container. Run by the Containerfiles.
set -eu
export RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --profile minimal --component clippy
