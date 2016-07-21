[![Linux Build Status](https://travis-ci.org/rschmitt/heatseeker.svg)](https://travis-ci.org/rschmitt/heatseeker)
[![Windows Build Status](https://ci.appveyor.com/api/projects/status/github/rschmitt/heatseeker?svg=true)](https://ci.appveyor.com/project/rschmitt/heatseeker)

Heatseeker is a rewrite of Gary Bernhardt's
[selecta](https://github.com/garybernhardt/selecta), a general-purpose fuzzy selector. It looks like this:

![ps-readline-demo](https://cloud.githubusercontent.com/assets/3725049/8273451/0ac04144-1824-11e5-8338-99e4b861c898.gif)

The purpose of the rewrite is to combine the simplicity and generality of Selecta with the speed and portability of native code.

## Installation

Compiled binaries for the latest version can be downloaded [from GitHub](https://github.com/rschmitt/heatseeker/releases/latest).

To install on Windows using [Chocolatey](https://chocolatey.org/), run:

```shell
choco install heatseeker
```

To install on OS X using [Homebrew](http://brew.sh/), run:

```shell
brew install https://raw.githubusercontent.com/rschmitt/heatseeker/master/heatseeker.rb
```

To install on Linux, run:

```shell
wget -q -O hs https://github.com/rschmitt/heatseeker/releases/download/v1.4.0/hs-linux && sudo install hs /usr/local/bin/
```

Or install it in your home directory instead by running:

```shell
wget -q -O hs https://github.com/rschmitt/heatseeker/releases/download/v1.4.0/hs-linux && install -D hs ~/bin/hs
```

## Use

Heatseeker supports basically the same keys as Selecta, along with a few others to support multi-select:

* ^T to select or deselect the currently highlighted match
* Enter to select the currently highlighted match, *or* any matches previously highlighted with ^T
* ^G to quit without selecting a match
* Backspace to delete the last query character typed
* ^U to delete the entire query
* ^N, down arrow, or tab to highlight the next match
* ^P or up arrow to highlight the previous match

### PowerShell

With [PSReadLine](https://github.com/lzybkr/PSReadLine), Heatseeker can be integrated directly into the Windows command line. Add this code to your `$profile`. The file selector can be summoned with Ctrl-S.

```posh
$ps = $null
try {
    # On Windows 10, PSReadLine ships with PowerShell
    $ps = [Microsoft.PowerShell.PSConsoleReadline]
} catch [Exception] {
    # Otherwise, it can be installed from the PowerShell Gallery:
    # https://github.com/lzybkr/PSReadLine#installation
    Import-Module PSReadLine
    $ps = [PSConsoleUtilities.PSConsoleReadLine]
}

Set-PSReadlineKeyHandler `
     -Chord 'Ctrl+s' `
     -BriefDescription "InsertHeatseekerPathInCommandLine" `
     -LongDescription "Run Heatseeker in the PWD, appending any selected paths to the current command" `
     -ScriptBlock {
         $choices = $(Get-ChildItem -Name -Attributes !D -Recurse | hs)
         $ps::Insert($choices -join " ")
    }
```

### Vim

With a bit of Vimscript, you can use Heatseeker to open files in Vim, without any need for a special plugin.

![vim-demo](https://cloud.githubusercontent.com/assets/3725049/8273517/2a2f9afa-1826-11e5-9e1e-a15e84751bd0.gif)

The Vimscript [samples](https://github.com/garybernhardt/selecta) from the Selecta README basically work, but it is preferable to modify them for use with Heatseeker in order to add support for Windows and multi-select.

```vim
function! HeatseekerCommand(choice_command, hs_args, first_command, rest_command)
    try
        let selections = system(a:choice_command . " | hs " . a:hs_args)
    catch /Vim:Interrupt/
        redraw!
        return
    endtry
    redraw!
    let first = 1
    for selection in split(selections, "\n")
        if first
            exec a:first_command . " " . selection
            let first = 0
        else
            exec a:rest_command . " " . selection
        endif
    endfor
endfunction

if has('win32')
    nnoremap <leader>f :call HeatseekerCommand("dir /a-d /s /b", "", ':e', ':tabe')<CR>
else
    nnoremap <leader>f :call HeatseekerCommand("find . ! -path '*/.git/*' -type f -follow", "", ':e', ':tabe')<cr>
endif
```

The same goes for buffer selection. This is a bit trickier on Windows, because the most straightforward way to send the list of buffers to Heatseeker is to write a temp file.

```posh
function! HeatseekerBuffer()
    let bufnrs = filter(range(1, bufnr("$")), 'buflisted(v:val)')
    let buffers = map(bufnrs, 'bufname(v:val)')
    let named_buffers = filter(buffers, '!empty(v:val)')
    if has('win32')
        let filename = tempname()
        call writefile(named_buffers, filename)
        call HeatseekerCommand("type " . filename, "", ":b", ":b")
        silent let _ = system("del " . filename)
    else
        call HeatseekerCommand('echo "' . join(named_buffers, "\n") . '"', "", ":b", ":b")
    endif
endfunction

" Fuzzy select a buffer. Open the selected buffer with :b.
nnoremap <leader>b :call HeatseekerBuffer()<cr>
```

## Project Status

* Heatseeker is fully implemented. It works smoothly on all supported platforms, including Windows; it has even been successfully smoke tested (both building and running) on Windows 10 Technical Preview.
* Heatseeker requires no unstable language features and can be compiled with the stable Rust toolchain (currently version 1.2.0).
* Heatseeker contains a fully working implementation of multi-threaded matching, but because it depends on an unstable feature (scoped threads) it is disabled by default. Since Heatseeker is extremely fast even with a single thread, this is not a big deal.
* In a few places in the Heatseeker code, there are workarounds to avoid the use of experimental features, such as libc, scoped, collections, and old_io. As Rust matures, these workarounds will be eliminated.

## Building

Building Heatseeker requires Rust 1.2.0 stable or later. Perform the build by invoking:

```
$ cargo build --release
```

The resulting binary will be located in the `target/release` directory. (Note that omitting the `--release` flag will cause compiler optimizations to be skipped; this speeds up compilation but results in a remarkably sluggish program.) The unit tests can be invoked by running:

```
$ cargo test
```
