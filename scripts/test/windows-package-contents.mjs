import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const candidates = [
  join(root, 'release', 'app', 'windows', 'yian-windows-host-0.1.0-1-prototype.zip'),
  join(root, 'release', 'web', 'public', 'downloads', 'yian-windows-host-0.1.0-1-prototype.zip'),
  join(root, 'src', 'yian-web', 'public', 'downloads', 'yian-windows-host-0.1.0-1-prototype.zip'),
];

const packagePath = candidates.find((candidate) => existsSync(candidate));
assert.ok(packagePath, 'Windows package zip must exist in release or bundled downloads');

const entries = new Set(execFileSync('tar', ['-tf', packagePath], {
  encoding: 'utf8',
}).split(/\r?\n/u).filter(Boolean).map((entry) => entry.replace(/\\/gu, '/')));

for (const requiredEntry of [
  'yian-windows-host.exe',
  'README.md',
  'start-yian-windows-host.cmd',
]) {
  assert.ok(entries.has(requiredEntry), `${relative(root, packagePath)} must include ${requiredEntry}`);
}

const mainSource = readFileSync(join(root, 'src', 'host', 'windows-host', 'src', 'main.rs'), 'utf8');
const consoleLauncher = readFileSync(join(root, 'src', 'host', 'windows-host', 'start-yian-windows-host.cmd'), 'utf8');

assert.match(mainSource, /windows_subsystem = "windows"/u, 'Windows desktop build must use the GUI subsystem when tray UI is enabled');
assert.match(mainSource, /fn run_default_entry[\s\S]*run_desktop_ui_entry/u, 'Windows host default entry must start the native desktop UI');
assert.match(mainSource, /fn run_desktop_ui_entry[\s\S]*slint_ui::run/u, 'Windows desktop entry must use the Rust Slint UI');
assert.match(consoleLauncher, /--console/u, 'debug console launcher must opt into --console mode');

console.log(JSON.stringify({
  status: 'passed',
  package: relative(root, packagePath),
  requiredEntries: 3,
}, null, 2));
