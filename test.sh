#! /bin/bash
set -o errexit -o pipefail -o nounset

# Warnings are errors for every crate in this workspace and for every target
# within those crates, which covers the library, the `pdu` binary, the test
# targets, the generated documentation, and the helper crates under `tools/` and
# `utils/`. Flags that the caller has already placed in these variables are
# preserved. The "Warnings Are Errors" section of CONTRIBUTING.md explains why
# the policy lives in this script rather than in the source, and which cases
# it leaves out.
export RUSTFLAGS="-D warnings ${RUSTFLAGS:-}"
export RUSTDOCFLAGS="-D warnings ${RUSTDOCFLAGS:-}"

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
  run_if "${LINT:-true}" cargo clippy --all-targets "$@" -- -D warnings
  run_if "${DOC:-false}" cargo doc --document-private-items --no-deps "$@"
  run_if "${BUILD:-true}" cargo build ${build_flags[@]+"${build_flags[@]}"} "$@"
  if [[ ${#skip_args[@]} -gt 0 ]]; then
    run_if "${TEST:-true}" cargo test ${test_flags[@]+"${test_flags[@]}"} "$@" -- "${skip_args[@]}"
  else
    run_if "${TEST:-true}" cargo test ${test_flags[@]+"${test_flags[@]}"} "$@"
  fi
)

helper_crates() (
  read -ra build_flags <<<"${BUILD_FLAGS:-}"
  read -ra test_flags <<<"${TEST_FLAGS:-}"
  selection=(--workspace --exclude parallel-disk-usage)
  run_if "${LINT:-true}" cargo clippy --all-targets "${selection[@]}" "$@" -- -D warnings
  run_if "${DOC:-false}" cargo doc --document-private-items --no-deps "${selection[@]}" "$@"
  run_if "${BUILD:-true}" cargo build ${build_flags[@]+"${build_flags[@]}"} "${selection[@]}" "$@"
  run_if "${TEST:-true}" cargo test ${test_flags[@]+"${test_flags[@]}"} "${selection[@]}" "$@"
)

run_if "${FMT:-true}" cargo fmt -- --check
# Dylint inspects the source under all feature gates in a single pass, so it is
# run once with `--all-features` rather than across every feature combination.
run_if "${DYLINT:-false}" cargo dylint --all -- --workspace --all-features --all-targets
unit "$@"
unit --no-default-features "$@"
unit --all-features "$@"
unit --features cli "$@"
helper_crates "$@"

if [[ -f "$failure_marker" ]]; then
  echo >&2
  echo 'error: Some checks have failed. Review the output above for details.' >&2
  exit 1
fi
