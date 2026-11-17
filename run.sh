#! /bin/bash
set -o errexit -o pipefail -o nounset

# Every binary other than `pdu` lives in its own crate under `tools/`, and
# `cargo run --bin` only looks inside the current package, so the package has
# to be named. Each tool crate shares its name with the binary it provides.
bin="$1"
case "$bin" in
pdu) package='parallel-disk-usage' ;;
*) package="$bin" ;;
esac

exec cargo run --package="$package" --bin="$bin" -- "${@:2}"
