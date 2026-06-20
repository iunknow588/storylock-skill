import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const downloadsDir = join(root, 'release', 'web', 'public', 'downloads');
const requiredPlatforms = new Set(['android', 'windows', 'linux']);
const checkedPlatforms = new Set();

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function sha256For(path) {
  return `sha256:${createHash('sha256').update(readFileSync(path)).digest('hex')}`;
}

assert.ok(existsSync(downloadsDir), 'release web downloads directory must exist');

const metadataFiles = readdirSync(downloadsDir)
  .filter((fileName) => fileName.endsWith('.json'))
  .sort();
assert.ok(metadataFiles.length > 0, 'release downloads must include metadata json files');

for (const metadataFile of metadataFiles) {
  const metadataPath = join(downloadsDir, metadataFile);
  const metadata = readJson(metadataPath);
  const displayPath = relative(root, metadataPath);

  assert.ok(metadata.fileName, `${displayPath} must include fileName`);
  assert.ok(metadata.platform, `${displayPath} must include platform`);
  assert.ok(metadata.packageKind, `${displayPath} must include packageKind`);
  assert.ok(metadata.releaseChannel, `${displayPath} must include releaseChannel`);
  assert.ok(metadata.checksum, `${displayPath} must include checksum`);
  assert.match(metadata.checksum, /^sha256:[0-9a-f]{64}$/u, `${displayPath} checksum must be sha256 hex`);

  const artifactPath = join(downloadsDir, metadata.fileName);
  assert.notEqual(metadataFile, metadata.fileName, `${displayPath} metadata must not reuse artifact file name`);
  assert.ok(existsSync(artifactPath), `${displayPath} artifact ${metadata.fileName} must exist`);
  assert.equal(statSync(artifactPath).size, metadata.fileSizeBytes, `${displayPath} fileSizeBytes must match artifact`);
  assert.equal(sha256For(artifactPath), metadata.checksum, `${displayPath} checksum must match artifact`);

  checkedPlatforms.add(metadata.platform);
}

for (const platform of requiredPlatforms) {
  assert.ok(checkedPlatforms.has(platform), `release metadata must include ${platform}`);
}

console.log(JSON.stringify({
  status: 'passed',
  metadataFiles: metadataFiles.length,
  platforms: [...checkedPlatforms].sort(),
}, null, 2));
