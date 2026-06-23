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
echo Starting the Slint desktop UI.
echo.

"%HOST_EXE%"
