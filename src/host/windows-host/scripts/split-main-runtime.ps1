$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$srcRoot = Join-Path $projectRoot "src"
$mainPath = Join-Path $srcRoot "main.rs"
$moduleDir = Join-Path $srcRoot "host_runtime"
$modPath = Join-Path $moduleDir "mod.rs"
$statePath = Join-Path $moduleDir "state.rs"
$requestSupportPath = Join-Path $moduleDir "request_support.rs"

$lines = [System.IO.File]::ReadAllLines($mainPath)

$stateStart = 28
$stateEnd = 778
$requestStart = 779
$requestEnd = 1469

if ($lines.Length -lt $requestEnd) {
    throw "main.rs is shorter than expected; aborting split"
}

[System.IO.Directory]::CreateDirectory($moduleDir) | Out-Null

$header = New-Object System.Collections.Generic.List[string]
for ($i = 0; $i -lt 26; $i++) {
    $header.Add($lines[$i])
}
$header.Add("mod host_runtime;")
$header.Add("use host_runtime::*;")
$header.Add("")

$state = New-Object System.Collections.Generic.List[string]
$state.Add("use super::*;")
$state.Add("")
for ($i = $stateStart - 1; $i -lt $stateEnd; $i++) {
    $line = $lines[$i]
    if ($line -match '^struct ') {
        $line = $line -replace '^struct ', 'pub(crate) struct '
    } elseif ($line -match '^impl ') {
        $line = $line
    } elseif ($line -match '^fn ') {
        $line = $line -replace '^fn ', 'pub(crate) fn '
    }
    $state.Add($line)
}

$requestSupport = New-Object System.Collections.Generic.List[string]
$requestSupport.Add("use super::*;")
$requestSupport.Add("")
for ($i = $requestStart - 1; $i -lt $requestEnd; $i++) {
    $line = $lines[$i]
    if ($line -match '^fn ') {
        $line = $line -replace '^fn ', 'pub(crate) fn '
    }
    $requestSupport.Add($line)
}

$remainder = New-Object System.Collections.Generic.List[string]
for ($i = $requestEnd; $i -lt $lines.Length; $i++) {
    $remainder.Add($lines[$i])
}

$newMain = New-Object System.Collections.Generic.List[string]
foreach ($line in $header) { $newMain.Add($line) }
foreach ($line in $remainder) { $newMain.Add($line) }

$modLines = @(
    "use super::*;",
    "",
    "mod state;",
    "pub(crate) use state::*;",
    "",
    "mod request_support;",
    "pub(crate) use request_support::*;"
)

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($statePath, $state, $utf8NoBom)
[System.IO.File]::WriteAllLines($requestSupportPath, $requestSupport, $utf8NoBom)
[System.IO.File]::WriteAllLines($modPath, $modLines, $utf8NoBom)
[System.IO.File]::WriteAllLines($mainPath, $newMain, $utf8NoBom)

Write-Output "main runtime split complete"
