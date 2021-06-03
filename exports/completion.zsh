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
'--bytes-format=[How to display the numbers of bytes]: :(plain metric binary)' \
'--quantity=[Aspect of the files/directories to be measured]: :(len blksize blocks)' \
'--max-depth=[Maximum depth to display the data (must be greater than 0)]' \
'(--column-width)--total-width=[Width of the visualization]' \
'*--column-width=[Maximum widths of the tree column and width of the bar column]' \
'--min-ratio=[Minimal size proportion required to appear]' \
'(--quantity)--json-input[Read JSON data from stdin]' \
'--json-output[Print JSON data instead of an ASCII chart]' \
'--top-down[Print the tree top-down instead of bottom-up]' \
'--no-sort[Preserve order of entries]' \
'--silent-errors[Prevent filesystem error messages from appearing in stderr]' \
'--progress[Report progress being made at the expense of performance]' \
'-h[Prints help information]' \
'--help[Prints help information]' \
'-V[Prints version information]' \
'--version[Prints version information]' \
'::files -- List of files and/or directories:_files' \
&& ret=0
    
}

(( $+functions[_pdu_commands] )) ||
_pdu_commands() {
    local commands; commands=(
        
    )
    _describe -t commands 'pdu commands' commands "$@"
}

_pdu "$@"