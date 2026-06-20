param(
  [string]$Distro = "Ubuntu-22.04",
  [string]$RepoRoot = "",
  [string]$EnvFile = "",
  [switch]$Build,
  [switch]$SiteHttpSmoke,
  [switch]$Preflight,
  [switch]$Prod,
  [switch]$Execute,
  [int]$DeployRetries = 2
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $PSCommandPath
if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
  $RepoRoot = Resolve-Path (Join-Path $scriptRoot "..\..")
} else {
  $RepoRoot = Resolve-Path -LiteralPath $RepoRoot
}
if ([string]::IsNullOrWhiteSpace($EnvFile)) {
  $EnvFile = Join-Path $scriptRoot ".env"
}

function Convert-ToWslPath {
  param([string]$Path)
  $resolvedPath = Resolve-Path -LiteralPath $Path
  $root = [System.IO.Path]::GetPathRoot($resolvedPath.Path)
  if ([string]::IsNullOrWhiteSpace($root) -or -not $root.EndsWith(":\")) {
    throw "Only drive-letter Windows paths are supported for WSL deployment: $Path"
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
$wslEnvFile = ""
if (-not [string]::IsNullOrWhiteSpace($EnvFile) -and (Test-Path -LiteralPath $EnvFile)) {
  $wslEnvFile = Convert-ToWslPath -Path $EnvFile
}
$quotedEnvFile = Convert-ToShellSingleQuoted -Value $wslEnvFile
$buildFlag = if ($Build) { "1" } else { "0" }
$siteHttpSmokeFlag = if ($SiteHttpSmoke) { "1" } else { "0" }
$preflightFlag = if ($Preflight) { "1" } else { "0" }
$prodFlag = if ($Prod) { "1" } else { "0" }
$executeFlag = if ($Execute) { "1" } else { "0" }
$quotedRetries = Convert-ToShellSingleQuoted -Value ([string]$DeployRetries)

$deployScript = @"
set -eu
repo_root=$quotedRepoRoot
env_file=$quotedEnvFile
build_flag=$buildFlag
site_http_smoke_flag=$siteHttpSmokeFlag
preflight_flag=$preflightFlag
prod_flag=$prodFlag
execute_flag=$executeFlag
deploy_retries=$quotedRetries

if [ ! -d "`$repo_root" ]; then
  echo "Repository path is not mounted in WSL: `$repo_root" >&2
  exit 10
fi

if [ -n "`$env_file" ] && [ -f "`$env_file" ]; then
  set -a
  # shellcheck disable=SC1090
  . "`$env_file"
  set +a
fi

export NVM_DIR="`${NVM_DIR:-`$HOME/.nvm}"
if [ -s "`$NVM_DIR/nvm.sh" ]; then
  # shellcheck disable=SC1090
  . "`$NVM_DIR/nvm.sh"
  if command -v nvm >/dev/null 2>&1; then
    best_node_version=`$(nvm ls --no-colors 2>/dev/null | sed -n 's/.*v\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\).*/\1/p' | awk -F. '`$1 >= 22 { print `$0 }' | sort -V | tail -n 1)
    if [ -n "`$best_node_version" ]; then
      nvm use "v`$best_node_version" >/dev/null 2>&1 || true
    fi
  fi
fi

command -v node >/dev/null
node_major=`$(node -p "Number(process.versions.node.split('.')[0])")
if [ "`$node_major" -lt 22 ]; then
  echo "Node.js >=22 is required in WSL; found `$(node -v)" >&2
  exit 11
fi
echo "Using WSL Node.js `$(node -v) at `$(command -v node)"

cd "`$repo_root"

if [ "`$build_flag" = "1" ]; then
  npm run build
fi
if [ "`$site_http_smoke_flag" = "1" ]; then
  npm run test:site-http
fi

project_name="`${VERCEL_PROJECT_NAME:-storylock-gateway}"
if [ ! -f ".vercel/project.json" ]; then
  echo ".vercel/project.json was not found under `$repo_root; run link_project first or deploy with VERCEL_PROJECT_ID in CI." >&2
  exit 12
fi
linked_project=`$(node -e "const fs=require('fs'); const p=JSON.parse(fs.readFileSync('.vercel/project.json','utf8')); console.log(p.projectName || '')")
if [ "`$linked_project" != "`$project_name" ]; then
  echo "Local Vercel project link mismatch: expected `$project_name, got `$linked_project" >&2
  exit 13
fi
echo "Vercel project link ok: `$linked_project"

vercel_cmd="vercel"
if ! command -v vercel >/dev/null 2>&1; then
  vercel_cmd="npx --yes vercel@54.5.1"
fi

vercel_common_args=""
if [ -n "`${VERCEL_TOKEN:-}" ]; then
  vercel_common_args="`$vercel_common_args --token `$VERCEL_TOKEN"
fi
if [ -n "`${VERCEL_SCOPE:-}" ]; then
  vercel_common_args="`$vercel_common_args --scope `$VERCEL_SCOPE"
fi

if ! sh -lc "`$vercel_cmd whoami `$vercel_common_args" >/tmp/storylock-vercel-whoami.out 2>/tmp/storylock-vercel-whoami.err; then
  if [ -n "`${VERCEL_TOKEN:-}" ]; then
    echo "Vercel token auth failed in WSL. Check VERCEL_TOKEN permissions for project `$project_name." >&2
  else
    echo "Vercel is not authenticated in WSL and VERCEL_TOKEN is empty. Run vercel login inside WSL or export VERCEL_TOKEN." >&2
  fi
  cat /tmp/storylock-vercel-whoami.err >&2 || true
  exit 14
fi
echo "Vercel user: `$(cat /tmp/storylock-vercel-whoami.out)"

custom_domain="`${VERCEL_CUSTOM_DOMAIN:-yian.cdao.online}"
bind_custom_domain="`${VERCEL_BIND_CUSTOM_DOMAIN:-false}"
if [ -n "`$custom_domain" ]; then
  if sh -lc "`$vercel_cmd domains inspect `$custom_domain `$vercel_common_args" >/tmp/storylock-vercel-domain.out 2>/tmp/storylock-vercel-domain.err; then
    echo "Vercel domain access ok: `$custom_domain"
  else
    echo "Vercel domain inspect did not succeed for `$custom_domain. If production still returns 404, confirm the domain is owned by the same Vercel project/account." >&2
    cat /tmp/storylock-vercel-domain.err >&2 || true
  fi
fi

deploy_args="deploy --yes"
if [ "`$prod_flag" = "1" ]; then
  deploy_args="`$deploy_args --prod"
fi
deploy_args="`$deploy_args `$vercel_common_args"

if [ "`$execute_flag" != "1" ]; then
  echo "WSL Vercel release plan:"
  echo "  project: `$project_name"
  echo "  command: `$vercel_cmd `$deploy_args"
  echo "No deployment command was executed. Re-run with -Execute."
  exit 0
fi

attempt=1
while [ "`$attempt" -le "`$deploy_retries" ]; do
  echo "Running: `$vercel_cmd `$deploy_args (attempt `$attempt/`$deploy_retries)"
  if sh -lc "`$vercel_cmd `$deploy_args" 2>&1 | tee /tmp/storylock-vercel-deploy.out; then
    break
  fi
  if [ "`$attempt" -eq "`$deploy_retries" ]; then
    echo "Vercel deploy failed after `$deploy_retries attempt(s)." >&2
    exit 15
  fi
  sleep "`$((attempt * 2))"
  attempt="`$((attempt + 1))"
done

deployment_url=`$(grep -o 'https://[^ ]*\.vercel\.app' /tmp/storylock-vercel-deploy.out | tail -n 1 || true)
if [ -n "`$deployment_url" ]; then
  echo "Deployment URL: `$deployment_url"
else
  echo "Deployment URL could not be parsed from Vercel output; custom-domain alias binding will be skipped." >&2
fi
if [ "`$prod_flag" = "1" ] && [ "`$bind_custom_domain" = "true" ] && [ -n "`$custom_domain" ] && [ -n "`$deployment_url" ]; then
  echo "Binding custom domain to this deployment: `$custom_domain -> `$deployment_url"
  sh -lc "`$vercel_cmd alias set `$deployment_url `$custom_domain `$vercel_common_args"
elif [ "`$prod_flag" = "1" ] && [ -n "`$custom_domain" ]; then
  echo "Custom domain binding was not forced. If `$custom_domain returns 404 after a successful deploy, set VERCEL_BIND_CUSTOM_DOMAIN=true after confirming domain ownership."
fi

if [ "`$preflight_flag" = "1" ] && [ -n "`${STORYLOCK_GATEWAY_PUBLIC_URL:-}" ]; then
  python3 - <<'PY'
import sys
import time
import urllib.request
import os

base = os.environ.get("STORYLOCK_GATEWAY_PUBLIC_URL", "").rstrip("/")
checks = [
    (base + "/", "text/html"),
    (base + "/main.js", "javascript"),
    (base + "/styles.css", "css"),
    (base + "/api/storylock-gateway", "json"),
    (base + "/app/download", "json"),
]
failures = []
for url, expected in checks:
    last_error = None
    for _ in range(6):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "storylock-wsl-preflight"})
            with urllib.request.urlopen(req, timeout=30) as response:
                status = response.status
                content_type = response.headers.get("content-type", "")
            if status == 200 and expected in content_type:
                last_error = None
                break
            last_error = f"{url} returned {status} {content_type}"
        except Exception as exc:
            last_error = f"{url} failed: {exc}"
        time.sleep(10)
    if last_error:
        failures.append(last_error)
if failures:
    print("Post-deploy preflight failed:", file=sys.stderr)
    for item in failures:
        print(" - " + item, file=sys.stderr)
    sys.exit(16)
print("Post-deploy preflight passed.")
PY
fi
"@

$tempScript = Join-Path ([System.IO.Path]::GetTempPath()) ("storylock-vercel-wsl-" + [System.Guid]::NewGuid().ToString("N") + ".sh")
[System.IO.File]::WriteAllText($tempScript, $deployScript, [System.Text.UTF8Encoding]::new($false))
$wslTempScript = Convert-ToWslPath -Path $tempScript

try {
  wsl.exe -d $Distro -- bash $wslTempScript
  if ($LASTEXITCODE -ne 0) {
    throw "WSL Vercel deployment failed with exit code $LASTEXITCODE"
  }
} finally {
  Remove-Item -LiteralPath $tempScript -Force -ErrorAction SilentlyContinue
}
