#!/bin/bash
set -euo pipefail
yard_packet=$PWD/validation-packet
yard_build=$(mktemp -d "$RUNNER_TEMP/yard-apple-build.XXXXXX")
mkdir -p "$GITHUB_WORKSPACE/shim-dist"
curl -fsSL https://go.dev/dl/go1.25.12.linux-amd64.tar.gz -o "$yard_build/go.tar.gz"
printf '234828b7a89e0e303d2556310ee549fbcf253d28de937bac3da13d6294262ac1  %s\n' "$yard_build/go.tar.gz" | sha256sum -c -
tar -xzf "$yard_build/go.tar.gz" -C "$yard_build"
git clone --no-checkout https://github.com/apple/container-builder-shim.git "$yard_build/source"
cd "$yard_build/source"
git checkout --detach e18d2182fd060dbf1c68113a74e7564d563dde27
test -z "$(git status --porcelain)"
printf '77f48c21db678529c676726d4fdad46686b370e72196255525ae2b054fae75e1  %s\n' "$yard_packet/proposed-fix.patch" | sha256sum -c -
git apply --check "$yard_packet/proposed-fix.patch"
git apply "$yard_packet/proposed-fix.patch"
git diff --check
git diff --binary > "$GITHUB_WORKSPACE/shim-dist/source.diff"
export GOTOOLCHAIN=local GOOS=linux GOARCH=arm64 CGO_ENABLED=0
"$yard_build/go/bin/go" build -mod=vendor -trimpath -buildvcs=false \
  -ldflags "-w -s -extldflags '-static' -X main.VERSION=y80-transfer-private-e18d218" \
  -tags 'osusergo netgo static_build seccomp' \
  -o "$GITHUB_WORKSPACE/shim-dist/container-builder-shim" .
"$yard_build/go/bin/go" version -m "$GITHUB_WORKSPACE/shim-dist/container-builder-shim" > "$GITHUB_WORKSPACE/shim-dist/go-build-info.txt"
cp "$yard_packet/proposed-fix.patch" "$GITHUB_WORKSPACE/shim-dist/"
cd "$GITHUB_WORKSPACE/shim-dist"
sha256sum container-builder-shim source.diff proposed-fix.patch go-build-info.txt > SHA256SUMS
python3 - <<'PY'
from pathlib import Path
import hashlib,json
Path('manifest.json').write_text(json.dumps({
 'base':'e18d2182fd060dbf1c68113a74e7564d563dde27',
 'patch_sha256':hashlib.sha256(Path('proposed-fix.patch').read_bytes()).hexdigest(),
 'binary_sha256':hashlib.sha256(Path('container-builder-shim').read_bytes()).hexdigest(),
 'toolchain':'go1.25.12 linux/amd64','target':'linux/arm64 CGO_ENABLED=0',
 'flags':"-mod=vendor -trimpath -buildvcs=false -ldflags [-w -s -extldflags '-static' -X main.VERSION=y80-transfer-private-e18d218] -tags [osusergo netgo static_build seccomp]"
},indent=2)+'\n')
PY
