import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

export function loadSqliteSchema() {
  return readFileSync(join(__dirname, 'sqlite-schema.sql'), 'utf8');
}

export async function openStoryLockDatabase(dbPath) {
  const sqlite = await import('node:sqlite');
  const db = new sqlite.DatabaseSync(dbPath);
  db.exec(loadSqliteSchema());
  return db;
}
