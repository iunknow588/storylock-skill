param(
  [string]$ApkPath = "",
  [string]$GatewayBaseUrl = "",
  [string]$DeepLink = "",
  [string]$ReportPath = (Join-Path ([System.IO.Path]::GetTempPath()) "android-device-loop-report.local.md")
)

$ErrorActionPreference = "Stop"

function Add-Result {
  param(
    [string]$Check,
    [string]$Status,
    [string]$Detail = ""
  )
  [PSCustomObject]@{
    Check = $Check
    Status = $Status
    Detail = $Detail
  }
}

$rows = @()
$adb = Get-Command adb -ErrorAction SilentlyContinue
if (-not $adb) {
  $rows += Add-Result "adb" "blocked" "adb was not found in PATH"
} else {
  $devices = & adb devices
  $deviceLines = $devices | Where-Object { $_ -match "\tdevice$" }
  $rows += Add-Result "adb devices" $(if ($deviceLines) { "ok" } else { "blocked" }) (($deviceLines -join "; ").Trim())
}

if ($ApkPath) {
  if (Test-Path -LiteralPath $ApkPath) {
    $rows += Add-Result "apk exists" "ok" $ApkPath
    if ($adb) {
      try {
        & adb install -r $ApkPath | Out-String | ForEach-Object {
          $rows += Add-Result "adb install" "ok" $_.Trim()
        }
      } catch {
        $rows += Add-Result "adb install" "failed" $_.Exception.Message
      }
    }
  } else {
    $rows += Add-Result "apk exists" "blocked" "$ApkPath not found"
  }
} else {
  $rows += Add-Result "apk exists" "skipped" "Pass -ApkPath to install a real APK"
}

if ($DeepLink -and $adb) {
  try {
    & adb shell am start -a android.intent.action.VIEW -d $DeepLink | Out-String | ForEach-Object {
      $rows += Add-Result "deep link" "ok" $_.Trim()
    }
  } catch {
    $rows += Add-Result "deep link" "failed" $_.Exception.Message
  }
} elseif ($DeepLink) {
  $rows += Add-Result "deep link" "blocked" "adb unavailable"
} else {
  $rows += Add-Result "deep link" "skipped" "Pass -DeepLink from /android-host/bind"
}

if ($GatewayBaseUrl) {
  $base = $GatewayBaseUrl.TrimEnd("/")
  foreach ($path in @("/", "/api/storylock-gateway", "/android-host/bind")) {
    try {
      $response = Invoke-WebRequest -Uri "$base$path" -UseBasicParsing -MaximumRedirection 0 -ErrorAction Stop
      $rows += Add-Result "gateway $path" "ok" "$($response.StatusCode) $($response.Headers['Content-Type'])"
    } catch {
      $response = $_.Exception.Response
      $detail = if ($response) { "$([int]$response.StatusCode) $($response.Headers['Content-Type'])" } else { $_.Exception.Message }
      $rows += Add-Result "gateway $path" "failed" $detail
    }
  }
} else {
  $rows += Add-Result "gateway" "skipped" "Pass -GatewayBaseUrl to verify Yian endpoints"
}

$rows | Format-Table -AutoSize

$reportDir = Split-Path -Parent $ReportPath
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
$lines = @(
  "# Android Device Loop Check Report",
  "",
  "GeneratedAt: $(Get-Date -Format s)",
  "",
  "| Check | Status | Detail |",
  "| --- | --- | --- |"
)
foreach ($row in $rows) {
  $detail = $row.Detail
  if ($null -eq $detail) {
    $detail = ""
  }
  $detail = $detail.Replace("|", "\|").Replace("`r", " ").Replace("`n", " ")
  $lines += "| $($row.Check) | $($row.Status) | $detail |"
}
$lines | Set-Content -Encoding utf8 -Path $ReportPath

Write-Host "Report written to $ReportPath" -ForegroundColor Green
