@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0release_windows_host.ps1" %*
