import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const scriptPath = join(root, 'scripts', 'release', 'linux', 'package_linux_host_wsl.ps1');
const packageScriptPath = join(root, 'scripts', 'release', 'linux', 'package_linux_host.mjs');

assert.ok(existsSync(scriptPath), 'WSL packaging PowerShell script must exist');
assert.ok(existsSync(packageScriptPath), 'Linux packaging Node script must exist');

const wslScript = readFileSync(scriptPath, 'utf8');
assert.match(wslScript, /\/mnt\/\$drive\/\$relativePath/);
assert.match(wslScript, /Repository path is not mounted in WSL/);
assert.match(wslScript, /storylock-linux-package-/);
assert.match(wslScript, /bash \$wslTempScript/);
assert.match(wslScript, /NVM_DIR/);
assert.match(wslScript, /best_node_version/);
assert.match(wslScript, /sort -V/);
assert.match(wslScript, /nvm use default/);
assert.match(wslScript, /nvm use --lts/);
assert.match(wslScript, /STORYLOCK_WSL_NODE_BIN/);
assert.match(wslScript, /command -v node/);
assert.match(wslScript, /node_major/);
assert.match(wslScript, /command -v dpkg-deb/);
assert.match(wslScript, /STORYLOCK_LINUX_PACKAGE_KIND="tar\.gz"/);
assert.match(wslScript, /node scripts\/release\/linux\/package_linux_host\.mjs/);

const packageScript = readFileSync(packageScriptPath, 'utf8');
assert.match(packageScript, /process\.platform === 'win32' \? 'zip' : 'tar\.gz'/);
assert.match(packageScript, /dpkg-deb/);
assert.match(packageScript, /desktopIntegration/);

console.log(JSON.stringify({
  status: 'passed',
  script: 'scripts/release/linux/package_linux_host_wsl.ps1',
}, null, 2));
