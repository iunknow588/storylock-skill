param(
    [string]$ExePath = "",
    [string]$OutputDir = "",
    [int]$TimeoutSeconds = 20
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$skillRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")
$toolPath = Join-Path $skillRoot "src\host\windows-host\tools\ui-doc-screenshots\capture_storylock_ui.ps1"

if (-not (Test-Path -LiteralPath $toolPath)) {
    throw "Moved capture helper not found: $toolPath"
}

& $toolPath @PSBoundParameters
