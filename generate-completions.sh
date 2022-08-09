#! /bin/bash
set -o errexit -o pipefail -o nounset

cd "$(dirname "$0")"
mkdir -p exports

gen() {
  ./run.sh pdu-completions --name='pdu' --shell="$1" --output="exports/$2"
}

gen bash completion.bash
gen fish completion.fish
gen zsh completion.zsh
gen powershell completion.ps1
gen elvish completion.elv
