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
  foreach ($rawLine in Get-Content -LiteralPath $Path) {
    $line = $rawLine.Trim()
    if (-not $line -or $line.StartsWith('#')) {
      continue
    }
    $parts = $line -split '=', 2
    if ($parts.Count -ne 2) {
      continue
    }
    if (-not [string]::IsNullOrWhiteSpace([System.Environment]::GetEnvironmentVariable($parts[0].Trim(), 'Process'))) {
      continue
    }
    Set-Item -Path ("Env:{0}" -f $parts[0].Trim()) -Value $parts[1].Trim()
  }
}

Import-EnvFile -Path $envPath
Import-EnvFile -Path $exampleEnvPath

if (-not $env:VERCEL_PROJECT_NAME) {
  throw 'VERCEL_PROJECT_NAME is required. Set it in scripts/vercel/.env'
}
if (-not $env:VERCEL_SCOPE) {
  throw 'VERCEL_SCOPE is required. storylock-gateway production should use VERCEL_SCOPE=iunknow588, not the LUCKEE team scope.'
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
  Write-Host "[INFO] Linking Vercel project $($env:VERCEL_PROJECT_NAME) under scope $($env:VERCEL_SCOPE)" -ForegroundColor Green
  & vercel @args
} finally {
  Pop-Location
}
