param(
  [string]$ManifestPath = "",
  [string]$EnvPath = "",
  [string]$BaseUrl = "",
  [string]$DownloadStatusUrl = "",
  [string]$MetadataUrl = "",
  [string]$Platform = "windows",
  [string]$PreferredKind = "zip",
  [switch]$SkipMetadataRequest
)

$ErrorActionPreference = "Stop"

function Get-MetadataFileName {
  param([string]$FileName)
  if ([string]::IsNullOrWhiteSpace($FileName)) {
    return ""
  }
  if ($FileName -match "\.tar\.gz$") {
    return ($FileName -replace "\.tar\.gz$", "-tar-gz.json")
  }
  return ($FileName -replace "\.([^.]+)$", '-$1.json')
}

function Import-SimpleEnvFile {
  param([string]$LiteralPath)
  $values = @{}
  if ([string]::IsNullOrWhiteSpace($LiteralPath) -or -not (Test-Path -LiteralPath $LiteralPath)) {
    return $values
  }
  foreach ($rawLine in Get-Content -LiteralPath $LiteralPath) {
    $line = $rawLine.Trim()
    if (-not $line -or $line.StartsWith("#")) {
      continue
    }
    $parts = $line -split "=", 2
    if ($parts.Count -ne 2) {
      continue
    }
    $values[$parts[0].Trim()] = $parts[1].Trim()
  }
  return $values
}

function Get-LocalReleaseInfo {
  param(
    [string]$ManifestPath,
    [string]$EnvPath,
    [string]$PreferredKind
  )

  if (-not [string]::IsNullOrWhiteSpace($ManifestPath)) {
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
    $artifactPath = [string]$artifact.Path
    return [PSCustomObject]@{
      Source = "manifest"
      Version = [string]$manifest.version
      VersionCode = [string]$manifest.versionCode
      ReleaseChannel = [string]$manifest.releaseChannel
      PackageKind = [string]$artifact.Kind
      FileName = Split-Path -Leaf $artifactPath
      FileSizeBytes = [string]$artifact.SizeBytes
      Checksum = "sha256:{0}" -f ([string]$artifact.Sha256).ToLowerInvariant()
      ArtifactPath = $artifactPath
    }
  }

  if (-not [string]::IsNullOrWhiteSpace($EnvPath)) {
    if (-not (Test-Path -LiteralPath $EnvPath)) {
      throw "Env file was not found: $EnvPath"
    }
    $envValues = Import-SimpleEnvFile -LiteralPath $EnvPath
    $artifactPath = [string]$envValues["STORYLOCK_WINDOWS_PACKAGE_PATH"]
    return [PSCustomObject]@{
      Source = "env"
      Version = [string]$envValues["STORYLOCK_WINDOWS_PACKAGE_VERSION"]
      VersionCode = [string]$envValues["STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE"]
      ReleaseChannel = [string]$envValues["STORYLOCK_WINDOWS_RELEASE_CHANNEL"]
      PackageKind = [string]$envValues["STORYLOCK_WINDOWS_PACKAGE_KIND"]
      FileName = if ([string]::IsNullOrWhiteSpace($artifactPath)) { "" } else { Split-Path -Leaf $artifactPath }
      FileSizeBytes = [string]$envValues["STORYLOCK_WINDOWS_PACKAGE_SIZE_BYTES"]
      Checksum = ([string]$envValues["STORYLOCK_WINDOWS_PACKAGE_CHECKSUM"]).ToLowerInvariant()
      ArtifactPath = $artifactPath
    }
  }

  throw "Provide either -ManifestPath or -EnvPath."
}

function Get-RemoteReleaseInfo {
  param(
    [string]$BaseUrl,
    [string]$DownloadStatusUrl,
    [string]$MetadataUrl,
    [string]$Platform,
    [string]$LocalFileName,
    [switch]$SkipMetadataRequest
  )

  $normalizedBaseUrl = ""
  if (-not [string]::IsNullOrWhiteSpace($BaseUrl)) {
    $normalizedBaseUrl = $BaseUrl.TrimEnd("/")
  }

  if ([string]::IsNullOrWhiteSpace($DownloadStatusUrl) -and -not [string]::IsNullOrWhiteSpace($normalizedBaseUrl)) {
    $DownloadStatusUrl = "$normalizedBaseUrl/app/download"
  }

  $platformStatus = $null
  if (-not [string]::IsNullOrWhiteSpace($DownloadStatusUrl)) {
    $downloadStatus = Invoke-RestMethod -Uri $DownloadStatusUrl -TimeoutSec 30 -ErrorAction Stop
    $platformStatus = $downloadStatus.platforms.$Platform
    if ($null -eq $platformStatus) {
      throw "Platform '$Platform' was not found in download status: $DownloadStatusUrl"
    }
  }

  $remoteFileName = ""
  if ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.fileName)) {
    $remoteFileName = [string]$platformStatus.fileName
  } elseif (-not [string]::IsNullOrWhiteSpace($LocalFileName)) {
    $remoteFileName = $LocalFileName
  }

  if ([string]::IsNullOrWhiteSpace($MetadataUrl) -and -not [string]::IsNullOrWhiteSpace($normalizedBaseUrl) -and -not [string]::IsNullOrWhiteSpace($remoteFileName)) {
    $MetadataUrl = "$normalizedBaseUrl/downloads/$(Get-MetadataFileName -FileName $remoteFileName)"
  }

  $metadata = $null
  if (-not $SkipMetadataRequest -and -not [string]::IsNullOrWhiteSpace($MetadataUrl)) {
    $metadata = Invoke-RestMethod -Uri $MetadataUrl -TimeoutSec 30 -ErrorAction Stop
  }

  $checksum = ""
  if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.checksum)) {
    $checksum = ([string]$metadata.checksum).ToLowerInvariant()
  } elseif ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.checksum)) {
    $checksum = ([string]$platformStatus.checksum).ToLowerInvariant()
  }

  $fileSizeBytes = ""
  if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.fileSizeBytes)) {
    $fileSizeBytes = [string]$metadata.fileSizeBytes
  } elseif ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.fileSizeBytes)) {
    $fileSizeBytes = [string]$platformStatus.fileSizeBytes
  }

  $version = ""
  if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.versionName)) {
    $version = [string]$metadata.versionName
  } elseif ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.versionName)) {
    $version = [string]$platformStatus.versionName
  }

  $versionCode = ""
  if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.versionCode)) {
    $versionCode = [string]$metadata.versionCode
  } elseif ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.versionCode)) {
    $versionCode = [string]$platformStatus.versionCode
  }

  $releaseChannel = ""
  if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.releaseChannel)) {
    $releaseChannel = [string]$metadata.releaseChannel
  } elseif ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.releaseChannel)) {
    $releaseChannel = [string]$platformStatus.releaseChannel
  }

  $packageKind = ""
  if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.packageKind)) {
    $packageKind = [string]$metadata.packageKind
  } elseif ($platformStatus -and -not [string]::IsNullOrWhiteSpace([string]$platformStatus.packageKind)) {
    $packageKind = [string]$platformStatus.packageKind
  }

  return [PSCustomObject]@{
    DownloadStatusUrl = $DownloadStatusUrl
    MetadataUrl = $MetadataUrl
    Version = $version
    VersionCode = $versionCode
    ReleaseChannel = $releaseChannel
    PackageKind = $packageKind
    FileName = if ($metadata -and -not [string]::IsNullOrWhiteSpace([string]$metadata.fileName)) { [string]$metadata.fileName } else { $remoteFileName }
    FileSizeBytes = $fileSizeBytes
    Checksum = $checksum
    DownloadUrl = if ($platformStatus) { [string]$platformStatus.downloadUrl } else { "" }
  }
}

function Compare-ReleaseField {
  param(
    [string]$Field,
    [string]$LocalValue,
    [string]$RemoteValue
  )

  $normalizedLocal = if ($null -eq $LocalValue) { "" } else { [string]$LocalValue }
  $normalizedRemote = if ($null -eq $RemoteValue) { "" } else { [string]$RemoteValue }
  [PSCustomObject]@{
    Field = $Field
    Local = $normalizedLocal
    Remote = $normalizedRemote
    Status = if ($normalizedLocal -eq $normalizedRemote) { "ok" } else { "mismatch" }
  }
}

$local = Get-LocalReleaseInfo -ManifestPath $ManifestPath -EnvPath $EnvPath -PreferredKind $PreferredKind
$remote = Get-RemoteReleaseInfo `
  -BaseUrl $BaseUrl `
  -DownloadStatusUrl $DownloadStatusUrl `
  -MetadataUrl $MetadataUrl `
  -Platform $Platform `
  -LocalFileName $local.FileName `
  -SkipMetadataRequest:$SkipMetadataRequest

$rows = @(
  Compare-ReleaseField -Field "version" -LocalValue $local.Version -RemoteValue $remote.Version
  Compare-ReleaseField -Field "versionCode" -LocalValue $local.VersionCode -RemoteValue $remote.VersionCode
  Compare-ReleaseField -Field "releaseChannel" -LocalValue $local.ReleaseChannel -RemoteValue $remote.ReleaseChannel
  Compare-ReleaseField -Field "packageKind" -LocalValue $local.PackageKind -RemoteValue $remote.PackageKind
  Compare-ReleaseField -Field "fileName" -LocalValue $local.FileName -RemoteValue $remote.FileName
  Compare-ReleaseField -Field "fileSizeBytes" -LocalValue $local.FileSizeBytes -RemoteValue $remote.FileSizeBytes
  Compare-ReleaseField -Field "checksum" -LocalValue $local.Checksum -RemoteValue $remote.Checksum
)

Write-Host "Windows release consistency check" -ForegroundColor Cyan
Write-Host ("Local source: {0}" -f $local.Source)
if (-not [string]::IsNullOrWhiteSpace($local.ArtifactPath)) {
  Write-Host ("Local artifact: {0}" -f $local.ArtifactPath)
}
if (-not [string]::IsNullOrWhiteSpace($remote.DownloadStatusUrl)) {
  Write-Host ("Remote status: {0}" -f $remote.DownloadStatusUrl)
}
if (-not [string]::IsNullOrWhiteSpace($remote.MetadataUrl)) {
  Write-Host ("Remote metadata: {0}" -f $remote.MetadataUrl)
}
if (-not [string]::IsNullOrWhiteSpace($remote.DownloadUrl)) {
  Write-Host ("Remote artifact: {0}" -f $remote.DownloadUrl)
}
Write-Host ""
$rows | Format-Table -AutoSize

$mismatches = @($rows | Where-Object { $_.Status -ne "ok" })
if ($mismatches.Count -gt 0) {
  throw "Release consistency check failed: $($mismatches.Count) field(s) differ between local and remote."
}

Write-Host ""
Write-Host "Release consistency check passed." -ForegroundColor Green
