#! /bin/bash
set -o errexit -o pipefail -o nounset

cd "$(dirname "$0")"

# Build a release binary inside a pinned container image. The GNU target takes
# its C runtime from the build environment rather than from the Rust toolchain,
# so pinning the image is what makes the output identical on every host.
#
# The image ships rustc 1.97.1 on Debian bullseye, whose glibc 2.31 keeps the
# required glibc version low for broad compatibility.
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

# The workspace is mounted read-only, so only the target directory is writable.
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
  bash /build/ci/build-in-container.sh "$target"

echo "reproducible-build: built target/$target/release/pdu" >&2
