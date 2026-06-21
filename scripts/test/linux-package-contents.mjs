import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const zipPackagePath = join(root, 'release', 'app', 'linux', 'yian-linux-host-0.1.0-1-prototype.zip');
const tarPackagePath = join(root, 'release', 'app', 'linux', 'yian-linux-host-0.1.0-1-prototype.tar.gz');
const debPackagePath = join(root, 'release', 'app', 'linux', 'yian-linux-host-0.1.0-1-prototype.deb');
const wslDistro = process.env.STORYLOCK_WSL_DISTRO ?? 'Ubuntu-22.04';
const requiredEntries = [
  'README.md',
  'package.json',
  'bin/yian-linux-host',
  'src/host/linux-host/server.mjs',
  'src/host/linux-host/bin/yian-linux-host',
  'src/host/linux-host/desktop/yian-linux-host.desktop',
  'src/host/linux-host/systemd/yian-linux-host.service',
  'src/host/linux-host/packaging/debian/control',
  'src/host/linux-host/assets/question-bank.json',
  'src/skills/local-story-access/index.js',
  'src/skills/local-story-access/access-host.js',
  'src/shared/secret-store.js',
  'src/shared/sqlite-schema.sql',
];

const archivePath = existsSync(tarPackagePath) ? tarPackagePath : zipPackagePath;
assert.ok(existsSync(archivePath), 'Linux prototype package must exist');

function listArchiveEntries(path) {
  if (path.endsWith('.tar.gz')) {
    return new Set(execFileSync('tar', ['-tzf', path], {
      encoding: 'utf8',
      windowsHide: true,
    }).split(/\r?\n/u)
      .map((line) => line.trim().replace(/^\.\//u, ''))
      .filter(Boolean));
  }

  const script = [
    '$ErrorActionPreference = "Stop"',
    `Add-Type -AssemblyName System.IO.Compression.FileSystem`,
    `$zip = [System.IO.Compression.ZipFile]::OpenRead(${JSON.stringify(path)})`,
    'try { $zip.Entries | ForEach-Object { $_.FullName.Replace("\\", "/") } } finally { $zip.Dispose() }',
  ].join('; ');
  const encodedScript = Buffer.from(script, 'utf16le').toString('base64');
  return new Set(execFileSync('powershell.exe', [
    '-NoProfile',
    '-NonInteractive',
    '-EncodedCommand',
    encodedScript,
  ], {
    encoding: 'utf8',
    windowsHide: true,
  }).split(/\r?\n/u).map((line) => line.trim()).filter(Boolean));
}

const entries = listArchiveEntries(archivePath);

for (const requiredEntry of requiredEntries) {
  assert.ok(entries.has(requiredEntry), `Linux package must include ${requiredEntry}`);
}

let debEntries = 0;
if (existsSync(debPackagePath)) {
  const debContents = execFileSync('wsl.exe', [
    '-d',
    wslDistro,
    '--',
    'bash',
    '-lc',
    `dpkg-deb --contents ${JSON.stringify(debPackagePath.replace(/^([A-Za-z]):\\/u, (_, drive) => `/mnt/${drive.toLowerCase()}/`).replaceAll('\\', '/'))}`,
  ], {
    encoding: 'utf8',
    windowsHide: true,
  });
  for (const requiredDebEntry of [
    './opt/yian-linux-host/bin/yian-linux-host',
    './usr/share/applications/yian-linux-host.desktop',
    './usr/lib/systemd/user/yian-linux-host.service',
  ]) {
    assert.ok(debContents.includes(requiredDebEntry), `Linux deb must include ${requiredDebEntry}`);
  }
  debEntries = debContents.split(/\r?\n/u).filter(Boolean).length;
}

console.log(JSON.stringify({
  status: 'passed',
  packagePath: archivePath,
  requiredEntries: requiredEntries.length,
  totalEntries: entries.size,
  debPackagePath: existsSync(debPackagePath) ? debPackagePath : null,
  wslDistro: existsSync(debPackagePath) ? wslDistro : null,
  debEntries,
}, null, 2));
