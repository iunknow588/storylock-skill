import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const files = {
  executable: join(root, 'src', 'host', 'linux-host', 'bin', 'yian-linux-host'),
  desktop: join(root, 'src', 'host', 'linux-host', 'desktop', 'yian-linux-host.desktop'),
  systemd: join(root, 'src', 'host', 'linux-host', 'systemd', 'yian-linux-host.service'),
  control: join(root, 'src', 'host', 'linux-host', 'packaging', 'debian', 'control'),
  postinst: join(root, 'src', 'host', 'linux-host', 'packaging', 'debian', 'postinst'),
  prerm: join(root, 'src', 'host', 'linux-host', 'packaging', 'debian', 'prerm'),
  manifest: join(root, 'release', 'app', 'linux', 'release-manifest-0.1.0-1-prototype.json'),
  stagedExecutable: join(root, '.temp', 'dist', 'linux-host-deb', 'opt', 'yian-linux-host', 'bin', 'yian-linux-host'),
};

for (const [name, path] of Object.entries(files)) {
  assert.ok(existsSync(path), `${name} must exist at ${path}`);
}

const executable = readFileSync(files.executable, 'utf8');
assert.match(executable, /^#!\/usr\/bin\/env sh/u);
assert.match(executable, /STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE/);
assert.match(executable, /STORYLOCK_LINUX_DEVELOPMENT_MODE/);

const desktop = readFileSync(files.desktop, 'utf8');
assert.match(desktop, /\[Desktop Entry\]/);
assert.match(desktop, /Exec=\/opt\/yian-linux-host\/bin\/yian-linux-host/);
assert.match(desktop, /Categories=Utility;Security;/);

const systemd = readFileSync(files.systemd, 'utf8');
assert.match(systemd, /\[Service\]/);
assert.match(systemd, /ExecStart=\/opt\/yian-linux-host\/bin\/yian-linux-host/);
assert.match(systemd, /STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE=1/);

const control = readFileSync(files.control, 'utf8');
assert.match(control, /Package: yian-linux-host/);
assert.match(control, /Depends: nodejs \(>= 22\), libsecret-tools/);

const manifest = JSON.parse(readFileSync(files.manifest, 'utf8'));
assert.equal(manifest.desktopIntegration.executable, '/opt/yian-linux-host/bin/yian-linux-host');
assert.equal(manifest.desktopIntegration.desktopEntry, '/usr/share/applications/yian-linux-host.desktop');
assert.equal(manifest.desktopIntegration.systemdUserUnit, '/usr/lib/systemd/user/yian-linux-host.service');
assert.equal(manifest.productionSecretStore.env.STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE, '1');

console.log(JSON.stringify({
  status: 'passed',
  filesChecked: Object.keys(files).length,
  debBuilt: manifest.desktopIntegration.debBuilt,
}, null, 2));
