#!/bin/sh
# Sets up apt in a container before anything is installed (T-133): a short timeout and retries,
# so that a stalled download is tried again soon, as the CI runner's own apt does, and, when
# UBUNTU_MIRROR is set, that mirror in place of Ubuntu's own, which on a slow day took CI past
# its 30 minutes. CI sets it to the runner's mirror. Used in the image Babawus is built in
# (apt.Containerfile) and in the clean images the .deb is installed in (package.sh).
set -eu

cat > /etc/apt/apt.conf.d/99babawus <<'EOF'
Acquire::http::Timeout "20";
Acquire::Retries "3";
EOF

if [ -n "${UBUNTU_MIRROR:-}" ]; then
    for sources in /etc/apt/sources.list.d/ubuntu.sources /etc/apt/sources.list; do
        if [ -f "$sources" ]; then
            sed -i -E "s#https?://(archive|security)\.ubuntu\.com/ubuntu/?#$UBUNTU_MIRROR#g" \
                "$sources"
        fi
    done
fi
