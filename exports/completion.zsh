#compdef pdu

autoload -U is-at-least

_pdu() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
'--bytes-format=[How to display the numbers of bytes]:BYTES_FORMAT:((plain\:"Display the value as-is"
metric\:"Display the value with a unit suffix in \[metric scale\](formatter::METRIC)"
binary\:"Display the value with a unit suffix in \[binary scale\](formatter::BINARY)"))' \
'--quantity=[Aspect of the files/directories to be measured]:QUANTITY:((len\:"Measure apparent sizes, equivalent to the \[len\](std::fs::Metadata::len) method"
blksize\:"Measure block sizes, equivalent to the \[blksize\](std::os::unix::prelude::MetadataExt::blksize) method (POSIX only)"
blocks\:"Count numbers of blocks, equivalent to the \[blocks\](std::os::unix::prelude::MetadataExt::blocks) method (POSIX only)"))' \
'--max-depth=[Maximum depth to display the data (must be greater than 0)]:MAX_DEPTH: ' \
'(--column-width)--total-width=[Width of the visualization]:TOTAL_WIDTH: ' \
'*--column-width=[Maximum widths of the tree column and width of the bar column]:TREE_WIDTH: :TREE_WIDTH: ' \
'--min-ratio=[Minimal size proportion required to appear]:MIN_RATIO: ' \
'(--quantity)--json-input[Read JSON data from stdin]' \
'--json-output[Print JSON data instead of an ASCII chart]' \
'--top-down[Print the tree top-down instead of bottom-up]' \
'--align-left[Fill the bars from left to right]' \
'--no-sort[Preserve order of entries]' \
'--silent-errors[Prevent filesystem error messages from appearing in stderr]' \
'--progress[Report progress being made at the expense of performance]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'*::files -- List of files and/or directories:_files' \
&& ret=0
}

(( $+functions[_pdu_commands] )) ||
_pdu_commands() {
    local commands; commands=()
    _describe -t commands 'pdu commands' commands "$@"
}

_pdu "$@"
