#! /bin/bash
set -o errexit -o pipefail -o nounset

# `cargo run` takes no `--workspace` and resolves `--bin` within one package.
case "$1" in
pdu) package='parallel-disk-usage' ;;
*) package="$1" ;;
esac

exec cargo run --package="$package" --bin="$1" -- "${@:2}"
