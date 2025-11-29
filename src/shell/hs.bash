# Heatseeker bash integration

__hs_setup_heatseeker() {
    if [[ ${BASH_VERSINFO[0]:-0} -lt 4 ]]; then
        printf 'heatseeker: bash %s lacks READLINE_LINE/READLINE_POINT support; key bindings disabled. Install a newer bash (e.g., via Homebrew) to enable Heatseeker bindings.\n' "${BASH_VERSION:-unknown}" >&2
        return
    fi

    # Free Ctrl+S for bindings in interactive shells
    if [[ $- == *i* ]]; then
        stty -ixon
    fi

    __hs_insert_paths() {
        local selected_paths
        printf '\n'
        if command -v fd >/dev/null 2>&1; then
            selected_paths=$(fd --type f --color=never | hs | paste -sd' ' -) || { printf '\033[1A'; return; }
        elif command -v rg >/dev/null 2>&1; then
            selected_paths=$(rg --files | hs | paste -sd' ' -) || { printf '\033[1A'; return; }
        elif [[ "$OS" == "Windows_NT" ]]; then
            selected_paths=$(cmd /c 'dir /a-d /s /b | findstr /V "\\.git\\\\"' | hs | paste -sd' ' -) || { printf '\033[1A'; return; }
        else
            selected_paths=$(find . -type f -not -path '*/.git/*' | hs | paste -sd' ' -) || { printf '\033[1A'; return; }
        fi

        printf '\033[1A'
        if [[ -n $selected_paths ]]; then
            READLINE_LINE+="${selected_paths} "
            READLINE_POINT=${#READLINE_LINE}
        fi
    }

    __hs_fuzzy_history() {
        local selection

        printf '\n'
        selection=$(
            HISTTIMEFORMAT= history | awk '
                {
                    $1 = "";
                    cmd = substr($0,2);
                    lines[NR] = cmd
                }
                END {
                    for (i = NR; i >= 1; i--)
                        if (!seen[lines[i]]++)
                            print lines[i]
                }
            ' | hs --filter-only
        )
        printf '\033[1A'
        if [[ -n $selection ]]; then
            READLINE_LINE=$selection
            READLINE_POINT=${#READLINE_LINE}
        fi
    }

    bind -x '"\C-s":__hs_insert_paths'
    bind -x '"\C-r":__hs_fuzzy_history'
}

__hs_setup_heatseeker
unset -f __hs_setup_heatseeker
