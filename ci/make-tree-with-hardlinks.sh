#! /bin/bash
set -o errexit -o pipefail -o nounset

target="${1}"
root="${target}/__tree_with_hardlinks__"
mkdir -pv "${root}"

mk() { mkdir -p "$(dirname "${root}/$1")"; head -c "$2" /dev/zero >"${root}/$1"; }
hl() { mkdir -p "$(dirname "${root}/$2")"; ln "${root}/$1" "${root}/$2"; }

# file 01: 128 bytes, 1 hardlink
mk a/file01 128
hl a/file01 b/link01

# file 02: 256 bytes, 3 hardlinks
mk a/b/file02 256
hl a/b/file02 c/link02a
hl a/b/file02 c/d/link02b
hl a/b/file02 e/link02c

# file 03: 512 bytes, 5 hardlinks
mk b/c/file03 512
hl b/c/file03 f/link03a
hl b/c/file03 f/g/link03b
hl b/c/file03 f/g/h/link03c
hl b/c/file03 i/link03d
hl b/c/file03 j/k/link03e

# file 04: 1024 bytes, 7 hardlinks
mk c/d/e/file04 1024
hl c/d/e/file04 l/link04a
hl c/d/e/file04 l/m/link04b
hl c/d/e/file04 l/m/n/link04c
hl c/d/e/file04 o/link04d
hl c/d/e/file04 o/p/link04e
hl c/d/e/file04 q/link04f
hl c/d/e/file04 r/s/link04g

# file 05: 2048 bytes, 2 hardlinks
mk d/file05 2048
hl d/file05 s/t/link05a
hl d/file05 u/link05b

# file 06: 4096 bytes, 4 hardlinks
mk e/f/file06 4096
hl e/f/file06 v/link06a
hl e/f/file06 v/w/link06b
hl e/f/file06 x/link06c
hl e/f/file06 x/y/z/link06d

# file 07: 8192 bytes, 10 hardlinks
mk f/g/h/file07 8192
for i in $(seq 1 10); do
    hl f/g/h/file07 "aa/bb${i}/link07"
done

# file 08: 100 bytes, 15 hardlinks
mk g/file08 100
for i in $(seq 1 15); do
    hl g/file08 "cc/dd${i}/link08"
done

# file 09: 200 bytes, 20 hardlinks
mk h/i/file09 200
for i in $(seq 1 20); do
    hl h/i/file09 "ee/ff${i}/link09"
done

# file 10: 300 bytes, 30 hardlinks
mk i/j/k/file10 300
for i in $(seq 1 30); do
    hl i/j/k/file10 "gg/hh${i}/link10"
done

# file 11: 400 bytes, 1 hardlink
mk j/file11 400
hl j/file11 ii/link11

# file 12: 600 bytes, 2 hardlinks
mk k/l/file12 600
hl k/l/file12 jj/link12a
hl k/l/file12 jj/kk/link12b

# file 13: 700 bytes, 8 hardlinks
mk l/m/n/file13 700
for i in $(seq 1 8); do
    hl l/m/n/file13 "ll/mm${i}/link13"
done

# file 14: 900 bytes, 12 hardlinks
mk m/file14 900
for i in $(seq 1 12); do
    hl m/file14 "nn/oo${i}/link14"
done

# file 15: 1500 bytes, 25 hardlinks
mk n/o/file15 1500
for i in $(seq 1 25); do
    hl n/o/file15 "pp/qq${i}/link15"
done

# file 16: 3000 bytes, 40 hardlinks
mk o/p/q/file16 3000
for i in $(seq 1 40); do
    hl o/p/q/file16 "rr/ss${i}/link16"
done

# file 17: 5000 bytes, 50 hardlinks
mk p/file17 5000
for i in $(seq 1 50); do
    hl p/file17 "tt/uu${i}/link17"
done

# file 18: 750 bytes, 6 hardlinks
mk q/r/file18 750
hl q/r/file18 vv/link18a
hl q/r/file18 vv/ww/link18b
hl q/r/file18 vv/ww/xx/link18c
hl q/r/file18 yy/link18d
hl q/r/file18 yy/zz/link18e
hl q/r/file18 aaa/link18f

# file 19: 1234 bytes, 9 hardlinks
mk r/s/t/file19 1234
for i in $(seq 1 9); do
    hl r/s/t/file19 "bbb/ccc${i}/link19"
done

# file 20: 10240 bytes, 11 hardlinks
mk s/file20 10240
for i in $(seq 1 11); do
    hl s/file20 "ddd/eee${i}/link20"
done
