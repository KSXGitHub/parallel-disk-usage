#! /bin/bash
set -o errexit -o pipefail -o nounset

bin=$1
case "$bin" in
pdu) package='parallel-disk-usage' ;;
*) package="$bin" ;;
esac

exec cargo run --package="$package" --bin="$bin" -- "${@:2}"
