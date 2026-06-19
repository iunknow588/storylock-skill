param(
  [ValidateSet("vercel", "static")]
  [string]$Target = "vercel",
  [string]$ProjectDir = "",
  [string]$EnvFile = "",
  [string]$WindowsEnvFile = "",
  [string]$StaticOutputDir = "",
  [string]$VercelToken = "",
  [switch]$SyncWindowsEnvToVercel,
  [string[]]$VercelEnvTargets = @("preview", "production"),
  [switch]$Build,
  [switch]$Preflight,
  [switch]$Prod,
  [switch]$Execute
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $PSCommandPath
if ([string]::IsNullOrWhiteSpace($ProjectDir)) {
  $ProjectDir = Join-Path $scriptRoot "..\.."
}
if ([string]::IsNullOrWhiteSpace($EnvFile)) {
  $EnvFile = Join-Path $scriptRoot ".env"
}
if ([string]::IsNullOrWhiteSpace($StaticOutputDir)) {
  $StaticOutputDir = Join-Path $scriptRoot "..\..\.temp\dist\site-release"
}

$exampleEnvPath = Join-Path $scriptRoot ".env.example"
$preflightScript = Join-Path $scriptRoot "preflight.ps1"
$syncEnvScript = Join-Path $scriptRoot "sync_env_file_to_vercel.ps1"

function Import-EnvFile {
  param([string]$Path)
  if (-not $Path -or -not (Test-Path -LiteralPath $Path)) {
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

Import-EnvFile -Path $exampleEnvPath
Import-EnvFile -Path $EnvFile
Import-EnvFile -Path $WindowsEnvFile

$resolvedProjectDir = (Resolve-Path -LiteralPath $ProjectDir).Path
$resolvedWindowsEnvFile = if (-not [string]::IsNullOrWhiteSpace($WindowsEnvFile) -and (Test-Path -LiteralPath $WindowsEnvFile)) {
  (Resolve-Path -LiteralPath $WindowsEnvFile).Path
} else {
  $WindowsEnvFile
}
$releaseOutputDir = Join-Path $StaticOutputDir (Get-Date -Format "yyyyMMdd-HHmmss")
New-Item -ItemType Directory -Force -Path $releaseOutputDir | Out-Null

Push-Location $resolvedProjectDir
try {
  if ($Build) {
    Write-Output "Running site build..."
    npm run build
  }

  if ($Preflight) {
    if (-not (Test-Path -LiteralPath $preflightScript)) {
      throw "Preflight script not found: $preflightScript"
    }
    & $preflightScript
  }

  $publicDir = Join-Path $resolvedProjectDir "release\web\public"
  if (-not (Test-Path -LiteralPath $publicDir)) {
    throw "Static release web public directory not found: $publicDir"
  }

  if ($Target -eq "static") {
    $staticDir = Join-Path $releaseOutputDir "release\web\public"
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $staticDir) | Out-Null
    Copy-Item -LiteralPath $publicDir -Destination $staticDir -Recurse -Force
    $summaryPath = Join-Path $releaseOutputDir "static-release-summary.json"
    [PSCustomObject]@{
      target = "static"
      projectDir = $resolvedProjectDir
      publicDir = $publicDir
      releaseDir = $staticDir
      gatewayPublicUrl = [System.Environment]::GetEnvironmentVariable("STORYLOCK_GATEWAY_PUBLIC_URL", "Process")
      preparedAt = (Get-Date).ToUniversalTime().ToString("o")
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $summaryPath -Encoding UTF8
    Write-Output "Static site release skeleton prepared."
    Write-Output "Release dir: $staticDir"
    Write-Output "Summary: $summaryPath"
    return
  }

  $projectName = [System.Environment]::GetEnvironmentVariable("VERCEL_PROJECT_NAME", "Process")
  if ([string]::IsNullOrWhiteSpace($projectName)) {
    throw "VERCEL_PROJECT_NAME is required for Vercel target."
  }

  $vercelArgs = @("deploy")
  if ($Prod) {
    $vercelArgs += "--prod"
  }
  $tokenValue = if ([string]::IsNullOrWhiteSpace($VercelToken)) {
    [System.Environment]::GetEnvironmentVariable("VERCEL_TOKEN", "Process")
  } else {
    $VercelToken
  }
  if (-not [string]::IsNullOrWhiteSpace($tokenValue)) {
    $vercelArgs += @("--token", $tokenValue)
  }
  $scopeValue = [System.Environment]::GetEnvironmentVariable("VERCEL_SCOPE", "Process")
  if (-not [string]::IsNullOrWhiteSpace($scopeValue)) {
    $vercelArgs += @("--scope", $scopeValue)
  }

  $planPath = Join-Path $releaseOutputDir "vercel-release-plan.json"
  [PSCustomObject]@{
    target = "vercel"
    projectDir = $resolvedProjectDir
    projectName = $projectName
    buildRequested = [bool]$Build
    preflightRequested = [bool]$Preflight
    prod = [bool]$Prod
    execute = [bool]$Execute
    command = @("vercel") + $vercelArgs
    windowsEnvFile = $resolvedWindowsEnvFile
    syncWindowsEnvToVercel = [bool]$SyncWindowsEnvToVercel
    vercelEnvTargets = $VercelEnvTargets
    preparedAt = (Get-Date).ToUniversalTime().ToString("o")
  } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $planPath -Encoding UTF8

  if ($SyncWindowsEnvToVercel -and -not [string]::IsNullOrWhiteSpace($resolvedWindowsEnvFile)) {
    $syncArgs = @{
      EnvFilePath = $resolvedWindowsEnvFile
      Environments = $VercelEnvTargets
      ProjectDir = $resolvedProjectDir
      VercelToken = $tokenValue
      OutputDir = (Join-Path $releaseOutputDir "env-sync")
    }
    if ($Execute) {
      $syncArgs.Execute = $true
    }
    & $syncEnvScript @syncArgs
  }

  if (-not $Execute) {
    Write-Output "Vercel release plan created."
    Write-Output "Plan: $planPath"
    Write-Output "No deployment command was executed. Re-run with -Execute after confirming env and project binding."
    return
  }

  if (-not (Get-Command vercel -ErrorAction SilentlyContinue)) {
    throw "Vercel CLI not found. Install it first or run scripts\\vercel\\link_project.cmd."
  }

  Write-Output ("Running: vercel {0}" -f ($vercelArgs -join " "))
  & vercel @vercelArgs
} finally {
  Pop-Location
}
