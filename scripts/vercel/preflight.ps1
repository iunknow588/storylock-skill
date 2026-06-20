param(
  [string]$BaseUrl = "",
  [string]$EnvFile = (Join-Path $PSScriptRoot ".env"),
  [switch]$RequireDomain,
  [switch]$RequireApkSource,
  [switch]$SkipHttp
)

$ErrorActionPreference = "Stop"

$exampleEnvPath = Join-Path $PSScriptRoot ".env.example"

function Import-EnvFile {
  param([string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    return
  }
  Get-Content -LiteralPath $Path | ForEach-Object {
    $line = $_.Trim()
    if (-not $line -or $line.StartsWith("#")) {
      return
    }
    $parts = $line -split "=", 2
    if ($parts.Count -ne 2) {
      return
    }
    [System.Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1].Trim(), "Process")
  }
}

function Test-Env {
  param(
    [string]$Name,
    [switch]$Required
  )
  $value = [System.Environment]::GetEnvironmentVariable($Name, "Process")
  $ok = -not [string]::IsNullOrWhiteSpace($value)
  if ($Required -and -not $ok) {
    throw "$Name is required"
  }
  [PSCustomObject]@{
    Check = "env:$Name"
    Status = if ($ok) { "ok" } elseif ($Required) { "missing" } else { "optional" }
    Value = if ($Name -like "*SECRET*" -or $Name -like "*TOKEN*") { if ($ok) { "[set]" } else { "" } } else { $value }
  }
}

function Test-VercelProjectLink {
  param(
    [string]$ExpectedProjectName
  )
  $projectJsonPath = Join-Path (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path ".vercel\project.json"
  if (-not (Test-Path -LiteralPath $projectJsonPath)) {
    return [PSCustomObject]@{
      Check = "vercel:project-link"
      Status = "missing"
      Value = "Missing .vercel/project.json; run scripts\vercel\link_project.cmd from skill/"
    }
  }
  try {
    $project = Get-Content -Raw -LiteralPath $projectJsonPath | ConvertFrom-Json
    $actualProjectName = [string]$project.projectName
    $matchesExpected = [string]::IsNullOrWhiteSpace($ExpectedProjectName) -or $actualProjectName -eq $ExpectedProjectName
    return [PSCustomObject]@{
      Check = "vercel:project-link"
      Status = if ($matchesExpected) { "ok" } else { "failed" }
      Value = if ($matchesExpected) {
        "linked project: $actualProjectName"
      } else {
        "VERCEL_PROJECT_NAME='$ExpectedProjectName' but .vercel/project.json is linked to '$actualProjectName'"
      }
    }
  } catch {
    return [PSCustomObject]@{
      Check = "vercel:project-link"
      Status = "failed"
      Value = "Unable to parse .vercel/project.json: $($_.Exception.Message)"
    }
  }
}

function Test-Http {
  param(
    [string]$Url,
    [int[]]$AllowedStatus = @(200)
  )
  try {
    $response = Invoke-WebRequest -Uri $Url -UseBasicParsing -MaximumRedirection 0 -ErrorAction Stop
    $status = [int]$response.StatusCode
    [PSCustomObject]@{
      Check = "http:$Url"
      Status = if ($AllowedStatus -contains $status) { "ok" } else { "unexpected_status" }
      Value = "$status $($response.Headers['Content-Type'])"
    }
  } catch {
    $response = $_.Exception.Response
    $status = if ($response) { [int]$response.StatusCode } else { 0 }
    $body = ""
    if ($response) {
      try {
        $stream = $response.GetResponseStream()
        if ($stream) {
          $reader = [System.IO.StreamReader]::new($stream)
          $body = $reader.ReadToEnd()
          $reader.Dispose()
        }
      } catch {
        $body = ""
      }
    }
    $hint = if ($body -match '"code"\s*:\s*"NOT_FOUND"') {
      "Vercel NOT_FOUND; check project binding, outputDirectory, api/ function entry, and production deployment"
    } elseif ($status -eq 404 -and ($Url -match '/$' -or $Url -match '/(main\.js|styles\.css|api/storylock-gateway)$')) {
      "$status $($response.Headers['Content-Type']); deployment-level 404, check domain project binding, repo root, build outputDirectory, and latest production deployment"
    } elseif ($response) {
      "$status $($response.Headers['Content-Type'])"
    } else {
      $_.Exception.Message
    }
    [PSCustomObject]@{
      Check = "http:$Url"
      Status = if ($AllowedStatus -contains $status) { "ok" } else { "failed" }
      Value = $hint
    }
  }
}

function Test-JsonField {
  param(
    [string]$Url,
    [string]$Path,
    [switch]$Required
  )
  try {
    $value = Invoke-RestMethod -Uri $Url -TimeoutSec 30 -ErrorAction Stop
    $current = $value
    foreach ($segment in ($Path -split "\.")) {
      if ($null -eq $current) {
        break
      }
      $current = $current.$segment
    }
    $ok = $null -ne $current -and -not [string]::IsNullOrWhiteSpace([string]$current)
    [PSCustomObject]@{
      Check = "json:$Url#$Path"
      Status = if ($ok) { "ok" } elseif ($Required) { "missing" } else { "optional" }
      Value = if ($ok) { [string]$current } else { "" }
    }
  } catch {
    [PSCustomObject]@{
      Check = "json:$Url#$Path"
      Status = if ($Required) { "failed" } else { "optional" }
      Value = $_.Exception.Message
    }
  }
}

Import-EnvFile -Path $exampleEnvPath
Import-EnvFile -Path $EnvFile

$rows = @()
$rows += Test-Env -Name "VERCEL_PROJECT_NAME" -Required
$expectedProjectName = [System.Environment]::GetEnvironmentVariable("VERCEL_PROJECT_NAME", "Process")
$rows += Test-VercelProjectLink -ExpectedProjectName $expectedProjectName
$rows += Test-Env -Name "STORYLOCK_GATEWAY_PUBLIC_URL" -Required:$RequireDomain
$rows += Test-Env -Name "STORYLOCK_ANDROID_CONNECT_MODE" -Required
$rows += Test-Env -Name "STORYLOCK_ANDROID_DEEP_LINK" -Required
$rows += Test-Env -Name "STORYLOCK_ANDROID_REGISTRY_FILE" -Required
$rows += Test-Env -Name "STORYLOCK_ANDROID_SHARED_SECRET" -Required
$rows += Test-Env -Name "STORYLOCK_ANDROID_APK_PATH"
$rows += Test-Env -Name "STORYLOCK_ANDROID_APP_DOWNLOAD_URL"
$rows += Test-Env -Name "STORYLOCK_ANDROID_APK_VERSION"
$rows += Test-Env -Name "STORYLOCK_ANDROID_APK_VERSION_CODE"
$rows += Test-Env -Name "STORYLOCK_ANDROID_APK_CHECKSUM"
$rows += Test-Env -Name "STORYLOCK_WINDOWS_PACKAGE_PATH"
$rows += Test-Env -Name "STORYLOCK_WINDOWS_APP_DOWNLOAD_URL"
$rows += Test-Env -Name "STORYLOCK_WINDOWS_PACKAGE_VERSION"
$rows += Test-Env -Name "STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE"
$rows += Test-Env -Name "STORYLOCK_WINDOWS_PACKAGE_CHECKSUM"
$rows += Test-Env -Name "STORYLOCK_LINUX_PACKAGE_PATH"
$rows += Test-Env -Name "STORYLOCK_LINUX_PACKAGE_VERSION"
$rows += Test-Env -Name "STORYLOCK_LINUX_PACKAGE_VERSION_CODE"
$rows += Test-Env -Name "STORYLOCK_LINUX_PACKAGE_CHECKSUM"

$apkPath = [System.Environment]::GetEnvironmentVariable("STORYLOCK_ANDROID_APK_PATH", "Process")
$apkUrl = [System.Environment]::GetEnvironmentVariable("STORYLOCK_ANDROID_APP_DOWNLOAD_URL", "Process")
if ($RequireApkSource -and [string]::IsNullOrWhiteSpace($apkPath) -and [string]::IsNullOrWhiteSpace($apkUrl)) {
  throw "Either STORYLOCK_ANDROID_APK_PATH or STORYLOCK_ANDROID_APP_DOWNLOAD_URL is required"
}
if (-not [string]::IsNullOrWhiteSpace($apkPath)) {
  $resolved = Resolve-Path -LiteralPath $apkPath -ErrorAction SilentlyContinue
  $rows += [PSCustomObject]@{
    Check = "file:STORYLOCK_ANDROID_APK_PATH"
    Status = if ($resolved) { "ok" } else { "missing" }
    Value = $apkPath
  }
}

$windowsPackagePath = [System.Environment]::GetEnvironmentVariable("STORYLOCK_WINDOWS_PACKAGE_PATH", "Process")
if (-not [string]::IsNullOrWhiteSpace($windowsPackagePath)) {
  $resolved = Resolve-Path -LiteralPath $windowsPackagePath -ErrorAction SilentlyContinue
  $rows += [PSCustomObject]@{
    Check = "file:STORYLOCK_WINDOWS_PACKAGE_PATH"
    Status = if ($resolved) { "ok" } else { "missing" }
    Value = $windowsPackagePath
  }
}

$linuxPackagePath = [System.Environment]::GetEnvironmentVariable("STORYLOCK_LINUX_PACKAGE_PATH", "Process")
if (-not [string]::IsNullOrWhiteSpace($linuxPackagePath)) {
  $resolved = Resolve-Path -LiteralPath $linuxPackagePath -ErrorAction SilentlyContinue
  $rows += [PSCustomObject]@{
    Check = "file:STORYLOCK_LINUX_PACKAGE_PATH"
    Status = if ($resolved) { "ok" } else { "missing" }
    Value = $linuxPackagePath
  }
}

$targetBaseUrl = if ($BaseUrl) {
  $BaseUrl.TrimEnd("/")
} else {
  $configuredBaseUrl = [System.Environment]::GetEnvironmentVariable("STORYLOCK_GATEWAY_PUBLIC_URL", "Process")
  if ($null -eq $configuredBaseUrl) {
    $configuredBaseUrl = ""
  }
  $configuredBaseUrl.TrimEnd("/")
}

if (-not $SkipHttp -and -not [string]::IsNullOrWhiteSpace($targetBaseUrl)) {
  $windowsZipUrl = "$targetBaseUrl/downloads/yian-windows-host-0.1.0-1-prototype.zip"
  $windowsMetadataUrl = "$targetBaseUrl/downloads/yian-windows-host-0.1.0-1-prototype-zip.json"
  $linuxPackageUrl = "$targetBaseUrl/downloads/yian-linux-host-0.1.0-1-prototype.deb"
  $linuxMetadataUrl = "$targetBaseUrl/downloads/yian-linux-host-0.1.0-1-prototype-deb.json"
  $downloadStatusUrl = "$targetBaseUrl/app/download"
  $rows += Test-Http -Url "$targetBaseUrl/" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/main.js" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/styles.css" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/api/storylock-gateway" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/android-host/bind" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/download/android-host" -AllowedStatus @(200, 307, 302)
  $rows += Test-Http -Url "$targetBaseUrl/app/download/windows" -AllowedStatus @(200, 307, 302)
  $rows += Test-Http -Url "$targetBaseUrl/app/download/linux" -AllowedStatus @(200, 307, 302)
  $rows += Test-Http -Url "$targetBaseUrl/download/windows-host" -AllowedStatus @(200, 307, 302)
  $rows += Test-Http -Url "$targetBaseUrl/download/linux-host" -AllowedStatus @(200, 307, 302)
  $rows += Test-Http -Url $windowsZipUrl -AllowedStatus @(200)
  $rows += Test-Http -Url $windowsMetadataUrl -AllowedStatus @(200)
  $rows += Test-Http -Url $linuxPackageUrl -AllowedStatus @(200)
  $rows += Test-Http -Url $linuxMetadataUrl -AllowedStatus @(200)
  $rows += Test-JsonField -Url $windowsMetadataUrl -Path "checksum" -Required
  $rows += Test-JsonField -Url $windowsMetadataUrl -Path "fileSizeBytes" -Required
  $rows += Test-JsonField -Url $linuxMetadataUrl -Path "checksum" -Required
  $rows += Test-JsonField -Url $linuxMetadataUrl -Path "fileSizeBytes" -Required
  $rows += Test-JsonField -Url $downloadStatusUrl -Path "platforms.windows.checksum" -Required
  $rows += Test-JsonField -Url $downloadStatusUrl -Path "platforms.windows.fileSizeBytes" -Required
  $rows += Test-JsonField -Url $downloadStatusUrl -Path "platforms.linux.checksum" -Required
  $rows += Test-JsonField -Url $downloadStatusUrl -Path "platforms.linux.fileSizeBytes" -Required
}

$rows | Format-Table -AutoSize

if (-not $SkipHttp -and -not [string]::IsNullOrWhiteSpace($targetBaseUrl)) {
  $deployment404Checks = @(
    "http:$targetBaseUrl/",
    "http:$targetBaseUrl/main.js",
    "http:$targetBaseUrl/styles.css",
    "http:$targetBaseUrl/api/storylock-gateway"
  )
  $deployment404Rows = @($rows | Where-Object {
      $deployment404Checks -contains $_.Check -and
      $_.Status -eq "failed" -and
      (
        $_.Value -match "deployment-level 404" -or
        $_.Value -match "Vercel NOT_FOUND" -or
        $_.Value -match "^404\b"
      )
    })
  if ($deployment404Rows.Count -eq $deployment404Checks.Count) {
    Write-Host ""
    Write-Host "Deployment-level 404 detected for yian site." -ForegroundColor Yellow
    Write-Host "Local project binding and env checks ran before HTTP checks. If vercel:project-link is ok, verify that yian.cdao.online is bound to this Vercel project and that the latest production deployment was created from the skill/ directory with release/web/public as outputDirectory." -ForegroundColor Yellow
    Write-Host "Suggested command after confirming the domain owner project:" -ForegroundColor Yellow
    Write-Host "scripts\vercel\publish_site_release.cmd -Target vercel -Build -SiteHttpSmoke -Preflight -Prod -Execute" -ForegroundColor Yellow
    Write-Host ""
  }
}

$failed = $rows | Where-Object { $_.Status -in @("missing", "failed", "unexpected_status") }
if ($failed) {
  throw "Preflight failed: $($failed.Count) check(s) need attention"
}

if ($SkipHttp) {
  Write-Host "Yian Vercel local preflight passed. HTTP checks were skipped." -ForegroundColor Green
} else {
  Write-Host "Yian Vercel preflight passed." -ForegroundColor Green
}
