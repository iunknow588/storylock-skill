$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$storylockPath = Join-Path (Join-Path $projectRoot "src\slint_ui") "storylock.rs"
$moduleDir = Join-Path (Join-Path $projectRoot "src\slint_ui") "storylock"
$coreDataPath = Join-Path $moduleDir "core_data.rs"

$lines = [System.IO.File]::ReadAllLines($storylockPath)

$startLine = 3
$endLine = 678

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
$newRoot.Add("mod core_data;")
$newRoot.Add("use core_data::*;")
$newRoot.Add("")
foreach ($line in $after) {
    $newRoot.Add($line)
}

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($coreDataPath, $extracted, $utf8NoBom)
[System.IO.File]::WriteAllLines($storylockPath, $newRoot, $utf8NoBom)

Write-Output "storylock core data extracted"
