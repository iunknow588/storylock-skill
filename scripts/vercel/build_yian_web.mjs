import { cp, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const source = join(root, 'src', 'yian-web', 'public');
const target = join(root, 'public');

await rm(target, { recursive: true, force: true });
await cp(source, target, { recursive: true });

console.log(`Copied ${source} -> ${target}`);
