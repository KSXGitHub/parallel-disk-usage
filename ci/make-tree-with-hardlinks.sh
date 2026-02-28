#! /bin/bash
set -o errexit -o pipefail -o nounset

target="${1}"
root="${target}/__tree_with_hardlinks__"
mkdir -pv "${root}"

create_file() {
  mkdir -p "$(dirname "${root}/$1")"
  head -c "$2" /dev/zero >"${root}/$1"
}

create_hardlink() {
  mkdir -p "$(dirname "${root}/$2")"
  ln "${root}/$1" "${root}/$2"
}

# file 01: 128 bytes, 1 hardlink
create_file a/file01 128
create_hardlink a/file01 b/link01

# file 02: 256 bytes, 3 hardlinks
create_file a/b/file02 256
create_hardlink a/b/file02 c/link02a
create_hardlink a/b/file02 c/d/link02b
create_hardlink a/b/file02 e/link02c

# file 03: 512 bytes, 5 hardlinks
create_file b/c/file03 512
create_hardlink b/c/file03 f/link03a
create_hardlink b/c/file03 f/g/link03b
create_hardlink b/c/file03 f/g/h/link03c
create_hardlink b/c/file03 i/link03d
create_hardlink b/c/file03 j/k/link03e

# file 04: 1024 bytes, 7 hardlinks
create_file c/d/e/file04 1024
create_hardlink c/d/e/file04 l/link04a
create_hardlink c/d/e/file04 l/m/link04b
create_hardlink c/d/e/file04 l/m/n/link04c
create_hardlink c/d/e/file04 o/link04d
create_hardlink c/d/e/file04 o/p/link04e
create_hardlink c/d/e/file04 q/link04f
create_hardlink c/d/e/file04 r/s/link04g

# file 05: 2048 bytes, 2 hardlinks
create_file d/file05 2048
create_hardlink d/file05 s/t/link05a
create_hardlink d/file05 u/link05b

# file 06: 4096 bytes, 4 hardlinks
create_file e/f/file06 4096
create_hardlink e/f/file06 v/link06a
create_hardlink e/f/file06 v/w/link06b
create_hardlink e/f/file06 x/link06c
create_hardlink e/f/file06 x/y/z/link06d

# file 07: 8192 bytes, 10 hardlinks
create_file f/g/h/file07 8192
for i in $(seq 1 10); do
    create_hardlink f/g/h/file07 "aa/bb${i}/link07"
done

# file 08: 100 bytes, 15 hardlinks
create_file g/file08 100
for i in $(seq 1 15); do
    create_hardlink g/file08 "cc/dd${i}/link08"
done

# file 09: 200 bytes, 20 hardlinks
create_file h/i/file09 200
for i in $(seq 1 20); do
    create_hardlink h/i/file09 "ee/ff${i}/link09"
done

# file 10: 300 bytes, 30 hardlinks
create_file i/j/k/file10 300
for i in $(seq 1 30); do
    create_hardlink i/j/k/file10 "gg/hh${i}/link10"
done

# file 11: 400 bytes, 1 hardlink
create_file j/file11 400
create_hardlink j/file11 ii/link11

# file 12: 600 bytes, 2 hardlinks
create_file k/l/file12 600
create_hardlink k/l/file12 jj/link12a
create_hardlink k/l/file12 jj/kk/link12b

# file 13: 700 bytes, 8 hardlinks
create_file l/m/n/file13 700
for i in $(seq 1 8); do
    create_hardlink l/m/n/file13 "ll/mm${i}/link13"
done

# file 14: 900 bytes, 12 hardlinks
create_file m/file14 900
for i in $(seq 1 12); do
    create_hardlink m/file14 "nn/oo${i}/link14"
done

# file 15: 1500 bytes, 25 hardlinks
create_file n/o/file15 1500
for i in $(seq 1 25); do
    create_hardlink n/o/file15 "pp/qq${i}/link15"
done

# file 16: 3000 bytes, 40 hardlinks
create_file o/p/q/file16 3000
for i in $(seq 1 40); do
    create_hardlink o/p/q/file16 "rr/ss${i}/link16"
done

# file 17: 5000 bytes, 50 hardlinks
create_file p/file17 5000
for i in $(seq 1 50); do
    create_hardlink p/file17 "tt/uu${i}/link17"
done

# file 18: 750 bytes, 6 hardlinks
create_file q/r/file18 750
create_hardlink q/r/file18 vv/link18a
create_hardlink q/r/file18 vv/ww/link18b
create_hardlink q/r/file18 vv/ww/xx/link18c
create_hardlink q/r/file18 yy/link18d
create_hardlink q/r/file18 yy/zz/link18e
create_hardlink q/r/file18 aaa/link18f

# file 19: 1234 bytes, 9 hardlinks
create_file r/s/t/file19 1234
for i in $(seq 1 9); do
    create_hardlink r/s/t/file19 "bbb/ccc${i}/link19"
done

# file 20: 10240 bytes, 11 hardlinks
create_file s/file20 10240
for i in $(seq 1 11); do
    create_hardlink s/file20 "ddd/eee${i}/link20"
done
