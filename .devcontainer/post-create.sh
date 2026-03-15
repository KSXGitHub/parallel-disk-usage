#!/usr/bin/env bash
set -euo pipefail

echo "Installing Python dependencies..." >&2
pip install toml

echo "Installing pnpm and project Node dependencies..." >&2
npm install -g pnpm
(cd ci/github-actions && pnpm install)

echo "Installing hyperfine via Cargo..." >&2
cargo install hyperfine
