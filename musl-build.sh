#!/bin/bash

set -eux

cargo clean
cargo build --release --target=x86_64-unknown-linux-gnu
perl -i -pe 's/^.*build\.rs.*$//' Cargo.toml
cargo update
perl -i -pe 's/-gnu/-musl/' src/version.rs

RUSTC=./musl-rustc cargo rustc --release --target=x86_64-unknown-linux-musl -- -C link-args=/usr/lib/x86_64-linux-gnu/liblzma.a -C linker=/usr/local/musl/bin/musl-gcc

git checkout Cargo.toml
