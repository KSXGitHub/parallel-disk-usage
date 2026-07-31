#! /bin/bash
set -o errexit -o pipefail -o nounset

# Emit the environment variables that make the release binary reproducible into
# "$GITHUB_ENV", so that every later step in the job inherits them.

cargo_home="${CARGO_HOME:-$HOME/.cargo}"
workspace="${GITHUB_WORKSPACE:-$PWD}"

# Replace the machine-specific absolute prefixes that the compiler would
# otherwise embed into the binary. This is the stable equivalent of the
# still-unstable "-Z trim-paths" option.
remap="--remap-path-prefix=$cargo_home=/cargo"
remap="$remap --remap-path-prefix=$workspace=/build"

# When the "rust-src" component is installed, the compiler rewrites the standard
# library's "/rustc/<commit-hash>/..." paths to the local toolchain directory.
# Remap them back, so the result does not depend on whether it is installed.
commit_hash="$(rustc --version --verbose | awk '/^commit-hash:/ { print $2 }')"
rust_src="$(rustc --print sysroot)/lib/rustlib/src/rust"
if [[ -n "$commit_hash" ]]; then
  remap="$remap --remap-path-prefix=$rust_src=/rustc/$commit_hash"
fi

if [[ -n "${RUSTFLAGS:-}" ]]; then
  remap="$RUSTFLAGS $remap"
fi

{
  echo "RUSTFLAGS=$remap"
  echo "CARGO_INCREMENTAL=0"
  # A fixed timestamp for any build script that would otherwise read the clock.
  echo "SOURCE_DATE_EPOCH=$(git log -1 --pretty=%ct)"
} >>"$GITHUB_ENV"
