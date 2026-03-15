#!/usr/bin/env bash
set -euo pipefail

mkdir -p "$HOME/.local/bin"

bash "$(dirname "$0")/install-rust-toolchain.sh"
bash "$(dirname "$0")/install-hyperfine.sh"

arch="$(uname -m)"

echo "Installing shellcheck from GitHub release..." >&2
shellcheck_version="0.11.0"
case "$arch" in
  x86_64)  shellcheck_arch="linux.x86_64" ;;
  aarch64) shellcheck_arch="linux.aarch64" ;;
  *)
    echo "ERROR: Unsupported architecture for shellcheck prebuilt binary: $arch" >&2
    exit 1
    ;;
esac
shellcheck_archive="shellcheck-v${shellcheck_version}.${shellcheck_arch}"
shellcheck_url="https://github.com/koalaman/shellcheck/releases/download/v${shellcheck_version}/${shellcheck_archive}.tar.gz"
curl -fsSL "$shellcheck_url" | tar -xz --strip-components=1 -C "$HOME/.local/bin" "shellcheck-v${shellcheck_version}/shellcheck"

echo "Installing shfmt from GitHub release..." >&2
shfmt_version="3.13.0"
case "$arch" in
  x86_64)  shfmt_arch="linux_amd64" ;;
  aarch64) shfmt_arch="linux_arm64" ;;
  *)
    echo "ERROR: Unsupported architecture for shfmt prebuilt binary: $arch" >&2
    exit 1
    ;;
esac
shfmt_url="https://github.com/mvdan/sh/releases/download/v${shfmt_version}/shfmt_v${shfmt_version}_${shfmt_arch}"
curl -fsSL "$shfmt_url" -o "$HOME/.local/bin/shfmt"
chmod +x "$HOME/.local/bin/shfmt"
