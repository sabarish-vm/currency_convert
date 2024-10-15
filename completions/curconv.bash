#/usr/bin/env bash
_dothis_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD - 1]}"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=($(compgen -W "-u --update" -- ${cur}))
        return 0
    fi

    if [[ "${COMP_WORDS[1]}" == "-u" || "${COMP_WORDS[1]}" == "--update" ]]; then
        return 0
    fi

    if [[ ${COMP_CWORD} -eq 2 ]]; then
        if [[ "$prev" =~ ^[0-9]+$ ]]; then
            opts="AUD BGN BRL CAD CHF CNY CZK DKK EUR GBP HKD HUF IDR ILS INR ISK JPY KRW MXN MYR NOK NZD PHP PLN RON SEK SGD THB TRY USD ZAR"
            COMPREPLY=($(compgen -W "${opts}" -- ${cur}))
            return 0
        fi
    fi

    if [[ ${COMP_CWORD} -eq 3 ]]; then
        COMPREPLY=($(compgen -W "--to" -- ${cur}))
        return 0
    fi

    if [[ ${COMP_CWORD} -eq 4 ]]; then
        opts="AUD-Australia BGN BRL CAD CHF CNY CZK DKK EUR GBP HKD HUF IDR ILS INR ISK JPY KRW MXN MYR NOK NZD PHP PLN RON SEK SGD THB TRY USD ZAR"
        COMPREPLY=($(compgen -W "${opts}" -- ${cur}))
        return 0
    fi
    return 0
    if [[ ${#COMPREPLY[*]} -eq 1 ]]; then #Only one completion
        COMPREPLY=(${COMPREPLY[0]%% - *}) #Remove ' - ' and everything after
    fi
}

complete -F _dothis_completions curconv
