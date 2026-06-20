param(
  [Parameter(Mandatory = $true)]
  [string]$EnvFilePath,
  [string[]]$Environments = @("preview", "production"),
  [string]$ProjectDir = "",
  [string]$OutputDir = "",
  [string]$VercelToken = "",
  [switch]$Execute
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $PSCommandPath
if ([string]::IsNullOrWhiteSpace($ProjectDir)) {
  $ProjectDir = Join-Path $scriptRoot "..\.."
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
  $OutputDir = Join-Path $scriptRoot "..\..\.temp\dist\vercel-env-sync"
}

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
    [System.Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1], "Process")
  }
}

function Read-EnvEntries {
  param([string]$Path)
  $entries = New-Object System.Collections.Generic.List[object]
  foreach ($rawLine in Get-Content -LiteralPath $Path) {
    if ($null -eq $rawLine) {
      continue
    }
    $line = $rawLine.Trim()
    if (-not $line -or $line.StartsWith("#")) {
      continue
    }
    $parts = $rawLine -split "=", 2
    if ($parts.Count -ne 2) {
      continue
    }
    $entries.Add([PSCustomObject]@{
        name = $parts[0].Trim()
        value = $parts[1]
      }) | Out-Null
  }
  return $entries
}

function Get-LocalVercelProjectName {
  param([string]$RootDir)
  $projectJsonPath = Join-Path $RootDir ".vercel\project.json"
  if (-not (Test-Path -LiteralPath $projectJsonPath)) {
    return ""
  }
  try {
    $project = Get-Content -Raw -LiteralPath $projectJsonPath | ConvertFrom-Json
    return [string]$project.projectName
  } catch {
    throw "Unable to parse local Vercel project link: $projectJsonPath"
  }
}

function Assert-VercelProjectLink {
  param(
    [string]$RootDir,
    [string]$ExpectedProjectName
  )
  $localProjectName = Get-LocalVercelProjectName -RootDir $RootDir
  if ([string]::IsNullOrWhiteSpace($localProjectName)) {
    throw "Local Vercel project link was not found. Run scripts\vercel\link_project.cmd from the skill/ directory before syncing env."
  }
  if (-not [string]::IsNullOrWhiteSpace($ExpectedProjectName) -and $localProjectName -ne $ExpectedProjectName) {
    throw "Local Vercel project link mismatch. VERCEL_PROJECT_NAME='$ExpectedProjectName' but .vercel/project.json is linked to '$localProjectName'. Re-run scripts\vercel\link_project.cmd after confirming which Vercel project owns yian.cdao.online."
  }
  return $localProjectName
}

function Invoke-VercelEnvWrite {
  param(
    [string]$Action,
    [string]$Name,
    [string]$Environment,
    [string]$Value,
    [string]$Token,
    [string]$Scope
  )

  $tempFile = [System.IO.Path]::GetTempFileName()
  try {
    Set-Content -LiteralPath $tempFile -Value $Value -Encoding UTF8
    $command = "Get-Content -LiteralPath '$tempFile' -Raw | vercel env $Action $Name $Environment"
    if (-not [string]::IsNullOrWhiteSpace($Scope)) {
      $command += " --scope $Scope"
    }
    if (-not [string]::IsNullOrWhiteSpace($Token)) {
      $command += " --token $Token"
    }
    & powershell -NoProfile -Command $command
    return $LASTEXITCODE
  } finally {
    Remove-Item -LiteralPath $tempFile -Force -ErrorAction SilentlyContinue
  }
}

if (-not (Test-Path -LiteralPath $EnvFilePath)) {
  throw "Env file was not found: $EnvFilePath"
}

$exampleEnvPath = Join-Path $scriptRoot ".env.example"
$defaultEnvPath = Join-Path $scriptRoot ".env"
Import-EnvFile -Path $exampleEnvPath
Import-EnvFile -Path $defaultEnvPath

$entries = Read-EnvEntries -Path $EnvFilePath
if ($entries.Count -eq 0) {
  throw "Env file does not contain any key=value pairs: $EnvFilePath"
}

$resolvedProjectDir = (Resolve-Path -LiteralPath $ProjectDir).Path
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$projectName = [System.Environment]::GetEnvironmentVariable("VERCEL_PROJECT_NAME", "Process")
$localVercelProjectName = Assert-VercelProjectLink -RootDir $resolvedProjectDir -ExpectedProjectName $projectName
$scopeValue = [System.Environment]::GetEnvironmentVariable("VERCEL_SCOPE", "Process")
$tokenValue = if ([string]::IsNullOrWhiteSpace($VercelToken)) {
  [System.Environment]::GetEnvironmentVariable("VERCEL_TOKEN", "Process")
} else {
  $VercelToken
}

$planItems = @()
foreach ($environment in $Environments) {
  foreach ($entry in $entries) {
    $planItems += [PSCustomObject]@{
      name = $entry.name
      environment = $environment
      valueLength = [string]$entry.value | ForEach-Object { $_.Length }
      action = "update-or-add"
    }
  }
}

$planPath = Join-Path $OutputDir ("vercel-env-sync-plan-{0}.json" -f (Get-Date -Format "yyyyMMdd-HHmmss"))
[PSCustomObject]@{
  projectName = $projectName
  localVercelProjectName = $localVercelProjectName
  projectDir = $resolvedProjectDir
  envFilePath = (Resolve-Path -LiteralPath $EnvFilePath).Path
  environments = $Environments
  execute = [bool]$Execute
  generatedAt = (Get-Date).ToUniversalTime().ToString("o")
  items = $planItems
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $planPath -Encoding UTF8

if (-not $Execute) {
  Write-Output "Vercel env sync plan created."
  Write-Output "Plan: $planPath"
  Write-Output "No Vercel env command was executed. Re-run with -Execute after confirming project link and token."
  $planItems | Format-Table name, environment, action -AutoSize
  return
}

if ([string]::IsNullOrWhiteSpace($projectName)) {
  throw "VERCEL_PROJECT_NAME is required for Vercel env sync."
}
if (-not (Get-Command vercel -ErrorAction SilentlyContinue)) {
  throw "Vercel CLI not found. Install it first or run scripts\\vercel\\link_project.cmd."
}

Push-Location $resolvedProjectDir
try {
  foreach ($environment in $Environments) {
    foreach ($entry in $entries) {
      Write-Output ("Syncing {0} -> {1}" -f $entry.name, $environment)
      $updateExit = Invoke-VercelEnvWrite -Action "update" -Name $entry.name -Environment $environment -Value $entry.value -Token $tokenValue -Scope $scopeValue
      if ($updateExit -ne 0) {
        Write-Output ("Update failed for {0} ({1}); falling back to add." -f $entry.name, $environment)
        $addExit = Invoke-VercelEnvWrite -Action "add" -Name $entry.name -Environment $environment -Value $entry.value -Token $tokenValue -Scope $scopeValue
        if ($addExit -ne 0) {
          throw "Failed to sync $($entry.name) for environment '$environment'"
        }
      }
    }
  }
} finally {
  Pop-Location
}

Write-Output "Vercel env sync completed."
Write-Output "Plan: $planPath"
