_pdu() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            pdu)
                cmd="pdu"
                ;;
            
            *)
                ;;
        esac
    done

    case "${cmd}" in
        pdu)
            opts=" -h -V  --json-input --json-output --top-down --align-left --no-sort --silent-errors --progress --help --version --bytes-format --quantity --max-depth --total-width --column-width --min-ratio  <files>... "
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
                    COMPREPLY=($(compgen -W "len blksize blocks" -- "${cur}"))
                    return 0
                    ;;
                --max-depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --total-width)
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
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        
    esac
}

complete -F _pdu -o bashdefault -o default pdu
