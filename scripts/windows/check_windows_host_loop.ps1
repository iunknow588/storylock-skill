param(
  [string]$ProjectDir = (Join-Path $PSScriptRoot "..\..\src\host\windows-host"),
  [string]$DataDir = (Join-Path $PSScriptRoot "..\..\.temp\runtime\windows-host-loop-data"),
  [string]$BindHost = "127.0.0.1",
  [int]$Port = 4510
)

$ErrorActionPreference = "Stop"

function Invoke-JsonPost {
  param(
    [string]$Uri,
    [hashtable]$Body
  )
  Invoke-RestMethod -Method Post -Uri $Uri -ContentType "application/json" -Body ($Body | ConvertTo-Json -Depth 8)
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

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "Cargo was not found. Install Rust from https://rustup.rs/ and rerun this script."
}

$project = Resolve-Path -LiteralPath $ProjectDir
$data = [System.IO.Path]::GetFullPath($DataDir)
New-Item -ItemType Directory -Force -Path $data | Out-Null

$env:STORYLOCK_WINDOWS_DATA_DIR = $data
$env:STORYLOCK_WINDOWS_APPROVAL_MODE = "auto_approve"
$env:STORYLOCK_WINDOWS_HOST_PORT = "$Port"

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
  $proc = Start-Process cargo -ArgumentList @("run") -WorkingDirectory $project -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog

  $health = $null
  for ($attempt = 0; $attempt -lt 20; $attempt++) {
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

  Add-Result $rows "health" "ok" ("approvalMode={0}; questionBankPath={1}" -f $health.approvalMode, $health.storage.path)

  $questionBankStatus = Invoke-RestMethod -Method Get -Uri "$baseUrl/question-bank/status"
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
  Add-Result $rows "question-bank-import" "ok" ("questionSetVersion={0}; questionCount={1}" -f $importResponse.result.questionSetVersion, $importResponse.result.questionCount)

  $verify = Invoke-JsonPost -Uri "$baseUrl/verify" -Body @{
    requestId = "req-verify-loop-001"
    capability = "requestSignature"
    keyId = "wallet-loop"
  }
  $cells = @($verify.result.grid.cells)
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
  Add-Result $rows "authorize" "ok" ("authorizationId={0}" -f $authorize.result.authorizationId)

  $execute = Invoke-JsonPost -Uri "$baseUrl/execute" -Body @{
    requestId = "req-execute-loop-001"
    capability = "requestSignature"
    keyId = "wallet-loop"
    authorizationId = $authorize.result.authorizationId
  }
  Add-Result $rows "execute" "ok" ("signature={0}" -f $execute.result.signature)

  $revoke = Invoke-JsonPost -Uri "$baseUrl/revoke" -Body @{
    requestId = "req-revoke-loop-001"
    authorizationId = $authorize.result.authorizationId
  }
  Add-Result $rows "revoke" "ok" ("status={0}; authorizationId={1}" -f $revoke.result.status, $revoke.result.authorizationId)

  $rows | Format-Table -AutoSize
  Write-Output "Windows host loop check passed."
}
finally {
  if ($proc -and -not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force
  }
}
