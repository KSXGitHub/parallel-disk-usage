#! /bin/bash
set -o errexit -o pipefail -o nounset

verify_var() {
  if (("$1" <= 0)); then
    echo "error: $2 ($1) is not a positive number" >&2
    exit 1
  fi
}

verify_var "$BENCHMARK_EXECUTION_COUNT" BENCHMARK_EXECUTION_COUNT
verify_var "$BENCHMARK_MEASUREMENT_COUNT" BENCHMARK_MEASUREMENT_COUNT

if (("$BENCHMARK_MEASUREMENT_COUNT" == 1)); then
  display_unit() { true; }
else
  display_unit() {
    echo >&2
    echo "benchmark unit $1..." >&2
  }
fi

stderr_file="$(mktemp)"

unit() (
  for ((i = 0; i < "$BENCHMARK_EXECUTION_COUNT"; i++)); do
    "$@" >/dev/null 2>"$stderr_file" || {
      code="$?"
      cat "$stderr_file" >&2
      exit "$code"
    }
  done
)

echo "benchmark command $*..." >&2
for ((i = 0; i < "$BENCHMARK_MEASUREMENT_COUNT"; i++)); do
  display_unit "$i"
  (time unit "$@")
done
echo >&2
