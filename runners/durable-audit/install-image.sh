#!/bin/sh
# Run only during an isolated trusted image build, never on the host.
set -eu
cd /opt/yard/durable-audit
case "$1" in
    amd64|x86_64) arch=x64 ;;
    arm64|aarch64) arch=arm64 ;;
    *) echo 'unsupported image architecture' >&2; exit 1 ;;
esac
archive="node-v26.11.1-linux-$arch.tar.gz"
expected=$(awk -v file="$archive" '$2 == file {print $1}' node-SHASUMS256.txt)
test -n "$expected"
printf '%s  %s\n' "$expected" "/build-inputs/$archive" | sha256sum --strict -c -
mkdir node
tar -xzf "/build-inputs/$archive" -C node --strip-components=1
export PATH="/opt/yard/durable-audit/node/bin:/usr/bin:/bin"
unset NODE_OPTIONS NODE_PATH LD_PRELOAD LD_LIBRARY_PATH
test "$(node --version)" = v26.11.1
for file in /build-inputs/packages/*.tgz; do
    npm cache add --offline --ignore-scripts --cache /tmp/audit-npm-cache "$file"
done
npm ci --offline --ignore-scripts --no-audit --no-fund --cache /tmp/audit-npm-cache
# npm's executable links are unnecessary; no executable package is selected.
rm -rf node_modules/.bin
node --no-addons seal.mjs
node --no-addons --check runner.mjs
rm -rf /tmp/audit-npm-cache /build-inputs
