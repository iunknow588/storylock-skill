$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$srcRoot = Join-Path $projectRoot "src"
$slintUiPath = Join-Path $srcRoot "slint_ui.rs"
$storylockModulePath = Join-Path (Join-Path $srcRoot "slint_ui") "storylock.rs"

$lines = [System.IO.File]::ReadAllLines($slintUiPath)

$startLine = 419
$endLine = 4134

if ($lines.Length -lt $endLine) {
    throw "slint_ui.rs is shorter than expected; aborting refactor"
}

$before = New-Object System.Collections.Generic.List[string]
for ($i = 0; $i -lt ($startLine - 1); $i++) {
    $before.Add($lines[$i])
}

$moved = New-Object System.Collections.Generic.List[string]
$moved.Add("use super::*;")
$moved.Add("")
for ($i = $startLine - 1; $i -lt $endLine; $i++) {
    $moved.Add($lines[$i])
}

$after = New-Object System.Collections.Generic.List[string]
for ($i = $endLine; $i -lt $lines.Length; $i++) {
    $after.Add($lines[$i])
}

$newRoot = New-Object System.Collections.Generic.List[string]
foreach ($line in $before) {
    $newRoot.Add($line)
}
$newRoot.Add("mod storylock;")
$newRoot.Add("use storylock::*;")
$newRoot.Add("")
foreach ($line in $after) {
    $newRoot.Add($line)
}

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($storylockModulePath, $moved, $utf8NoBom)
[System.IO.File]::WriteAllLines($slintUiPath, $newRoot, $utf8NoBom)

Write-Output "storylock module extracted"
