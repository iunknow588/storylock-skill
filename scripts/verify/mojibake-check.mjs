import { readdirSync, readFileSync, statSync } from 'node:fs';
import { extname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));

const scanRoots = [
  'api',
  'docs',
  'scripts',
  'src',
  'web-api',
  'package.json',
  'vercel.json',
  'README.md',
];

const skipDirs = new Set([
  '.git',
  '.temp',
  '.vercel',
  'node_modules',
  'target',
]);

const textExtensions = new Set([
  '.cmd',
  '.css',
  '.html',
  '.js',
  '.json',
  '.md',
  '.mjs',
  '.ps1',
  '.py',
  '.rs',
  '.sh',
  '.slint',
  '.sql',
  '.toml',
  '.txt',
  '.xml',
  '.yaml',
  '.yml',
]);

const fromCodePoints = (...values) => String.fromCodePoint(...values);

const suspiciousPatterns = [
  new RegExp(fromCodePoints(0xfffd), 'u'),
  new RegExp(fromCodePoints(0x00ef, 0x00bf, 0x00bd), 'u'),
  /Ã[\u0080-\u00bf]/u,
  /Â[\u0080-\u00bf]/u,
  /â[\u0080-\u00bf]/u,
  new RegExp(fromCodePoints(0x6fb6, 0x8f9, 0x798c), 'u'),
  new RegExp(fromCodePoints(0x9a9e, 0x51b2, 0x5f34), 'u'),
  new RegExp(fromCodePoints(0x694a, 0x5c6, 0x656a), 'u'),
  new RegExp(fromCodePoints(0x942d, 0x256, 0x6a00), 'u'),
  new RegExp(fromCodePoints(0x942a, 0x719f, 0x6e80), 'u'),
  new RegExp(fromCodePoints(0x5997, 0x5cac, 0x6f70), 'u'),
  new RegExp(fromCodePoints(0x5bf0, 0x544a, 0x59d9), 'u'),
  new RegExp(fromCodePoints(0x95c2, 0xe155, 0x5e06), 'u'),
  new RegExp(fromCodePoints(0x59ab, 0x20ac, 0x93cc), 'u'),
  new RegExp(fromCodePoints(0x5a62, 0x5806), 'u'),
  new RegExp(fromCodePoints(0x7f01, 0x694a), 'u'),
  new RegExp(fromCodePoints(0x5a11), 'u'),
  new RegExp(fromCodePoints(0x745c, 0x7248), 'u'),
];

function* walk(path) {
  const stats = statSync(path);
  if (stats.isDirectory()) {
    const name = path.split(/[\\/]/u).at(-1);
    if (skipDirs.has(name)) {
      return;
    }
    for (const entry of readdirSync(path)) {
      yield* walk(join(path, entry));
    }
    return;
  }

  if (stats.isFile() && textExtensions.has(extname(path))) {
    yield path;
  }
}

const findings = [];
let filesChecked = 0;

for (const scanRoot of scanRoots) {
  const target = join(root, scanRoot);
  for (const file of walk(target)) {
    filesChecked += 1;
    const text = readFileSync(file, 'utf8');
    const lines = text.split(/\r?\n/u);
    lines.forEach((line, index) => {
      const matched = suspiciousPatterns.find((pattern) => pattern.test(line));
      if (!matched) {
        return;
      }
      findings.push({
        file: relative(root, file).replaceAll('\\', '/'),
        line: index + 1,
        pattern: matched.source,
      });
    });
  }
}

if (findings.length > 0) {
  console.error(JSON.stringify({
    status: 'failed',
    filesChecked,
    findings,
  }, null, 2));
  process.exit(1);
}

console.log(JSON.stringify({
  status: 'passed',
  filesChecked,
}, null, 2));
