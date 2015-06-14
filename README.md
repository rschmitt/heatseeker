[![Build Status](http://img.shields.io/travis/rschmitt/heatseeker.svg?label=Linux%20build)](https://travis-ci.org/rschmitt/heatseeker)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/github/rschmitt/heatseeker?svg=true)](https://ci.appveyor.com/project/rschmitt/heatseeker)

# Heatseeker

Heatseeker is a rewrite of Gary Bernhardt's
[selecta](https://github.com/garybernhardt/selecta), a fuzzy selector. The project has the following goals:

* Produce a drop-in replacement for Selecta
* Be as fast as possible (for usability with a large number of choices)
* Support Windows

## Installation

Compiled binaries for the latest version can be downloaded [from GitHub](https://github.com/rschmitt/heatseeker/releases/tag/v0.3.0).

To install on OS X using [Homebrew](http://brew.sh/), run:

```shell
brew install https://raw.githubusercontent.com/rschmitt/heatseeker/master/heatseeker.rb
```

## Project Status

* Heatseeker is fully implemented. It works smoothly on all supported platforms, including Windows; it has even been successfully smoke tested (both building and running) on Windows 10 Technical Preview.
* Heatseeker requires no unstable language features and can be compiled with the stable Rust toolchain (currently version 1.0.0).
* Heatseeker contains a fully working implementation of multi-threaded matching, but because it depends on an unstable feature (scoped threads) it is disabled by default. Since Heatseeker is extremely fast even with a single thread, this is not a big deal.
* In a few places in the Heatseeker code, there are workarounds to avoid the use of experimental features, such as libc, scoped, collections, and old_io. As Rust matures, these workarounds will be eliminated.

## Building

Building Heatseeker requires Rust 1.0.0 stable or later. On Windows, MinGW-w64 must also be installed to build some dependencies.

Perform the build by invoking:

```
$ cargo build --release
```

The resulting binary will be located in the `target/release` directory. (Note that omitting the `--release` flag will cause compiler optimizations to be skipped; this speeds up compilation but results in a remarkably sluggish program.) The unit tests can be invoked by running:

```
$ cargo test
```
