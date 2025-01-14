#! /bin/bash
set -o errexit -o pipefail -o nounset
mkdir ./flatten

[ -d ./downloads ] || {
	echo Folder ./downloads does not exist >/dev/stderr
	exit 1
}

# shellcheck disable=SC2012
ls ./downloads | while read -r name; do
	case "$name" in
	*wasm*) suffix=.wasm ;;
	*windows*) suffix=.exe ;;
	*) suffix='' ;;
	esac

	src="./downloads/${name}/pdu${suffix}"
	dst="./flatten/${name}${suffix}"
	echo Copying "$src" to "$dst"...
	cp "$src" "$dst" || exit $?
done
