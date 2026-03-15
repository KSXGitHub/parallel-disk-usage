#!/usr/bin/env bash
set -euo pipefail

echo "Installing Python dependencies..." >&2
pip install toml

echo "Installing pnpm and project Node dependencies..." >&2
npm install -g pnpm@7.33.7
(cd ci/github-actions && pnpm install)

echo "Installing hyperfine from GitHub release..." >&2
hyperfine_version="1.20.0"
hyperfine_archive_name="hyperfine-v${hyperfine_version}-x86_64-unknown-linux-musl"
hyperfine_url="https://github.com/sharkdp/hyperfine/releases/download/v${hyperfine_version}/${hyperfine_archive_name}.tar.gz"
mkdir -p "$HOME/.local/bin"
curl -fsSL "$hyperfine_url" | tar -xz --strip-components=1 -C "$HOME/.local/bin" "${hyperfine_archive_name}/hyperfine"
