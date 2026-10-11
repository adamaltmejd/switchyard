#!/bin/sh
# Fetch public archives as data. Installation and package code stay in the image.
set -eu
package_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
inputs=$1
case "$(uname -m)" in
    x86_64) arch=x64 ;;
    aarch64|arm64) arch=arm64 ;;
    *) echo 'unsupported build input architecture' >&2; exit 1 ;;
esac
mkdir "$inputs"
mkdir "$inputs/packages"
archive="node-v26.11.1-linux-$arch.tar.gz"
expected=$(awk -v file="$archive" '$2 == file {print $1}' "$package_dir/node-SHASUMS256.txt")
test -n "$expected"
curl --proto '=https' --tlsv1.2 -fsSL -o "$inputs/$archive" "https://nodejs.org/dist/v26.11.1/$archive"
printf '%s  %s\n' "$expected" "$inputs/$archive" | sha256sum --strict -c -
jq -er '.packages | to_entries[] | select(.key != "") | .value |
    if (.resolved | startswith("https://registry.npmjs.org/")) and
       (.integrity | test("^sha512-[A-Za-z0-9+/]{86}==$"))
    then [.resolved, .integrity] | @tsv
    else error("locked archive lacks registry URL or exact SHA512") end' \
    "$package_dir/package-lock.json" > "$inputs/archives.tsv"
index=0
while read -r url integrity; do
    index=$((index + 1))
    file="$inputs/packages/$index.tgz"
    curl --proto '=https' --tlsv1.2 -fsSL -o "$file" "$url"
    actual="sha512-$(openssl dgst -sha512 -binary "$file" | openssl base64 -A)"
    if [ "$actual" != "$integrity" ]; then
        echo "locked archive integrity mismatch: $url" >&2
        exit 1
    fi
done < "$inputs/archives.tsv"
rm "$inputs/archives.tsv"
