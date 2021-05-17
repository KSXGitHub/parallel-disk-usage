#! /bin/bash
set -o errexit -o pipefail -o nounset

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
  echo >&2
  echo "benchmark unit $i..." >&2
  (time unit "$@")
done
echo >&2
