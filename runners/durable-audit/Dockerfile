# Use a reviewed target base, containing sh, tar, awk and sha256sum, with glibc.
ARG BASE_IMAGE=pinfold/profile-default:latest
FROM ${BASE_IMAGE}
ARG TARGETARCH
COPY runners/durable-audit/ /opt/yard/durable-audit/
# Public, integrity-verified archives supplied by the trusted image builder.
COPY build-inputs/ /build-inputs/
RUN sh /opt/yard/durable-audit/install-image.sh "$TARGETARCH"
