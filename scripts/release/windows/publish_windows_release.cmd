@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0publish_windows_release.ps1" %*
