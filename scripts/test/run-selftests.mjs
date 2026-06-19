import { spawnSync } from 'node:child_process';

const packages = [
  ['skills/local-story-processing', ['npm', ['run', 'selftest']]],
  ['skills/local-story-access', ['npm', ['run', 'selftest']]],
  ['skills/remote-gateway', ['npm', ['run', 'selftest']]],
  ['skills/remote-gateway', ['npm', ['run', 'check:agent-capabilities']]],
  ['skills/remote-gateway', ['npm', ['run', 'selftest:e2e']]],
  ['skills/remote-gateway', ['npm', ['run', 'selftest:web-api-android']]],
  ['ui', ['npm', ['run', 'selftest']]],
  ['engine', ['npm', ['run', 'selftest']]],
];

for (const [packageName, [command, args]] of packages) {
  const cwd = new URL(`../../src/${packageName}/`, import.meta.url);
  const result = spawnSync(command, args, {
    cwd,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

console.log('StoryLock workspace selftest passed.');
