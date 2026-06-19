# Yian Windows Host

This directory is the Windows desktop local host skeleton for Yian / StoryLock.

Recommended implementation language: Rust.

Why Rust for Windows:

1. produces a direct-run `.exe`, `.msi`, or `.zip` package for regular users
2. avoids requiring Python or a separate runtime on the user's machine
3. fits a long-running local host with localhost HTTP, relay polling, tray integration, and Windows credential APIs
4. can integrate with Windows Credential Manager or DPAPI for local secret protection

Current scope in this directory:

1. Rust project skeleton
2. default Windows host configuration
3. shared registration / relay contract notes
4. local `GET /health` and `POST /execute` HTTP endpoints
5. gateway registration and relay polling prototype
6. StoryLock Local Core call chain prototype with Windows confirmation dialog and DPAPI-protected local object storage

Target runtime flow:

1. Yian website exposes `/app/download/windows`
2. user downloads and runs the Windows package
3. Windows host registers with the remote gateway using relay mode
4. Windows host polls relay requests and routes them through verify / authorize / execute / revoke
5. StoryLock Local Core remains local, uses DPAPI-protected objects, and returns minimal results through the private assistant boundary

Default ports and endpoints:

1. local health: `http://127.0.0.1:4510/health`
2. local verify: `http://127.0.0.1:4510/verify`
3. local authorize: `http://127.0.0.1:4510/authorize`
4. local revoke: `http://127.0.0.1:4510/revoke`
5. local execute: `http://127.0.0.1:4510/execute`
6. gateway register: `/local-host/register`
7. relay poll: `/local-host/relay/poll`
8. relay respond: `/local-host/relay/respond`

Build:

```powershell
cargo build --release
```

Print config only:

```powershell
cargo run -- --print-config
```

Run prototype:

```powershell
$env:STORYLOCK_GATEWAY_URL="https://yian.cdao.online"
$env:STORYLOCK_ANDROID_SHARED_SECRET="replace-with-strong-shared-secret"
cargo run
```

Prototype local execute request:

```powershell
Invoke-RestMethod -Method Post -Uri http://127.0.0.1:4510/execute -ContentType 'application/json' -Body (@{
  requestId = 'req-demo-001'
  capability = 'requestSignature'
  keyId = 'wallet-main'
} | ConvertTo-Json -Depth 4)
```

Prototype local verification flow:

```powershell
$verification = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:4510/verify -ContentType 'application/json' -Body (@{
  requestId = 'req-verify-001'
  capability = 'requestSignature'
  keyId = 'wallet-main'
} | ConvertTo-Json -Depth 4)

$authorization = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:4510/authorize -ContentType 'application/json' -Body (@{
  requestId = 'req-authorize-001'
  verificationId = $verification.result.verificationId
  answers = @(
    @{ cellId = 'cell-1'; answer = 'DAWN' }
    @{ cellId = 'cell-2'; answer = 'RIVER' }
    @{ cellId = 'cell-3'; answer = 'LANTERN' }
  )
} | ConvertTo-Json -Depth 5)

Invoke-RestMethod -Method Post -Uri http://127.0.0.1:4510/execute -ContentType 'application/json' -Body (@{
  requestId = 'req-execute-001'
  capability = 'requestSignature'
  keyId = 'wallet-main'
  authorizationId = $authorization.result.authorizationId
} | ConvertTo-Json -Depth 4)
```

Default local state path:

1. `%LOCALAPPDATA%\\Yian\\windows-host\\keys`
2. `%LOCALAPPDATA%\\Yian\\windows-host\\credentials`
3. `%LOCALAPPDATA%\\Yian\\windows-host\\authorizations`
4. `%LOCALAPPDATA%\\Yian\\windows-host\\question-bank.json`

Override local state path:

```powershell
$env:STORYLOCK_WINDOWS_DATA_DIR="E:\\2026OPC大赛\\skill\\.temp\\runtime\\windows-host-data"
```

Question bank bootstrap:

1. the runtime seeds `question-bank.json` from `src/host/windows-host/assets/question-bank.json` on first start
2. later runs read the local file from the data directory
3. `/verify` challenge cells are selected from that local question bank and include question-set and normalization metadata

Question bank CLI:

```powershell
cargo run -- --print-question-bank-path
cargo run -- --validate-question-bank
cargo run -- --import-question-bank E:\path\to\question-bank.json
```

Question bank HTTP endpoints:

1. `GET /question-bank/status`
2. `POST /question-bank/import`

Example import request:

```powershell
Invoke-RestMethod -Method Post -Uri http://127.0.0.1:4510/question-bank/import -ContentType 'application/json' -Body (@{
  requestId = 'req-import-bank-001'
  sourcePath = 'E:\path\to\question-bank.json'
} | ConvertTo-Json -Depth 4)
```

Windows loop check:

```powershell
..\..\scripts\windows\check_windows_host_loop.cmd
```

Package:

```powershell
..\..\scripts\release\windows\build_windows_host.cmd
..\..\scripts\release\windows\build_windows_host.ps1 -BuildMsi
..\..\scripts\release\windows\build_windows_host.ps1 -BuildMsi -Version 0.1.0 -VersionCode 1 -ReleaseChannel prototype
..\..\scripts\release\windows\build_windows_host.ps1 -BuildMsi -SignArtifacts
..\..\scripts\release\windows\release_windows_host.cmd
..\..\scripts\release\windows\publish_windows_release.cmd -ManifestPath E:\path\to\release-manifest.json -PublicDownloadUrl https://example.test/yian-windows-host.zip
..\..\scripts\release\windows\publish_windows_release.cmd -ManifestPath E:\path\to\release-manifest.json -CopyArtifacts
..\..\scripts\release\windows\upload_windows_release_to_object_storage.cmd -UploadManifestPath E:\path\to\upload-manifest.json
..\..\scripts\vercel\sync_env_file_to_vercel.cmd -EnvFilePath ..\..\scripts\vercel\.env.windows-package.publish
..\..\scripts\vercel\publish_site_release.cmd -Target vercel -Build -Preflight
..\..\scripts\vercel\publish_site_release.cmd -Target vercel -WindowsEnvFile ..\..\scripts\vercel\.env.windows-package.publish -SyncWindowsEnvToVercel
..\..\scripts\vercel\publish_site_release.cmd -Target static -Build
```

Future package names:

1. `yian-windows-host-0.1.0-1-prototype.zip`
2. `yian-windows-host-0.1.0-1.exe`
3. `yian-windows-host-0.1.0-1.msi`

Installer scaffold:

1. WiX source: `src/host/windows-host/installer/product.wxs`
2. fixed `UpgradeCode`: `6F0A7D8B-7F59-4E6B-B4E8-0EAC6959B301`
3. MSI build is optional in the current script and requires a local `wix` CLI
4. the build script injects `ProductVersion`, emits MSI size/checksum metadata, and keeps major upgrade continuity through `MajorUpgrade`
5. signing entry: `scripts/release/windows/sign_windows_package.ps1`

Release and upgrade policy scaffold:

1. `prototype` channel keeps rapid iteration artifacts such as `yian-windows-host-0.1.0-1-prototype.zip`
2. future `release` channel should reuse the same `UpgradeCode` for in-place MSI major upgrades
3. `Version` and `VersionCode` are now explicit build inputs in `build_windows_host.ps1`
4. signing expects `STORYLOCK_WINDOWS_SIGN_CERT_THUMBPRINT` and `STORYLOCK_WINDOWS_SIGN_TIMESTAMP_URL`
5. `release_windows_host.ps1` produces a `release-manifest-<version>-<versionCode>-<channel>.json` summary for downstream publishing
6. `manifest_to_windows_env.ps1` converts that manifest into the Windows distribution env fragment used by the download entry
7. `publish_windows_release.ps1` prepares the final publish summary, upload manifest, env file, and optional copied artifact directory for deployment or manual upload
8. `upload_windows_release_to_object_storage.ps1` turns the upload manifest into an object-storage upload plan and can optionally execute an S3-compatible upload skeleton
9. `sync_env_file_to_vercel.ps1` bridges the generated Windows env fragment into a Vercel env sync plan and can optionally execute `vercel env` updates
10. `publish_site_release.ps1` can now optionally call that env-sync step before the Vercel deploy plan or execution

Important limitation:

The current Rust host is still a prototype, but the local execution path is now shaped as a StoryLock Local Core call chain. It can register, expose health, create local grid-like verification challenges with required cells, exchange them for authorization sessions, reject revoked or expired sessions, execute signature and password-fill requests through a `storylock-local-core-call-v1` envelope, poll relay requests, show a Windows Yes/No confirmation dialog for fallback approval, persist signature keys / credential objects / verification records / authorization records under DPAPI protection, and return structured success/error envelopes with `verificationId`, `authorizationId`, `coreCallId`, required strength, allowed action, and expiry metadata. It also includes a WiX-based MSI scaffold, checksum metadata, and a reserved upgrade code for installer continuity. It does not yet implement richer approval UI, tray UI, production certificate provisioning, or automatic update delivery.

Distribution environment variables:

1. `STORYLOCK_WINDOWS_PACKAGE_PATH`
2. `STORYLOCK_WINDOWS_APP_DOWNLOAD_URL`
3. `STORYLOCK_WINDOWS_PACKAGE_VERSION`
4. `STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE`
5. `STORYLOCK_WINDOWS_PACKAGE_SIZE_BYTES`
6. `STORYLOCK_WINDOWS_PACKAGE_CHECKSUM`
7. `STORYLOCK_WINDOWS_PACKAGE_KIND`
8. `STORYLOCK_WINDOWS_RELEASE_CHANNEL`
9. `STORYLOCK_WINDOWS_APPROVAL_MODE`
10. `STORYLOCK_WINDOWS_DATA_DIR`
11. `STORYLOCK_WINDOWS_MSI_PATH`
12. `STORYLOCK_WINDOWS_MSI_SIZE_BYTES`
13. `STORYLOCK_WINDOWS_MSI_CHECKSUM`
14. `STORYLOCK_WINDOWS_MSI_UPGRADE_CODE`
15. `STORYLOCK_WINDOWS_SIGN_CERT_THUMBPRINT`
16. `STORYLOCK_WINDOWS_SIGN_TIMESTAMP_URL`
