#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Installing Rust toolchain from rust-toolchain..." >&2
rustup toolchain install "$(<"$REPO_ROOT/rust-toolchain")"
rustup component add clippy rustfmt rust-analyzer
