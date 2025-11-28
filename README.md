[![Linux Build Status](https://app.travis-ci.com/rschmitt/heatseeker.svg?branch=master)](https://app.travis-ci.com/rschmitt/heatseeker)
[![Windows Build Status](https://ci.appveyor.com/api/projects/status/github/rschmitt/heatseeker?svg=true)](https://ci.appveyor.com/project/rschmitt/heatseeker)

![ps-readline-demo](https://cloud.githubusercontent.com/assets/3725049/8273451/0ac04144-1824-11e5-8338-99e4b861c898.gif)

Heatseeker is a fast, portable fuzzy finder that emphasizes speed and simplicity.

## Installation

The recommended way to install is through Cargo:

```sh
cargo install heatseeker
```

To install on Windows using [Chocolatey](https://chocolatey.org/), run:

```powershell
choco install heatseeker
```

To install on OS X using [Homebrew](http://brew.sh/), run:

```sh
brew tap rschmitt/heatseeker
brew install heatseeker
```

## Use

Heatseeker's usage is mostly intuitive, but there are a few commands worth knowing:

* `^T` (that is, Control-T) to select or deselect the currently highlighted choice
* Enter to select the currently highlighted choice, *or* any matches previously highlighted with `^T`
* `^G`, `^C`, or Escape to quit without selecting a match
* Backspace to delete the last query character typed
* `^W` to delete the last word in the query
* `^U` to delete the entire query
* `^N`, down arrow, or Tab to highlight the next match
* `^P`, up arrow, or Shift-Tab to highlight the previous match
* `^B` or Page Up to move up by one page
* `^F` or Page Down to move down by one page
* Home/End to move to the first or last choice

### Shell integration

The shell integration adds the following commands:

* `^S` selects files to add to the current command.
* `^R` performs a history search.

#### Zsh

Add this to your `~/.zshrc`:

```sh
eval "$(hs shell zsh)"
```

Note that the default integration sets the `noflowcontrol` option in order to free up the `^S` binding.

#### Bash

Add this to your `~/.bashrc`:

```bash
eval "$(hs shell bash)"
```

This requires a modern version of bash; the ancient `/bin/bash` that ships with macOS is unsupported. Additionally, note that the default integration disables flow control in order to free up the `^S` binding.

#### PowerShell

Add this to your `$profile`:

```powershell
(&hs shell pwsh) | Out-String | Invoke-Expression
```

Be sure to add it after any other readline configuration, such as `Set-PSReadlineOption -EditMode Emacs`, which will overwrite Heatseeker's bindings.

#### Nushell

Add this to the end of your `config.nu`, which can be found by running `$nu.config-path` in Nushell:

```nu
mkdir ($nu.data-dir | path join "vendor/autoload")
hs shell nu | save -f ($nu.data-dir | path join "vendor/autoload/hs.nu")
```

#### Fish

Add this to your `~/.config/fish/config.fish`:

```fish
hs shell fish | source
```

### Vim integration

The built-in plugin supports both Vim and Neovim:

```vim
Plug 'rschmitt/heatseeker'
```

![vim-demo](https://cloud.githubusercontent.com/assets/3725049/8273517/2a2f9afa-1826-11e5-9e1e-a15e84751bd0.gif)

This plugin adds the following key bindings:

* `<leader>f` to open one or more files. Multiple files will be opened in tabs.
  * The `<leader>` key defaults to `\`, but people frequently change it to `,`:
    ```
    let g:mapleader = ","
    ```
* `<leader>b` to select a buffer to open.
* `^G` to take the identifier currently under the cursor and select files to open containing that string.

## Project Information

Heatseeker has been actively used and maintained for over ten years. It is considered feature-complete. Development focuses on general maintenance, integration support, bugfixes, and portability improvements.

Heatseeker originated as a Rust rewrite of Gary Bernhardt's [selecta](https://github.com/garybernhardt/selecta) with the goal of improving speed and portability.

## Building

Heatseeker uses a typical Cargo build. Perform the build by invoking:

```
$ cargo build --release
```

The resulting binary will be located in the `target/release` directory. Alternatively, you can install from the repository by running:

```
cargo install --path . --locked
```

The unit tests can be invoked by running:

```
$ cargo test
```

Finally, a debug build can be produced by running:

```
$ cargo build
```

Debug builds of Heatseeker write a special log file, `heatseeker-debug.log`, to the current working directory. This can be used to debug issues with things like input decoding and signal handling.
