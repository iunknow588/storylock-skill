import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));

function read(relativePath) {
  return readFileSync(join(root, relativePath), 'utf8');
}

const wslPackageScript = read('scripts/release/linux/package_linux_host_wsl.ps1');
assert.doesNotMatch(wslPackageScript, /nvm use 22\b/u, 'WSL packaging must not pin Node.js to v22');
assert.match(wslPackageScript, /best_node_version/u, 'WSL packaging must select the highest installed Node.js >=22');
assert.match(wslPackageScript, /STORYLOCK_WSL_NODE_BIN/u, 'WSL packaging must allow explicit Node binary override');

const preflightScript = read('scripts/vercel/preflight.ps1');
assert.match(preflightScript, /yian-windows-host-0\.1\.0-1-prototype-zip\.json/u);
assert.doesNotMatch(preflightScript, /yian-windows-host-0\.1\.0-1-prototype\.json/u);
assert.match(preflightScript, /yian-linux-host-0\.1\.0-1-prototype-deb\.json/u);
assert.match(preflightScript, /\[switch\]\$SkipHttp/u, 'Vercel preflight must support local-only checks before first deploy');
assert.match(preflightScript, /vercel:project-link/u, 'Vercel preflight must check local project binding');
assert.match(preflightScript, /Deployment-level 404 detected/u, 'Vercel preflight must summarize deployment-level 404 failures');

const publishScript = read('scripts/vercel/publish_site_release.ps1');
assert.match(publishScript, /Assert-VercelProjectLink/u, 'Vercel publish must guard against deploying to the wrong project');
assert.match(publishScript, /-SkipHttp/u, 'Vercel publish must not block first deploy on stale online HTTP preflight');
assert.match(publishScript, /post-deploy Vercel preflight/u, 'Vercel publish must run HTTP preflight after deployment');
assert.match(publishScript, /deploy", "--yes"/u, 'Vercel publish must use non-interactive deploy mode');
assert.match(publishScript, /Test-VercelCliReady/u, 'Vercel publish must fail fast on auth/network problems');
assert.match(publishScript, /Invoke-VercelDeployWithRetry/u, 'Vercel publish must retry transient deploy failures');
assert.match(publishScript, /openid-configuration/u, 'Vercel publish must explain OIDC/TLS failures');
assert.match(publishScript, /Test-VercelDomainAccess/u, 'Vercel publish must diagnose custom-domain ownership/access');
assert.match(publishScript, /Set-VercelDeploymentAlias/u, 'Vercel publish must support binding the production custom domain to the deployment');

const syncEnvScript = read('scripts/vercel/sync_env_file_to_vercel.ps1');
assert.match(syncEnvScript, /Assert-VercelProjectLink/u, 'Vercel env sync must guard against updating the wrong project');

const vercelProductionWorkflow = read('.github/workflows/vercel-production.yml');
assert.match(vercelProductionWorkflow, /workflow_dispatch/u, 'Vercel production workflow must be manually triggered');
assert.match(vercelProductionWorkflow, /VERCEL_TOKEN/u, 'Vercel production workflow must use token auth');
assert.match(vercelProductionWorkflow, /VERCEL_ORG_ID/u, 'Vercel production workflow must bind org id');
assert.match(vercelProductionWorkflow, /VERCEL_PROJECT_ID/u, 'Vercel production workflow must bind project id');
assert.match(vercelProductionWorkflow, /npm test/u, 'Vercel production workflow must run tests before deploy');
assert.match(vercelProductionWorkflow, /vercel@54\.5\.1 deploy --prod --yes/u, 'Vercel production workflow must pin CLI deploy command');

const publishWslScript = read('scripts/vercel/publish_site_release_wsl.ps1');
assert.match(publishWslScript, /npx --yes vercel@54\.5\.1/u, 'WSL Vercel publish must pin fallback CLI version');
assert.match(publishWslScript, /VERCEL_TOKEN/u, 'WSL Vercel publish must support token auth');
assert.match(publishWslScript, /nvm ls --no-colors/u, 'WSL Vercel publish must load nvm Node >=22');
assert.match(publishWslScript, /STORYLOCK_GATEWAY_PUBLIC_URL/u, 'WSL Vercel publish must post-check production URL');
assert.match(publishWslScript, /VERCEL_BIND_CUSTOM_DOMAIN/u, 'WSL Vercel publish must support explicit custom-domain alias binding');

const vercelEnvExample = read('scripts/vercel/.env.example');
assert.match(vercelEnvExample, /VERCEL_PROJECT_NAME=storylock-gateway/u, 'Vercel env example must point to the gateway project, not the generic repo name');

const linuxPackageTest = read('scripts/test/linux-package-contents.mjs');
assert.match(linuxPackageTest, /STORYLOCK_WSL_DISTRO/u, 'Linux package test must allow WSL distro override');
assert.match(linuxPackageTest, /tarPackagePath/u, 'Linux package test must support tar.gz artifacts');
assert.match(linuxPackageTest, /debPackagePath/u, 'Linux package test must support deb artifacts');
assert.match(linuxPackageTest, /EncodedCommand/u, 'Linux zip package test must preserve Unicode paths on Windows');

const androidReadinessTest = read('scripts/test/android-readiness.mjs');
assert.match(androidReadinessTest, /AndroidKeyStore/u, 'Android readiness must guard AndroidKeyStore usage');
assert.match(androidReadinessTest, /BiometricPrompt/u, 'Android readiness must guard BiometricPrompt usage');
assert.match(androidReadinessTest, /validate_android_question_set\.mjs/u, 'Android readiness must validate the bundled question set');
assert.match(androidReadinessTest, /STORYLOCK_ANDROID_APK_CHECKSUM/u, 'Android readiness must guard APK metadata output');

const buildScript = read('scripts/vercel/build_yian_web.mjs');
assert.match(buildScript, /metadataNameFor/u);
assert.match(buildScript, /-tar-gz\.json/u);
assert.match(buildScript, /-\$1\.json/u);

const windowsTrayManualCheck = read('scripts/windows/start_windows_host_tray_manual_check.ps1');
assert.match(windowsTrayManualCheck, /--features ui-tray -- --tray/u, 'Windows tray manual check must run the tray feature');
assert.match(windowsTrayManualCheck, /Get-NetTCPConnection -LocalPort \$Port -State Listen/u, 'Windows tray manual check must fail fast when the port is occupied');
assert.match(windowsTrayManualCheck, /STORYLOCK_WINDOWS_DATA_DIR/u, 'Windows tray manual check must isolate runtime data');
assert.match(windowsTrayManualCheck, /Copy Diagnostics/u, 'Windows tray manual check must prompt the diagnostics clipboard check');

const windowsTrayManualRecord = read('docs/test/Windows托盘人工验收记录_20260620.md');
for (const requiredToken of ['TRAY-01', 'TRAY-02', 'TRAY-03', 'TRAY-04', 'TRAY-05']) {
  assert.match(
    windowsTrayManualRecord,
    new RegExp(requiredToken, 'u'),
    `Windows tray manual record must include ${requiredToken}`,
  );
}

console.log(JSON.stringify({
  status: 'passed',
  checks: [
    'wsl-node-selection',
    'preflight-metadata-names',
    'vercel-project-link-guard',
    'vercel-deployment-404-summary',
    'vercel-preflight-deploy-order',
    'vercel-auth-network-diagnostics',
    'vercel-ci-token-deploy',
    'vercel-wsl-token-deploy',
    'linux-package-format-flexibility',
    'download-metadata-name-collision',
    'windows-tray-manual-check-entry',
  ],
}, null, 2));
