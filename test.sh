#! /bin/bash
set -o errexit -o pipefail -o nounset

run() {
  echo >&2
  echo "exec> $*" >&2
  "$@"
}

skip() {
  echo >&2
  echo "skip> $*" >&2
}

run_if() {
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
}

flags=("$@")
build_flags=()
test_flags=()
eval "build_flags+=(${BUILD_FLAGS:-})"
eval "test_flags+=(${TEST_FLAGS:-})"
unit() {
  arguments=("$@" "${flags[@]}")
  run_if "${LINT:-true}" cargo clippy "${arguments[@]}" -- -D warnings
  run_if "${BUILD:-true}" cargo build "${build_flags[@]}" "${arguments[@]}"
  run_if "${TEST:-true}" cargo test "${test_flags[@]}" "${arguments[@]}"
}

run_if "${FMT:-true}" cargo fmt
unit
unit --no-default-features
unit --all-features
unit --features cli
unit --features cli-completions
