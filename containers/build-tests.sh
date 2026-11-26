#!/bin/sh
# Builds the tests of babawus-core on real file systems in a distribution's container, without
# running them, and prints where each test program is on the host, one per line (T-128). The
# smoke test copies them into that distribution's VM and runs them there, since a container has
# no loop devices or udisks2.
#
#   containers/build-tests.sh fedora-45
#
# The test programs know the folder they were built in, through CARGO_TARGET_TMPDIR, as
# /src/target/containers/<name>/target/tmp, which the smoke test makes in the VM.
set -eu
. "$(dirname "$0")/common.sh"

if [ $# -ne 1 ]; then
    echo "Usage: $0 <distribution>" >&2
    exit 2
fi
name=$1
distribution_image "$name" > /dev/null
echo "==> $name: building the image" >&2
tag=$(build_image "$name")
echo "==> $name: building the tests" >&2
# Cargo's messages go to standard error, and its JSON, which names each test program, to standard
# output. Only the tests in crates/babawus-core/tests/ use real file systems.
run_in_build_image "$name" "$tag" \
    "cargo test --locked -p babawus-core --test '*' --no-run --message-format=json" |
    grep -o '"executable":"/src/[^"]*"' |
    sed "s|^\"executable\":\"/src/|$repo/|; s|\"\$||"
