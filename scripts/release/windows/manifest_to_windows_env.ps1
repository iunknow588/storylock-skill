param(
  [Parameter(Mandatory = $true)]
  [string]$ManifestPath,
  [string]$EnvOutput = "",
  [string]$PreferredKind = "zip",
  [string]$PublicDownloadUrl = ""
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
if ([string]::IsNullOrWhiteSpace($EnvOutput)) {
  $EnvOutput = Join-Path $repoRoot ".temp\vercel\windows-package.env"
}

if (-not (Test-Path -LiteralPath $ManifestPath)) {
  throw "Manifest file was not found: $ManifestPath"
}

$manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
if (-not $manifest.artifacts -or $manifest.artifacts.Count -eq 0) {
  throw "Manifest does not contain any artifacts: $ManifestPath"
}

$artifact = $manifest.artifacts | Where-Object { $_.Kind -eq $PreferredKind } | Select-Object -First 1
if (-not $artifact) {
  $artifact = $manifest.artifacts | Select-Object -First 1
}

$envDir = Split-Path -Parent $EnvOutput
if (-not [string]::IsNullOrWhiteSpace($envDir)) {
  New-Item -ItemType Directory -Force -Path $envDir | Out-Null
}

$kind = [string]$artifact.Kind
$path = [string]$artifact.Path
$sizeBytes = [string]$artifact.SizeBytes
$sha256 = [string]$artifact.Sha256
$version = [string]$manifest.version
$versionCode = [string]$manifest.versionCode
$releaseChannel = [string]$manifest.releaseChannel

$envLines = @(
  "STORYLOCK_WINDOWS_PACKAGE_PATH=$path"
  "STORYLOCK_WINDOWS_PACKAGE_VERSION=$version"
  "STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE=$versionCode"
  "STORYLOCK_WINDOWS_PACKAGE_SIZE_BYTES=$sizeBytes"
  "STORYLOCK_WINDOWS_PACKAGE_CHECKSUM=sha256:$sha256"
  "STORYLOCK_WINDOWS_PACKAGE_KIND=$kind"
  "STORYLOCK_WINDOWS_RELEASE_CHANNEL=$releaseChannel"
  "STORYLOCK_WINDOWS_APP_DOWNLOAD_URL=$PublicDownloadUrl"
)
Set-Utf8NoBomContent -LiteralPath $EnvOutput -Value $envLines

$packageOutputScript = Join-Path $repoRoot "scripts\vercel\write_package_output.mjs"
if (Test-Path -LiteralPath $packageOutputScript) {
  node $packageOutputScript windows $EnvOutput (Join-Path $repoRoot ".temp\vercel\output.json")
}

Write-Output "Windows env file generated from manifest."
Write-Output "Manifest: $ManifestPath"
Write-Output "Env file: $EnvOutput"
Write-Output "Artifact kind: $kind"
