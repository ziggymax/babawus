# Shared by check.sh and package.sh: the supported distributions, and how their containers are
# built and run. Sourced, not run.

# Name, Containerfile and image of each distribution. The tags are pinned (issue 169); a new
# Ubuntu or Fedora is taken up in a task of its own. Debian is no longer supported (T-133). .github/workflows/ci.yml lists the same names.
DISTRIBUTIONS="
ubuntu-24.04 apt docker.io/library/ubuntu:24.04
ubuntu-26.04 apt docker.io/library/ubuntu:26.04
fedora-45 dnf docker.io/library/fedora:45
"

here=$(cd "$(dirname "$0")" && pwd)
repo=$(dirname "$here")

# The names of every distribution, one per line.
distribution_names() {
    echo "$DISTRIBUTIONS" | awk 'NF { print $1 }'
}

# The distribution's image, as the distribution ships it, without anything of Babawus's.
distribution_image() {
    line=$(echo "$DISTRIBUTIONS" | awk -v n="$1" '$1 == n')
    if [ -z "$line" ]; then
        echo "Unknown distribution: $1 (see --list)" >&2
        exit 2
    fi
    echo "$line" | awk '{ print $3 }'
}

# Builds the image that Babawus is built in, for the distribution, and prints its tag.
build_image() {
    image=$(distribution_image "$1")
    kind=$(echo "$DISTRIBUTIONS" | awk -v n="$1" '$1 == n { print $2 }')
    tag="localhost/babawus-check:$1"
    podman build --quiet --pull=newer --build-arg IMAGE="$image" \
        --build-arg UBUNTU_MIRROR="${UBUNTU_MIRROR:-}" \
        -f "$here/$kind.Containerfile" -t "$tag" "$here" > /dev/null
    echo "$tag"
}

# Runs a shell command in the distribution's build image, in the repository, with its build
# output and downloaded crates kept in target/containers/<name>/.
#
# No debug information, which nobody uses in a container, and in CI no incremental builds, which
# only pay off on a second build of the same code: both keep target/containers/ small enough for
# GitHub's cache. Labels are turned off rather than relabelling the repository, which matters on
# a host with SELinux, such as Fedora.
run_in_build_image() {
    name=$1
    tag=$2
    command=$3
    build_out="$repo/target/containers/$name"
    mkdir -p "$build_out/target" "$build_out/registry"
    incremental=true
    if [ -n "${CI:-}" ]; then
        incremental=false
    fi
    podman run --rm --security-opt label=disable \
        -v "$repo:/src" -w /src \
        -v "$build_out/registry:/usr/local/cargo/registry" \
        -e CARGO_TARGET_DIR="/src/target/containers/$name/target" \
        -e CARGO_PROFILE_DEV_DEBUG=false -e CARGO_PROFILE_TEST_DEBUG=false \
        -e CARGO_INCREMENTAL="$incremental" \
        "$tag" sh -c "$command"
}
