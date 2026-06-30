import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));

const requiredFiles = [
  'docs/test/StoryLock平台验收矩阵_20260620.md',
  'docs/management/StoryLock真机真实桌面验收待办_20260622.md',
  'docs/ref/10-Android真机闭环检查.md',
  'release/app/README.md',
  'scripts/android/check_device_loop.ps1',
  'scripts/windows/check_windows_host_loop.ps1',
  'scripts/linux/check_linux_host_loop.mjs',
  'scripts/linux/check_linux_secret_service_wsl.ps1',
  'scripts/linux/check_linux_secret_service_wsl.sh',
  'scripts/release/linux/package_linux_host.mjs',
  'scripts/release/linux/package_linux_host_wsl.ps1',
  'scripts/release/android/build_apk.ps1',
  'scripts/test/android-readiness.mjs',
  'scripts/release/windows/release_windows_host.ps1',
  'scripts/test/release-metadata-consistency.mjs',
  'scripts/test/linux-package-contents.mjs',
  'scripts/test/linux-desktop-integration.mjs',
  'scripts/test/linux-wsl-packaging.mjs',
  'scripts/test/platform-validation-status.mjs',
  'scripts/test/script-regression-guards.mjs',
  'docs/test/Android真机验收记录_20260622.md',
  'docs/test/Linux桌面WSL验收记录_20260622.md',
  '.github/workflows/vercel-production.yml',
  'scripts/vercel/publish_site_release_wsl.ps1',
  'api/storylock-gateway.mjs',
  'vercel.json',
  'src/host/linux-host/server.mjs',
  'src/host/linux-host/assets/question-bank.json',
];

const matrixPath = join(root, 'docs/test/StoryLock平台验收矩阵_20260620.md');
const matrixText = readFileSync(matrixPath, 'utf8');

for (const requiredFile of requiredFiles) {
  assert.ok(existsSync(join(root, requiredFile)), `${requiredFile} must exist`);
}

for (const requiredToken of [
  'npm run test:release',
  'npm run test:android-readiness',
  'npm run test:platform-readiness',
  'npm run test:non-device-validation',
  'npm run test:delivery',
  'npm run test:linux-host',
  'npm run package:linux-host',
  'npm run package:linux-host:wsl',
  'npm run test:linux-package',
  'npm run test:linux-desktop',
  'npm run test:linux-wsl',
  'npm run diagnose:linux-secret-service:wsl',
  'npm run status:platform-validation',
  'scripts\\android\\check_device_loop.ps1',
  'scripts\\windows\\check_windows_host_loop.cmd',
  'Android真机验收记录_20260622.md',
  'Linux桌面WSL验收记录_20260622.md',
  'StoryLock真机真实桌面验收待办_20260622.md',
  'WIN-10',
  'AND-09',
  'LIN-12',
  'AUTO-15',
  'scripts\\release\\android\\build_apk.cmd -Variant release',
  'scripts\\release\\windows\\release_windows_host.cmd',
  'SHA-256 checksum',
  'releaseChannel',
]) {
  assert.ok(matrixText.includes(requiredToken), `delivery matrix must mention ${requiredToken}`);
}

const packageJson = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
assert.ok(packageJson.scripts['test:release'], 'package.json must expose test:release');
assert.ok(packageJson.scripts['test:android-readiness'], 'package.json must expose test:android-readiness');
assert.ok(packageJson.scripts['test:platform-readiness'], 'package.json must expose test:platform-readiness');
assert.match(packageJson.scripts['test:platform-readiness'], /test:android-readiness/u);
assert.match(packageJson.scripts['test:platform-readiness'], /test:windows-ui-readiness/u);
assert.match(packageJson.scripts['test:platform-readiness'], /test:linux-desktop/u);
assert.ok(packageJson.scripts['test:delivery'], 'package.json must expose test:delivery');
assert.ok(packageJson.scripts['test:scripts'], 'package.json must expose test:scripts');
assert.ok(packageJson.scripts['test:windows-ui-readiness'], 'package.json must expose test:windows-ui-readiness');
assert.match(packageJson.scripts['test:windows-ui-readiness'], /test:windows-host-features/u);
assert.match(packageJson.scripts.test, /test:delivery/u, 'npm test must include test:delivery');

const vercelConfig = JSON.parse(readFileSync(join(root, 'vercel.json'), 'utf8'));
assert.equal(vercelConfig.outputDirectory, 'release/web/public', 'Vercel must deploy the built web public directory');
const rewrites = vercelConfig.rewrites ?? [];
for (const requiredRoute of [
  '/api/storylock-gateway',
  '/app/download',
  '/app/download/android',
  '/app/download/windows',
  '/app/download/linux',
  '/app/bind',
  '/app/registrations',
  '/download/linux-host',
]) {
  assert.ok(rewrites.some((rewrite) => rewrite.source === requiredRoute), `vercel.json must route ${requiredRoute}`);
}
for (const rewrite of rewrites) {
  assert.notEqual(
    rewrite.destination,
    '/web-api/storylock-gateway.mjs',
    'Vercel rewrites must target /api/storylock-gateway, not the internal web-api directory',
  );
}

const vercelWorkflow = readFileSync(join(root, '.github', 'workflows', 'vercel-production.yml'), 'utf8');
for (const requiredToken of [
  'workflow_dispatch',
  'VERCEL_TOKEN',
  'VERCEL_ORG_ID',
  'VERCEL_PROJECT_ID',
  'npm test',
  'npm run build',
  'vercel@54.5.1 deploy --prod --yes',
  'https://yian.cdao.online/',
]) {
  assert.ok(vercelWorkflow.includes(requiredToken), `Vercel workflow must mention ${requiredToken}`);
}

console.log(JSON.stringify({
  status: 'passed',
  filesChecked: requiredFiles.length,
  matrixId: 'storylock-platform-acceptance-matrix-20260620',
}, null, 2));
