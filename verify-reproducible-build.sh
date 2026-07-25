#! /bin/bash
set -o errexit -o pipefail -o nounset

cd "$(dirname "$0")"

# Rebuild a release binary with the environment that makes it reproducible, then
# print its checksum. Compare the checksum against the matching published release
# to confirm that the release follows from this source and this toolchain.
#
# The build mirrors the deployment workflow, including the explicit target, so
# the result matches the released artifact for that target. The target defaults
# to the primary Linux target and may be overridden, for example with
# x86_64-unknown-linux-musl.
target="${1:-x86_64-unknown-linux-gnu}"

export CARGO_INCREMENTAL=0
export RUSTFLAGS="--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=$PWD=/build"

cargo build --release --all-features --target "$target"
sha256sum "target/$target/release/pdu"
