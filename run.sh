#! /bin/bash
set -o errexit -o pipefail -o nounset

case "$1" in
pdu) package='parallel-disk-usage' ;;
*) package="$1" ;;
esac

exec cargo run --package="$package" --bin="$1" -- "${@:2}"
