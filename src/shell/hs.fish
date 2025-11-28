# Heatseeker fish integration

function _hs_insert_paths --description "Insert file paths selected via heatseeker"
    printf "\n" # Go down to next line
    set -l choices
    if type -q fd
        set choices (fd --type f --color=never | hs)
    else if type -q rg
        set choices (rg --files | hs)
    else
        set choices (find . -type f -not -path '*/.git/*' | hs)
    end

    printf "\e[1A" # Return to the prompt line
    if test (count $choices) -gt 0
        commandline -i (string join ' ' $choices)" "
    end
    commandline -f repaint
end

function _hs_fuzzy_history --description "Search shell history via heatseeker"
    set -l dedup
    set -l seen
    for cmd in (history)
        if test -n "$cmd"
            if not contains -- $cmd $seen
                set -a dedup $cmd
                set -a seen $cmd
            end
        end
    end

    printf "\n" # Go down to next line
    set -l selection (printf '%s\n' $dedup | hs --filter-only | string trim)
    printf "\e[1A" # Return to the prompt line
    commandline -f repaint
    if test -n "$selection"
        commandline -r $selection
    end
end

bind ctrl-s _hs_insert_paths
bind ctrl-r _hs_fuzzy_history
