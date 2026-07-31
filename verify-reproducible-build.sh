#! /bin/bash
set -o errexit -o pipefail -o nounset

cd "$(dirname "$0")"

# Rebuild a release binary and print its checksum for comparison against a
# published release.
target="${1:-x86_64-unknown-linux-gnu}"

./reproducible-build.sh "$target"
sha256sum "target/$target/release/pdu"
