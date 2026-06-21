@echo off
setlocal
set SCRIPT_DIR=%~dp0
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%start_windows_host_tray_manual_check.ps1" %*
exit /b %ERRORLEVEL%
