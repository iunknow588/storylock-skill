import { createReadStream, existsSync, readdirSync, statSync } from 'node:fs';
import { readFile, stat } from 'node:fs/promises';
import { basename, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { StoryLockRemoteGateway } from './index.js';
import {
  resolveAndroidConnectionConfig,
  resolveAppDistributionConfig,
} from './connection-config.js';
import { StoryLockHostRegistry } from './host-registry.js';

let cachedRegistry = null;
let cachedRegistryFile = null;

function json(res, statusCode, value) {
  res.statusCode = statusCode;
  res.setHeader('content-type', 'application/json; charset=utf-8');
  res.end(JSON.stringify(value, null, 2));
}

function html(res, statusCode, value) {
  res.statusCode = statusCode;
  res.setHeader('content-type', 'text/html; charset=utf-8');
  res.end(value);
}

function redirect(res, statusCode, location) {
  res.statusCode = statusCode;
  res.setHeader('location', location);
  res.end();
}

function setCors(res) {
  res.setHeader('access-control-allow-origin', '*');
  res.setHeader('access-control-allow-methods', 'GET,POST,OPTIONS');
  res.setHeader('access-control-allow-headers', 'content-type,x-storylock-shared-secret');
}

async function readBody(req) {
  if (req.body && typeof req.body === 'object') {
    return req.body;
  }
  let body = '';
  for await (const chunk of req) {
    body += chunk;
    if (body.length > 1_000_000) {
      throw new Error('request body too large');
    }
  }
  return body.trim() ? JSON.parse(body) : {};
}

function optionalString(value) {
  if (typeof value !== 'string') {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function wantsHtml(req) {
  const accept = optionalString(req.headers.accept) ?? '';
  return accept.includes('text/html');
}

function escapeHtml(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function renderSimplePage({ title, lead, actions = [], notes = [] }) {
  const actionHtml = actions.map((action) => (
    `<a class="button" href="${escapeHtml(action.href)}">${escapeHtml(action.label)}</a>`
  )).join('');
  const notesHtml = notes.length
    ? `<ul>${notes.map((note) => `<li>${escapeHtml(note)}</li>`).join('')}</ul>`
    : '';
  return `<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>${escapeHtml(title)} | 易安</title>
    <style>
      body{margin:0;background:#f4f7f8;color:#12202b;font-family:"Segoe UI","Microsoft YaHei",sans-serif}
      main{width:min(760px,calc(100vw - 32px));margin:0 auto;padding:72px 0}
      h1{margin:0 0 16px;font-size:32px;line-height:1.25}
      p,li{color:#5c6d79;line-height:1.75}
      .panel{border:1px solid #d5e0e5;border-radius:8px;background:#fff;padding:28px;box-shadow:0 18px 40px rgba(18,32,43,.08)}
      .actions{display:flex;flex-wrap:wrap;gap:12px;margin-top:24px}
      .button{display:inline-flex;min-height:42px;align-items:center;justify-content:center;padding:0 16px;border-radius:8px;background:#0d6d77;color:#fff;text-decoration:none}
      .button.secondary{background:#fff;color:#12202b;border:1px solid #d5e0e5}
    </style>
  </head>
  <body>
    <main>
      <div class="panel">
        <h1>${escapeHtml(title)}</h1>
        <p>${escapeHtml(lead)}</p>
        ${notesHtml}
        <div class="actions">${actionHtml}</div>
      </div>
    </main>
  </body>
</html>`;
}

function getRegistry(env = process.env) {
  const filePath = optionalString(env.STORYLOCK_ANDROID_REGISTRY_FILE);
  if (!cachedRegistry || cachedRegistryFile !== filePath) {
    cachedRegistry = new StoryLockHostRegistry({ filePath });
    cachedRegistryFile = filePath;
  }
  return cachedRegistry;
}

function ensureEnv(name, env = process.env) {
  const value = env[name];
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${name} is required`);
  }
  return value.trim();
}

function ensureSharedSecret(req, env = process.env) {
  const expected = env.STORYLOCK_ANDROID_SHARED_SECRET;
  if (!expected) {
    return;
  }
  const received = req.headers['x-storylock-shared-secret'];
  if (received !== expected) {
    const error = new Error('unauthorized android host request');
    error.statusCode = 401;
    throw error;
  }
}

function getGatewayBaseUrl(req, env = process.env) {
  const configured = optionalString(env.STORYLOCK_GATEWAY_PUBLIC_URL);
  if (configured) {
    return configured.replace(/\/$/, '');
  }
  const protocol = optionalString(req.headers['x-forwarded-proto']) ?? 'http';
  const host = optionalString(req.headers['x-forwarded-host'])
    ?? optionalString(req.headers.host)
    ?? '127.0.0.1';
  return `${protocol}://${host}`;
}

function publicDownloadUrl(gatewayBaseUrl) {
  return new URL('/app/download', gatewayBaseUrl).toString();
}

function publicAndroidDownloadUrl(gatewayBaseUrl) {
  return new URL('/app/download/android', gatewayBaseUrl).toString();
}

function publicWindowsDownloadUrl(gatewayBaseUrl) {
  return new URL('/app/download/windows', gatewayBaseUrl).toString();
}

function publicLinuxDownloadUrl(gatewayBaseUrl) {
  return new URL('/app/download/linux', gatewayBaseUrl).toString();
}

function publicBindingUrl(gatewayBaseUrl) {
  return new URL('/app/bind', gatewayBaseUrl).toString();
}

function publicRegistrationsUrl(gatewayBaseUrl) {
  return new URL('/app/registrations', gatewayBaseUrl).toString();
}

function bundledApkDownloadUrl(gatewayBaseUrl) {
  return new URL(`/downloads/${DEFAULT_ANDROID_RELEASE_APK_FILE_NAME}`, gatewayBaseUrl).toString();
}

function bundledWindowsDownloadUrl(gatewayBaseUrl) {
  return new URL('/downloads/yian-windows-host-0.1.0-1-prototype.zip', gatewayBaseUrl).toString();
}

function bundledLinuxDownloadUrl(gatewayBaseUrl) {
  return new URL(`/downloads/${DEFAULT_LINUX_PACKAGE_FILE_NAME}`, gatewayBaseUrl).toString();
}

const DEFAULT_ANDROID_RELEASE_APK_FILE_NAME = 'storylock-android-host-0.1.0-1-release.apk';
const DEFAULT_ANDROID_RELEASE_APK_METADATA_FILE_NAME = 'storylock-android-host-0.1.0-1-release-apk.json';
const DEFAULT_ANDROID_DEBUG_APK_FILE_NAME = 'storylock-android-host-0.1.0-1-debug.apk';
const DEFAULT_WINDOWS_PACKAGE_FILE_NAME = 'yian-windows-host-0.1.0-1-prototype.zip';
const DEFAULT_WINDOWS_PACKAGE_METADATA_FILE_NAME = 'yian-windows-host-0.1.0-1-prototype-zip.json';
const DEFAULT_LINUX_PACKAGE_FILE_NAME = 'yian-linux-host-0.1.0-1-prototype.deb';
const DEFAULT_LINUX_PACKAGE_METADATA_FILE_NAME = 'yian-linux-host-0.1.0-1-prototype-deb.json';
const FALLBACK_LINUX_PACKAGE_FILE_NAMES = [
  'yian-linux-host-0.1.0-1-prototype.tar.gz',
  'yian-linux-host-0.1.0-1-prototype.zip',
];

function firstExistingPath(paths) {
  return paths.find((candidate) => existsSync(candidate)) ?? null;
}

function findNewestFile(directory, predicate) {
  if (!existsSync(directory)) {
    return null;
  }
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && predicate(entry.name))
    .map((entry) => {
      const absolutePath = resolve(directory, entry.name);
      return {
        absolutePath,
        modifiedTimeMs: statSync(absolutePath).mtimeMs,
      };
    })
    .sort((left, right) => right.modifiedTimeMs - left.modifiedTimeMs)[0]?.absolutePath ?? null;
}

function resolveApkFilePath(env = process.env, appDistribution = resolveAppDistributionConfig(env)) {
  const configured = optionalString(appDistribution.androidApkPath);
  if (configured) {
    const absolute = resolve(configured);
    return existsSync(absolute) ? absolute : null;
  }
  return firstExistingPath([
    resolve(process.cwd(), 'release', 'app', 'android', DEFAULT_ANDROID_RELEASE_APK_FILE_NAME),
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_ANDROID_RELEASE_APK_FILE_NAME),
  ]) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'app', 'android'),
    (fileName) => /-release\.apk$/iu.test(fileName),
  ) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads'),
    (fileName) => /-release\.apk$/iu.test(fileName),
  ) ?? firstExistingPath([
    resolve(process.cwd(), 'release', 'app', 'android', DEFAULT_ANDROID_DEBUG_APK_FILE_NAME),
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_ANDROID_DEBUG_APK_FILE_NAME),
    resolve(process.cwd(), 'android-host', 'app', 'build', 'outputs', 'apk', 'debug', 'app-debug.apk'),
  ]) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'app', 'android'),
    (fileName) => fileName.toLowerCase().endsWith('.apk'),
  ) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads'),
    (fileName) => fileName.toLowerCase().endsWith('.apk'),
  );
}

function resolveWindowsPackageFilePath(env = process.env, appDistribution = resolveAppDistributionConfig(env)) {
  const configured = optionalString(appDistribution.windowsPackagePath);
  if (configured) {
    const absolute = resolve(configured);
    return existsSync(absolute) ? absolute : null;
  }
  return firstExistingPath([
    resolve(process.cwd(), 'release', 'app', 'windows', DEFAULT_WINDOWS_PACKAGE_FILE_NAME),
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_WINDOWS_PACKAGE_FILE_NAME),
  ]) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'app', 'windows'),
    (fileName) => /\.(zip|msi|exe)$/iu.test(fileName),
  ) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads'),
    (fileName) => /^yian-windows-host-.+\.(zip|msi|exe)$/iu.test(fileName),
  );
}

function resolveLinuxPackageFilePath(env = process.env) {
  const configured = optionalString(env.STORYLOCK_LINUX_PACKAGE_PATH);
  if (configured) {
    const absolute = resolve(configured);
    return existsSync(absolute) ? absolute : null;
  }
  return firstExistingPath([
    resolve(process.cwd(), 'release', 'app', 'linux', DEFAULT_LINUX_PACKAGE_FILE_NAME),
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_LINUX_PACKAGE_FILE_NAME),
    ...FALLBACK_LINUX_PACKAGE_FILE_NAMES.flatMap((fileName) => [
      resolve(process.cwd(), 'release', 'app', 'linux', fileName),
      resolve(process.cwd(), 'release', 'web', 'public', 'downloads', fileName),
    ]),
  ]) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'app', 'linux'),
    (fileName) => /^yian-linux-host-.+\.deb$/iu.test(fileName),
  ) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads'),
    (fileName) => /^yian-linux-host-.+\.deb$/iu.test(fileName),
  ) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'app', 'linux'),
    (fileName) => /^yian-linux-host-.+(\.tar\.gz|\.(zip|deb|rpm|appimage))$/iu.test(fileName),
  ) ?? findNewestFile(
    resolve(process.cwd(), 'release', 'web', 'public', 'downloads'),
    (fileName) => /^yian-linux-host-.+(\.tar\.gz|\.(zip|deb|rpm|appimage))$/iu.test(fileName),
  );
}

async function readBundledAndroidApkMetadata() {
  const metadataPath = resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_ANDROID_RELEASE_APK_METADATA_FILE_NAME);
  if (!existsSync(metadataPath)) {
    return null;
  }
  try {
    return JSON.parse(await readFile(metadataPath, 'utf8'));
  } catch {
    return null;
  }
}

async function readBundledWindowsPackageMetadata() {
  const metadataPath = resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_WINDOWS_PACKAGE_METADATA_FILE_NAME);
  if (!existsSync(metadataPath)) {
    return null;
  }
  try {
    return JSON.parse(await readFile(metadataPath, 'utf8'));
  } catch {
    return null;
  }
}

async function readBundledLinuxPackageMetadata() {
  const metadataPath = resolve(process.cwd(), 'release', 'web', 'public', 'downloads', DEFAULT_LINUX_PACKAGE_METADATA_FILE_NAME);
  if (!existsSync(metadataPath)) {
    return null;
  }
  try {
    return JSON.parse(await readFile(metadataPath, 'utf8'));
  } catch {
    return null;
  }
}

async function checksumForFile(absolutePath) {
  if (!absolutePath || !existsSync(absolutePath)) {
    return null;
  }
  const bytes = await readFile(absolutePath);
  return `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
}

async function serveApkFile(res, absolutePath) {
  const info = await stat(absolutePath);
  res.statusCode = 200;
  res.setHeader('content-type', 'application/vnd.android.package-archive');
  res.setHeader('content-length', String(info.size));
  res.setHeader('content-disposition', `attachment; filename="${basename(absolutePath)}"`);
  await new Promise((resolvePromise, rejectPromise) => {
    const stream = createReadStream(absolutePath);
    stream.on('error', rejectPromise);
    stream.on('end', resolvePromise);
    stream.pipe(res);
  });
}

async function serveBinaryFile(res, absolutePath, {
  contentType = 'application/octet-stream',
} = {}) {
  const info = await stat(absolutePath);
  res.statusCode = 200;
  res.setHeader('content-type', contentType);
  res.setHeader('content-length', String(info.size));
  res.setHeader('content-disposition', `attachment; filename="${basename(absolutePath)}"`);
  await new Promise((resolvePromise, rejectPromise) => {
    const stream = createReadStream(absolutePath);
    stream.on('error', rejectPromise);
    stream.on('end', resolvePromise);
    stream.pipe(res);
  });
}

function inferAndroidPackageKind(fileName, configuredKind = null) {
  if (configuredKind) {
    return configuredKind;
  }
  const normalized = String(fileName ?? '').toLowerCase();
  if (normalized.includes('release')) {
    return 'release';
  }
  if (normalized.includes('debug')) {
    return 'debug';
  }
  return 'apk';
}

function inferHostPackageKind(fileName, configuredKind = null) {
  if (configuredKind) {
    return configuredKind;
  }
  const normalized = String(fileName ?? '').toLowerCase();
  if (normalized.endsWith('.tar.gz')) {
    return 'tar.gz';
  }
  if (normalized.endsWith('.zip')) {
    return 'zip';
  }
  if (normalized.endsWith('.msi')) {
    return 'msi';
  }
  if (normalized.endsWith('.exe')) {
    return 'exe';
  }
  if (normalized.endsWith('.deb')) {
    return 'deb';
  }
  if (normalized.endsWith('.rpm')) {
    return 'rpm';
  }
  if (normalized.endsWith('.appimage')) {
    return 'appimage';
  }
  return 'package';
}

function inferReleaseChannel(packageKind, configuredChannel = null) {
  if (configuredChannel) {
    return configuredChannel;
  }
  if (packageKind === 'debug') {
    return 'internal';
  }
  if (packageKind === 'release') {
    return 'candidate';
  }
  return 'unspecified';
}

function optionalPositiveInteger(value) {
  const parsed = Number.parseInt(String(value ?? ''), 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

async function buildAppDistributionStatus(env, gatewayBaseUrl) {
  const appDistribution = resolveAppDistributionConfig(env);
  const apkFilePath = resolveApkFilePath(env, appDistribution);
  const windowsPackageFilePath = resolveWindowsPackageFilePath(env, appDistribution);
  const linuxPackageFilePath = resolveLinuxPackageFilePath(env);
  const artifactName = apkFilePath ? basename(apkFilePath) : DEFAULT_ANDROID_RELEASE_APK_FILE_NAME;
  const windowsArtifactName = windowsPackageFilePath ? basename(windowsPackageFilePath) : DEFAULT_WINDOWS_PACKAGE_FILE_NAME;
  const linuxArtifactName = linuxPackageFilePath ? basename(linuxPackageFilePath) : DEFAULT_LINUX_PACKAGE_FILE_NAME;
  const packageKind = inferAndroidPackageKind(artifactName, optionalString(appDistribution.androidPackageKind));
  const windowsPackageKind = inferHostPackageKind(windowsArtifactName, optionalString(appDistribution.windowsPackageKind));
  const linuxPackageKind = inferHostPackageKind(linuxArtifactName, optionalString(env.STORYLOCK_LINUX_PACKAGE_KIND));
  const artifactInfo = apkFilePath
    ? await stat(apkFilePath)
    : null;
  const windowsArtifactInfo = windowsPackageFilePath
    ? await stat(windowsPackageFilePath)
    : null;
  const linuxArtifactInfo = linuxPackageFilePath
    ? await stat(linuxPackageFilePath)
    : null;
  const bundledAndroidMetadata = await readBundledAndroidApkMetadata();
  const bundledWindowsMetadata = await readBundledWindowsPackageMetadata();
  const bundledLinuxMetadata = await readBundledLinuxPackageMetadata();
  const apkChecksum = apkFilePath ? await checksumForFile(apkFilePath) : null;
  const windowsPackageChecksum = windowsPackageFilePath ? await checksumForFile(windowsPackageFilePath) : null;
  const linuxPackageChecksum = linuxPackageFilePath ? await checksumForFile(linuxPackageFilePath) : null;

  return {
    ...appDistribution,
    androidAppDownloadUrl: appDistribution.androidAppDownloadUrl
      ?? publicAndroidDownloadUrl(gatewayBaseUrl),
    windowsAppDownloadUrl: appDistribution.windowsAppDownloadUrl
      ?? publicWindowsDownloadUrl(gatewayBaseUrl),
    artifact: {
      available: Boolean(apkFilePath || appDistribution.androidAppDownloadUrl || artifactName),
      localFilePath: apkFilePath,
      fileName: artifactName,
      fileSizeBytes: artifactInfo?.size
        ?? optionalPositiveInteger(appDistribution.androidApkSizeBytes)
        ?? optionalPositiveInteger(bundledAndroidMetadata?.fileSizeBytes),
      versionName: optionalString(appDistribution.androidApkVersion)
        ?? optionalString(bundledAndroidMetadata?.versionName),
      versionCode: optionalString(appDistribution.androidApkVersionCode)
        ?? optionalString(bundledAndroidMetadata?.versionCode),
      checksum: optionalString(appDistribution.androidApkChecksum)
        ?? apkChecksum
        ?? optionalString(bundledAndroidMetadata?.checksum),
      packageKind,
      releaseChannel: inferReleaseChannel(
        packageKind,
        optionalString(appDistribution.androidReleaseChannel) ?? optionalString(bundledAndroidMetadata?.releaseChannel),
      ),
      downloadStrategy: apkFilePath
        ? 'local_apk_file'
        : appDistribution.androidAppDownloadUrl
          ? 'external_download_url'
          : 'bundled_static_apk',
      installGuideUrl: appDistribution.androidInstallGuideUrl,
    },
    platforms: {
      android: {
        platform: 'android',
        label: 'Android',
        downloadUrl: appDistribution.androidAppDownloadUrl ?? publicAndroidDownloadUrl(gatewayBaseUrl),
        available: Boolean(apkFilePath || appDistribution.androidAppDownloadUrl || artifactName),
        fileName: artifactName,
        fileSizeBytes: artifactInfo?.size
          ?? optionalPositiveInteger(appDistribution.androidApkSizeBytes)
          ?? optionalPositiveInteger(bundledAndroidMetadata?.fileSizeBytes),
        versionName: optionalString(appDistribution.androidApkVersion)
          ?? optionalString(bundledAndroidMetadata?.versionName),
        versionCode: optionalString(appDistribution.androidApkVersionCode)
          ?? optionalString(bundledAndroidMetadata?.versionCode),
        checksum: optionalString(appDistribution.androidApkChecksum)
          ?? apkChecksum
          ?? optionalString(bundledAndroidMetadata?.checksum),
        packageKind,
        releaseChannel: inferReleaseChannel(
          packageKind,
          optionalString(appDistribution.androidReleaseChannel) ?? optionalString(bundledAndroidMetadata?.releaseChannel),
        ),
      },
      windows: {
        platform: 'windows',
        label: 'Windows',
        downloadUrl: appDistribution.windowsAppDownloadUrl ?? publicWindowsDownloadUrl(gatewayBaseUrl),
        available: Boolean(windowsPackageFilePath || appDistribution.windowsAppDownloadUrl || windowsArtifactName),
        fileName: windowsArtifactName,
        fileSizeBytes: windowsArtifactInfo?.size
          ?? optionalPositiveInteger(appDistribution.windowsPackageSizeBytes)
          ?? optionalPositiveInteger(bundledWindowsMetadata?.fileSizeBytes),
        versionName: optionalString(appDistribution.windowsPackageVersion)
          ?? optionalString(bundledWindowsMetadata?.versionName),
        versionCode: optionalString(appDistribution.windowsPackageVersionCode)
          ?? optionalString(bundledWindowsMetadata?.versionCode),
        checksum: optionalString(appDistribution.windowsPackageChecksum)
          ?? windowsPackageChecksum
          ?? optionalString(bundledWindowsMetadata?.checksum),
        packageKind: windowsPackageKind ?? optionalString(bundledWindowsMetadata?.packageKind),
        releaseChannel: optionalString(appDistribution.windowsReleaseChannel)
          ?? optionalString(bundledWindowsMetadata?.releaseChannel)
          ?? 'prototype',
        implementation: 'rust',
        downloadStrategy: windowsPackageFilePath
          ? 'local_package_file'
          : appDistribution.windowsAppDownloadUrl
            ? 'external_download_url'
            : 'bundled_static_package',
        status: windowsPackageFilePath || appDistribution.windowsAppDownloadUrl || windowsArtifactName ? 'configured' : 'not_configured',
      },
      linux: {
        platform: 'linux',
        label: 'Linux',
        downloadUrl: publicLinuxDownloadUrl(gatewayBaseUrl),
        available: Boolean(linuxPackageFilePath || linuxArtifactName),
        fileName: linuxArtifactName,
        fileSizeBytes: linuxArtifactInfo?.size
          ?? optionalPositiveInteger(bundledLinuxMetadata?.fileSizeBytes),
        versionName: optionalString(env.STORYLOCK_LINUX_PACKAGE_VERSION)
          ?? optionalString(bundledLinuxMetadata?.versionName),
        versionCode: optionalString(env.STORYLOCK_LINUX_PACKAGE_VERSION_CODE)
          ?? optionalString(bundledLinuxMetadata?.versionCode),
        checksum: optionalString(env.STORYLOCK_LINUX_PACKAGE_CHECKSUM)
          ?? linuxPackageChecksum
          ?? optionalString(bundledLinuxMetadata?.checksum),
        packageKind: linuxPackageKind ?? optionalString(bundledLinuxMetadata?.packageKind),
        releaseChannel: optionalString(env.STORYLOCK_LINUX_RELEASE_CHANNEL)
          ?? optionalString(bundledLinuxMetadata?.releaseChannel)
          ?? 'prototype',
        implementation: 'node',
        downloadStrategy: linuxPackageFilePath ? 'local_package_file' : 'bundled_static_package',
        status: linuxPackageFilePath || linuxArtifactName ? 'configured' : 'not_configured',
      },
    },
  };
}

function renderPlatformDownloadPage({ gatewayBaseUrl, appDistribution }) {
  const android = appDistribution.platforms.android;
  const windows = appDistribution.platforms.windows;
  const linux = appDistribution.platforms.linux;
  return renderSimplePage({
    title: '选择易安本地版本',
    lead: '请选择当前设备对应的本地宿主。Windows 电脑下载 Windows 版本；Android 手机下载 Android APK；Linux 当前提供原型包。',
    actions: [
      { href: windows.downloadUrl, label: '下载 Windows 版本' },
      { href: android.downloadUrl, label: '下载 Android 版本' },
      { href: linux.downloadUrl, label: '下载 Linux 原型包' },
      { href: new URL('/app/bind', gatewayBaseUrl).toString(), label: '安装后打开绑定入口' },
    ],
    notes: [
      `Windows: ${windows.status === 'configured' ? windows.fileName : '暂未配置真实安装包'}`,
      `Android: ${android.fileName}`,
      `Linux: ${linux.status === 'configured' ? linux.fileName : '暂未配置 Linux 包'}`,
      '安装前请核对版本、文件大小和 SHA-256 校验值。',
    ],
  });
}

async function forwardToAndroid(request, {
  androidHostUrl,
  fetchImpl = globalThis.fetch,
  sharedSecret = process.env.STORYLOCK_ANDROID_SHARED_SECRET ?? '',
} = {}) {
  const response = await fetchImpl(androidHostUrl, {
    method: 'POST',
    headers: {
      'content-type': 'application/json; charset=utf-8',
      ...(sharedSecret ? { 'x-storylock-shared-secret': sharedSecret } : {}),
    },
    body: JSON.stringify(request),
  });
  const text = await response.text();
  const payload = text.trim() ? JSON.parse(text) : null;
  if (!response.ok) {
    const message = payload?.error?.message ?? payload?.message ?? `android host returned ${response.status}`;
    throw new Error(message);
  }
  return payload;
}

function summarizeHost(host) {
  if (!host) {
    return null;
  }
  return {
    registrationId: host.registrationId,
    hostId: host.hostId,
    identityId: host.identityId,
    deviceId: host.deviceId,
    appInstanceId: host.appInstanceId,
    preferredMode: host.preferredMode,
    online: host.online,
    lastSeenAt: host.lastSeenAt,
    ageMs: host.ageMs,
    reachability: host.reachability,
    install: host.install,
    device: host.device,
    directUrl: host.directUrl,
    relayUrl: host.relayUrl,
    deepLink: host.deepLink,
  };
}

async function dispatchViaRelay(remoteRequest, {
  identityId,
  preferredMode,
  env,
}) {
  const registry = getRegistry(env);
  const host = registry.findHostForInvocation({
    identityId,
    preferredMode,
  });
  if (!host) {
    throw new Error('no registered android host is available for relay execution');
  }
  const { response } = registry.createRelayRequest({
    hostId: host.hostId,
    request: remoteRequest,
  });
  return response;
}

async function transportForMode(remoteRequest, {
  env,
  fetchImpl,
  preferredMode,
}) {
  const registry = getRegistry(env);
  const identityId = remoteRequest?.payload?.identityId ?? null;
  if (preferredMode === 'direct_url') {
    const androidHostUrl = optionalString(env.STORYLOCK_ANDROID_HOST_URL)
      ?? registry.findHostForInvocation({ identityId, preferredMode: 'direct_url' })?.executeUrl;
    if (!androidHostUrl) {
      throw new Error('STORYLOCK_ANDROID_HOST_URL is required for direct_url mode');
    }
    return forwardToAndroid(remoteRequest, {
      androidHostUrl,
      fetchImpl,
      sharedSecret: env.STORYLOCK_ANDROID_SHARED_SECRET ?? '',
    });
  }
  return dispatchViaRelay(remoteRequest, {
    identityId,
    preferredMode,
    env,
  });
}

function buildRegistrationResponse(req, env, host) {
  const gatewayBaseUrl = getGatewayBaseUrl(req, env);
  return {
    status: 'ok',
    registration: summarizeHost(host),
    relay: {
      relayUrl: new URL('/local-host/relay', gatewayBaseUrl).toString(),
      pollUrl: new URL('/local-host/relay/poll', gatewayBaseUrl).toString(),
      respondUrl: new URL('/local-host/relay/respond', gatewayBaseUrl).toString(),
    },
    gateway: {
      baseUrl: gatewayBaseUrl,
      executeUrl: new URL('/api/storylock-gateway', gatewayBaseUrl).toString(),
      downloadUrl: publicDownloadUrl(gatewayBaseUrl),
    },
  };
}

async function handleGet(req, res, url, env) {
  const gatewayBaseUrl = getGatewayBaseUrl(req, env);
  const appDistribution = await buildAppDistributionStatus(env, gatewayBaseUrl);
  const registry = getRegistry(env);
  const connection = resolveAndroidConnectionConfig(env, {
    gatewayBaseUrl,
    bindingDeepLink: optionalString(env.STORYLOCK_ANDROID_DEEP_LINK),
  });
  const registrySummary = registry.getStatusSummary();

  if (url.pathname === '/app/download') {
    if (wantsHtml(req)) {
      html(res, 200, renderPlatformDownloadPage({
        gatewayBaseUrl,
        appDistribution,
      }));
      return;
    }
    json(res, 200, {
      status: 'ok',
      message: 'choose a local host package for your platform',
      downloadPageUrl: publicDownloadUrl(gatewayBaseUrl),
      platforms: appDistribution.platforms,
    });
    return;
  }

  if (url.pathname === '/app/download/windows' || url.pathname === '/download/windows-host') {
    const windowsPackageFilePath = resolveWindowsPackageFilePath(env, appDistribution);
    if (windowsPackageFilePath) {
      await serveBinaryFile(res, windowsPackageFilePath, {
        contentType: 'application/zip',
      });
      return;
    }
    const currentDownloadUrl = new URL(url.pathname, gatewayBaseUrl).toString();
    const configuredDownloadUrl = appDistribution.windowsAppDownloadUrl;
    const targetDownloadUrl = configuredDownloadUrl && configuredDownloadUrl !== currentDownloadUrl
      ? configuredDownloadUrl
      : bundledWindowsDownloadUrl(gatewayBaseUrl);
    if (targetDownloadUrl !== currentDownloadUrl) {
      redirect(res, 307, targetDownloadUrl);
      return;
    }
    if (wantsHtml(req)) {
      html(res, 404, renderSimplePage({
        title: 'Windows 版本暂未开放下载',
        lead: '当前站点还没有配置 Windows 本地宿主安装包。Windows 版本推荐使用 Rust 实现，目标是下载后可直接运行。',
        actions: [
          { href: publicDownloadUrl(gatewayBaseUrl), label: '返回版本选择' },
          { href: publicAndroidDownloadUrl(gatewayBaseUrl), label: '下载 Android 版本' },
        ],
        notes: [
          '后续 Windows 包建议提供 .exe、.msi 或 .zip。',
          '未配置真实 Windows 包前，请不要把 Android APK 当作电脑版本使用。',
        ],
      }));
      return;
    }
    json(res, 404, {
      status: 'error',
      message: 'windows app download source is not configured',
      platform: appDistribution.platforms.windows,
    });
    return;
  }

  if (url.pathname === '/app/download/linux' || url.pathname === '/download/linux-host') {
    const linuxPackageFilePath = resolveLinuxPackageFilePath(env);
    if (linuxPackageFilePath) {
      await serveBinaryFile(res, linuxPackageFilePath, {
        contentType: 'application/octet-stream',
      });
      return;
    }
    const currentDownloadUrl = new URL(url.pathname, gatewayBaseUrl).toString();
    const targetDownloadUrl = bundledLinuxDownloadUrl(gatewayBaseUrl);
    if (targetDownloadUrl !== currentDownloadUrl) {
      redirect(res, 307, targetDownloadUrl);
      return;
    }
    if (wantsHtml(req)) {
      html(res, 404, renderSimplePage({
        title: 'Linux 原型包暂未开放下载',
        lead: '当前站点还没有配置 Linux 本地宿主原型包。Linux 包仍处于原型和发行版验收阶段。',
        actions: [
          { href: publicDownloadUrl(gatewayBaseUrl), label: '返回版本选择' },
        ],
        notes: [
          'Linux 当前需要 Node.js >=22。',
          '生产模式需要 secret-tool 和 Secret Service 可用。',
        ],
      }));
      return;
    }
    json(res, 404, {
      status: 'error',
      message: 'linux app download source is not configured',
      platform: appDistribution.platforms.linux,
    });
    return;
  }

  if (url.pathname === '/app/download/android' || url.pathname === '/download/android-host') {
    const apkFilePath = resolveApkFilePath(env, appDistribution);
    if (apkFilePath) {
      await serveApkFile(res, apkFilePath);
      return;
    }
    const currentDownloadUrl = new URL(url.pathname, gatewayBaseUrl).toString();
    const configuredDownloadUrl = appDistribution.androidAppDownloadUrl;
    const targetDownloadUrl = configuredDownloadUrl && configuredDownloadUrl !== currentDownloadUrl
      ? configuredDownloadUrl
      : bundledApkDownloadUrl(gatewayBaseUrl);
    if (targetDownloadUrl !== currentDownloadUrl) {
      redirect(res, 307, targetDownloadUrl);
      return;
    }
    if (wantsHtml(req)) {
      html(res, 404, renderSimplePage({
        title: '易安 App 暂未开放下载',
        lead: '当前站点还没有配置可下载的安装包。请稍后再试，或等待页面展示正式版本号、文件大小和校验值后再安装。',
        actions: [
          { href: '/', label: '返回易安首页' },
        ],
        notes: [
          '不要从不可信来源下载安装包。',
          '安装前请核对版本号、文件大小和校验值。',
        ],
      }));
      return;
    }
    json(res, 404, {
      status: 'error',
      message: 'android app download source is not configured',
    });
    return;
  }

  if (url.pathname === '/app/bind' || url.pathname === '/android-host/bind' || url.pathname === '/local-host/bind') {
    const binding = registry.issueBindingToken({
      identityId: url.searchParams.get('identityId') ?? 'android-demo-001',
      preferredMode: url.searchParams.get('preferredMode') ?? optionalString(env.STORYLOCK_ANDROID_CONNECT_MODE) ?? 'relay_url',
      gatewayBaseUrl,
      deepLinkBase: optionalString(env.STORYLOCK_ANDROID_DEEP_LINK),
    });
    if (wantsHtml(req)) {
      html(res, 200, renderSimplePage({
        title: '打开本地 StoryLock App 完成绑定',
        lead: '请在自己的手机上打开本地 StoryLock App。它会像一个随身安全确认器一样保存本地确认能力；远程页面只负责连接，不替你完成确认。',
        actions: [
          { href: binding.deepLink ?? '/', label: '打开本地 StoryLock App' },
          { href: '/', label: '返回易安首页' },
        ],
        notes: [
          '手机 App 是本地 StoryLock 应用，不是远程服务节点。',
          '如果手机没有反应，请先确认已经安装易安 App。',
          '如果状态仍显示离线，请重新打开 App 或重新绑定。',
        ],
      }));
      return;
    }
    json(res, 200, {
      status: 'ok',
      binding,
      connection: resolveAndroidConnectionConfig(env, {
        gatewayBaseUrl,
        bindingDeepLink: binding.deepLink,
      }),
    });
    return;
  }

  if (url.pathname === '/app/registrations' || url.pathname === '/android-host/registrations' || url.pathname === '/local-host/registrations') {
    json(res, 200, {
      status: 'ok',
      hosts: registry.listHosts({
        identityId: url.searchParams.get('identityId') ?? null,
      }),
    });
    return;
  }

  if (url.pathname === '/api/site/registrations') {
    json(res, 200, {
      status: 'ok',
      hosts: registry.listHosts({
        identityId: url.searchParams.get('identityId') ?? null,
      }),
    });
    return;
  }

  json(res, 200, {
    status: 'ok',
    service: 'storylock-remote-gateway-web-api',
    onlineStatus: {
      websiteEntryUrl: `${gatewayBaseUrl}/`,
      gatewayEntryUrl: new URL('/api/storylock-gateway', gatewayBaseUrl).toString(),
      downloadUrl: publicDownloadUrl(gatewayBaseUrl),
      androidDownloadUrl: appDistribution.platforms.android.downloadUrl,
      windowsDownloadUrl: appDistribution.platforms.windows.downloadUrl,
      bindingEntryUrl: publicBindingUrl(gatewayBaseUrl),
      activeConnectionMode: connection.activeOption?.mode ?? connection.preferredMode ?? 'unconfigured',
      activeHostCount: registrySummary.activeHostCount,
      onlineHostCount: registrySummary.onlineHostCount,
      totalHostCount: registrySummary.totalHostCount,
    },
    website: {
      brand: 'Yian',
      layer: 'layer3_site',
      entryUrl: `${gatewayBaseUrl}/`,
      summary: 'public website, app distribution, first-bind entry, and remote gateway surface',
    },
    gateway: {
      entryUrl: new URL('/api/storylock-gateway', gatewayBaseUrl).toString(),
      executeUrl: new URL('/api/storylock-gateway', gatewayBaseUrl).toString(),
      registrationsUrl: publicRegistrationsUrl(gatewayBaseUrl),
    },
    androidHostUrl: optionalString(env.STORYLOCK_ANDROID_HOST_URL),
    capabilities: ['requestSignature', 'requestPasswordFill'],
    appDistribution,
    binding: {
      entryUrl: publicBindingUrl(gatewayBaseUrl),
      deepLinkBase: optionalString(env.STORYLOCK_ANDROID_DEEP_LINK) ?? 'storylock-host://bind',
      tokenPolicy: registrySummary.bindingTokenPolicy,
    },
    secondLayerConnection: connection,
    bindingEndpoint: publicBindingUrl(gatewayBaseUrl),
    hostRegistry: {
      ...registrySummary,
      hosts: registrySummary.hosts.map(summarizeHost),
    },
    relayPolicy: {
      transport: 'poll_respond',
      timeoutMs: registrySummary.relay.timeoutMs,
      pollIntervalMs: registrySummary.relay.pollIntervalMs,
      retryBackoffMs: registrySummary.relay.retryBackoffMs,
      failureTracking: {
        pendingResponseCount: registrySummary.relay.pendingResponseCount,
        totalRequests: registrySummary.relay.totalRequests,
        resolvedResponses: registrySummary.relay.resolvedResponses,
        timeoutCount: registrySummary.relay.timeoutCount,
        lastTimeoutAt: registrySummary.relay.lastTimeoutAt,
        lastResolvedAt: registrySummary.relay.lastResolvedAt,
      },
      recoveryStrategy: 'host continues polling after timeout; gateway reissues on next request',
    },
    routingStrategy: {
      identitySelection: 'prefer request.identityId when present',
      onlinePreference: 'recent online hosts first',
      preferredModePriority: true,
      scoringOrder: ['identity_match', 'preferred_mode', 'reachability', 'most_recent_last_seen'],
      defaultMode: connection.activeOption?.mode ?? connection.preferredMode ?? 'relay_url',
    },
    pharos: {
      role: 'optional_anchor_layer',
      executionRole: 'not_local_execution_layer',
    },
  });
}

async function handleAndroidHostRegister(req, res, env) {
  const registry = getRegistry(env);
  const body = await readBody(req);
  const bindingToken = optionalString(body.bindingToken);
  const binding = bindingToken ? registry.consumeBindingToken(bindingToken) : null;
  if (bindingToken && !binding) {
    json(res, 400, {
      status: 'error',
      message: 'binding token is invalid or expired',
    });
    return;
  }
  if (!binding) {
    ensureSharedSecret(req, env);
  }

  const host = registry.upsertHost({
    deviceId: body.deviceId,
    appInstanceId: body.appInstanceId,
    identityId: binding?.identityId ?? body.identityId,
    preferredMode: body.preferredMode ?? binding?.preferredMode ?? 'relay_url',
    directUrl: body.host?.directUrl ?? body.host?.executeUrl ?? null,
    healthUrl: body.host?.healthUrl ?? null,
    executeUrl: body.host?.executeUrl ?? null,
    deepLink: body.deepLink ?? binding?.deepLink ?? optionalString(env.STORYLOCK_ANDROID_DEEP_LINK),
    relayUrl: body.relayUrl ?? new URL('/android-host/relay', getGatewayBaseUrl(req, env)).toString(),
    install: body.install,
    device: body.device,
    reachability: body.reachability,
    bindingTokenId: binding?.tokenId ?? null,
  });
  json(res, 200, buildRegistrationResponse(req, env, host));
}

async function handleRelayPoll(req, res, env) {
  const registry = getRegistry(env);
  ensureSharedSecret(req, env);
  const body = await readBody(req);
  const deviceId = optionalString(body.deviceId);
  const appInstanceId = optionalString(body.appInstanceId);
  if (!deviceId || !appInstanceId) {
    json(res, 400, {
      status: 'error',
      message: 'deviceId and appInstanceId are required',
    });
    return;
  }
  const dispatched = registry.takeRelayRequest({ deviceId, appInstanceId });
  if (!dispatched) {
    json(res, 200, {
      status: 'idle',
      registration: registry.touchHost(deviceId, appInstanceId),
    });
    return;
  }
  json(res, 200, {
    status: 'ok',
    relayRequestId: dispatched.relayRequestId,
    request: dispatched.request,
    registration: registry.touchHost(deviceId, appInstanceId),
  });
}

async function handleRelayRespond(req, res, env) {
  const registry = getRegistry(env);
  ensureSharedSecret(req, env);
  const body = await readBody(req);
  const relayRequestId = optionalString(body.relayRequestId);
  if (!relayRequestId) {
    json(res, 400, {
      status: 'error',
      message: 'relayRequestId is required',
    });
    return;
  }
  const accepted = registry.resolveRelayResponse({
    relayRequestId,
    response: body.response ?? null,
  });
  json(res, accepted ? 200 : 404, {
    status: accepted ? 'ok' : 'error',
    message: accepted ? 'relay response accepted' : 'relay request not found',
  });
}

export async function handleStoryLockGatewayRequest(req, res, {
  env = process.env,
  fetchImpl = globalThis.fetch,
} = {}) {
  setCors(res);
  if (req.method === 'OPTIONS') {
    res.statusCode = 204;
    res.end();
    return;
  }

  try {
    const url = new URL(req.url ?? '/', getGatewayBaseUrl(req, env));
    if (req.method === 'GET') {
      await handleGet(req, res, url, env);
      return;
    }
    if (req.method !== 'POST') {
      json(res, 405, { status: 'error', message: 'method not allowed' });
      return;
    }

    if (url.pathname === '/android-host/register' || url.pathname === '/local-host/register') {
      await handleAndroidHostRegister(req, res, env);
      return;
    }
    if (url.pathname === '/android-host/relay/poll' || url.pathname === '/local-host/relay/poll') {
      await handleRelayPoll(req, res, env);
      return;
    }
    if (url.pathname === '/android-host/relay/respond' || url.pathname === '/local-host/relay/respond') {
      await handleRelayRespond(req, res, env);
      return;
    }

    const connection = resolveAndroidConnectionConfig(env, {
      gatewayBaseUrl: getGatewayBaseUrl(req, env),
    });
    ensureSharedSecret(req, env);
    const request = await readBody(req);
    const gateway = new StoryLockRemoteGateway({
      transport(remoteRequest) {
        return transportForMode(remoteRequest, {
          env,
          fetchImpl,
          preferredMode: connection.activeOption?.mode ?? connection.preferredMode,
        });
      },
      eip712Env: env,
    });
    const response = await gateway.invoke(request);
    json(res, 200, {
      ...response,
      executionLocation: 'remote_gateway',
      auditMeta: {
        ...(response?.auditMeta ?? {}),
        remoteGateway: 'web_api',
      },
    });
  } catch (error) {
    json(res, error.statusCode ?? 500, {
      status: 'error',
      message: error.message,
    });
  }
}
