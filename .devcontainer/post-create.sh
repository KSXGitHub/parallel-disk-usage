#!/usr/bin/env bash
set -euo pipefail

mkdir -p "$HOME/.local/bin"

echo "Installing hyperfine from GitHub release..." >&2
hyperfine_version="1.20.0"
hyperfine_archive_name="hyperfine-v${hyperfine_version}-x86_64-unknown-linux-musl"
hyperfine_url="https://github.com/sharkdp/hyperfine/releases/download/v${hyperfine_version}/${hyperfine_archive_name}.tar.gz"
curl -fsSL "$hyperfine_url" | tar -xz --strip-components=1 -C "$HOME/.local/bin" "${hyperfine_archive_name}/hyperfine"

echo "Installing shellcheck from GitHub release..." >&2
shellcheck_version="0.11.0"
shellcheck_archive_name="shellcheck-v${shellcheck_version}.linux.x86_64"
shellcheck_url="https://github.com/koalaman/shellcheck/releases/download/v${shellcheck_version}/${shellcheck_archive_name}.tar.gz"
curl -fsSL "$shellcheck_url" | tar -xz --strip-components=1 -C "$HOME/.local/bin" "shellcheck-v${shellcheck_version}/shellcheck"

echo "Installing shfmt from GitHub release..." >&2
shfmt_version="3.13.0"
shfmt_url="https://github.com/mvdan/sh/releases/download/v${shfmt_version}/shfmt_v${shfmt_version}_linux_amd64"
curl -fsSL "$shfmt_url" -o "$HOME/.local/bin/shfmt"
chmod +x "$HOME/.local/bin/shfmt"
