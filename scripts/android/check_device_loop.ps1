param(
  [string]$ApkPath = "",
  [string]$GatewayBaseUrl = "",
  [string]$DeepLink = "",
  [string]$IdentityId = "android-demo-001",
  [string]$PackageKind = "",
  [string]$ReleaseChannel = "",
  [int]$AndroidHostPort = 4500,
  [int]$LocalForwardPort = 14500,
  [string]$SharedSecret = "replace-with-strong-shared-secret",
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

if ($adb -and ($deviceLines.Count -gt 0)) {
  try {
    & adb forward "tcp:$LocalForwardPort" "tcp:$AndroidHostPort" | Out-Null
    $rows += Add-Result "adb forward" "ok" "tcp:$LocalForwardPort -> tcp:$AndroidHostPort"

    $headers = @{}
    if ($SharedSecret) {
      $headers["x-storylock-shared-secret"] = $SharedSecret
    }

    foreach ($path in @("/health", "/permission-summary")) {
      try {
        $localUrl = "http://127.0.0.1:$LocalForwardPort$path"
        $response = Invoke-WebRequest -Uri $localUrl -Headers $headers -UseBasicParsing -ErrorAction Stop
        $body = $response.Content
        $redactionOk = ($body -notmatch "canonicalAnswer|acceptedAnswers|privateKey|signingKeyBytes|`"password`"\s*:")
        $detail = "$($response.StatusCode) $($response.Headers['Content-Type'])"
        if ($path -eq "/permission-summary") {
          $detail = "$detail; redactionOk=$redactionOk"
        }
        $rows += Add-Result "android host $path" $(if ($redactionOk) { "ok" } else { "failed" }) $detail
      } catch {
        $response = $_.Exception.Response
        $detail = if ($response) { "$([int]$response.StatusCode) $($response.Headers['Content-Type'])" } else { $_.Exception.Message }
        $rows += Add-Result "android host $path" "failed" $detail
      }
    }
  } catch {
    $rows += Add-Result "adb forward" "failed" $_.Exception.Message
  } finally {
    try {
      & adb forward --remove "tcp:$LocalForwardPort" | Out-Null
    } catch {
      # Best-effort cleanup only.
    }
  }
} elseif ($adb) {
  $rows += Add-Result "android host local http" "blocked" "Connect a device, start Android Host, then rerun to test /health and /permission-summary"
}

if ($GatewayBaseUrl) {
  $base = $GatewayBaseUrl.TrimEnd("/")
  foreach ($path in @("/", "/api/storylock-gateway", "/app/download", "/app/download/android", "/android-host/bind")) {
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

if ($GatewayBaseUrl -and $IdentityId) {
  $bindUrl = "{0}/app/bind?identityId={1}&preferredMode=relay_url" -f $GatewayBaseUrl.TrimEnd("/"), $IdentityId
  try {
    $bindingResponse = Invoke-WebRequest -Uri $bindUrl -UseBasicParsing -ErrorAction Stop
    $rows += Add-Result "bind request" "ok" "$($bindingResponse.StatusCode) identityId=$IdentityId"
  } catch {
    $response = $_.Exception.Response
    $detail = if ($response) { "$([int]$response.StatusCode) $($response.Headers['Content-Type'])" } else { $_.Exception.Message }
    $rows += Add-Result "bind request" "failed" $detail
  }
}

if ($PackageKind) {
  $rows += Add-Result "package kind" "info" $PackageKind
}

if ($ReleaseChannel) {
  $rows += Add-Result "release channel" "info" $ReleaseChannel
}

$rows | Format-Table -AutoSize

$reportDir = Split-Path -Parent $ReportPath
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
$lines = @(
  "# Android Device Loop Check Report",
  "",
  "GeneratedAt: $(Get-Date -Format s)",
  "IdentityId: $IdentityId",
  "PackageKind: $PackageKind",
  "ReleaseChannel: $ReleaseChannel",
  "AndroidHostPort: $AndroidHostPort",
  "LocalForwardPort: $LocalForwardPort",
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
