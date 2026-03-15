#!/usr/bin/env bash
set -euo pipefail

echo "Installing Python dependencies..." >&2
pip install toml

echo "Installing pnpm and project Node dependencies..." >&2
npm install -g pnpm@7.33.7
(cd ci/github-actions && pnpm install)

echo "Installing hyperfine from GitHub release..." >&2
HYPERFINE_VERSION="1.20.0"
HYPERFINE_ARCHIVE="hyperfine-v${HYPERFINE_VERSION}-x86_64-unknown-linux-musl"
HYPERFINE_URL="https://github.com/sharkdp/hyperfine/releases/download/v${HYPERFINE_VERSION}/${HYPERFINE_ARCHIVE}.tar.gz"
mkdir -p "$HOME/.local/bin"
curl -fsSL "$HYPERFINE_URL" \
  | tar -xz --strip-components=1 -C "$HOME/.local/bin" "${HYPERFINE_ARCHIVE}/hyperfine"
