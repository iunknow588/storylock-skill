param(
  [ValidateSet("debug", "release")]
  [string]$Variant = "debug",
  [string]$ProjectDir = (Join-Path $PSScriptRoot "..\..\android-host"),
  [string]$EnvOutput = (Join-Path $PSScriptRoot "..\vercel\.env.android-apk")
)

$ErrorActionPreference = "Stop"

$resolvedProject = Resolve-Path -LiteralPath $ProjectDir
$gradlew = Join-Path $resolvedProject "gradlew.bat"

if (Test-Path -LiteralPath $gradlew) {
  $gradleCommand = $gradlew
  $gradleArgs = @("assemble$($Variant.Substring(0, 1).ToUpper())$($Variant.Substring(1))")
} else {
  $gradle = Get-Command gradle -ErrorAction SilentlyContinue
  if (-not $gradle) {
    throw "Gradle was not found. Install Gradle or add an Android Gradle wrapper under android-host, then rerun this script."
  }
  $gradleCommand = $gradle.Source
  $gradleArgs = @("assemble$($Variant.Substring(0, 1).ToUpper())$($Variant.Substring(1))")
}

Push-Location $resolvedProject
try {
  & $gradleCommand @gradleArgs
  if ($LASTEXITCODE -ne 0) {
    throw "Gradle build failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}

$apkDir = Join-Path $resolvedProject "app\build\outputs\apk\$Variant"
$apk = Get-ChildItem -LiteralPath $apkDir -Filter "*.apk" -File |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

if (-not $apk) {
  throw "No APK was produced under $apkDir"
}

$hash = Get-FileHash -Algorithm SHA256 -LiteralPath $apk.FullName
$versionName = "0.1.0"
$versionCode = "1"
$releaseChannel = if ($Variant -eq "debug") { "internal" } else { "candidate" }

$envDir = Split-Path -Parent $EnvOutput
New-Item -ItemType Directory -Force -Path $envDir | Out-Null
@(
  "STORYLOCK_ANDROID_APK_PATH=$($apk.FullName)"
  "STORYLOCK_ANDROID_APK_VERSION=$versionName"
  "STORYLOCK_ANDROID_APK_VERSION_CODE=$versionCode"
  "STORYLOCK_ANDROID_APK_CHECKSUM=sha256:$($hash.Hash.ToLowerInvariant())"
  "STORYLOCK_ANDROID_PACKAGE_KIND=$Variant"
  "STORYLOCK_ANDROID_RELEASE_CHANNEL=$releaseChannel"
) | Set-Content -Encoding utf8 -Path $EnvOutput

[PSCustomObject]@{
  ApkPath = $apk.FullName
  SizeBytes = $apk.Length
  Sha256 = $hash.Hash.ToLowerInvariant()
  EnvOutput = $EnvOutput
}
