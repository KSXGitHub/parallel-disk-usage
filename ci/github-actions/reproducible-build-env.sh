#! /bin/bash
set -o errexit -o pipefail -o nounset

# Emit the environment variables that make the release binary reproducible into
# "$GITHUB_ENV", so that every later step in the job inherits them. The goal is
# for the compiled artifact to depend only on the source code and the pinned
# toolchain, never on the host that performs the build.
#
# None of these variables reduce parallelism. Parallel code generation is
# already deterministic on a fixed toolchain, so the build stays multi-threaded.

cargo_home="${CARGO_HOME:-$HOME/.cargo}"
workspace="${GITHUB_WORKSPACE:-$PWD}"

# Replace the machine-specific absolute prefixes that the compiler would
# otherwise embed into the binary. The dependency sources live under the cargo
# home, and the crate itself is built from the workspace directory. Both paths
# differ from one machine to the next. This is the stable equivalent of the
# still-unstable "-Z trim-paths" option, and it does not affect codegen.
remap="--remap-path-prefix=$cargo_home=/cargo"
remap="$remap --remap-path-prefix=$workspace=/build"
if [[ -n "${RUSTFLAGS:-}" ]]; then
  remap="$RUSTFLAGS $remap"
fi

{
  echo "RUSTFLAGS=$remap"
  # Incremental compilation caches carry host-specific state and are already
  # disabled for the release profile. Pin the value so a stray environment
  # setting cannot turn it back on.
  echo "CARGO_INCREMENTAL=0"
  # Provide a fixed timestamp for any build script that would otherwise read the
  # wall clock. The current sources embed no timestamp, so this is a safeguard
  # rather than a fix for a present problem.
  echo "SOURCE_DATE_EPOCH=$(git log -1 --pretty=%ct)"
} >>"$GITHUB_ENV"
