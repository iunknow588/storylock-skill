param(
  [string]$Root = ".",
  [switch]$OnlyChanged
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspaceRoot = Resolve-Path (Join-Path $scriptRoot "..\..")
$targetRoot = Resolve-Path (Join-Path (Get-Location) $Root)
$checker = Join-Path $workspaceRoot "scripts\text\check_line_endings.py"

$args = @(
  $checker,
  "--root",
  $targetRoot,
  "--fail-on-bom",
  "--fail-on-crlf"
)

if ($OnlyChanged) {
  $args += "--only-changed"
}

python @args
