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
    _arguments "${_arguments_options[@]}" : \
'--bytes-format=[How to display the numbers of bytes]:BYTES_FORMAT:((plain\:"Display plain number of bytes without units"
metric\:"Use metric scale, i.e. 1K = 1000B, 1M = 1000K, and so on"
binary\:"Use binary scale, i.e. 1K = 1024B, 1M = 1024K, and so on"))' \
'--quantity=[Aspect of the files/directories to be measured]:QUANTITY:((apparent-size\:"Measure apparent sizes"
block-size\:"Measure block sizes (block-count * 512B)"
block-count\:"Count numbers of blocks"))' \
'--max-depth=[Maximum depth to display the data (must be greater than 0)]:MAX_DEPTH:_default' \
'--depth=[Maximum depth to display the data (must be greater than 0)]:MAX_DEPTH:_default' \
'(--column-width)--total-width=[Width of the visualization]:TOTAL_WIDTH:_default' \
'(--column-width)--width=[Width of the visualization]:TOTAL_WIDTH:_default' \
'*--column-width=[Maximum widths of the tree column and width of the bar column]:TREE_WIDTH:_default:TREE_WIDTH:_default' \
'--min-ratio=[Minimal size proportion required to appear]:MIN_RATIO:_default' \
'--threads=[Set the maximum number of threads to spawn. Could be either "auto", "max", or a number]:THREADS:_default' \
'(--quantity --deduplicate-hardlinks)--json-input[Read JSON data from stdin]' \
'--json-output[Print JSON data instead of an ASCII chart]' \
'--deduplicate-hardlinks[Detect duplicated hardlinks and remove their sizes from total]' \
'--detect-links[Detect duplicated hardlinks and remove their sizes from total]' \
'--dedupe-links[Detect duplicated hardlinks and remove their sizes from total]' \
'--top-down[Print the tree top-down instead of bottom-up]' \
'--align-right[Set the root of the bars to the right]' \
'--no-sort[Preserve order of entries]' \
'--silent-errors[Prevent filesystem error messages from appearing in stderr]' \
'--no-errors[Prevent filesystem error messages from appearing in stderr]' \
'--progress[Report progress being made at the expense of performance]' \
'--omit-json-shared-details[Do not output \`.shared.details\` in the JSON output]' \
'--omit-json-shared-summary[Do not output \`.shared.summary\` in the JSON output]' \
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

if [ "$funcstack[1]" = "_pdu" ]; then
    _pdu "$@"
else
    compdef _pdu pdu
fi
