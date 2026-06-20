@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0publish_site_release_wsl.ps1" %*
exit /b %errorlevel%
