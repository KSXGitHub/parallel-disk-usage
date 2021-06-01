#! /bin/bash
set -o errexit -o pipefail -o nounset

run() (
  echo >&2
  echo "exec> $*" >&2
  "$@"
)

skip() (
  echo >&2
  echo "skip> $*" >&2
)

run_if() (
  condition="$1"
  shift
  case "$condition" in
  true) run "$@" ;;
  false) skip "$@" ;;
  *)
    echo "error: Invalid condition: $condition" >&2
    exit 1
    ;;
  esac
)

unit() (
  eval run_if "${LINT:-true}" cargo clippy "$@" -- -D warnings
  eval run_if "${DOC:-false}" cargo doc "$@"
  eval run_if "${BUILD:-true}" cargo build "${BUILD_FLAGS:-}" "$@"
  eval run_if "${TEST:-true}" cargo test "${TEST_FLAGS:-}" "$@"
)

run_if "${FMT:-true}" cargo fmt -- --check
unit "$@"
unit --no-default-features "$@"
unit --all-features "$@"
unit --features cli "$@"
unit --features cli-completions "$@"
