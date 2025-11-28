# Heatseeker Nushell integration

def --env hs-insert-paths [] {
    print "" # Go down to the next line
    let selection = (
        if ((which fd | length) > 0) {
            ^fd --type f --color=never | ^hs
        } else if ((which rg | length) > 0) {
            ^rg --files | ^hs
        } else if ($nu.os-info.name == "windows") {
            ^cmd /c 'dir /a-d /s /b | findstr /V "\\.git\\"' | ^hs
        } else {
            ^find . -type f -not -path '*/.git/*' | ^hs
        }
        | lines
        | where {|it| $it != ""}
        | str join " "
    )

    print -n (ansi cursor_up) # Return to the prompt line
    if $selection != "" { commandline edit --insert $"($selection) " }
}

def --env hs-fuzzy-history [] {
    print "" # Go down to the next line
    mut deduped = []
    for cmd in (history | get command | reverse) {
        # TODO: This is asymptotically slow; we need a better data structure
        if not ($cmd in $deduped) {
            $deduped = ($deduped | append $cmd)
        }
    }

    let selection = (
        $deduped
        | str join (char nl)
        | ^hs --filter-only
        | str trim
    )

    print -n (ansi cursor_up) # Return to the prompt line
    if $selection != "" { commandline edit --replace $selection }
}

let _hs_keybindings = [
    {
        name: "hs_insert_path"
        modifier: control
        keycode: char_s
        mode: [emacs vi_normal vi_insert]
        event: { send: ExecuteHostCommand cmd: "hs-insert-paths" }
    }
    {
        name: "hs_fuzzy_history"
        modifier: control
        keycode: char_r
        mode: [emacs vi_normal vi_insert]
        event: { send: ExecuteHostCommand cmd: "hs-fuzzy-history" }
    }
]

$env.config = (
    $env.config
    | upsert keybindings ($_hs_keybindings)
)
