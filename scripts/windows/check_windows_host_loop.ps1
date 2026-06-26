param(
  [string]$ProjectDir = (Join-Path $PSScriptRoot "..\..\src\host\windows-host"),
  [string]$DataDir = (Join-Path $PSScriptRoot "..\..\.temp\runtime\windows-host-loop-data"),
  [string]$TargetDir = (Join-Path $PSScriptRoot "..\..\.temp\runtime\windows-host-loop-target"),
  [string]$BindHost = "127.0.0.1",
  [int]$Port = 4510
)

$ErrorActionPreference = "Stop"

function Invoke-JsonPost {
  param(
    [string]$Uri,
    [hashtable]$Body
  )
  $json = $Body | ConvertTo-Json -Depth 8
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
  Invoke-RestMethod -Method Post -Uri $Uri -ContentType "application/json; charset=utf-8" -Body $bytes
}

function Add-Result {
  param(
    [System.Collections.Generic.List[object]]$Rows,
    [string]$Name,
    [string]$Status,
    [string]$Detail
  )
  $Rows.Add([PSCustomObject]@{
      Check = $Name
      Status = $Status
      Detail = $Detail
    }) | Out-Null
}

function Assert-Success {
  param(
    [object]$Response,
    [string]$Name
  )
  if ($null -eq $Response) {
    throw "$Name returned no response"
  }
  if ($Response.status -ne "success") {
    $message = if ($Response.error -and $Response.error.message) { $Response.error.message } else { ($Response | ConvertTo-Json -Depth 8) }
    throw "$Name failed: $message"
  }
}

function Assert-Value {
  param(
    [object]$Value,
    [string]$Name
  )
  if ($null -eq $Value -or [string]::IsNullOrWhiteSpace([string]$Value)) {
    throw "$Name was empty"
  }
}

function Assert-NoSensitiveText {
  param(
    [object]$Value,
    [string]$Name
  )
  $text = $Value | ConvertTo-Json -Depth 16
  foreach ($pattern in @("SUMMIT", "ORBIT", "ANCHOR", "shared_secret", "shared-secret-value", "story raw text")) {
    if ($text -match $pattern) {
      throw "$Name leaked sensitive marker: $pattern"
    }
  }
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Cargo was not found. Install Rust from https://rustup.rs/ and rerun this script."
}

$project = Resolve-Path -LiteralPath $ProjectDir
$data = [System.IO.Path]::GetFullPath($DataDir)
$target = [System.IO.Path]::GetFullPath($TargetDir)
if (Test-Path -LiteralPath $data) {
  Remove-Item -LiteralPath $data -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $data | Out-Null
if (Test-Path -LiteralPath $target) {
  Remove-Item -LiteralPath $target -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $target | Out-Null

$env:STORYLOCK_WINDOWS_DATA_DIR = $data
$env:STORYLOCK_WINDOWS_APPROVAL_MODE = "auto_approve"
$env:STORYLOCK_WINDOWS_HOST_PORT = "$Port"
$env:CARGO_TARGET_DIR = $target

$listener = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
if ($listener) {
  throw "Port $Port is already in use. Stop the existing listener or choose another port."
}

$rows = [System.Collections.Generic.List[object]]::new()
$baseUrl = "http://$BindHost`:$Port"
$proc = $null
$stdoutLog = Join-Path $data "windows-host.stdout.log"
$stderrLog = Join-Path $data "windows-host.stderr.log"

try {
  if (Test-Path -LiteralPath $stdoutLog) { Remove-Item -LiteralPath $stdoutLog -Force }
  if (Test-Path -LiteralPath $stderrLog) { Remove-Item -LiteralPath $stderrLog -Force }
  $proc = Start-Process cargo -ArgumentList @("run", "--no-default-features") -WorkingDirectory $project -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog

  $health = $null
  for ($attempt = 0; $attempt -lt 80; $attempt++) {
    Start-Sleep -Milliseconds 750
    try {
      $health = Invoke-RestMethod -Method Get -Uri "$baseUrl/health"
      break
    } catch {
      if ($proc.HasExited) {
        break
      }
    }
  }
  if (-not $health) {
    $stdoutTail = if (Test-Path -LiteralPath $stdoutLog) { Get-Content -LiteralPath $stdoutLog -Tail 20 | Out-String } else { "" }
    $stderrTail = if (Test-Path -LiteralPath $stderrLog) { Get-Content -LiteralPath $stderrLog -Tail 20 | Out-String } else { "" }
    throw "Windows host did not become healthy. Stdout:`n$stdoutTail`nStderr:`n$stderrTail"
  }

  Assert-Value $health.approvalMode "health.approvalMode"
  Assert-Value $health.storage.provider "health.storage.provider"
  Assert-Value $health.storage.visibility "health.storage.visibility"
  Assert-Value $health.questionBank.visibility "health.questionBank.visibility"
  Add-Result $rows "health" "ok" ("approvalMode={0}; storageProvider={1}; questionBankVisibility={2}" -f $health.approvalMode, $health.storage.provider, $health.questionBank.visibility)

  $uiPage = Invoke-WebRequest -Method Get -Uri "$baseUrl/ui"
  if ($uiPage.StatusCode -ne 200 -or $uiPage.Content -notmatch "Yian Windows Host") {
    throw "GET /ui did not return the local management page"
  }
  $uiStatus = Invoke-RestMethod -Method Get -Uri "$baseUrl/ui/status"
  if ($uiStatus.status -ne "success") {
    throw "GET /ui/status did not return success"
  }
  Assert-Value $uiStatus.result.host.identityId "ui-status.result.host.identityId"
  Assert-Value $uiStatus.result.questionBank.questionSetVersion "ui-status.result.questionBank.questionSetVersion"
  Add-Result $rows "ui-status" "ok" ("managementUrl={0}; relay={1}" -f $uiStatus.result.ui.managementUrl, $uiStatus.result.relay.status)

  $diagnostics = Invoke-RestMethod -Method Get -Uri "$baseUrl/diagnostics"
  Assert-Success $diagnostics "diagnostics"
  Assert-Value $diagnostics.result.localEndpoints.management "diagnostics.result.localEndpoints.management"
  Assert-Value $diagnostics.result.localEndpoints.shutdown "diagnostics.result.localEndpoints.shutdown"
  if (@($diagnostics.result.redaction.hidden) -notcontains "sharedSecret") {
    throw "diagnostics.result.redaction.hidden did not include sharedSecret"
  }
  Assert-NoSensitiveText $diagnostics "diagnostics"
  Add-Result $rows "diagnostics" "ok" ("redaction={0}; shutdown={1}" -f $diagnostics.result.redaction.level, $diagnostics.result.localEndpoints.shutdown)

  $questionBankStatus = Invoke-RestMethod -Method Get -Uri "$baseUrl/question-bank/status"
  Assert-Success $questionBankStatus "question-bank-status"
  Assert-Value $questionBankStatus.result.questionSetVersion "question-bank-status.result.questionSetVersion"
  Assert-Value $questionBankStatus.result.questionCount "question-bank-status.result.questionCount"
  Add-Result $rows "question-bank-status" "ok" ("questionSetVersion={0}; questionCount={1}" -f $questionBankStatus.result.questionSetVersion, $questionBankStatus.result.questionCount)

  $tempImport = Join-Path $data "import-question-bank.json"
  @'
{
  "schemaVersion": "windows-local-question-bank-v1",
  "questionSetVersion": "windows-loop-v2",
  "normalizationVersion": "upper-ascii-v1",
  "questions": [
    {
      "questionId": "loop-q-01",
      "promptRef": "loop-prompt-01",
      "versionTag": "v2",
      "promptText": "Imported loop question 1.",
      "answer": "SUMMIT"
    },
    {
      "questionId": "loop-q-02",
      "promptRef": "loop-prompt-02",
      "versionTag": "v2",
      "promptText": "Imported loop question 2.",
      "answer": "ORBIT"
    },
    {
      "questionId": "loop-q-03",
      "promptRef": "loop-prompt-03",
      "versionTag": "v2",
      "promptText": "Imported loop question 3.",
      "answer": "ANCHOR"
    }
  ]
}
'@ | Set-Content -LiteralPath $tempImport -Encoding UTF8

  $importResponse = Invoke-JsonPost -Uri "$baseUrl/question-bank/import" -Body @{
    requestId = "req-import-loop-001"
    sourcePath = $tempImport
  }
  Assert-Success $importResponse "question-bank-import"
  Assert-Value $importResponse.result.questionSetVersion "question-bank-import.result.questionSetVersion"
  Assert-Value $importResponse.result.questionCount "question-bank-import.result.questionCount"
  Add-Result $rows "question-bank-import" "ok" ("questionSetVersion={0}; questionCount={1}" -f $importResponse.result.questionSetVersion, $importResponse.result.questionCount)

  $verify = Invoke-JsonPost -Uri "$baseUrl/verify" -Body @{
    requestId = "req-verify-loop-001"
    capability = "requestSignature"
    keyId = "wallet-loop"
  }
  Assert-Success $verify "verify"
  Assert-Value $verify.result.verificationId "verify.result.verificationId"
  $cells = @($verify.result.grid.cells)
  if ($cells.Count -lt 1) {
    throw "verify.result.grid.cells was empty"
  }
  $answers = @()
  foreach ($cell in $cells) {
    $answer = switch ($cell.questionId) {
      "loop-q-01" { "summit" }
      "loop-q-02" { "orbit" }
      "loop-q-03" { "anchor" }
      default { "UNKNOWN" }
    }
    $answers += @{
      cellId = $cell.cellId
      answer = $answer
    }
  }
  Add-Result $rows "verify" "ok" ("verificationId={0}; cells={1}" -f $verify.result.verificationId, $cells.Count)

  $authorize = Invoke-JsonPost -Uri "$baseUrl/authorize" -Body @{
    requestId = "req-authorize-loop-001"
    verificationId = $verify.result.verificationId
    answers = $answers
  }
  Assert-Success $authorize "authorize"
  Assert-Value $authorize.result.authorizationId "authorize.result.authorizationId"
  Add-Result $rows "authorize" "ok" ("authorizationId={0}" -f $authorize.result.authorizationId)

  $execute = Invoke-JsonPost -Uri "$baseUrl/execute" -Body @{
    requestId = "req-execute-loop-001"
    capability = "requestSignature"
    keyId = "wallet-loop"
    authorizationId = $authorize.result.authorizationId
  }
  Assert-Success $execute "execute"
  Assert-Value $execute.result.signature "execute.result.signature"
  Add-Result $rows "execute" "ok" ("signature={0}" -f $execute.result.signature)

  $directExecute = Invoke-JsonPost -Uri "$baseUrl/execute" -Body @{
    requestId = "req-ui-confirm-loop-001"
    capability = "requestSignature"
    keyId = "wallet-ui-confirm"
    requester = "windows-host-loop"
    origin = "http://127.0.0.1:$Port"
  }
  Assert-Success $directExecute "direct-execute-for-ui-confirmation"
  Assert-Value $directExecute.result.signature "direct-execute-for-ui-confirmation.result.signature"
  Add-Result $rows "direct-execute-ui-confirmation" "ok" ("signature={0}" -f $directExecute.result.signature)

  $uiStatusAfterExecute = Invoke-RestMethod -Method Get -Uri "$baseUrl/ui/status"
  Assert-Value $uiStatusAfterExecute.result.lastConfirmation.requestId "ui-status-after-execute.result.lastConfirmation.requestId"
  Assert-Value $uiStatusAfterExecute.result.lastConfirmation.capability "ui-status-after-execute.result.lastConfirmation.capability"
  Assert-Value $uiStatusAfterExecute.result.lastConfirmation.requiredStrength "ui-status-after-execute.result.lastConfirmation.requiredStrength"
  Assert-Value $uiStatusAfterExecute.result.lastConfirmation.risk "ui-status-after-execute.result.lastConfirmation.risk"
  if ($uiStatusAfterExecute.result.lastConfirmation.requestId -ne "req-ui-confirm-loop-001") {
    throw "ui lastConfirmation did not track the latest execute request"
  }
  Assert-Value $uiStatusAfterExecute.result.lastExecution.requestId "ui-status-after-execute.result.lastExecution.requestId"
  if ($uiStatusAfterExecute.result.lastExecution.requestId -ne "req-ui-confirm-loop-001") {
    throw "ui lastExecution did not track the latest execute request"
  }
  Add-Result $rows "ui-confirmation" "ok" ("requestId={0}; strength={1}" -f $uiStatusAfterExecute.result.lastConfirmation.requestId, $uiStatusAfterExecute.result.lastConfirmation.requiredStrength)
  Add-Result $rows "ui-last-execution" "ok" ("requestId={0}; capability={1}" -f $uiStatusAfterExecute.result.lastExecution.requestId, $uiStatusAfterExecute.result.lastExecution.capability)

  $revoke = Invoke-JsonPost -Uri "$baseUrl/revoke" -Body @{
    requestId = "req-revoke-loop-001"
    authorizationId = $authorize.result.authorizationId
  }
  Assert-Success $revoke "revoke"
  Assert-Value $revoke.result.status "revoke.result.status"
  Assert-Value $revoke.result.authorizationId "revoke.result.authorizationId"
  if ($revoke.result.status -ne "revoked") {
    throw "revoke.result.status expected revoked but got $($revoke.result.status)"
  }
  Add-Result $rows "revoke" "ok" ("status={0}; authorizationId={1}" -f $revoke.result.status, $revoke.result.authorizationId)

  $shutdown = Invoke-RestMethod -Method Post -Uri "$baseUrl/shutdown"
  Assert-Success $shutdown "shutdown"
  Start-Sleep -Milliseconds 750
  if ($proc -and -not $proc.HasExited) {
    try {
      Invoke-RestMethod -Method Get -Uri "$baseUrl/health" -TimeoutSec 1 | Out-Null
      throw "Host still responded to /health after shutdown"
    } catch {
      if ($_.Exception.Message -eq "Host still responded to /health after shutdown") {
        throw
      }
    }
  }
  $listenerAfterShutdown = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
  if ($listenerAfterShutdown) {
    throw "Port $Port was still listening after /shutdown"
  }
  Add-Result $rows "shutdown" "ok" "host accepted local shutdown and released the port"

  $rows | Format-Table -AutoSize
  Write-Output "Windows host loop check passed."
}
finally {
  if ($proc -and -not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force
  }
}
