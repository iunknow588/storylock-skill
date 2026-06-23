param(
  [string]$Distro = "Ubuntu-22.04",
  [string]$RepoRoot = "",
  [string]$ReportPath = ".temp\linux-secret-service-wsl-report.local.md"
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
  $RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
} else {
  $RepoRoot = Resolve-Path -LiteralPath $RepoRoot
}

function Convert-ToWslPath {
  param([string]$Path)
  $resolvedPath = Resolve-Path -LiteralPath $Path
  $root = [System.IO.Path]::GetPathRoot($resolvedPath.Path)
  if ([string]::IsNullOrWhiteSpace($root) -or -not $root.EndsWith(":\")) {
    throw "Only drive-letter Windows paths are supported for WSL diagnostics: $Path"
  }
  $drive = $root.Substring(0, 1).ToLowerInvariant()
  $relativePath = $resolvedPath.Path.Substring($root.Length).Replace("\", "/")
  return "/mnt/$drive/$relativePath"
}

if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
  throw "wsl.exe was not found. Enable WSL and install a Linux distribution before running this script."
}

$repoRootPath = [System.IO.Path]::GetFullPath($RepoRoot)
$reportFullPath = [System.IO.Path]::GetFullPath((Join-Path $repoRootPath $ReportPath))
$reportDir = Split-Path -Parent $reportFullPath
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

$wslRepoRoot = Convert-ToWslPath -Path $repoRootPath
$scriptPath = Join-Path $repoRootPath "scripts\linux\check_linux_secret_service_wsl.sh"
$wslScriptPath = Convert-ToWslPath -Path $scriptPath

try {
  $previousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  $output = @(& wsl.exe -d $Distro -- bash $wslScriptPath $wslRepoRoot 2>&1) | ForEach-Object {
    $_.ToString()
  }
  $exitCode = $LASTEXITCODE
} catch {
  $output = @($_.Exception.Message)
  $exitCode = 1
} finally {
  $ErrorActionPreference = $previousErrorActionPreference
}

$lines = @(
  "# Linux Secret Service WSL Diagnostic Report",
  "",
  "GeneratedAt: $(Get-Date -Format s)",
  "Distro: $Distro",
  "RepoRoot: $repoRootPath",
  "ExitCode: $exitCode",
  "",
  '```text'
)
$lines += $output
$lines += '```'
$lines | Set-Content -Encoding utf8 -Path $reportFullPath

$output
Write-Host "Report written to $reportFullPath" -ForegroundColor Green

if ($exitCode -ne 0) {
  exit $exitCode
}
