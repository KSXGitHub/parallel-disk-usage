#! /bin/bash
set -o errexit -o pipefail -o nounset

cd "$(dirname "$0")"

# Rebuild a release binary inside the pinned container and print its checksum.
# Building inside the container makes the result independent of the host, so the
# checksum can be compared against a published release. The target defaults to
# the primary Linux target and may be overridden, for example with
# x86_64-unknown-linux-musl.
target="${1:-x86_64-unknown-linux-gnu}"

./reproducible-build.sh "$target"
sha256sum "target/$target/release/pdu"
