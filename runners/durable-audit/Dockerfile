# Use a reviewed Debian target base with sh, tar, awk, sha256sum and glibc.
ARG BASE_IMAGE=pinfold/profile-default:latest
FROM ${BASE_IMAGE}
ARG TARGETARCH
# Official Node 26 needs libatomic.so.1 in addition to the base's C/C++ runtime.
RUN apt-get update \
    && apt-get install -y --no-install-recommends libatomic1 \
    && rm -rf /var/lib/apt/lists/*
COPY runners/durable-audit/ /opt/yard/durable-audit/
# Public, integrity-verified archives supplied by the trusted image builder.
COPY build-inputs/ /build-inputs/
RUN sh /opt/yard/durable-audit/install-image.sh "$TARGETARCH"
