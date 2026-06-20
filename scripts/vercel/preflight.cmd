@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0preflight.ps1" %*
exit /b %errorlevel%
