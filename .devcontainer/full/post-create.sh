#!/usr/bin/env bash
set -euo pipefail

echo "Installing Python dependencies..." >&2
python3 -m pip install --user toml

echo "Installing pnpm and project Node dependencies..." >&2
npm install -g pnpm@7.9.0
(cd ci/github-actions && pnpm install --frozen-lockfile)

bash "$(dirname "$0")/../install-rust-toolchain.sh"
bash "$(dirname "$0")/../install-hyperfine.sh"
