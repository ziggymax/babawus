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
# target/containers/, so that a second run only builds what changed.
set -eu

# Name, Containerfile and image of each distribution. The tags are pinned (issue 169); a new
# Debian or Fedora is taken up in a task of its own. .github/workflows/ci.yml lists the same names.
DISTRIBUTIONS="
ubuntu-24.04 apt docker.io/library/ubuntu:24.04
ubuntu-26.04 apt docker.io/library/ubuntu:26.04
debian-13 apt docker.io/library/debian:13
fedora-45 dnf docker.io/library/fedora:45
"

here=$(cd "$(dirname "$0")" && pwd)
repo=$(dirname "$here")

if [ "${1:-}" = "--list" ]; then
    echo "$DISTRIBUTIONS" | awk 'NF { print $1 }'
    exit 0
fi

wanted="$*"
if [ -z "$wanted" ]; then
    wanted=$(echo "$DISTRIBUTIONS" | awk 'NF { print $1 }')
fi

failed=""
for name in $wanted; do
    line=$(echo "$DISTRIBUTIONS" | awk -v n="$name" '$1 == n')
    if [ -z "$line" ]; then
        echo "Unknown distribution: $name (see --list)" >&2
        exit 2
    fi
    set -- $line
    kind=$2
    image=$3
    tag="localhost/babawus-check:$name"

    echo "==> $name: building the image from $image"
    podman build --quiet --pull=newer --build-arg IMAGE="$image" \
        -f "$here/$kind.Containerfile" -t "$tag" "$here" > /dev/null

    out="$repo/target/containers/$name"
    mkdir -p "$out/target" "$out/registry"

    # No debug information, which nobody uses in a container, and in CI no incremental builds,
    # which only pay off on a second build of the same code: both keep target/containers/ small
    # enough for GitHub's cache.
    incremental=true
    if [ -n "${CI:-}" ]; then
        incremental=false
    fi

    # Labels are turned off rather than relabelling the repository, which matters on a host with
    # SELinux, such as Fedora.
    echo "==> $name: clippy and unit tests"
    if podman run --rm --security-opt label=disable \
        -v "$repo:/src" -w /src \
        -v "$out/registry:/usr/local/cargo/registry" \
        -e CARGO_TARGET_DIR="/src/target/containers/$name/target" \
        -e CARGO_PROFILE_DEV_DEBUG=false -e CARGO_PROFILE_TEST_DEBUG=false \
        -e CARGO_INCREMENTAL="$incremental" \
        "$tag" \
        sh -c 'cargo --version && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked'
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
