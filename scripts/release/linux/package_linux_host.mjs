import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { mkdir, rm, cp, writeFile, readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { readdir } from 'node:fs/promises';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(dirname(dirname(fileURLToPath(import.meta.url)))));
const version = process.env.STORYLOCK_LINUX_PACKAGE_VERSION ?? '0.1.0';
const versionCode = process.env.STORYLOCK_LINUX_PACKAGE_VERSION_CODE ?? '1';
const releaseChannel = process.env.STORYLOCK_LINUX_RELEASE_CHANNEL ?? 'prototype';
const packageKind = process.env.STORYLOCK_LINUX_PACKAGE_KIND ?? (process.platform === 'win32' ? 'zip' : 'tar.gz');
const outputDir = join(root, 'release', 'app', 'linux');
const stageDir = join(root, '.temp', 'dist', 'linux-host-package');
const packageExtension = packageKind === 'tar.gz' ? 'tar.gz' : 'zip';
const packageName = `yian-linux-host-${version}-${versionCode}-${releaseChannel}.${packageExtension}`;
const packagePath = join(outputDir, packageName);
const debPath = join(outputDir, `yian-linux-host-${version}-${versionCode}-${releaseChannel}.deb`);
const manifestPath = join(outputDir, `release-manifest-${version}-${versionCode}-${releaseChannel}.json`);
const debStageDir = join(root, '.temp', 'dist', 'linux-host-deb');
let debBuildError = null;

function run(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    execFile(command, args, { windowsHide: true, ...options }, (error, stdout, stderr) => {
      if (error) {
        error.stdout = stdout;
        error.stderr = stderr;
        reject(error);
      } else {
        resolve({ stdout, stderr });
      }
    });
  });
}

async function copyRequiredFiles() {
  await rm(stageDir, { recursive: true, force: true });
  await mkdir(join(stageDir, 'src', 'host'), { recursive: true });
  await mkdir(join(stageDir, 'src', 'skills'), { recursive: true });
  await mkdir(join(stageDir, 'src'), { recursive: true });
  await mkdir(join(stageDir, 'bin'), { recursive: true });
  await cp(join(root, 'src', 'host', 'linux-host'), join(stageDir, 'src', 'host', 'linux-host'), { recursive: true });
  await cp(join(root, 'src', 'host', 'linux-host', 'bin', 'yian-linux-host'), join(stageDir, 'bin', 'yian-linux-host'));
  await cp(join(root, 'src', 'skills', 'local-story-access'), join(stageDir, 'src', 'skills', 'local-story-access'), { recursive: true });
  await cp(join(root, 'src', 'shared'), join(stageDir, 'src', 'shared'), { recursive: true });
  await cp(join(root, 'package.json'), join(stageDir, 'package.json'));
  await writeFile(join(stageDir, 'README.md'), `# Yian Linux Host Package

Prototype package generated from the StoryLock workspace.

Run:

\`\`\`bash
node src/host/linux-host/server.mjs
\`\`\`

Self-check from the full workspace:

\`\`\`bash
npm run test:linux-host
\`\`\`

Production-mode start:

\`\`\`bash
STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE=1 STORYLOCK_LINUX_DEVELOPMENT_MODE=0 ./src/host/linux-host/bin/yian-linux-host
\`\`\`
`, 'utf8');
}

async function createArchive() {
  await mkdir(outputDir, { recursive: true });
  for (const fileName of await readdir(outputDir)) {
    if (fileName.startsWith(`yian-linux-host-${version}-${versionCode}-${releaseChannel}.`)) {
      await rm(join(outputDir, fileName), { force: true });
    }
  }
  await rm(packagePath, { force: true });
  if (packageKind === 'zip') {
    if (process.platform !== 'win32') {
      await run('zip', ['-qr', packagePath, '.'], { cwd: stageDir });
      return;
    }
    const command = [
      '$ErrorActionPreference = "Stop"',
      `Compress-Archive -Path ${JSON.stringify(join(stageDir, '*'))} -DestinationPath ${JSON.stringify(packagePath)} -Force`,
    ].join('; ');
    await run('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', command]);
    return;
  }
  await run('tar', ['-czf', packagePath, '-C', stageDir, '.']);
}

async function createDebStaging() {
  await rm(debStageDir, { recursive: true, force: true });
  const appDir = join(debStageDir, 'opt', 'yian-linux-host');
  const desktopDir = join(debStageDir, 'usr', 'share', 'applications');
  const systemdDir = join(debStageDir, 'usr', 'lib', 'systemd', 'user');
  const debianDir = join(debStageDir, 'DEBIAN');
  await mkdir(appDir, { recursive: true });
  await mkdir(desktopDir, { recursive: true });
  await mkdir(systemdDir, { recursive: true });
  await mkdir(debianDir, { recursive: true });
  await cp(stageDir, appDir, { recursive: true });
  await cp(join(stageDir, 'src', 'host', 'linux-host', 'desktop', 'yian-linux-host.desktop'), join(desktopDir, 'yian-linux-host.desktop'));
  await cp(join(stageDir, 'src', 'host', 'linux-host', 'systemd', 'yian-linux-host.service'), join(systemdDir, 'yian-linux-host.service'));
  await cp(join(stageDir, 'src', 'host', 'linux-host', 'packaging', 'debian', 'control'), join(debianDir, 'control'));
  await cp(join(stageDir, 'src', 'host', 'linux-host', 'packaging', 'debian', 'postinst'), join(debianDir, 'postinst'));
  await cp(join(stageDir, 'src', 'host', 'linux-host', 'packaging', 'debian', 'prerm'), join(debianDir, 'prerm'));
  if (process.platform === 'win32') {
    await writeFile(join(debStageDir, 'DEBIAN-STAGING-README.txt'), [
      'This directory is a Debian package staging tree.',
      'Build on Linux with: dpkg-deb --build <staging-dir> <output.deb>',
      'Windows local packaging skips Unix chmod and dpkg-deb; use npm run package:linux-host:wsl for a .deb.',
      '',
    ].join('\n'), 'utf8');
    return;
  }
  await run('find', [debStageDir, '-type', 'd', '-exec', 'chmod', '0755', '{}', ';']);
  await run('find', [debStageDir, '-type', 'f', '-exec', 'chmod', '0644', '{}', ';']);
  await run('chmod', ['0644', join(debianDir, 'control')]);
  await run('chmod', ['0755', join(debianDir, 'postinst')]);
  await run('chmod', ['0755', join(debianDir, 'prerm')]);
  await run('chmod', ['0755', join(appDir, 'bin', 'yian-linux-host')]);
  await run('chmod', ['0755', join(appDir, 'src', 'host', 'linux-host', 'bin', 'yian-linux-host')]);
  await writeFile(join(debStageDir, 'DEBIAN-STAGING-README.txt'), [
    'This directory is a Debian package staging tree.',
    'Build on Linux with: dpkg-deb --build <staging-dir> <output.deb>',
    '',
  ].join('\n'), 'utf8');
}

async function createDebIfAvailable() {
  await rm(debPath, { force: true });
  await createDebStaging();
  if (process.platform === 'win32') {
    debBuildError = 'Windows local packaging created Debian staging only; use npm run package:linux-host:wsl to build .deb.';
    return false;
  }
  try {
    await run('dpkg-deb', ['--build', debStageDir, debPath]);
    return existsSync(debPath);
  } catch (error) {
    debBuildError = [
      error.message,
      error.stderr?.trim(),
      error.stdout?.trim(),
    ].filter(Boolean).join('\n');
    return false;
  }
}

async function writeManifest() {
  const bytes = await readFile(packagePath);
  const checksum = `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
  const artifacts = [
    {
      kind: packageKind,
      path: packagePath,
      fileName: basename(packagePath),
      sizeBytes: bytes.length,
      checksum,
    },
  ];
  if (existsSync(debPath)) {
    const debBytes = await readFile(debPath);
    artifacts.push({
      kind: 'deb',
      path: debPath,
      fileName: basename(debPath),
      sizeBytes: debBytes.length,
      checksum: `sha256:${createHash('sha256').update(debBytes).digest('hex')}`,
    });
  }
  const manifest = {
    product: 'yian-linux-host',
    version,
    versionCode,
    releaseChannel,
    packageKind,
    builtAt: new Date().toISOString(),
    artifacts,
    desktopIntegration: {
      executable: '/opt/yian-linux-host/bin/yian-linux-host',
      desktopEntry: '/usr/share/applications/yian-linux-host.desktop',
      systemdUserUnit: '/usr/lib/systemd/user/yian-linux-host.service',
      debianStagingDir: debStageDir,
      debBuilt: existsSync(debPath),
      debBuildError,
    },
    productionSecretStore: {
      env: {
        STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE: '1',
        STORYLOCK_LINUX_DEVELOPMENT_MODE: '0',
      },
      dependencies: ['nodejs >= 22', 'libsecret-tools', 'Secret Service provider'],
    },
    limitations: [
      'prototype package',
      'requires Node.js >=22',
      'production mode requires Linux secret-tool and Secret Service',
      'desktop integration is included as a prototype and still needs distribution-level validation',
      'signed Linux packages are not included yet',
    ],
  };
  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  return manifest;
}

await copyRequiredFiles();
await createArchive();
if (!existsSync(packagePath)) {
  throw new Error(`Linux package was not created: ${packagePath}`);
}
await createDebIfAvailable();
const manifest = await writeManifest();
console.log(JSON.stringify({
  status: 'success',
  packagePath,
  manifestPath,
  artifacts: manifest.artifacts,
  debianStagingDir: debStageDir,
}, null, 2));
