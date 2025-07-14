_pdu() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="pdu"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        pdu)
            opts="-h -V --json-input --json-output --bytes-format --top-down --align-right --quantity --depth --max-depth --width --total-width --column-width --min-ratio --no-sort --no-errors --silent-errors --progress --threads --help --version [FILES]..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --bytes-format)
                    COMPREPLY=($(compgen -W "plain metric binary" -- "${cur}"))
                    return 0
                    ;;
                --quantity)
                    COMPREPLY=($(compgen -W "apparent-size block-size block-count" -- "${cur}"))
                    return 0
                    ;;
                --max-depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --total-width)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --width)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --column-width)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --min-ratio)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _pdu -o nosort -o bashdefault -o default pdu
else
    complete -F _pdu -o bashdefault -o default pdu
fi
