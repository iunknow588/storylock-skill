param(
  [string]$ProjectDir = "",
  [string]$OutputDir = "",
  [string]$EnvOutput = "",
  [string]$Version = "0.1.0",
  [string]$VersionCode = "1",
  [string]$ReleaseChannel = "prototype",
  [switch]$BuildMsi,
  [switch]$SignArtifacts
)

$ErrorActionPreference = "Stop"

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

$buildScript = Join-Path $PSScriptRoot "build_windows_host.ps1"
if (-not (Test-Path -LiteralPath $buildScript)) {
  throw "Build script not found: $buildScript"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

& $buildScript `
  -ProjectDir $ProjectDir `
  -OutputDir $OutputDir `
  -EnvOutput $EnvOutput `
  -Version $Version `
  -VersionCode $VersionCode `
  -ReleaseChannel $ReleaseChannel `
  -BuildMsi:$BuildMsi `
  -SignArtifacts:$SignArtifacts

$zipPath = Join-Path $OutputDir "yian-windows-host-$Version-$VersionCode-$ReleaseChannel.zip"
$msiPath = Join-Path $OutputDir "yian-windows-host-$Version-$VersionCode-$ReleaseChannel.msi"

$artifacts = [System.Collections.Generic.List[object]]::new()
if (Test-Path -LiteralPath $zipPath) {
  $zip = Get-Item -LiteralPath $zipPath
  $zipHash = Get-FileHash -LiteralPath $zip.FullName -Algorithm SHA256
  $artifacts.Add([PSCustomObject]@{
      Kind = "zip"
      Path = $zip.FullName
      SizeBytes = $zip.Length
      Sha256 = $zipHash.Hash.ToLowerInvariant()
    }) | Out-Null
}
if (Test-Path -LiteralPath $msiPath) {
  $msi = Get-Item -LiteralPath $msiPath
  $msiHash = Get-FileHash -LiteralPath $msi.FullName -Algorithm SHA256
  $artifacts.Add([PSCustomObject]@{
      Kind = "msi"
      Path = $msi.FullName
      SizeBytes = $msi.Length
      Sha256 = $msiHash.Hash.ToLowerInvariant()
    }) | Out-Null
}

$manifestPath = Join-Path $OutputDir "release-manifest-$Version-$VersionCode-$ReleaseChannel.json"
$manifest = [PSCustomObject]@{
  product = "yian-windows-host"
  version = $Version
  versionCode = $VersionCode
  releaseChannel = $ReleaseChannel
  signed = [bool]$SignArtifacts
  builtAt = (Get-Date).ToUniversalTime().ToString("o")
  artifacts = $artifacts
  envFile = (Resolve-Path -LiteralPath $EnvOutput).Path
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8

$manifestEnvScript = Join-Path $PSScriptRoot "manifest_to_windows_env.ps1"
if (Test-Path -LiteralPath $manifestEnvScript) {
  & $manifestEnvScript -ManifestPath $manifestPath -EnvOutput $EnvOutput
}

Write-Output "Windows host release flow completed."
Write-Output "Release manifest: $manifestPath"
$artifacts | Format-Table -AutoSize
