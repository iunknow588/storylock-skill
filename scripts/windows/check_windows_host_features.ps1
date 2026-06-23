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
  cargo check --no-default-features
  Write-Output "Windows host Slint UI checks passed."
}
finally {
  Pop-Location
}
