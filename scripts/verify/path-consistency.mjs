import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));

const skippedDirectories = new Set([
  '.git',
  '.vercel',
  'node_modules',
  'target',
]);

const skippedRelativePrefixes = [
  'docs/management/back/',
  'src/storylock-local-story-access-skill/',
  'src/storylock-local-story-processing-skill/',
  'src/storylock-remote-gateway-skill/',
  'src/storylock-skill-engine/',
];

const searchableExtensions = new Set([
  '.cmd',
  '.js',
  '.json',
  '.md',
  '.mjs',
  '.ps1',
  '.txt',
  '.yaml',
  '.yml',
]);

const stalePathPatterns = [
  {
    pattern: /(?:^|[^A-Za-z0-9_-])(?:skill\/)?src\/storylock-local-story-processing-skill(?:\/|`|'|"|\s|$)/u,
    replacement: 'src/skills/local-story-processing',
  },
  {
    pattern: /(?:^|[^A-Za-z0-9_-])(?:skill\/)?src\/storylock-local-story-access-skill(?:\/|`|'|"|\s|$)/u,
    replacement: 'src/skills/local-story-access',
  },
  {
    pattern: /(?:^|[^A-Za-z0-9_-])(?:skill\/)?src\/storylock-remote-gateway-skill(?:\/|`|'|"|\s|$)/u,
    replacement: 'src/skills/remote-gateway',
  },
  {
    pattern: /(?:^|[^A-Za-z0-9_-])(?:skill\/)?src\/storylock-skill-engine(?:\/|`|'|"|\s|$)/u,
    replacement: 'src/engine',
  },
];

function normalizePath(value) {
  return value.replaceAll('\\', '/');
}

function extensionOf(fileName) {
  const index = fileName.lastIndexOf('.');
  return index === -1 ? '' : fileName.slice(index).toLowerCase();
}

function shouldSkip(relativePath, entryName = '') {
  const normalized = normalizePath(relativePath);
  if (skippedDirectories.has(entryName)) {
    return true;
  }
  if (normalized === 'scripts/verify/path-consistency.mjs') {
    return true;
  }
  return skippedRelativePrefixes.some((prefix) => normalized.startsWith(prefix));
}

function walk(directory, output = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolutePath = join(directory, entry.name);
    const relativePath = normalizePath(relative(root, absolutePath));
    if (entry.isDirectory()) {
      if (shouldSkip(relativePath, entry.name)) {
        continue;
      }
      walk(absolutePath, output);
      continue;
    }
    if (shouldSkip(relativePath) || !searchableExtensions.has(extensionOf(entry.name))) {
      continue;
    }
    output.push(absolutePath);
  }
  return output;
}

const violations = [];
const files = walk(root);

for (const file of files) {
  const text = readFileSync(file, 'utf8');
  const lines = text.split(/\r?\n/u);
  lines.forEach((line, index) => {
    for (const { pattern, replacement } of stalePathPatterns) {
      if (pattern.test(line)) {
        violations.push({
          file: normalizePath(relative(root, file)),
          line: index + 1,
          replacement,
          text: line.trim(),
        });
      }
    }
  });
}

assert.equal(
  violations.length,
  0,
  violations
    .map((item) => `${item.file}:${item.line} stale path; use ${item.replacement}\n  ${item.text}`)
    .join('\n'),
);

console.log(JSON.stringify({
  status: 'passed',
  filesChecked: files.length,
}, null, 2));
