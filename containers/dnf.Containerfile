# Builds Babawus and runs its unit tests on a distribution with dnf: Fedora (T-119).
# The image is given when building, for example --build-arg IMAGE=docker.io/library/fedora:45.
# common.sh lists the distributions and their images.
ARG IMAGE
FROM ${IMAGE}

# What building Babawus needs, and what its package depends on, with Fedora's package names,
# without udisks2, which only the tests on real file systems use, and they don't run here.
RUN dnf install -y --setopt=install_weak_deps=False \
        gcc pkgconf-pkg-config gtk4-devel curl \
        exfatprogs e2fsprogs acl \
    && dnf clean all

# Fedora's image has an empty /etc/machine-id, which a real Fedora never has, and Babawus tells
# computers apart by it. On Debian and Ubuntu, installing GTK brings dbus, which writes one.
RUN [ -s /etc/machine-id ] || tr -d - < /proc/sys/kernel/random/uuid > /etc/machine-id

COPY install-rust.sh /usr/local/bin/
RUN install-rust.sh
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH

# Builds the .rpm (T-121). rpm-build brings Fedora's own find-requires, which lists only the
# libraries Babawus links to itself, as Fedora's packages do, not every library they need in turn.
RUN dnf install -y --setopt=install_weak_deps=False rpm-build && dnf clean all
RUN cargo install --locked cargo-generate-rpm && rm -rf /usr/local/cargo/registry
