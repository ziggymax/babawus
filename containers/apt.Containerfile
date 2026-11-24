# Builds Babawus and runs its unit tests on a distribution with apt: Ubuntu or Debian (T-119).
# The image is given when building, for example --build-arg IMAGE=docker.io/library/debian:13.
# check.sh lists the distributions and their images.
ARG IMAGE
FROM ${IMAGE}

# What building Babawus needs, and what its package depends on (crates/babawus/Cargo.toml),
# without udisks2, which only the tests on real file systems use, and they don't run here.
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential pkg-config libgtk-4-dev curl ca-certificates \
        exfatprogs e2fsprogs acl \
    && rm -rf /var/lib/apt/lists/*

COPY install-rust.sh /usr/local/bin/
RUN install-rust.sh
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
