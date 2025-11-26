# Heatseeker zsh integration

setopt noflowcontrol

# Replace the shell's built-in ^R handling
_hs_fuzzy_history() {
    echo
    BUFFER=$(fc -l 1 | tac | awk '{ $1=""; cmd=substr($0,2); if (!seen[cmd]++) print cmd }' | hs --filter-only | sed 's/\\n/\n/g')
    echo -n "\033[1A"
    zle reset-prompt
    zle end-of-buffer-or-history
}

# Run Heatseeker in the current working directory, appending the selected path, if
# any, to the current command.
_hs_insert_hs_path_in_command_line() {
    local selected_paths
    echo # Print a newline or we'll clobber the old prompt.
    # Find the path; abort if the user doesn't select anything.
    if command -v fd >/dev/null 2>&1; then
        selected_paths=$(fd --type f --color=never | hs | paste -sd' ' -) || return
    elif command -v rg >/dev/null 2>&1; then
        selected_paths=$(rg --files | hs | paste -sd' ' -) || return
    else
        selected_paths=$(find . -type f -not -path '*/.git/*' | hs | paste -sd' ' -) || return
    fi
    # Append the selection to the current command buffer.
    if [[ -n $selected_paths ]]; then
        LBUFFER="$LBUFFER$selected_paths "
    fi
    echo -n "\033[1A" # Move the cursor back up
    zle reset-prompt # Redraw the prompt
}

# Create the zle widgets
zle -N _hs_insert_hs_path_in_command_line
zle -N _hs_fuzzy_history

# Bind those keys to the newly created widgets
bindkey "^S" "_hs_insert_hs_path_in_command_line"
bindkey "^R" "_hs_fuzzy_history"
