#!/bin/bash

set -euo pipefail

version="$(git tag --points-at HEAD)"
if [[ -z "$version" ]]; then
    version="$(git rev-parse HEAD | cut -c 1-7)"
fi

cargo build --release --target=aarch64-apple-darwin
cargo build --release --target=x86_64-apple-darwin
lipo -create -output hs target/aarch64-apple-darwin/release/hs target/x86_64-apple-darwin/release/hs
tar -cf "heatseeker-$version-universal-apple-darwin.tar" hs
gzip "heatseeker-$version-universal-apple-darwin.tar"
