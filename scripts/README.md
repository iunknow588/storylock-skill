# Script Layout

- `git/`
  - `get.cmd`
  - `get.ps1`
  - `commit.cmd`
  - `commit.ps1`
  - `.env`
  - `.env.example`
- `vercel/`
  - `dev_local.cmd`
  - `dev_local.ps1`
  - `link_project.cmd`
  - `link_project.ps1`
  - `publish_site_release.cmd`
  - `publish_site_release.ps1`
  - `sync_env_file_to_vercel.cmd`
  - `sync_env_file_to_vercel.ps1`
  - `.env.example`
- `text/`
  - `normalize_text_files.py`
  - `check_line_endings.py`
- `verify/`
  - `path-consistency.mjs`
  - `encoding-check.ps1`
- `android/`
  - `check_device_loop.cmd`
  - `check_device_loop.ps1`
  - `validate_android_question_set.mjs`
- `windows/`
  - `check_windows_host_loop.cmd`
  - `check_windows_host_loop.ps1`
- `linux/`
  - Linux SecretStore / packaging notes placeholder
- `release/`
  - `android/`
    - `build_apk.cmd`
    - `build_apk.ps1`
  - `windows/`
    - Windows package build, release, publish, upload, and signing helpers

## Common Commands

```powershell
scripts\git\get.cmd
scripts\git\commit.cmd
scripts\vercel\dev_local.cmd
scripts\vercel\link_project.cmd
scripts\vercel\publish_site_release.cmd -Target static -Build
scripts\vercel\publish_site_release.cmd -Target vercel -Build -SiteHttpSmoke -Preflight
scripts\vercel\publish_site_release.cmd -Target vercel -Build -SiteHttpSmoke -Prod -Execute
scripts\vercel\sync_env_file_to_vercel.cmd -EnvFilePath .temp\vercel\windows-package.publish.env
scripts\vercel\publish_site_release.cmd -Target vercel -WindowsEnvFile .temp\vercel\windows-package.publish.env -SyncWindowsEnvToVercel
scripts\release\android\build_apk.cmd -Variant debug
node scripts\android\validate_android_question_set.mjs
scripts\release\windows\release_windows_host.cmd
scripts\release\windows\publish_windows_release.cmd -ManifestPath E:\path\to\release-manifest.json -CopyArtifacts
scripts\release\windows\upload_windows_release_to_object_storage.cmd -UploadManifestPath E:\path\to\upload-manifest.json
python scripts\text\normalize_text_files.py --root . --dry-run
python scripts\text\normalize_text_files.py --root . --dry-run --fail-on-change
python scripts\text\normalize_text_files.py --root . --fix
python scripts\text\check_line_endings.py --root .
node scripts\verify\path-consistency.mjs
scripts\verify\encoding-check.ps1 -OnlyChanged
scripts\verify\encoding-check.ps1
```

## Notes

- `scripts\git\*.ps1` and `scripts\git\*.cmd` resolve the repository root from the current repo or `REPO_ROOT_OVERRIDE`.
- Git remotes are read from `scripts/git/.env` first, then from the repository root `.env` for compatibility.
- If `PREFERRED_REMOTE_URL` is not set, the git scripts reuse the current `origin` URL.
- `scripts\text\` utilities skip binary files and common generated directories.
- `scripts\verify\encoding-check.ps1` now combines line-ending checks with UTF-8/LF normalization dry-run; add `-Fix` to rewrite text files to UTF-8 without BOM and LF.
- `scripts\vercel\` reads `scripts/vercel/.env` when present, otherwise falls back to `.env.example`.
- Generated package metadata should not be stored under `scripts\vercel\`. Android and Windows package scripts now write `.temp\vercel\android-package.env`, `.temp\vercel\windows-package.env`, and `.temp\vercel\output.json`; copy values into `scripts\vercel\.env` only when a manual deployment needs overrides.
- `scripts\vercel\preflight.ps1`, `publish_site_release.ps1`, and `sync_env_file_to_vercel.ps1` check `.vercel/project.json` against `VERCEL_PROJECT_NAME` before deployment or env sync. Re-run `scripts\vercel\link_project.cmd` if the local link does not match the Vercel project that owns `yian.cdao.online`.
- `scripts\vercel\publish_site_release.ps1` supports two skeleton targets: `vercel` for CLI deployment planning or execution and `static` for copying the built `release/web/public/` directory into a release folder. Add `-SiteHttpSmoke` before Vercel deployment to run the local rewrite-equivalent HTTP check for `/`, `/api/storylock-gateway`, `/app/download`, platform downloads, binding, and registrations. Add `-Preflight` to run local env/project checks before deploy and online HTTP checks after deploy, so stale production 404s do not block the first corrective deployment.
- `scripts\vercel\sync_env_file_to_vercel.ps1` converts a local env fragment into a Vercel env sync plan and can optionally execute `vercel env update` with `add` fallback for `preview` and `production`.
- `scripts\release\windows\upload_windows_release_to_object_storage.ps1` defaults to generating an upload plan and only executes uploads when `-Execute` is supplied.
- `release\app\` is the unified local source for downloadable app packages. Android packages go under `release\app\android\`, Windows packages under `release\app\windows\`, and Linux packages under `release\app\linux\`. The site build copies these packages into `release\web\public\downloads\` before deployment.
- App package build/release/upload scripts live under `scripts\release\`; `scripts\android\` and `scripts\windows\` are reserved for local loop checks.
- `node scripts\android\validate_android_question_set.mjs` verifies that the Android asset-backed question set has non-empty identity/version fields, at least 24 active questions, and no duplicate active `questionId`.
- `scripts\linux\` currently records Linux-specific material boundaries; the shared `node src/skills/local-story-access/scripts/check-secret-store.mjs` command is the current Linux SecretStore verification entry.
