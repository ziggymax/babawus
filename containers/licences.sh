#!/bin/sh
# Writes THIRD-PARTY-LICENSES, the licences of the crates Babawus is built from, into
# target/release/ (or $CARGO_TARGET_DIR/release/), where both packages take it from (T-130,
# issue 171). package.sh runs it in the containers; run it yourself before a `cargo deb` of your
# own. It needs cargo-about (`cargo install --locked --features cli cargo-about`).
#
# crates/babawus/about.toml says which crates and licences, and about.hbs how the file looks. A
# crate under a licence not accepted there stops it. It reads only the licence files in the
# crates themselves, offline, once cargo has downloaded them.
set -eu
cd "$(dirname "$0")/.."
release="${CARGO_TARGET_DIR:-target}/release"
cargo fetch --locked
mkdir -p "$release"
cargo about generate --frozen --fail -m crates/babawus/Cargo.toml crates/babawus/about.hbs \
    -o "$release/THIRD-PARTY-LICENSES"
