param(
  [ValidateSet("debug", "release")]
  [string]$Variant = "debug",
  [string]$ProjectDir = "",
  [string]$PackageOutputDir = "",
  [string]$EnvOutput = ""
)

$ErrorActionPreference = "Stop"

# Android host release packaging now depends on unified story-draft assets under
# app/src/main/assets/story-drafts/manifest.json and the referenced draft files.

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
if ([string]::IsNullOrWhiteSpace($ProjectDir)) {
  $ProjectDir = Join-Path $repoRoot "src\host\android-host"
}
if ([string]::IsNullOrWhiteSpace($PackageOutputDir)) {
  $PackageOutputDir = Join-Path $repoRoot "release\app\android"
}
if ([string]::IsNullOrWhiteSpace($EnvOutput)) {
  $EnvOutput = Join-Path $repoRoot ".temp\vercel\android-package.env"
}

$resolvedProject = Resolve-Path -LiteralPath $ProjectDir
$gradlew = Join-Path $resolvedProject "gradlew.bat"
$localGradle = Join-Path $repoRoot ".tools\gradle-8.7\bin\gradle.bat"
$defaultSdk = "C:\Program Files (x86)\Android\android-sdk"

if (-not $env:ANDROID_HOME -and (Test-Path -LiteralPath $defaultSdk)) {
  $env:ANDROID_HOME = $defaultSdk
}
if (-not $env:ANDROID_SDK_ROOT -and $env:ANDROID_HOME) {
  $env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
}
if ($env:ANDROID_HOME) {
  $escapedSdk = $env:ANDROID_HOME.Replace("\", "\\")
  "sdk.dir=$escapedSdk" | Set-Content -Encoding ascii -Path (Join-Path $resolvedProject "local.properties")
}
if (-not $env:JAVA_HOME -and (Test-Path -LiteralPath "C:\Program Files\Android\Android Studio\jbr")) {
  $env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
}

if (Test-Path -LiteralPath $gradlew) {
  $gradleCommand = $gradlew
  $gradleArgs = @("assemble$($Variant.Substring(0, 1).ToUpper())$($Variant.Substring(1))")
} elseif (Test-Path -LiteralPath $localGradle) {
  $gradleCommand = $localGradle
  $gradleArgs = @("assemble$($Variant.Substring(0, 1).ToUpper())$($Variant.Substring(1))")
} else {
  $gradle = Get-Command gradle -ErrorAction SilentlyContinue
  if (-not $gradle) {
    throw "Gradle was not found. Install Gradle or add an Android Gradle wrapper under src\host\android-host, then rerun this script."
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
$packageName = "storylock-android-host-$versionName-$versionCode-$Variant.apk"
$packagePath = Join-Path $PackageOutputDir $packageName

New-Item -ItemType Directory -Force -Path $PackageOutputDir | Out-Null
Copy-Item -LiteralPath $apk.FullName -Destination $packagePath -Force
$package = Get-Item -LiteralPath $packagePath
$packageHash = Get-FileHash -Algorithm SHA256 -LiteralPath $package.FullName

$envDir = Split-Path -Parent $EnvOutput
New-Item -ItemType Directory -Force -Path $envDir | Out-Null
@(
  "STORYLOCK_ANDROID_APK_PATH=$($package.FullName)"
  "STORYLOCK_ANDROID_APK_VERSION=$versionName"
  "STORYLOCK_ANDROID_APK_VERSION_CODE=$versionCode"
  "STORYLOCK_ANDROID_APK_SIZE_BYTES=$($package.Length)"
  "STORYLOCK_ANDROID_APK_CHECKSUM=sha256:$($packageHash.Hash.ToLowerInvariant())"
  "STORYLOCK_ANDROID_PACKAGE_KIND=$Variant"
  "STORYLOCK_ANDROID_RELEASE_CHANNEL=$releaseChannel"
) | Set-Content -Encoding utf8 -Path $EnvOutput

$packageOutputScript = Join-Path $repoRoot "scripts\vercel\write_package_output.mjs"
if (Test-Path -LiteralPath $packageOutputScript) {
  node $packageOutputScript android $EnvOutput (Join-Path $repoRoot ".temp\vercel\output.json")
}

[PSCustomObject]@{
  GradleApkPath = $apk.FullName
  ApkPath = $package.FullName
  SizeBytes = $package.Length
  Sha256 = $packageHash.Hash.ToLowerInvariant()
  EnvOutput = $EnvOutput
}
