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

if (-not (Test-Path -LiteralPath $envPath) -and (Test-Path -LiteralPath $exampleEnvPath)) {
  Write-Host "[INFO] scripts/vercel/.env not found. Falling back to .env.example" -ForegroundColor Yellow
}

$port = if ($env:STORYLOCK_VERCEL_PORT) { $env:STORYLOCK_VERCEL_PORT } else { '4318' }
$env:PORT = $port

Push-Location $repoRoot
try {
  Write-Host "[INFO] Starting StoryLock Web API gateway on port $port" -ForegroundColor Green
  npm run dev:web-api --prefix src/skills/remote-gateway
} finally {
  Pop-Location
}
