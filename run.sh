#! /bin/bash
set -o errexit -o pipefail -o nounset
exec cargo run --bin="$1" --features cli-completions,man-page,usage-md,ai-instructions -- "${@:2}"
