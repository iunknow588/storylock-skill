param(
  [Parameter(Mandatory = $true)]
  [string]$FilePath
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $FilePath)) {
  throw "File to sign was not found: $FilePath"
}

$signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
if (-not $signtool) {
  throw "signtool.exe was not found. Install the Windows SDK signing tools or add signtool.exe to PATH."
}

$certThumbprint = $env:STORYLOCK_WINDOWS_SIGN_CERT_THUMBPRINT
$timestampUrl = $env:STORYLOCK_WINDOWS_SIGN_TIMESTAMP_URL

if ([string]::IsNullOrWhiteSpace($certThumbprint)) {
  throw "STORYLOCK_WINDOWS_SIGN_CERT_THUMBPRINT is required for signing."
}
if ([string]::IsNullOrWhiteSpace($timestampUrl)) {
  throw "STORYLOCK_WINDOWS_SIGN_TIMESTAMP_URL is required for signing."
}

& $signtool.Source sign /sha1 $certThumbprint /fd SHA256 /tr $timestampUrl /td SHA256 $FilePath
if ($LASTEXITCODE -ne 0) {
  throw "signtool failed for $FilePath with exit code $LASTEXITCODE"
}

Write-Output "Signed package: $FilePath"
