param(
  [Parameter(Mandatory = $true)]
  [string]$UploadManifestPath,
  [string]$Provider = "s3-compatible",
  [string]$Bucket = "",
  [string]$Prefix = "downloads/windows-host",
  [string]$PublicBaseUrl = "",
  [string]$CliCommand = "aws",
  [string]$AwsProfile = "",
  [string]$AwsEndpointUrl = "",
  [string]$OutputDir = "",
  [switch]$Execute
)

$ErrorActionPreference = "Stop"
$scriptRoot = Split-Path -Parent $PSCommandPath

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
  $repoRoot = Resolve-Path (Join-Path $scriptRoot "..\..\..")
  $OutputDir = Join-Path $repoRoot ".temp\dist\windows-upload"
}

if (-not (Test-Path -LiteralPath $UploadManifestPath)) {
  throw "Upload manifest was not found: $UploadManifestPath"
}

$uploadManifest = Get-Content -LiteralPath $UploadManifestPath -Raw | ConvertFrom-Json
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$providerName = if ([string]::IsNullOrWhiteSpace($Provider)) { "s3-compatible" } else { $Provider.Trim() }
$bucketName = if ([string]::IsNullOrWhiteSpace($Bucket)) { "" } else { $Bucket.Trim() }
$keyPrefix = if ([string]::IsNullOrWhiteSpace($Prefix)) { "" } else { $Prefix.Trim().Trim("/") }
$baseUrl = if ([string]::IsNullOrWhiteSpace($PublicBaseUrl)) { "" } else { $PublicBaseUrl.Trim().TrimEnd("/") }

$planItems = @()
foreach ($item in $uploadManifest.uploadItems) {
  $sourcePath = [string]$item.sourcePath
  if (-not (Test-Path -LiteralPath $sourcePath)) {
    throw "Upload source file was not found: $sourcePath"
  }
  $targetFileName = [string]$item.targetFileName
  $objectKey = if ([string]::IsNullOrWhiteSpace($keyPrefix)) {
    $targetFileName
  } else {
    "$keyPrefix/$targetFileName"
  }
  $publicUrl = if ([string]::IsNullOrWhiteSpace($baseUrl)) {
    ""
  } else {
    "$baseUrl/$objectKey"
  }
  $commandPreview = if ($providerName -eq "s3-compatible" -and -not [string]::IsNullOrWhiteSpace($bucketName)) {
    "aws s3 cp `"$sourcePath`" `"s3://$bucketName/$objectKey`""
  } else {
    ""
  }
  $planItems += [PSCustomObject]@{
    kind = [string]$item.kind
    sourcePath = $sourcePath
    targetFileName = $targetFileName
    objectKey = $objectKey
    bucket = $bucketName
    publicUrl = $publicUrl
    provider = $providerName
    commandPreview = $commandPreview
  }
}

$planPath = Join-Path $OutputDir ("object-storage-upload-plan-{0}-{1}-{2}.json" -f $uploadManifest.version, $uploadManifest.versionCode, $uploadManifest.releaseChannel)
$plan = [PSCustomObject]@{
  product = [string]$uploadManifest.product
  version = [string]$uploadManifest.version
  versionCode = [string]$uploadManifest.versionCode
  releaseChannel = [string]$uploadManifest.releaseChannel
  provider = $providerName
  bucket = $bucketName
  prefix = $keyPrefix
  publicBaseUrl = $baseUrl
  execute = [bool]$Execute
  generatedAt = (Get-Date).ToUniversalTime().ToString("o")
  items = $planItems
}
$plan | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $planPath -Encoding UTF8

if (-not $Execute) {
  Write-Output "Object storage upload plan created."
  Write-Output "Plan: $planPath"
  Write-Output "No upload command was executed. Re-run with -Execute after filling provider settings."
  $planItems | Format-Table kind, targetFileName, objectKey, publicUrl -AutoSize
  return
}

if ($providerName -ne "s3-compatible") {
  throw "Only the s3-compatible execution skeleton is implemented. Provider '$providerName' currently supports plan generation only."
}
if ([string]::IsNullOrWhiteSpace($bucketName)) {
  throw "Bucket is required when -Execute is used."
}
if (-not (Get-Command $CliCommand -ErrorAction SilentlyContinue)) {
  throw "CLI command not found: $CliCommand"
}

foreach ($item in $planItems) {
  $args = @("s3", "cp", $item.sourcePath, "s3://$bucketName/$($item.objectKey)")
  if ($AwsProfile) {
    $args += @("--profile", $AwsProfile)
  }
  if ($AwsEndpointUrl) {
    $args += @("--endpoint-url", $AwsEndpointUrl)
  }
  Write-Output ("Uploading {0} -> s3://{1}/{2}" -f $item.sourcePath, $bucketName, $item.objectKey)
  & $CliCommand @args
}

Write-Output "Object storage upload skeleton execution completed."
Write-Output "Plan: $planPath"
