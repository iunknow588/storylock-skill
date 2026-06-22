param(
  [Parameter(Mandatory = $true)]
  [string]$ManifestPath,
  [string]$EnvOutput = "",
  [string]$PublicDownloadUrl = "",
  [string]$PublishOutputDir = "",
  [switch]$CopyArtifacts
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
if ([string]::IsNullOrWhiteSpace($EnvOutput)) {
  $EnvOutput = Join-Path $repoRoot ".temp\vercel\windows-package.publish.env"
}
if ([string]::IsNullOrWhiteSpace($PublishOutputDir)) {
  $PublishOutputDir = Join-Path $repoRoot ".temp\dist\windows-publish"
}

$manifestScript = Join-Path $PSScriptRoot "manifest_to_windows_env.ps1"
if (-not (Test-Path -LiteralPath $manifestScript)) {
  throw "Manifest adapter script not found: $manifestScript"
}
if (-not (Test-Path -LiteralPath $ManifestPath)) {
  throw "Manifest file was not found: $ManifestPath"
}

New-Item -ItemType Directory -Force -Path $PublishOutputDir | Out-Null

& $manifestScript -ManifestPath $ManifestPath -EnvOutput $EnvOutput -PublicDownloadUrl $PublicDownloadUrl

$manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
$envFilePath = (Resolve-Path -LiteralPath $EnvOutput).Path
$publishSummaryPath = Join-Path $PublishOutputDir ("publish-summary-{0}-{1}-{2}.json" -f $manifest.version, $manifest.versionCode, $manifest.releaseChannel)

$summary = [PSCustomObject]@{
  product = "yian-windows-host"
  version = $manifest.version
  versionCode = $manifest.versionCode
  releaseChannel = $manifest.releaseChannel
  publicDownloadUrl = $PublicDownloadUrl
  manifestPath = (Resolve-Path -LiteralPath $ManifestPath).Path
  envFilePath = $envFilePath
  artifactKinds = @($manifest.artifacts | ForEach-Object { $_.Kind })
  artifactPaths = @($manifest.artifacts | ForEach-Object { $_.Path })
  preparedAt = (Get-Date).ToUniversalTime().ToString("o")
}

$summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $publishSummaryPath -Encoding UTF8

$publishArtifactsDir = Join-Path $PublishOutputDir "artifacts"
New-Item -ItemType Directory -Force -Path $publishArtifactsDir | Out-Null

$uploadManifestPath = Join-Path $PublishOutputDir ("upload-manifest-{0}-{1}-{2}.json" -f $manifest.version, $manifest.versionCode, $manifest.releaseChannel)
$uploadItems = @()

foreach ($artifact in $manifest.artifacts) {
  $artifactPath = [string]$artifact.Path
  $artifactName = Split-Path -Leaf $artifactPath
  $publishPath = Join-Path $publishArtifactsDir $artifactName
  if ($CopyArtifacts) {
    Copy-Item -LiteralPath $artifactPath -Destination $publishPath -Force
  }
  $uploadItems += [PSCustomObject]@{
    kind = [string]$artifact.Kind
    sourcePath = $artifactPath
    publishPath = $publishPath
    targetFileName = $artifactName
    publicDownloadUrl = if ([string]::IsNullOrWhiteSpace($PublicDownloadUrl)) { "" } else { $PublicDownloadUrl }
  }
}

[PSCustomObject]@{
  product = "yian-windows-host"
  version = $manifest.version
  versionCode = $manifest.versionCode
  releaseChannel = $manifest.releaseChannel
  copiedArtifacts = [bool]$CopyArtifacts
  artifactsDirectory = $publishArtifactsDir
  uploadItems = $uploadItems
  generatedAt = (Get-Date).ToUniversalTime().ToString("o")
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $uploadManifestPath -Encoding UTF8

Write-Output "Windows publish preparation completed."
Write-Output "Manifest: $ManifestPath"
Write-Output "Env file: $envFilePath"
Write-Output "Publish summary: $publishSummaryPath"
Write-Output "Upload manifest: $uploadManifestPath"
