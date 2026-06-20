param(
  [string]$Root = ".",
  [switch]$OnlyChanged,
  [switch]$Fix
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspaceRoot = Resolve-Path (Join-Path $scriptRoot "..\..")
$targetRoot = Resolve-Path (Join-Path (Get-Location) $Root)
$checker = Join-Path $workspaceRoot "scripts\text\check_line_endings.py"
$normalizer = Join-Path $workspaceRoot "scripts\text\normalize_text_files.py"

$lineEndingArgs = @(
  $checker,
  "--root",
  $targetRoot,
  "--fail-on-bom",
  "--fail-on-crlf"
)

if ($OnlyChanged) {
  $lineEndingArgs += "--only-changed"
}

python @lineEndingArgs

if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}

$normalizeArgs = @(
  $normalizer,
  "--root",
  $targetRoot
)

if ($Fix) {
  $normalizeArgs += "--fix"
} else {
  $normalizeArgs += "--dry-run"
  $normalizeArgs += "--fail-on-change"
}

python @normalizeArgs

if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}
