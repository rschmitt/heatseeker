# Heatseeker PowerShell integration

# Check if PSReadLine is available (should be present in PowerShell 5.1+ and all PowerShell Core versions)
if (-not (Get-Module -Name PSReadLine)) {
    Write-Warning "PSReadLine module is not available. Heatseeker integration requires PSReadLine."
    return
}

$ps = [Microsoft.PowerShell.PSConsoleReadLine]

# Ctrl+S: Insert file path from current directory
Set-PSReadLineKeyHandler -Chord 'Ctrl+s' -ScriptBlock {
    $ps::InsertLineBelow()

    # Find files with fallback: fd -> rg -> Get-ChildItem
    if (Get-Command fd -ErrorAction SilentlyContinue) {
        $choices = $(fd --type f --color=never | hs)
    } elseif (Get-Command rg -ErrorAction SilentlyContinue) {
        $choices = $(rg --files | hs)
    } else {
        $choices = $(Get-ChildItem -Recurse -File | Select-Object -ExpandProperty FullName | hs)
    }

    $ps::Undo()
    $ps::InvokePrompt()
    if ($choices) {
        $ps::Insert($choices -join " ")
    }
}

# Ctrl+R: Fuzzy history search
Set-PSReadLineKeyHandler -Chord Ctrl+r -BriefDescription 'hs history' -ScriptBlock {
    $histFile = (Get-PSReadLineOption).HistorySavePath
    if (-not (Test-Path $histFile)) { return }

    $lines = (Get-Content -Path $histFile -Raw) -split '\r?\n'
    [array]::Reverse($lines)

    $seen  = [System.Collections.Generic.HashSet[string]]::new()
    $dedup = foreach ($l in $lines) { if ($l -and $seen.Add($l)) { $l } }

    $ps::InsertLineBelow()
    $selection = $dedup | & hs --filter-only
    if ($LASTEXITCODE -eq 0 -and $selection) {
        $ps::RevertLine()
        $ps::InvokePrompt()
        $ps::Insert($selection.Trim())
    } else {
        $ps::Undo()
        $ps::InvokePrompt()
    }
}
