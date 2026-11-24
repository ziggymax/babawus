#!/bin/sh
# Builds Babawus's packages in the containers, and checks that each one installs, with all its
# dependencies, in a clean image of each distribution it is for, as a user would install it, and
# that every library /usr/bin/babawus needs is found (T-121).
#
#   containers/package.sh          every package
#   containers/package.sh rpm      only this one
#
# The packages are left in target/containers/packages/.
set -eu
. "$(dirname "$0")/common.sh"

# Each package: its format, the distribution it is built on, and those it is installed on.
PACKAGES="
rpm fedora-45 fedora-45
"

out="$repo/target/containers/packages"

# Builds the package of the given format on the distribution, and leaves it in $out.
build_package() {
    format=$1
    name=$2
    echo "==> $format: building the image for $name"
    tag=$(build_image "$name")
    echo "==> $format: building the package on $name"
    rm -f "$out"/*."$format"
    mkdir -p "$out"
    case $format in
        rpm)
            run_in_build_image "$name" "$tag" '
                cargo build --locked --release -p babawus \
                && cargo generate-rpm -p crates/babawus --auto-req find-requires \
                    --target-dir "$CARGO_TARGET_DIR" -o /src/target/containers/packages/'
            ;;
        *)
            echo "Unknown package format: $format" >&2
            exit 2
            ;;
    esac
}

# Installs the package in a clean image of the distribution, and checks it there.
install_package() {
    file=$1
    name=$2
    image=$(distribution_image "$name")
    case $file in
        *.rpm) install='dnf install -y -q /packages/*.rpm' query='rpm -ql babawus' ;;
        *) echo "Unknown package: $file" >&2; exit 2 ;;
    esac
    podman pull --quiet --policy=newer "$image" > /dev/null
    podman run --rm --security-opt label=disable \
        -v "$(dirname "$file"):/packages:ro" "$image" sh -c "
            set -e
            $install
            echo '--- Installed files:'
            $query
            echo '--- Libraries:'
            if ldd /usr/bin/babawus | grep 'not found'; then
                echo 'A library of /usr/bin/babawus is missing'
                exit 1
            fi
            echo 'All found'
            ls /usr/share/babawus/languages/*.toml > /dev/null
        "
}

wanted="$*"
if [ -z "$wanted" ]; then
    wanted=$(echo "$PACKAGES" | awk 'NF { print $1 }')
fi

failed=""
for format in $wanted; do
    line=$(echo "$PACKAGES" | awk -v f="$format" '$1 == f')
    if [ -z "$line" ]; then
        echo "Unknown package format: $format" >&2
        exit 2
    fi
    set -- $line
    built_on=$2
    shift 2
    if ! build_package "$format" "$built_on"; then
        echo "==> $format: FAILED to build"
        failed="$failed $format"
        continue
    fi
    file=$(ls "$out"/*."$format")
    for name in "$@"; do
        echo "==> $format: installing on $name"
        if install_package "$file" "$name"; then
            echo "==> $format on $name: passed"
        else
            echo "==> $format on $name: FAILED"
            failed="$failed $format-on-$name"
        fi
    done
done

if [ -n "$failed" ]; then
    echo "Failed:$failed" >&2
    exit 1
fi
echo "All passed: $(echo $wanted)"
