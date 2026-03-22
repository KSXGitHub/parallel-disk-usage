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

failure_marker=$(mktemp)
rm -f "$failure_marker"

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
      run "$@" || touch "$failure_marker"
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
unit --features ai-instructions "$@"

if [[ -f $failure_marker ]]; then
  rm -f "$failure_marker"
  echo >&2
  echo 'error: Some checks have failed. Review the output above for details.' >&2
  exit 1
fi
