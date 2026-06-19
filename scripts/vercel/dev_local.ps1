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

if (-not (Test-Path -LiteralPath $envPath) -and (Test-Path -LiteralPath $exampleEnvPath)) {
  Write-Host "[INFO] scripts/vercel/.env not found. Falling back to .env.example" -ForegroundColor Yellow
}

Import-EnvFile -Path $exampleEnvPath
Import-EnvFile -Path $envPath

$port = if ($env:STORYLOCK_VERCEL_PORT) { $env:STORYLOCK_VERCEL_PORT } else { '4318' }
$env:PORT = $port

Push-Location $repoRoot
try {
  Write-Host "[INFO] Starting StoryLock Web API gateway on port $port" -ForegroundColor Green
  npm run dev:web-api --prefix src/skills/remote-gateway
} finally {
  Pop-Location
}
