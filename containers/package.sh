#!/bin/sh
# Builds Babawus's packages in the containers, and checks that each one installs, with all its
# dependencies, in a clean image of each distribution it is for, as a user would install it, that
# every library /usr/bin/babawus needs is found (T-121, T-122), and that its AppStream metainfo
# is valid (issue 175).
#
#   containers/package.sh          every package
#   containers/package.sh deb      only this one
#
# The packages are left in target/containers/packages/.
set -eu
. "$(dirname "$0")/common.sh"

# Each package: its format, the distribution it is built on, and those it is installed on. The
# .deb is built on the oldest, so that it works on the newer ones too (issue 169).
PACKAGES="
deb ubuntu-24.04 ubuntu-24.04 ubuntu-26.04
rpm fedora-45 fedora-45
"

out="$repo/target/containers/packages"

# Run first in the container: the tools the packages are built with are kept with the build
# output, which CI caches, rather than in the image, which would then have to build them for
# every job that uses it.
TOOLS='
    set -e
    tools="$CARGO_TARGET_DIR/../tools"
    export PATH="$tools/bin:$PATH"
    install_tool() {
        [ -x "$tools/bin/$1" ] || cargo install --locked --root "$tools" "$@"
    }
'

# THIRD-PARTY-LICENSES, which both packages install (T-130).
LICENCES='
    install_tool cargo-about --features cli
    containers/licences.sh
'

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
        deb)
            run_in_build_image "$name" "$tag" "$TOOLS
                install_tool cargo-deb
                $LICENCES
                cargo deb --locked -p babawus -o /src/target/containers/packages/"
            ;;
        rpm)
            # An rpm version can't have a "-", so a pre-release such as 1.2.0-alpha.1 becomes
            # 1.2.0~alpha.1, which sorts before 1.2.0, as Fedora numbers them and cargo-deb does
            # for the .deb on its own (T-123).
            version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repo/Cargo.toml" | sed 's/-/~/')
            run_in_build_image "$name" "$tag" "$TOOLS
                $LICENCES
                cargo build --locked --release -p babawus
                cargo generate-rpm -p crates/babawus --auto-req find-requires \\
                    -s 'version = \"$version\"' \\
                    --target-dir \"\$CARGO_TARGET_DIR\" -o /src/target/containers/packages/"
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
        *.deb)
            install='/setup/apt-setup.sh && apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq /packages/*.deb > /dev/null'
            validator='DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends appstream > /dev/null'
            query='dpkg -L babawus | while read -r f; do [ -d "$f" ] || echo "$f"; done'
            ;;
        *.rpm)
            install='dnf install -y -q /packages/*.rpm'
            validator='dnf install -y -q --setopt=install_weak_deps=False appstream'
            query='rpm -ql babawus'
            ;;
        *) echo "Unknown package: $file" >&2; exit 2 ;;
    esac
    # Without --policy, which the CI runner's older podman doesn't have (T-133).
    podman pull --quiet "$image" > /dev/null
    podman run --rm --security-opt label=disable \
        -v "$(dirname "$file"):/packages:ro" -v "$here:/setup:ro" \
        -e UBUNTU_MIRROR="${UBUNTU_MIRROR:-}" "$image" sh -c "
            set -e
            $install
            echo '--- Installed files:'
            $query | tee /tmp/files
            grep -q /THIRD-PARTY-LICENSES /tmp/files || { echo 'THIRD-PARTY-LICENSES is missing'; exit 1; }
            echo '--- Libraries:'
            if ldd /usr/bin/babawus | grep 'not found'; then
                echo 'A library of /usr/bin/babawus is missing'
                exit 1
            fi
            echo 'All found'
            ls /usr/share/babawus/languages/*.toml > /dev/null
            echo '--- AppStream:'
            $validator
            appstreamcli validate --no-net /usr/share/metainfo/io.github.ziggymax.Babawus.metainfo.xml
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
