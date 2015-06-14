@echo on

cargo clean
cargo test
cargo build --release

mkdir chocolatey
cp heatseeker.nuspec chocolatey\

mkdir chocolatey\tools
cp LICENSE chocolatey\tools\
cp README.md chocolatey\tools\README

mkdir chocolatey\tools\bin
cp target\release\hs.exe chocolatey\tools\bin

cpack chocolatey\heatseeker.nuspec
