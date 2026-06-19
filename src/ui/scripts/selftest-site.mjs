import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { spawn } from 'node:child_process';

const port = Number(process.env.STORYLOCK_UI_SELFTEST_PORT || 4197);
const baseUrl = `http://127.0.0.1:${port}`;
assert.equal(existsSync(new URL('../../yian-web/public/index.html', import.meta.url)), true);
assert.equal(existsSync(new URL('../../yian-web/public/main.js', import.meta.url)), true);
assert.equal(existsSync(new URL('../../yian-web/public/styles.css', import.meta.url)), true);
const server = spawn(process.execPath, ['server.mjs'], {
  cwd: new URL('../', import.meta.url),
  env: {
    ...process.env,
    PORT: String(port),
    STORYLOCK_ANDROID_CONNECT_MODE: 'relay_url',
    STORYLOCK_ANDROID_DEEP_LINK: 'storylock-host://bind',
    STORYLOCK_ANDROID_APP_DOWNLOAD_URL: 'https://example.test/storylock-android-host.apk',
    STORYLOCK_ANDROID_APK_VERSION: '0.1.0',
    STORYLOCK_ANDROID_APK_VERSION_CODE: '1',
    STORYLOCK_ANDROID_PACKAGE_KIND: 'debug',
    STORYLOCK_ANDROID_RELEASE_CHANNEL: 'internal',
    STORYLOCK_ANDROID_APK_CHECKSUM: 'sha256:selftest',
    STORYLOCK_WINDOWS_APP_DOWNLOAD_URL: 'https://example.test/yian-windows-host.zip',
    STORYLOCK_WINDOWS_PACKAGE_VERSION: '0.1.0',
    STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE: '1',
    STORYLOCK_WINDOWS_PACKAGE_KIND: 'zip',
    STORYLOCK_WINDOWS_RELEASE_CHANNEL: 'prototype',
    STORYLOCK_WINDOWS_PACKAGE_CHECKSUM: 'sha256:windows-selftest',
  },
  stdio: ['ignore', 'pipe', 'pipe'],
});

let output = '';
server.stdout.on('data', (chunk) => {
  output += chunk.toString('utf8');
});
server.stderr.on('data', (chunk) => {
  output += chunk.toString('utf8');
});

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function fetchWithRetry(path, attempts = 30) {
  let lastError = null;
  for (let index = 0; index < attempts; index += 1) {
    try {
      return await fetch(new URL(path, baseUrl));
    } catch (error) {
      lastError = error;
      await sleep(100);
    }
  }
  throw lastError;
}

try {
  const home = await fetchWithRetry('/');
  assert.equal(home.status, 200);
  assert.match(home.headers.get('content-type') ?? '', /text\/html/);
  const html = await home.text();
  assert.match(html, /data-locale="zh"/);
  assert.match(html, /data-locale="en"/);
  assert.match(html, /\/app\/download\/windows/);
  assert.match(html, /\/app\/download\/android/);
  assert.match(html, /id="faq"/);
  assert.doesNotMatch(html, /audience|评审|合作方|开发者|Judges|Partners|Developers/i);
  assert.doesNotMatch(html, /Layer 3|Keystore|BiometricPrompt|requestSignature|requestPasswordFill|gateway_url|binding_token|deep link|relay_url|direct_url|envelope/i);

  const script = await fetchWithRetry('/main.js');
  assert.equal(script.status, 200);
  const scriptText = await script.text();
  assert.match(scriptText, /const I18N =/);
  assert.match(scriptText, /zh:/);
  assert.match(scriptText, /en:/);
  assert.match(scriptText, /function applyLocale/);
  assert.match(scriptText, /状态显示离线怎么办/);
  assert.match(scriptText, /What if the status is offline/);
  assert.match(scriptText, /私人智能助理/);
  assert.match(scriptText, /StoryLock 本地核心/);
  assert.match(scriptText, /private assistant/);
  assert.match(scriptText, /StoryLock Local Core/);
  assert.doesNotMatch(scriptText, /audience|评审|合作方|开发者|Judges|Partners|Developers/i);
  assert.doesNotMatch(scriptText, /Layer 3|Keystore|BiometricPrompt|requestSignature|requestPasswordFill|gateway_url|binding_token|deep link|relay_url|direct_url|envelope/i);

  const styles = await fetchWithRetry('/styles.css');
  assert.equal(styles.status, 200);
  assert.match(styles.headers.get('content-type') ?? '', /text\/css/);

  const gateway = await fetchWithRetry('/api/storylock-gateway');
  assert.equal(gateway.status, 200);
  const gatewayStatus = await gateway.json();
  assert.equal(gatewayStatus.status, 'ok');
  assert.equal(gatewayStatus.onlineStatus.activeConnectionMode, 'relay_url');
  assert.equal(gatewayStatus.appDistribution.artifact.versionName, '0.1.0');
  assert.equal(gatewayStatus.appDistribution.artifact.checksum, 'sha256:selftest');
  assert.equal(gatewayStatus.appDistribution.platforms.android.versionName, '0.1.0');
  assert.equal(gatewayStatus.appDistribution.platforms.windows.implementation, 'rust');
  assert.equal(gatewayStatus.appDistribution.platforms.windows.checksum, 'sha256:windows-selftest');

  const appBinding = await fetch(new URL('/app/bind', baseUrl), {
    headers: {
      accept: 'text/html',
    },
  });
  assert.equal(appBinding.status, 200);
  assert.match(appBinding.headers.get('content-type') ?? '', /text\/html/);

  const appRegistrations = await fetchWithRetry('/app/registrations', 10);
  assert.equal(appRegistrations.status, 200);
  assert.match(appRegistrations.headers.get('content-type') ?? '', /application\/json/);

  console.log('Yian site homepage/i18n/runtime selftest passed.');
} finally {
  server.kill();
  const exit = await new Promise((resolve) => {
    server.once('exit', (code) => resolve(code));
    setTimeout(() => resolve(null), 1_000);
  });
  if (exit && exit !== 0) {
    process.stderr.write(output);
  }
}
