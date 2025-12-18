# Bash completion for cmdx
# Source this file or copy to /etc/bash_completion.d/
# Usage: source completions/cmdx.bash

_cmdx() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Commands
    local commands="exec shell translate"
    
    # Options
    local opts="--from --to --dry-run --quiet --verbose --no-color --help --version -n -q -v -h"
    
    # OS values for --from and --to
    local os_values="windows linux macos freebsd"
    
    # Handle previous argument
    case "${prev}" in
        --from|--to)
            COMPREPLY=( $(compgen -W "${os_values}" -- ${cur}) )
            return 0
            ;;
        cmdx)
            # First argument can be a command, option, or file
            COMPREPLY=( $(compgen -W "${commands} ${opts}" -- ${cur}) )
            COMPREPLY+=( $(compgen -f -- ${cur}) )
            return 0
            ;;
        exec|shell|translate)
            # After commands, suggest options
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
    esac
    
    # Default: suggest options and files
    if [[ ${cur} == -* ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    else
        # Suggest script files
        COMPREPLY=( $(compgen -f -X '!*.@(bat|cmd|ps1|sh|bash|zsh)' -- ${cur}) )
        # Also add all files if no matches
        if [ ${#COMPREPLY[@]} -eq 0 ]; then
            COMPREPLY=( $(compgen -f -- ${cur}) )
        fi
    fi
    
    return 0
}

complete -F _cmdx cmdx
