@echo on

choco push *.nupkg --source https://push.chocolatey.org/ || exit /b 1
