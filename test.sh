#! /bin/bash
set -o errexit -o pipefail -o nounset

# Validate PDU_NO_FAIL_FAST
no_fail_fast="${PDU_NO_FAIL_FAST:-false}"
case "$no_fail_fast" in
true | false) ;;
*)
  echo "error: Invalid value for PDU_NO_FAIL_FAST: $no_fail_fast (expected 'true' or 'false')" >&2
  exit 1
  ;;
esac

# A temporary file is used instead of a variable because run_if and unit are
# subshells, so variable assignments inside them don't propagate to the parent.
failure_dir=$(mktemp -d)
trap 'rm -rf "$failure_dir"' EXIT
failure_marker="$failure_dir/failed"

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
  true)
    if [[ $no_fail_fast == 'true' ]]; then
      run "$@" || {
        exit_status=$?
        printf 'error: Command failed with exit code %d: ' "$exit_status" >&2
        printf '%q ' "$@" >&2
        printf '\n' >&2
        touch "$failure_marker"
      }
    else
      run "$@"
    fi
    ;;
  false) skip "$@" ;;
  *)
    echo "error: Invalid condition: $condition" >&2
    exit 1
    ;;
  esac
)

unit() (
  read -ra build_flags <<<"${BUILD_FLAGS:-}"
  read -ra test_flags <<<"${TEST_FLAGS:-}"
  read -ra test_skip <<<"${TEST_SKIP:-}"
  skip_args=()
  for name in ${test_skip[@]+"${test_skip[@]}"}; do
    skip_args+=(--skip "$name")
  done
  run_if "${LINT:-true}" cargo clippy "$@" -- -D warnings
  run_if "${DOC:-false}" cargo doc "$@"
  run_if "${BUILD:-true}" cargo build ${build_flags[@]+"${build_flags[@]}"} "$@"
  if [[ ${#skip_args[@]} -gt 0 ]]; then
    run_if "${TEST:-true}" cargo test ${test_flags[@]+"${test_flags[@]}"} "$@" -- "${skip_args[@]}"
  else
    run_if "${TEST:-true}" cargo test ${test_flags[@]+"${test_flags[@]}"} "$@"
  fi
)

run_if "${FMT:-true}" cargo fmt -- --check
unit "$@"
unit --no-default-features "$@"
unit --all-features "$@"
unit --features cli "$@"

if [[ -f "$failure_marker" ]]; then
  echo >&2
  echo 'error: Some checks have failed. Review the output above for details.' >&2
  exit 1
fi
