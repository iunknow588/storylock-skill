param(
  [string]$ProjectDir = (Join-Path $PSScriptRoot "..\..\src\host\windows-host")
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Cargo was not found. Install Rust from https://rustup.rs/ and rerun this script."
}

$project = Resolve-Path -LiteralPath $ProjectDir

Push-Location $project
try {
  cargo check
  cargo check --features ui-slint
  cargo check --features ui-tray
  cargo check --features "ui-slint ui-tray"
  Write-Output "Windows host feature checks passed."
}
finally {
  Pop-Location
}
