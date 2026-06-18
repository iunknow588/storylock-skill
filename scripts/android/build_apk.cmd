@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build_apk.ps1" %*
