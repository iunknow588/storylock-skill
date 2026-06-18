$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir '..\..')
$envPath = Join-Path $scriptDir '.env'
$exampleEnvPath = Join-Path $scriptDir '.env.example'

function Import-EnvFile {
  param([string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    return
  }
  Get-Content -LiteralPath $Path | ForEach-Object {
    $line = $_.Trim()
    if (-not $line -or $line.StartsWith('#')) {
      return
    }
    $parts = $line -split '=', 2
    if ($parts.Count -ne 2) {
      return
    }
    [System.Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1].Trim(), 'Process')
  }
}

Import-EnvFile -Path $exampleEnvPath
Import-EnvFile -Path $envPath

if (-not $env:VERCEL_PROJECT_NAME) {
  throw 'VERCEL_PROJECT_NAME is required. Set it in scripts/vercel/.env'
}

if (-not (Get-Command vercel -ErrorAction SilentlyContinue)) {
  Write-Host "[WARN] Vercel CLI not found. Installing..." -ForegroundColor Yellow
  npm install -g vercel
}

Push-Location $repoRoot
try {
  $args = @('link', '--project', $env:VERCEL_PROJECT_NAME, '--yes')
  if ($env:VERCEL_SCOPE) {
    $args += @('--scope', $env:VERCEL_SCOPE)
  }
  if ($env:VERCEL_TOKEN) {
    $args += @('--token', $env:VERCEL_TOKEN)
  }
  Write-Host "[INFO] Linking Vercel project $($env:VERCEL_PROJECT_NAME)" -ForegroundColor Green
  & vercel @args
} finally {
  Pop-Location
}
