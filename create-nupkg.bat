@echo on

cargo test || exit /b 1
cargo build --release || exit /b 1

rd /s /q chocolatey
mkdir chocolatey || exit /b 1
copy heatseeker.nuspec chocolatey\ || exit /b 1

mkdir chocolatey\tools || exit /b 1
copy LICENSE chocolatey\tools\ || exit /b 1
copy README.md chocolatey\tools\README || exit /b 1

mkdir chocolatey\tools\bin || exit /b 1
copy target\release\hs.exe chocolatey\tools\bin || exit /b 1

choco pack chocolatey\heatseeker.nuspec || exit /b 1
