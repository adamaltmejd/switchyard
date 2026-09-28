# The toolchain rust-toolchain.toml pins, with its components. Boxes set
# their own HOME and PATH, so the toolchain is copied out of rustup and linked
# onto the default PATH; rust-toolchain.toml is not read inside a box.
FROM docker.io/library/rust:1.98.1-slim-trixie AS rust
RUN rustup component add rustfmt clippy && cp -a "$(rustc --print sysroot)" /toolchain

FROM pinfold/profile-default:latest
RUN apt-get update \
    && apt-get install -y --no-install-recommends gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust /toolchain /opt/rust
RUN ln -s /opt/rust/bin/* /usr/local/bin/ && cargo --version && cargo clippy --version
