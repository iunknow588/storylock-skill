@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0check_device_loop.ps1" %*
