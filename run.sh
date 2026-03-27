#! /bin/bash
set -o errexit -o pipefail -o nounset
exec cargo run --bin="$1" --features cli-completions,cli-man,ai-instructions -- "${@:2}"
