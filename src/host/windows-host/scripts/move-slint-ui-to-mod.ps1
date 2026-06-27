$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$srcRoot = Join-Path $projectRoot "src"
$oldPath = Join-Path $srcRoot "slint_ui.rs"
$newPath = Join-Path (Join-Path $srcRoot "slint_ui") "mod.rs"

if (-not (Test-Path -LiteralPath $oldPath)) {
    throw "source file not found: $oldPath"
}

Move-Item -LiteralPath $oldPath -Destination $newPath -Force
Write-Output "moved to mod.rs"
