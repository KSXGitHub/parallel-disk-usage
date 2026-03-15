#!/usr/bin/env bash
# Shared helper: install hyperfine binary from GitHub releases into ~/.local/bin
set -euo pipefail

arch="$(uname -m)"
case "$arch" in
  x86_64)  hyperfine_target="x86_64-unknown-linux-musl" ;;
  aarch64) hyperfine_target="aarch64-unknown-linux-musl" ;;
  *)
    echo "ERROR: Unsupported architecture for hyperfine prebuilt binary: $arch" >&2
    exit 1
    ;;
esac

mkdir -p "$HOME/.local/bin"

echo "Installing hyperfine from GitHub release..." >&2
hyperfine_version="1.20.0"
hyperfine_archive="hyperfine-v${hyperfine_version}-${hyperfine_target}"
hyperfine_url="https://github.com/sharkdp/hyperfine/releases/download/v${hyperfine_version}/${hyperfine_archive}.tar.gz"
curl -fsSL "$hyperfine_url" | tar -xz --strip-components=1 -C "$HOME/.local/bin" "${hyperfine_archive}/hyperfine"
