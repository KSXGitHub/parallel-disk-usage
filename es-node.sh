#! /bin/bash
set -o errexit -o pipefail -o nounset
options=(
  --experimental-modules
  --es-module-specifier-resolution='node'
)
exec node "${options[@]}" "$@"
