@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0check_windows_host_loop.ps1" %*
