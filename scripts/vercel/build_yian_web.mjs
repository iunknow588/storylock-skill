import { access, cp, mkdir, readdir, rm, readFile, stat, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { basename, dirname, join } from 'node:path';
import { createHash } from 'node:crypto';

const root = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const source = join(root, 'src', 'yian-web', 'public');
const target = join(root, 'release', 'web', 'public');
const appArtifactsRoot = join(root, 'release', 'app');
const windowsEnvFile = join(root, 'scripts', 'vercel', '.env.windows-package');
const defaultWindowsPackageName = 'yian-windows-host-0.1.0-1-prototype.zip';
const packagePattern = /\.(apk|zip|msi|exe|appimage|deb|rpm)$/iu;

async function readEnvFile(path) {
  if (!existsSync(path)) {
    return {};
  }
  const content = await readFile(path, 'utf8');
  return Object.fromEntries(
    content
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith('#'))
      .map((line) => {
        const index = line.indexOf('=');
        if (index < 0) {
          return null;
        }
        return [line.slice(0, index).trim(), line.slice(index + 1).trim()];
      })
      .filter(Boolean),
  );
}

async function listPackageFiles(directory, predicate = () => true) {
  if (!existsSync(directory)) {
    return [];
  }
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    if (!entry.isFile() || !packagePattern.test(entry.name) || !predicate(entry.name)) {
      continue;
    }
    const absolutePath = join(directory, entry.name);
    const info = await stat(absolutePath);
    files.push({
      absolutePath,
      fileName: entry.name,
      modifiedTimeMs: info.mtimeMs,
    });
  }
  return files.sort((left, right) => right.modifiedTimeMs - left.modifiedTimeMs);
}

function inferPlatform(fileName, platformDirectory = null) {
  if (platformDirectory) {
    return platformDirectory;
  }
  const normalized = fileName.toLowerCase();
  if (normalized.endsWith('.apk') || normalized.includes('android')) {
    return 'android';
  }
  if (normalized.includes('windows') || /\.(msi|exe)$/iu.test(normalized)) {
    return 'windows';
  }
  if (/\.(appimage|deb|rpm)$/iu.test(normalized) || normalized.includes('linux')) {
    return 'linux';
  }
  return 'unknown';
}

function inferPackageKind(fileName) {
  const normalized = fileName.toLowerCase();
  if (normalized.endsWith('.apk')) {
    return normalized.includes('release') ? 'release' : normalized.includes('debug') ? 'debug' : 'apk';
  }
  if (normalized.endsWith('.msi')) {
    return 'msi';
  }
  if (normalized.endsWith('.exe')) {
    return 'exe';
  }
  if (normalized.endsWith('.zip')) {
    return 'zip';
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

function inferReleaseChannel(fileName, packageKind) {
  const normalized = fileName.toLowerCase();
  if (normalized.includes('prototype')) {
    return 'prototype';
  }
  if (packageKind === 'debug') {
    return 'internal';
  }
  if (packageKind === 'release') {
    return 'candidate';
  }
  return 'local';
}

function inferVersionMetadata(fileName) {
  const withoutExtension = fileName.replace(/\.[^.]+$/u, '');
  const match = withoutExtension.match(/(?:storylock-android-host|yian-windows-host)-([0-9]+(?:\.[0-9]+)*)-([0-9]+)/u);
  if (!match) {
    return {};
  }
  return {
    versionName: match[1],
    versionCode: match[2],
  };
}

function metadataNameFor(fileName) {
  return fileName.replace(/\.[^.]+$/u, '.json');
}

async function copyPackageToDownloads(packagePath, {
  downloadsDir,
  platform = null,
  metadata = {},
} = {}) {
  const fileName = basename(packagePath);
  const outputPath = join(downloadsDir, fileName);
  await mkdir(downloadsDir, { recursive: true });
  if (packagePath !== outputPath) {
    await cp(packagePath, outputPath);
  }
  const packageBytes = await readFile(outputPath);
  const checksum = `sha256:${createHash('sha256').update(packageBytes).digest('hex')}`;
  const packageKind = inferPackageKind(fileName);
  await writeFile(
    join(downloadsDir, metadataNameFor(fileName)),
    `${JSON.stringify({
      fileName,
      platform: inferPlatform(fileName, platform),
      fileSizeBytes: packageBytes.length,
      checksum,
      packageKind,
      releaseChannel: inferReleaseChannel(fileName, packageKind),
      ...inferVersionMetadata(fileName),
      ...metadata,
    }, null, 2)}\n`,
    'utf8',
  );
  return outputPath;
}

async function copyAppArtifactsToDownloads(downloadsDir) {
  if (!existsSync(appArtifactsRoot)) {
    return [];
  }
  const platformDirs = await readdir(appArtifactsRoot, { withFileTypes: true });
  const copied = [];
  for (const entry of platformDirs) {
    if (!entry.isDirectory()) {
      continue;
    }
    const platform = entry.name;
    const packageFiles = await listPackageFiles(join(appArtifactsRoot, platform));
    for (const packageFile of packageFiles) {
      copied.push(await copyPackageToDownloads(packageFile.absolutePath, {
        downloadsDir,
        platform,
      }));
    }
  }
  return copied;
}

await rm(target, { recursive: true, force: true });
await cp(source, target, { recursive: true });
const downloadsDir = join(target, 'downloads');
await copyAppArtifactsToDownloads(downloadsDir);

const windowsEnv = await readEnvFile(windowsEnvFile);
const windowsPackagePath = windowsEnv.STORYLOCK_WINDOWS_PACKAGE_PATH;
const appWindowsPackagePath = join(appArtifactsRoot, 'windows', defaultWindowsPackageName);
const newestAppWindowsPackage = (await listPackageFiles(
  join(appArtifactsRoot, 'windows'),
  (fileName) => /\.(zip|msi|exe)$/iu.test(fileName),
))[0]?.absolutePath;
const bundledWindowsPackagePath = join(downloadsDir, defaultWindowsPackageName);
const resolvedWindowsPackagePath =
  windowsPackagePath && existsSync(windowsPackagePath)
    ? windowsPackagePath
    : existsSync(appWindowsPackagePath)
      ? appWindowsPackagePath
      : newestAppWindowsPackage
        ? newestAppWindowsPackage
        : existsSync(bundledWindowsPackagePath)
          ? bundledWindowsPackagePath
          : null;
if (resolvedWindowsPackagePath) {
  const fileName = basename(resolvedWindowsPackagePath);
  await copyPackageToDownloads(resolvedWindowsPackagePath, {
    downloadsDir,
    platform: 'windows',
    metadata: {
      versionName: windowsEnv.STORYLOCK_WINDOWS_PACKAGE_VERSION ?? '0.1.0',
      versionCode: windowsEnv.STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE ?? '1',
      packageKind: windowsEnv.STORYLOCK_WINDOWS_PACKAGE_KIND ?? 'zip',
      releaseChannel: windowsEnv.STORYLOCK_WINDOWS_RELEASE_CHANNEL ?? 'prototype',
    },
  });
}

for (const requiredFile of ['index.html', 'main.js', 'styles.css']) {
  await access(join(target, requiredFile));
}
if (existsSync(join(target, 'downloads', defaultWindowsPackageName))) {
  await access(join(target, 'downloads', defaultWindowsPackageName.replace(/\.[^.]+$/u, '.json')));
}

console.log(`Copied ${source} -> ${target}`);
