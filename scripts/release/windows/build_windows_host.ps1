param(
  [string]$ProjectDir = "",
  [string]$OutputDir = "",
  [string]$EnvOutput = "",
  [string]$Version = "0.1.0",
  [string]$VersionCode = "1",
  [string]$ReleaseChannel = "prototype",
  [string]$PackageKind = "zip",
  [switch]$BuildMsi,
  [switch]$SignArtifacts
)

$ErrorActionPreference = "Stop"

function Set-Utf8NoBomContent {
  param(
    [Parameter(Mandatory = $true)]
    [string]$LiteralPath,
    [Parameter(Mandatory = $true)]
    [string[]]$Value
  )
  $text = ($Value -join "`n") + "`n"
  [System.IO.File]::WriteAllText($LiteralPath, $text, [System.Text.UTF8Encoding]::new($false))
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
if ([string]::IsNullOrWhiteSpace($ProjectDir)) {
  $ProjectDir = Join-Path $repoRoot "src\host\windows-host"
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
  $OutputDir = Join-Path $repoRoot "release\app\windows"
}
if ([string]::IsNullOrWhiteSpace($EnvOutput)) {
  $EnvOutput = Join-Path $repoRoot "scripts\vercel\.env.windows-package"
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Cargo was not found. Install Rust from https://rustup.rs/ and rerun this script."
}

$project = Resolve-Path -LiteralPath $ProjectDir
Push-Location $project
try {
  cargo build --release --features "ui-slint ui-tray"
} finally {
  Pop-Location
}

$exe = Join-Path $project "target\release\yian-windows-host.exe"
if (-not (Test-Path -LiteralPath $exe)) {
  throw "Windows host executable was not produced: $exe"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$version = $Version
$versionCode = $VersionCode
$releaseChannel = $ReleaseChannel
$packageKind = $PackageKind
$zipName = "yian-windows-host-$version-$versionCode-$releaseChannel.zip"
$zipPath = Join-Path $OutputDir $zipName
if (Test-Path -LiteralPath $zipPath) {
  Remove-Item -LiteralPath $zipPath -Force
}

$packageFiles = @(
  $exe
  (Join-Path $project "README.md")
  (Join-Path $project "start-yian-windows-host.cmd")
)
Compress-Archive -LiteralPath $packageFiles -DestinationPath $zipPath
$hash = Get-FileHash -LiteralPath $zipPath -Algorithm SHA256
$item = Get-Item -LiteralPath $zipPath

$msiPath = $null
$msiItem = $null
$msiHash = $null
if ($BuildMsi) {
  $wix = Get-Command wix -ErrorAction SilentlyContinue
  if (-not $wix) {
    Write-Warning "WiX CLI was not found. Skipping MSI build."
  } else {
    $installerDir = Join-Path $project "installer"
    $msiName = "yian-windows-host-$version-$versionCode-$releaseChannel.msi"
    $msiPath = Join-Path $OutputDir $msiName
    if (Test-Path -LiteralPath $msiPath) {
      Remove-Item -LiteralPath $msiPath -Force
    }
    Push-Location $installerDir
    try {
      & $wix.Source "build" ".\product.wxs" "-d" "ProductVersion=$version" "-o" $msiPath
    } finally {
      Pop-Location
    }
    if (-not (Test-Path -LiteralPath $msiPath)) {
      throw "WiX build did not produce MSI output: $msiPath"
    }
    $msiItem = Get-Item -LiteralPath $msiPath
    $msiHash = Get-FileHash -LiteralPath $msiPath -Algorithm SHA256
  }
}

if ($SignArtifacts) {
  $signScript = Join-Path $PSScriptRoot "sign_windows_package.ps1"
  if (-not (Test-Path -LiteralPath $signScript)) {
    throw "Signing script not found: $signScript"
  }
  & $signScript -FilePath $zipPath
  if ($msiPath) {
    & $signScript -FilePath $msiPath
  }
}

$envLines = @(
  "STORYLOCK_WINDOWS_PACKAGE_PATH=$($item.FullName)"
  "STORYLOCK_WINDOWS_PACKAGE_VERSION=$version"
  "STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE=$versionCode"
  "STORYLOCK_WINDOWS_PACKAGE_SIZE_BYTES=$($item.Length)"
  "STORYLOCK_WINDOWS_PACKAGE_CHECKSUM=sha256:$($hash.Hash.ToLowerInvariant())"
  "STORYLOCK_WINDOWS_PACKAGE_KIND=$packageKind"
  "STORYLOCK_WINDOWS_RELEASE_CHANNEL=$releaseChannel"
  "STORYLOCK_WINDOWS_MSI_PATH=$msiPath"
  "STORYLOCK_WINDOWS_MSI_SIZE_BYTES=$(if ($msiItem) { $msiItem.Length } else { '' })"
  "STORYLOCK_WINDOWS_MSI_CHECKSUM=$(if ($msiHash) { 'sha256:' + $msiHash.Hash.ToLowerInvariant() } else { '' })"
  "STORYLOCK_WINDOWS_MSI_UPGRADE_CODE=6F0A7D8B-7F59-4E6B-B4E8-0EAC6959B301"
)
Set-Utf8NoBomContent -LiteralPath $EnvOutput -Value $envLines

Write-Output "Windows host package: $($item.FullName)"
Write-Output "SHA-256: $($hash.Hash.ToLowerInvariant())"
if ($msiPath) {
  Write-Output "Windows host MSI: $msiPath"
  Write-Output "MSI SHA-256: $($msiHash.Hash.ToLowerInvariant())"
}
Write-Output "Env file: $EnvOutput"
