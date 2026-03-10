#!/usr/bin/env bash
set -euo pipefail

PDU=/home/user/parallel-disk-usage/target/release/pdu
DUST=/tmp/dust/target/release/dust
DUA=/tmp/dua-cli/target/release/dua
TEST_DIR=/tmp/bench-hardlinks
WARMUP=2
RUNS=10

echo "=== Creating test data with hardlinks ==="
rm -rf "${TEST_DIR}"
mkdir -p "${TEST_DIR}"

for i in $(seq 1 20); do
  dir="${TEST_DIR}/dir_${i}"
  mkdir -p "${dir}"
  for j in $(seq 1 500); do
    dd if=/dev/urandom "of=${dir}/file_${j}.bin" bs=4096 count=1 status=none 2>/dev/null
  done
done

echo "Creating hardlinks..."
for i in $(seq 1 10); do
  src="${TEST_DIR}/dir_${i}"
  dst="${TEST_DIR}/dir_${i}_links"
  mkdir -p "${dst}"
  for f in "${src}"/*; do
    ln "${f}" "${dst}/$(basename "${f}")"
  done
done

echo "Creating multi-hardlinked files..."
mkdir -p "${TEST_DIR}/multi_links"
for i in $(seq 1 100); do
  src="${TEST_DIR}/dir_1/file_${i}.bin"
  for k in $(seq 1 5); do
    ln "${src}" "${TEST_DIR}/multi_links/file_${i}_link_${k}.bin"
  done
done

total_files=$(find "${TEST_DIR}" -type f | wc -l)
unique_inodes=$(find "${TEST_DIR}" -type f -printf '%i\n' | sort -u | wc -l)
echo "Total file entries: ${total_files}"
echo "Unique inodes: ${unique_inodes}"
echo "Hardlinked entries: $((total_files - unique_inodes))"
echo ""

echo "=== Benchmark 1: Hardlink Deduplication ==="
hyperfine \
  --warmup ${WARMUP} \
  --runs ${RUNS} \
  --export-markdown /tmp/bench-dedup.md \
  -n 'pdu --deduplicate-hardlinks' "${PDU} --deduplicate-hardlinks ${TEST_DIR}" \
  -n 'dust (default dedup)' "${DUST} --no-progress ${TEST_DIR}" \
  -n 'dua (default dedup)' "${DUA} ${TEST_DIR}"

echo ""
cat /tmp/bench-dedup.md
echo ""

echo "=== Benchmark 2: Apparent Size (no dedup) ==="
hyperfine \
  --warmup ${WARMUP} \
  --runs ${RUNS} \
  --export-markdown /tmp/bench-apparent.md \
  -n 'pdu apparent-size' "${PDU} --quantity=apparent-size ${TEST_DIR}" \
  -n 'dust apparent-size' "${DUST} --no-progress --apparent-size ${TEST_DIR}" \
  -n 'dua apparent-size' "${DUA} --count-hard-links --apparent-size ${TEST_DIR}"

echo ""
cat /tmp/bench-apparent.md
echo ""

echo "=== Benchmark 3: Block Size (no dedup) ==="
hyperfine \
  --warmup ${WARMUP} \
  --runs ${RUNS} \
  --export-markdown /tmp/bench-block.md \
  -n 'pdu block-size' "${PDU} --quantity=block-size ${TEST_DIR}" \
  -n 'dust block-size' "${DUST} --no-progress ${TEST_DIR}" \
  -n 'dua count-hard-links' "${DUA} --count-hard-links ${TEST_DIR}"

echo ""
cat /tmp/bench-block.md
echo ""

echo "=== Benchmark 4: Real-world (pdu repo, few hardlinks) ==="
REPO_DIR=/home/user/parallel-disk-usage
hyperfine \
  --warmup ${WARMUP} \
  --runs ${RUNS} \
  --export-markdown /tmp/bench-repo.md \
  -n 'pdu --deduplicate-hardlinks (repo)' "${PDU} --deduplicate-hardlinks ${REPO_DIR}" \
  -n 'dust (repo)' "${DUST} --no-progress ${REPO_DIR}" \
  -n 'dua (repo)' "${DUA} ${REPO_DIR}"

echo ""
cat /tmp/bench-repo.md
echo ""

rm -rf "${TEST_DIR}"
echo "=== Done ==="
