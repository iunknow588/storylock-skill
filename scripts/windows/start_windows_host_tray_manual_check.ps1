param(
  [string]$ProjectDir = (Join-Path $PSScriptRoot "..\..\src\host\windows-host"),
  [string]$DataDir = (Join-Path $PSScriptRoot "..\..\.temp\runtime\windows-host-tray-manual-data"),
  [int]$Port = 4510
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Cargo was not found. Install Rust from https://rustup.rs/ and rerun this script."
}

$project = Resolve-Path -LiteralPath $ProjectDir
$data = [System.IO.Path]::GetFullPath($DataDir)
New-Item -ItemType Directory -Force -Path $data | Out-Null

$listener = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
if ($listener) {
  throw "Port $Port is already in use. Stop the existing listener or choose another port."
}

$env:STORYLOCK_WINDOWS_DATA_DIR = $data
$env:STORYLOCK_WINDOWS_HOST_PORT = "$Port"

Write-Output "Starting Yian Windows Host tray manual check..."
Write-Output "Project: $project"
Write-Output "DataDir: $data"
Write-Output "Port: $Port"
Write-Output ""
Write-Output "Manual checks to record:"
Write-Output "1. Tray icon is visible after startup."
Write-Output "2. Open Local Management opens http://127.0.0.1:$Port/ui."
Write-Output "3. View Health opens http://127.0.0.1:$Port/health."
Write-Output "4. Copy Diagnostics places redacted diagnostics text on the clipboard."
Write-Output "5. Exit stops the host and releases port $Port."
Write-Output ""
Write-Output "Close the tray menu with Exit when the check is done."

Push-Location $project
try {
  cargo run --features ui-tray -- --tray
}
finally {
  Pop-Location
}
