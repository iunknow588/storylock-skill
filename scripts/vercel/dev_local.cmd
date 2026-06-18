@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0dev_local.ps1"
exit /b %errorlevel%
