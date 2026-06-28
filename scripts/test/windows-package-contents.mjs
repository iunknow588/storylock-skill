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

const requiredEntries = [
  'yian-windows-host.exe',
  'README.md',
  'start-yian-windows-host.cmd',
  'identity-package/package-manifest.json',
  'identity-package/resource-catalog.json',
  'identity-package/learning-policy.json',
  'config/login-sites.json',
  'config/signing-actions.json',
  'config/agent-tasks.json',
  'templates/shouzhudaitu-zh/README.md',
  'templates/shouzhudaitu-zh/story-template.json',
  'templates/zhizi-yilin-zh/README.md',
  'templates/zhizi-yilin-zh/story-template.json',
  'templates/emperor-new-clothes-en/README.md',
  'templates/emperor-new-clothes-en/story-template.json',
];

for (const requiredEntry of requiredEntries) {
  assert.ok(entries.has(requiredEntry), `${relative(root, packagePath)} must include ${requiredEntry}`);
}

const mainSource = readFileSync(join(root, 'src', 'host', 'windows-host', 'src', 'main.rs'), 'utf8');
const uiEntrySource = readFileSync(join(root, 'src', 'host', 'windows-host', 'src', 'host_runtime', 'ui', 'entry.rs'), 'utf8');
const windowsHostCargo = readFileSync(join(root, 'src', 'host', 'windows-host', 'Cargo.toml'), 'utf8');
const desktopLauncher = readFileSync(join(root, 'src', 'host', 'windows-host', 'start-yian-windows-host.cmd'), 'utf8');

assert.match(windowsHostCargo, /default = \["ui-slint"\]/u, 'Windows package must use the Slint UI by default');
assert.doesNotMatch(windowsHostCargo, /ui-tray/u, 'Windows package must not ship a tray UI feature');
assert.match(mainSource, /windows_subsystem = "windows"/u, 'Windows desktop build must use the GUI subsystem when Slint UI is enabled');
assert.match(uiEntrySource, /fn run_default_entry[\s\S]*run_desktop_ui_entry/u, 'Windows host default entry must start the native desktop UI');
assert.match(uiEntrySource, /fn run_desktop_ui_entry[\s\S]*slint_ui::run/u, 'Windows desktop entry must use the Rust Slint UI');
assert.doesNotMatch(`${mainSource}\n${uiEntrySource}`, /run_tray_entry|--tray|mod tray_ui/u, 'Windows desktop entry must not expose a tray UI path');
assert.match(desktopLauncher, /Starting the Slint desktop UI/u, 'desktop launcher must describe the Slint UI path');
assert.doesNotMatch(desktopLauncher, /--console|--tray/u, 'desktop launcher must not select console or tray modes');

console.log(JSON.stringify({
  status: 'passed',
  package: relative(root, packagePath),
  requiredEntries: requiredEntries.length,
}, null, 2));
