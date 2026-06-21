@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "HOST_EXE=%SCRIPT_DIR%yian-windows-host.exe"

if not exist "%HOST_EXE%" (
  echo yian-windows-host.exe was not found next to this script.
  echo Please extract the full zip package first, then run this script again.
  pause
  exit /b 1
)

if "%STORYLOCK_GATEWAY_URL%"=="" set "STORYLOCK_GATEWAY_URL=https://yian.cdao.online"

echo Starting Yian Windows Host debug console...
echo.
echo Gateway: %STORYLOCK_GATEWAY_URL%
echo Local health: http://127.0.0.1:4510/health
echo Local management: http://127.0.0.1:4510/ui
echo.
if /I "%STORYLOCK_WINDOWS_START_MODE%"=="tray" (
  echo Starting in tray mode. Use the tray menu to open local UI, copy diagnostics, or exit.
  echo.
  "%HOST_EXE%" --tray
  exit /b %ERRORLEVEL%
)

echo Keep this window open while using the Windows local host.
echo Double-click yian-windows-host.exe for the desktop tray app.
echo This script is kept for debug logs and console troubleshooting.
echo Press Ctrl+C to stop it.
echo.

"%HOST_EXE%" --console
