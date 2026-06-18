param(
  [string]$BaseUrl = "",
  [string]$EnvFile = (Join-Path $PSScriptRoot ".env"),
  [switch]$RequireDomain,
  [switch]$RequireApkSource
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
    [PSCustomObject]@{
      Check = "http:$Url"
      Status = if ($AllowedStatus -contains $status) { "ok" } else { "failed" }
      Value = if ($response) { "$status $($response.Headers['Content-Type'])" } else { $_.Exception.Message }
    }
  }
}

Import-EnvFile -Path $exampleEnvPath
Import-EnvFile -Path $EnvFile

$rows = @()
$rows += Test-Env -Name "VERCEL_PROJECT_NAME" -Required
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

$targetBaseUrl = if ($BaseUrl) {
  $BaseUrl.TrimEnd("/")
} else {
  $configuredBaseUrl = [System.Environment]::GetEnvironmentVariable("STORYLOCK_GATEWAY_PUBLIC_URL", "Process")
  if ($null -eq $configuredBaseUrl) {
    $configuredBaseUrl = ""
  }
  $configuredBaseUrl.TrimEnd("/")
}

if (-not [string]::IsNullOrWhiteSpace($targetBaseUrl)) {
  $rows += Test-Http -Url "$targetBaseUrl/" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/main.js" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/styles.css" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/api/storylock-gateway" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/android-host/bind" -AllowedStatus @(200)
  $rows += Test-Http -Url "$targetBaseUrl/download/android-host" -AllowedStatus @(200, 307, 302)
}

$rows | Format-Table -AutoSize

$failed = $rows | Where-Object { $_.Status -in @("missing", "failed", "unexpected_status") }
if ($failed) {
  throw "Preflight failed: $($failed.Count) check(s) need attention"
}

Write-Host "Yian Vercel preflight passed." -ForegroundColor Green
