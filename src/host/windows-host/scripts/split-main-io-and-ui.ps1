$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$srcRoot = Join-Path $projectRoot "src"
$mainPath = Join-Path $srcRoot "main.rs"
$moduleDir = Join-Path $srcRoot "host_runtime"
$ioPath = Join-Path $moduleDir "io.rs"
$uiPath = Join-Path $moduleDir "ui.rs"

$lines = [System.IO.File]::ReadAllLines($mainPath)

$ioStart = 277
$ioEnd = 703
$uiStart = 1713
$uiEnd = 2150

[System.IO.Directory]::CreateDirectory($moduleDir) | Out-Null

$before = New-Object System.Collections.Generic.List[string]
for ($i = 0; $i -lt ($ioStart - 1); $i++) { $before.Add($lines[$i]) }

$io = New-Object System.Collections.Generic.List[string]
$io.Add("use super::*;")
$io.Add("")
for ($i = $ioStart - 1; $i -lt $ioEnd; $i++) {
    $line = $lines[$i]
    if ($line -match '^fn ') {
        $line = $line -replace '^fn ', 'pub(crate) fn '
    }
    $io.Add($line)
}

$middle = New-Object System.Collections.Generic.List[string]
for ($i = $ioEnd; $i -lt ($uiStart - 1); $i++) { $middle.Add($lines[$i]) }

$ui = New-Object System.Collections.Generic.List[string]
$ui.Add("use super::*;")
$ui.Add("")
for ($i = $uiStart - 1; $i -lt $uiEnd; $i++) {
    $line = $lines[$i]
    if ($line -match '^fn ') {
        $line = $line -replace '^fn ', 'pub(crate) fn '
    }
    $ui.Add($line)
}

$after = New-Object System.Collections.Generic.List[string]
for ($i = $uiEnd; $i -lt $lines.Length; $i++) { $after.Add($lines[$i]) }

$newMain = New-Object System.Collections.Generic.List[string]
foreach ($line in $before) { $newMain.Add($line) }
$newMain.Add("mod host_runtime;")
$newMain.Add("use host_runtime::*;")
$newMain.Add("")
foreach ($line in $middle) { $newMain.Add($line) }
foreach ($line in $after) { $newMain.Add($line) }

$modLines = @(
    "use super::*;",
    "",
    "mod state;",
    "pub(crate) use state::*;",
    "",
    "mod request_support;",
    "pub(crate) use request_support::*;",
    "",
    "mod io;",
    "pub(crate) use io::*;",
    "",
    "mod ui;",
    "pub(crate) use ui::*;"
)

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($ioPath, $io, $utf8NoBom)
[System.IO.File]::WriteAllLines($uiPath, $ui, $utf8NoBom)
[System.IO.File]::WriteAllLines((Join-Path $moduleDir "mod.rs"), $modLines, $utf8NoBom)
[System.IO.File]::WriteAllLines($mainPath, $newMain, $utf8NoBom)

Write-Output "main IO/UI split complete"
