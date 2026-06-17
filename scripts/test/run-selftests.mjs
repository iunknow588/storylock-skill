import { spawnSync } from 'node:child_process';

const packages = [
  ['storylock-local-story-processing-skill', ['npm', ['run', 'selftest']]],
  ['storylock-local-story-access-skill', ['npm', ['run', 'selftest']]],
  ['storylock-remote-gateway-skill', ['npm', ['run', 'selftest']]],
  ['storylock-remote-gateway-skill', ['npm', ['run', 'selftest:e2e']]],
  ['storylock-skill-engine', ['npm', ['run', 'selftest']]],
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
