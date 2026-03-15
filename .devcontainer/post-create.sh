#!/usr/bin/env bash
set -euo pipefail

# Install Python dependencies
pip install toml

# Install pnpm and project Node dependencies
npm install -g pnpm
(cd ci/github-actions && pnpm install)

# Install hyperfine via Cargo
cargo install hyperfine
