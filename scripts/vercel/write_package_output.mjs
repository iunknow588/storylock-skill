import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const [, , platform, envFilePath, outputPath] = process.argv;

if (!platform || !envFilePath || !outputPath) {
  console.error('Usage: node scripts/vercel/write_package_output.mjs <android|windows> <env-file> <output-json>');
  process.exit(2);
}

function readEnv(content) {
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
        return [line.slice(0, index).trim(), line.slice(index + 1)];
      })
      .filter(Boolean),
  );
}

const resolvedEnvFile = resolve(envFilePath);
const resolvedOutput = resolve(outputPath);
const env = readEnv(await readFile(resolvedEnvFile, 'utf8'));
const current = existsSync(resolvedOutput)
  ? JSON.parse(await readFile(resolvedOutput, 'utf8'))
  : {};

const next = {
  ...current,
  generatedAt: new Date().toISOString(),
  note: 'Generated build/package summary. Copy values into scripts/vercel/.env only when a manual deployment needs them.',
  [platform]: {
    env,
    envFilePath: resolvedEnvFile,
    updatedAt: new Date().toISOString(),
  },
};

await mkdir(dirname(resolvedOutput), { recursive: true });
await writeFile(`${resolvedOutput}.tmp`, `${JSON.stringify(next, null, 2)}\n`, 'utf8');
await rename(`${resolvedOutput}.tmp`, resolvedOutput);
console.log(`Package output updated: ${resolvedOutput}`);
