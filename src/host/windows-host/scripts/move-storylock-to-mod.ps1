$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$slintUiDir = Join-Path $projectRoot "src\slint_ui"
$oldPath = Join-Path $slintUiDir "storylock.rs"
$newPath = Join-Path (Join-Path $slintUiDir "storylock") "mod.rs"

if (-not (Test-Path -LiteralPath $oldPath)) {
    throw "source file not found: $oldPath"
}

Move-Item -LiteralPath $oldPath -Destination $newPath -Force
Write-Output "storylock moved to mod.rs"
