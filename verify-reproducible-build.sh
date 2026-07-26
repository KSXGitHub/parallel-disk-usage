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

# Remap the machine-specific absolute prefixes that the compiler would otherwise
# embed into the binary: the dependency sources under the cargo home, the crate
# built from this directory, and the standard library sources under the toolchain
# sysroot. The last one matters when the "rust-src" component is installed,
# because the compiler then rewrites the standard library's "/rustc/<commit-hash>"
# paths to the local toolchain directory.
commit_hash="$(rustc --version --verbose | awk '/^commit-hash:/ { print $2 }')"
rust_src="$(rustc --print sysroot)/lib/rustlib/src/rust"
export CARGO_INCREMENTAL=0
export RUSTFLAGS="--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=$PWD=/build"
if [[ -n "$commit_hash" ]]; then
  export RUSTFLAGS="$RUSTFLAGS --remap-path-prefix=$rust_src=/rustc/$commit_hash"
fi

cargo build --release --all-features --target "$target"
sha256sum "target/$target/release/pdu"
