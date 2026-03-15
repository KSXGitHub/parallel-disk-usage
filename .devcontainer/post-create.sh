#!/usr/bin/env bash
set -euo pipefail

echo "Installing Python dependencies..." >&2
pip install toml

echo "Installing pnpm and project Node dependencies..." >&2
npm install -g pnpm
(cd ci/github-actions && pnpm install)

echo "Installing hyperfine from GitHub release..." >&2
HYPERFINE_VERSION="1.20.0"
mkdir -p "$HOME/.local/bin"
curl -fsSL "https://github.com/sharkdp/hyperfine/releases/download/v${HYPERFINE_VERSION}/hyperfine-v${HYPERFINE_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
  | tar -xz --strip-components=1 -C "$HOME/.local/bin" "hyperfine-v${HYPERFINE_VERSION}-x86_64-unknown-linux-musl/hyperfine"
