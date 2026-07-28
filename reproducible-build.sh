#! /bin/bash
set -o errexit -o pipefail -o nounset

cd "$(dirname "$0")"

# Build a release binary inside a pinned container image, so the result is
# identical no matter which host runs the build. The GNU target links its C
# runtime (the glibc startup objects and the GCC version string) from the build
# environment rather than from the Rust toolchain, so a bare host build varies
# between distributions. Fixing the image fixes that C runtime.
#
# The image is pinned by digest, not by tag, so the toolchain, GCC, and glibc
# can never drift. It ships rustc 1.97.1 on Debian bullseye, whose glibc 2.31
# keeps the required glibc version low for broad compatibility.
target="${1:-x86_64-unknown-linux-gnu}"
image="rust:1.97.1-slim-bullseye@sha256:dd159e8e44aaa84dffbb7f24a742a96b8d0a26920bd3b46ec2b1778be0da1cdf"

# Detect a container runtime. PDU_CONTAINER_RUNTIME overrides the choice.
runtime="${PDU_CONTAINER_RUNTIME:-}"
if [[ -z "$runtime" ]]; then
  for candidate in docker podman; do
    if command -v "$candidate" >/dev/null; then
      runtime="$candidate"
      break
    fi
  done
fi
if [[ -z "$runtime" ]]; then
  echo "error: no container runtime found; install docker or podman, or set PDU_CONTAINER_RUNTIME" >&2
  exit 1
fi
echo "reproducible-build: using $runtime with $image" >&2

mkdir -p target

# The workspace is mounted read-only so the build cannot modify the source, and
# only the target directory is writable. Proxy variables are forwarded for
# networks that require them, and they are absent on an ordinary network.
# PDU_CONTAINER_RUN_ARGS passes extra runtime flags for unusual environments,
# such as a custom CA certificate mount.
# shellcheck disable=SC2086
"$runtime" run --rm --interactive \
  --env HTTPS_PROXY --env HTTP_PROXY --env NO_PROXY \
  --env https_proxy --env http_proxy --env no_proxy \
  --volume "$PWD":/build:ro \
  --volume "$PWD/target":/build/target \
  ${PDU_CONTAINER_RUN_ARGS:-} \
  "$image" \
  bash -s -- "$target" <<'INNER'
set -o errexit -o pipefail -o nounset
target="$1"
cd /build

# The chosen target is not always the image's host target, so make sure its
# standard library is present.
rustup target add "$target" >/dev/null

# Remap the machine-independent but still absolute paths that the compiler
# embeds, exactly as an ordinary reproducible build would. Inside the pinned
# image these prefixes are already fixed, so this mainly keeps the embedded
# paths canonical and matches the non-container build environment.
commit_hash="$(rustc --version --verbose | awk '/^commit-hash:/ { print $2 }')"
rust_src="$(rustc --print sysroot)/lib/rustlib/src/rust"
export CARGO_INCREMENTAL=0
export RUSTFLAGS="--remap-path-prefix=${CARGO_HOME:-/usr/local/cargo}=/cargo --remap-path-prefix=$rust_src=/rustc/$commit_hash"

cargo build --release --all-features --target "$target"
INNER

echo "reproducible-build: built target/$target/release/pdu" >&2
