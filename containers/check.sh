#!/bin/sh
# Builds Babawus and runs clippy and its unit tests in a container for each supported distribution
# (T-119), with podman. CI runs the same script, one distribution per job.
#
#   containers/check.sh                 every distribution
#   containers/check.sh debian-13 ...   only these
#   containers/check.sh --list          the names
#
# The tests on real file systems need udisks2 and loop devices, so they don't run here, only on
# the CI runner and on the Mint VM. Build output and downloaded crates are kept per distribution in
# target/containers/, so that a second run only builds what changed. The distributions are listed
# in common.sh.
set -eu
. "$(dirname "$0")/common.sh"

if [ "${1:-}" = "--list" ]; then
    distribution_names
    exit 0
fi

wanted="$*"
if [ -z "$wanted" ]; then
    wanted=$(distribution_names)
fi

failed=""
for name in $wanted; do
    image=$(distribution_image "$name")
    echo "==> $name: building the image from $image"
    tag=$(build_image "$name")

    echo "==> $name: clippy and unit tests"
    if run_in_build_image "$name" "$tag" \
        'cargo --version && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked'
    then
        echo "==> $name: passed"
    else
        echo "==> $name: FAILED"
        failed="$failed $name"
    fi
done

if [ -n "$failed" ]; then
    echo "Failed:$failed" >&2
    exit 1
fi
echo "All passed: $(echo $wanted)"
