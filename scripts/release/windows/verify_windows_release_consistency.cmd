@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0verify_windows_release_consistency.ps1" %*
endlocal
