#! /bin/bash
set -o errexit -o pipefail -o nounset

# This script runs inside the container started by reproducible-build.sh, where
# the workspace is mounted at /build. It is not meant to be run on the host.
target="$1"
cd /build

# The chosen target is not always the image's host target.
rustup target add "$target" >/dev/null

# Keep the paths that the compiler embeds canonical, matching the remapping that
# a build outside the container performs.
commit_hash="$(rustc --version --verbose | awk '/^commit-hash:/ { print $2 }')"
rust_src="$(rustc --print sysroot)/lib/rustlib/src/rust"
export CARGO_INCREMENTAL=0
export RUSTFLAGS="--remap-path-prefix=${CARGO_HOME:-/usr/local/cargo}=/cargo --remap-path-prefix=$rust_src=/rustc/$commit_hash"

cargo build --release --all-features --target "$target"
