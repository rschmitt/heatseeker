@echo on

rd /s /q chocolatey
mkdir chocolatey || exit /b 1
copy heatseeker.nuspec chocolatey\ || exit /b 1

mkdir chocolatey\tools || exit /b 1
copy LICENSE chocolatey\tools\ || exit /b 1
copy VERIFICATION.txt chocolatey\tools\ || exit /b 1
copy README.md chocolatey\tools\README || exit /b 1

mkdir chocolatey\tools\bin || exit /b 1

wget -q https://github.com/rschmitt/heatseeker/releases/download/v1.6.0/heatseeker-v1.6.0-x86_64-pc-windows-msvc.zip || exit /b 1
7z x heatseeker-v1.6.0-x86_64-pc-windows-msvc.zip || exit /b 1
del heatseeker-v1.6.0-x86_64-pc-windows-msvc.zip
move hs.exe chocolatey\tools\bin || exit /b 1

choco pack chocolatey\heatseeker.nuspec || exit /b 1
