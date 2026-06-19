param(
  [string]$ProjectDir = (Join-Path $PSScriptRoot "..\..\windows-host"),
  [string]$OutputDir = (Join-Path $PSScriptRoot "..\..\dist"),
  [string]$EnvOutput = (Join-Path $PSScriptRoot "..\vercel\.env.windows-package")
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Cargo was not found. Install Rust from https://rustup.rs/ and rerun this script."
}

$project = Resolve-Path -LiteralPath $ProjectDir
Push-Location $project
try {
  cargo build --release
} finally {
  Pop-Location
}

$exe = Join-Path $project "target\release\yian-windows-host.exe"
if (-not (Test-Path -LiteralPath $exe)) {
  throw "Windows host executable was not produced: $exe"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$version = "0.1.0"
$versionCode = "1"
$zipName = "yian-windows-host-$version-$versionCode-prototype.zip"
$zipPath = Join-Path $OutputDir $zipName
if (Test-Path -LiteralPath $zipPath) {
  Remove-Item -LiteralPath $zipPath -Force
}

Compress-Archive -LiteralPath $exe, (Join-Path $project "README.md") -DestinationPath $zipPath
$hash = Get-FileHash -LiteralPath $zipPath -Algorithm SHA256
$item = Get-Item -LiteralPath $zipPath

@(
  "STORYLOCK_WINDOWS_PACKAGE_PATH=$($item.FullName)"
  "STORYLOCK_WINDOWS_PACKAGE_VERSION=$version"
  "STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE=$versionCode"
  "STORYLOCK_WINDOWS_PACKAGE_SIZE_BYTES=$($item.Length)"
  "STORYLOCK_WINDOWS_PACKAGE_CHECKSUM=sha256:$($hash.Hash.ToLowerInvariant())"
  "STORYLOCK_WINDOWS_PACKAGE_KIND=zip"
  "STORYLOCK_WINDOWS_RELEASE_CHANNEL=prototype"
) | Set-Content -LiteralPath $EnvOutput -Encoding UTF8

Write-Output "Windows host package: $($item.FullName)"
Write-Output "SHA-256: $($hash.Hash.ToLowerInvariant())"
Write-Output "Env file: $EnvOutput"

