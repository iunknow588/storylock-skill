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
  [switch]$SiteHttpSmoke,
  [switch]$Prod,
  [switch]$Execute,
  [string]$VercelCustomDomain = "",
  [switch]$BindCustomDomain,
  [int]$DeployRetries = 2
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
  foreach ($rawLine in Get-Content -LiteralPath $Path) {
    $line = $rawLine.Trim()
    if (-not $line -or $line.StartsWith("#")) {
      continue
    }
    $parts = $line -split "=", 2
    if ($parts.Count -ne 2) {
      continue
    }
    if (-not [string]::IsNullOrWhiteSpace([System.Environment]::GetEnvironmentVariable($parts[0].Trim(), "Process"))) {
      continue
    }
    Set-Item -Path ("Env:{0}" -f $parts[0].Trim()) -Value $parts[1].Trim()
  }
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

function Get-LocalVercelProjectLink {
  param([string]$RootDir)
  $projectJsonPath = Join-Path $RootDir ".vercel\project.json"
  if (-not (Test-Path -LiteralPath $projectJsonPath)) {
    return $null
  }
  try {
    return Get-Content -Raw -LiteralPath $projectJsonPath | ConvertFrom-Json
  } catch {
    throw "Unable to parse local Vercel project link: $projectJsonPath"
  }
}

function Assert-VercelProjectLink {
  param(
    [string]$RootDir,
    [string]$ExpectedProjectName,
    [string]$ExpectedScope
  )
  $project = Get-LocalVercelProjectLink -RootDir $RootDir
  if ($null -eq $project) {
    $localProjectName = ""
  } else {
    $localProjectName = [string]$project.projectName
  }
  if ([string]::IsNullOrWhiteSpace($localProjectName)) {
    throw "Local Vercel project link was not found. Run scripts\vercel\link_project.cmd from the skill/ directory before deploying."
  }
  if (-not [string]::IsNullOrWhiteSpace($ExpectedProjectName) -and $localProjectName -ne $ExpectedProjectName) {
    throw "Local Vercel project link mismatch. VERCEL_PROJECT_NAME='$ExpectedProjectName' but .vercel/project.json is linked to '$localProjectName'. Re-run scripts\vercel\link_project.cmd after confirming which Vercel project owns yian.cdao.online."
  }
  $localOrgId = [string]$project.orgId
  if ($ExpectedScope -eq "iunknow588" -and $localOrgId.StartsWith("team_")) {
    throw "Local Vercel project link is bound to a team org ($localOrgId), but storylock-gateway production must deploy under personal scope '$ExpectedScope'. Delete .vercel/project.json and re-run scripts\vercel\link_project.cmd after confirming the Vercel token belongs to iunknow588."
  }
  return $localProjectName
}

function Test-VercelCliReady {
  param(
    [string]$Token,
    [string]$Scope
  )
  if (-not (Get-Command vercel -ErrorAction SilentlyContinue)) {
    throw "Vercel CLI not found. Install it first or run scripts\vercel\link_project.cmd."
  }

  $whoamiArgs = @("whoami")
  if (-not [string]::IsNullOrWhiteSpace($Token)) {
    $whoamiArgs += @("--token", $Token)
  }
  if (-not [string]::IsNullOrWhiteSpace($Scope)) {
    $whoamiArgs += @("--scope", $Scope)
  }

  Write-Output "Checking Vercel CLI authentication and network..."
  $previousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try {
    $output = & vercel @whoamiArgs 2>&1
    $exitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $previousErrorActionPreference
  }
  if ($exitCode -eq 0) {
    Write-Output ($output -join [Environment]::NewLine)
    return
  }

  $message = ($output | Out-String).Trim()
  if ($message -match "openid-configuration|Client network socket disconnected|TLS connection|FetchError") {
    throw "Vercel CLI cannot reach the Vercel auth/OIDC endpoint. Configure VERCEL_TOKEN, check proxy/firewall/TLS interception, or retry from a network that can access https://vercel.com/.well-known/openid-configuration. Original error: $message"
  }
  if ([string]::IsNullOrWhiteSpace($Token) -and $message -match "not authenticated|login|token|Unauthorized|No existing credentials") {
    throw "Vercel CLI is not authenticated and VERCEL_TOKEN is empty. Run 'vercel login' or set VERCEL_TOKEN in scripts\vercel\.env before deploying."
  }
  throw "Vercel CLI authentication check failed with exit code $exitCode. $message"
}

function Invoke-VercelDeployWithRetry {
  param(
    [string[]]$Arguments,
    [int]$MaxAttempts
  )
  $attempts = [Math]::Max(1, $MaxAttempts)
  $lastOutput = ""
  for ($attempt = 1; $attempt -le $attempts; $attempt += 1) {
    Write-Output ("Running: vercel {0} (attempt {1}/{2})" -f ($Arguments -join " "), $attempt, $attempts)
    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
      $output = & vercel @Arguments 2>&1
      $exitCode = $LASTEXITCODE
    } finally {
      $ErrorActionPreference = $previousErrorActionPreference
    }
    $lastOutput = ($output | Out-String).Trim()
    if (-not [string]::IsNullOrWhiteSpace($lastOutput)) {
      Write-Output $lastOutput
    }
    if ($exitCode -eq 0) {
      return $lastOutput
    }
    if ($attempt -lt $attempts) {
      Start-Sleep -Seconds ([Math]::Min(10, 2 * $attempt))
    }
  }

  if ($lastOutput -match "openid-configuration|Client network socket disconnected|TLS connection|FetchError") {
    throw "Vercel deploy could not reach Vercel auth/OIDC over TLS after $attempts attempt(s). Set VERCEL_TOKEN, verify proxy/firewall/TLS settings, or deploy from a network that can reach Vercel. Original error: $lastOutput"
  }
  throw "Vercel deploy failed after $attempts attempt(s). Last output: $lastOutput"
}

function Get-VercelDeploymentUrl {
  param([string]$DeployOutput)
  if ([string]::IsNullOrWhiteSpace($DeployOutput)) {
    return ""
  }
  $matches = [regex]::Matches($DeployOutput, "https://[^\s]+\.vercel\.app")
  if ($matches.Count -eq 0) {
    return ""
  }
  return $matches[$matches.Count - 1].Value.Trim()
}

function Test-VercelDomainAccess {
  param(
    [string]$Domain,
    [string]$Token,
    [string]$Scope
  )
  if ([string]::IsNullOrWhiteSpace($Domain)) {
    return
  }

  $args = @("domains", "inspect", $Domain)
  if (-not [string]::IsNullOrWhiteSpace($Token)) {
    $args += @("--token", $Token)
  }
  if (-not [string]::IsNullOrWhiteSpace($Scope)) {
    $args += @("--scope", $Scope)
  }

  Write-Output "Checking Vercel domain access: $Domain"
  $previousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try {
    $output = & vercel @args 2>&1
    $exitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $previousErrorActionPreference
  }
  $message = ($output | Out-String).Trim()
  if (-not [string]::IsNullOrWhiteSpace($message)) {
    Write-Output $message
  }
  if ($exitCode -ne 0) {
    Write-Output "Vercel domain inspect did not succeed. If yian.cdao.online still returns deployment-level 404, confirm the domain is added to the same Vercel project/account as the production deployment."
  }
}

function Set-VercelDeploymentAlias {
  param(
    [string]$DeploymentUrl,
    [string]$Domain,
    [string]$Token,
    [string]$Scope
  )
  if ([string]::IsNullOrWhiteSpace($DeploymentUrl) -or [string]::IsNullOrWhiteSpace($Domain)) {
    return
  }

  $args = @("alias", "set", $DeploymentUrl, $Domain)
  if (-not [string]::IsNullOrWhiteSpace($Token)) {
    $args += @("--token", $Token)
  }
  if (-not [string]::IsNullOrWhiteSpace($Scope)) {
    $args += @("--scope", $Scope)
  }

  Write-Output "Binding custom domain to this deployment: $Domain -> $DeploymentUrl"
  $previousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try {
    $output = & vercel @args 2>&1
    $exitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $previousErrorActionPreference
  }
  $message = ($output | Out-String).Trim()
  if (-not [string]::IsNullOrWhiteSpace($message)) {
    Write-Output $message
  }
  if ($exitCode -ne 0) {
    throw "Vercel alias binding failed for $Domain. Confirm the domain belongs to this Vercel account/project and DNS is configured for Vercel."
  }
}

Import-EnvFile -Path $EnvFile
Import-EnvFile -Path $WindowsEnvFile
Import-EnvFile -Path $exampleEnvPath

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
    Write-Output "Running local Vercel preflight..."
    & $preflightScript -EnvFile $EnvFile -SkipHttp
  }

  if ($SiteHttpSmoke) {
    Write-Output "Running local site HTTP smoke test..."
    npm run test:site-http
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
  $vercelArgs = @("deploy", "--yes")
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
  $localVercelProjectName = Assert-VercelProjectLink -RootDir $resolvedProjectDir -ExpectedProjectName $projectName -ExpectedScope $scopeValue
  if (-not [string]::IsNullOrWhiteSpace($scopeValue)) {
    $vercelArgs += @("--scope", $scopeValue)
  }
  $customDomainValue = if ([string]::IsNullOrWhiteSpace($VercelCustomDomain)) {
    [System.Environment]::GetEnvironmentVariable("VERCEL_CUSTOM_DOMAIN", "Process")
  } else {
    $VercelCustomDomain
  }
  $bindCustomDomainValue = $BindCustomDomain -or ([System.Environment]::GetEnvironmentVariable("VERCEL_BIND_CUSTOM_DOMAIN", "Process") -eq "true")

  $planPath = Join-Path $releaseOutputDir "vercel-release-plan.json"
  [PSCustomObject]@{
    target = "vercel"
    projectDir = $resolvedProjectDir
    projectName = $projectName
    localVercelProjectName = $localVercelProjectName
    buildRequested = [bool]$Build
    preflightRequested = [bool]$Preflight
    siteHttpSmokeRequested = [bool]$SiteHttpSmoke
    prod = [bool]$Prod
    execute = [bool]$Execute
    deployRetries = $DeployRetries
    command = @("vercel") + $vercelArgs
    customDomain = $customDomainValue
    bindCustomDomain = [bool]$bindCustomDomainValue
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

  Test-VercelCliReady -Token $tokenValue -Scope $scopeValue
  Test-VercelDomainAccess -Domain $customDomainValue -Token $tokenValue -Scope $scopeValue
  $deployOutput = Invoke-VercelDeployWithRetry -Arguments $vercelArgs -MaxAttempts $DeployRetries
  $deploymentUrl = Get-VercelDeploymentUrl -DeployOutput $deployOutput
  if (-not [string]::IsNullOrWhiteSpace($deploymentUrl)) {
    Write-Output "Deployment URL: $deploymentUrl"
  } else {
    Write-Output "Deployment URL could not be parsed from Vercel output; custom-domain alias binding will be skipped."
  }
  if ($Prod -and $bindCustomDomainValue) {
    Set-VercelDeploymentAlias -DeploymentUrl $deploymentUrl -Domain $customDomainValue -Token $tokenValue -Scope $scopeValue
  } elseif ($Prod -and -not [string]::IsNullOrWhiteSpace($customDomainValue)) {
    Write-Output "Custom domain binding was not forced. If $customDomainValue returns 404 after a successful deploy, re-run with -BindCustomDomain after confirming domain ownership."
  }
  if ($Preflight) {
    $postDeployBaseUrl = [System.Environment]::GetEnvironmentVariable("STORYLOCK_GATEWAY_PUBLIC_URL", "Process")
    if (-not [string]::IsNullOrWhiteSpace($postDeployBaseUrl)) {
      Write-Output "Running post-deploy Vercel preflight..."
      & $preflightScript -EnvFile $EnvFile -BaseUrl $postDeployBaseUrl
    } else {
      Write-Output "Skipping post-deploy HTTP preflight because STORYLOCK_GATEWAY_PUBLIC_URL is empty."
    }
  }
} finally {
  Pop-Location
}
