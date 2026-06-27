$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$storylockPath = Join-Path (Join-Path $projectRoot "src\slint_ui") "storylock.rs"
$moduleDir = Join-Path (Join-Path $projectRoot "src\slint_ui") "storylock"
$modulePath = Join-Path $moduleDir "resource_export.rs"

$lines = [System.IO.File]::ReadAllLines($storylockPath)

$startLine = 567
$endLine = 1539

if ($lines.Length -lt $endLine) {
    throw "storylock.rs is shorter than expected; aborting extract"
}

[System.IO.Directory]::CreateDirectory($moduleDir) | Out-Null

$before = New-Object System.Collections.Generic.List[string]
for ($i = 0; $i -lt ($startLine - 1); $i++) {
    $before.Add($lines[$i])
}

$extracted = New-Object System.Collections.Generic.List[string]
$extracted.Add("use super::*;")
$extracted.Add("")
for ($i = $startLine - 1; $i -lt $endLine; $i++) {
    $line = $lines[$i]
    if ($line -match '^fn ') {
        $line = $line -replace '^fn ', 'pub(super) fn '
    } elseif ($line -match '^struct PreflightIssue') {
        $line = $line -replace '^struct PreflightIssue', 'pub(super) struct PreflightIssue'
    } elseif ($line -match '^struct PreflightResult') {
        $line = $line -replace '^struct PreflightResult', 'pub(super) struct PreflightResult'
    }
    $extracted.Add($line)
}

$after = New-Object System.Collections.Generic.List[string]
for ($i = $endLine; $i -lt $lines.Length; $i++) {
    $after.Add($lines[$i])
}

$newRoot = New-Object System.Collections.Generic.List[string]
foreach ($line in $before) {
    $newRoot.Add($line)
}
$newRoot.Add("mod resource_export;")
$newRoot.Add("pub(super) use resource_export::*;")
$newRoot.Add("")
foreach ($line in $after) {
    $newRoot.Add($line)
}

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($modulePath, $extracted, $utf8NoBom)
[System.IO.File]::WriteAllLines($storylockPath, $newRoot, $utf8NoBom)

Write-Output "storylock resource/export extracted"
