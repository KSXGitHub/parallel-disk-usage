#! /bin/bash
set -o errexit -o pipefail -o nounset
exec cargo run --bin="$1" --features cli-completions -- "${@:2}"
