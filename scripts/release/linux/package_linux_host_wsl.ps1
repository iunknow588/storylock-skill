param(
  [string]$Distro = "Ubuntu-22.04",
  [string]$RepoRoot = "",
  [string]$Version = "0.1.0",
  [string]$VersionCode = "1",
  [string]$ReleaseChannel = "prototype"
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
  $RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
} else {
  $RepoRoot = Resolve-Path -LiteralPath $RepoRoot
}

function Convert-ToWslPath {
  param([string]$Path)
  $resolvedPath = Resolve-Path -LiteralPath $Path
  $root = [System.IO.Path]::GetPathRoot($resolvedPath.Path)
  if ([string]::IsNullOrWhiteSpace($root) -or -not $root.EndsWith(":\")) {
    throw "Only drive-letter Windows paths are supported for WSL packaging: $Path"
  }
  $drive = $root.Substring(0, 1).ToLowerInvariant()
  $relativePath = $resolvedPath.Path.Substring($root.Length).Replace("\", "/")
  return "/mnt/$drive/$relativePath"
}

function Convert-ToShellSingleQuoted {
  param([string]$Value)
  return "'" + $Value.Replace("'", "'\''") + "'"
}

if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
  throw "wsl.exe was not found. Enable WSL and install a Linux distribution before running this script."
}

$wslRepoRoot = Convert-ToWslPath -Path $RepoRoot
$quotedRepoRoot = Convert-ToShellSingleQuoted -Value $wslRepoRoot
$quotedVersion = Convert-ToShellSingleQuoted -Value $Version
$quotedVersionCode = Convert-ToShellSingleQuoted -Value $VersionCode
$quotedReleaseChannel = Convert-ToShellSingleQuoted -Value $ReleaseChannel

$checkScript = @"
set -eu
repo_root=$quotedRepoRoot
if [ ! -d "`$repo_root" ]; then
  echo "Repository path is not mounted in WSL: `$repo_root" >&2
  exit 10
fi
if [ -z "`${STORYLOCK_WSL_NODE_BIN:-}" ]; then
  export NVM_DIR="`${NVM_DIR:-`$HOME/.nvm}"
  if [ -s "`$NVM_DIR/nvm.sh" ]; then
    . "`$NVM_DIR/nvm.sh"
    if command -v nvm >/dev/null 2>&1; then
      best_node_version=`$(nvm ls --no-colors 2>/dev/null | sed -n 's/.*v\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\).*/\1/p' | awk -F. '`$1 >= 22 { print `$0 }' | sort -V | tail -n 1)
      if [ -n "`$best_node_version" ]; then
        nvm use "v`$best_node_version" >/dev/null 2>&1 || true
      else
        nvm use default >/dev/null 2>&1 || nvm use --lts >/dev/null 2>&1 || nvm use node >/dev/null 2>&1 || true
      fi
    fi
  fi
else
  export PATH="`$(dirname "`$STORYLOCK_WSL_NODE_BIN"):`$PATH"
fi
command -v node >/dev/null
node_major=`$(node -p "Number(process.versions.node.split('.')[0])")
if [ "`$node_major" -lt 22 ]; then
  echo "Node.js >=22 is required in WSL; found `$(node -v)" >&2
  echo "If Node.js >=22 is installed with nvm, run: nvm alias default node" >&2
  echo "Or run: nvm install --lts && nvm alias default --lts" >&2
  echo "Or set STORYLOCK_WSL_NODE_BIN to the absolute WSL path of the Node.js >=22 binary." >&2
  exit 11
fi
echo "Using WSL Node.js `$(node -v) at `$(command -v node)"
command -v dpkg-deb >/dev/null
cd "`$repo_root"
STORYLOCK_LINUX_PACKAGE_KIND="tar.gz" STORYLOCK_LINUX_PACKAGE_VERSION=$quotedVersion STORYLOCK_LINUX_PACKAGE_VERSION_CODE=$quotedVersionCode STORYLOCK_LINUX_RELEASE_CHANNEL=$quotedReleaseChannel node scripts/release/linux/package_linux_host.mjs
"@

$tempScript = Join-Path ([System.IO.Path]::GetTempPath()) ("storylock-linux-package-" + [System.Guid]::NewGuid().ToString("N") + ".sh")
[System.IO.File]::WriteAllText($tempScript, $checkScript, [System.Text.UTF8Encoding]::new($false))
$wslTempScript = Convert-ToWslPath -Path $tempScript

try {
  wsl.exe -d $Distro -- bash $wslTempScript
  if ($LASTEXITCODE -ne 0) {
    throw "WSL Linux host packaging failed with exit code $LASTEXITCODE"
  }
} finally {
  Remove-Item -LiteralPath $tempScript -Force -ErrorAction SilentlyContinue
}
