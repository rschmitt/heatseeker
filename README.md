[![Build Status](http://img.shields.io/travis/rschmitt/heatseeker.svg)](https://travis-ci.org/rschmitt/heatseeker)

# Heatseeker

Heatseeker is a rewrite of Gary Bernhardt's
[selecta](https://github.com/garybernhardt/selecta), a fuzzy selector. The project has the following goals:

* Produce a drop-in replacement for Selecta
* Be as fast as possible (for usability with a large number of choices)
* Support Windows

Building Heatseeker requires the latest Rust compiler (available [here](http://www.rust-lang.org/install.html)), as well as the Cargo build system and package manager (available [here](https://github.com/rust-lang/cargo)).

Perform the build by invoking:

```
$ cargo build --release
```

The resulting binary will be located in the `target/release` directory. (Note that omitting the `--release` flag will cause compiler optimizations to be skipped; this speeds up compilation but results in a remarkably sluggish program.) The unit tests can be invoked by running:

```
$ cargo test
```
