@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0sync_env_file_to_vercel.ps1" %*
exit /b %errorlevel%
