import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

export function loadSqliteSchema() {
  return readFileSync(join(__dirname, 'sqlite-schema.sql'), 'utf8');
}

function tableColumns(db, tableName) {
  return new Set(db.prepare(`PRAGMA table_info(${tableName})`).all().map((row) => row.name));
}

function addColumnIfMissing(db, tableName, columnName, definition) {
  const columns = tableColumns(db, tableName);
  if (!columns.has(columnName)) {
    db.exec(`ALTER TABLE ${tableName} ADD COLUMN ${columnName} ${definition}`);
  }
}

export function migrateSqliteSchema(db) {
  addColumnIfMissing(db, 'request_store', 'request_hash', "TEXT NOT NULL DEFAULT ''");
  addColumnIfMissing(db, 'request_store', 'response_json', 'TEXT');
  addColumnIfMissing(db, 'request_store', 'response_status', 'TEXT');
  addColumnIfMissing(db, 'request_store', 'response_created_at', 'INTEGER');

  addColumnIfMissing(db, 'challenge_state', 'challenge_manifest_json', "TEXT NOT NULL DEFAULT '{}'");
  addColumnIfMissing(db, 'challenge_state', 'required_threshold', 'INTEGER NOT NULL DEFAULT 1');
  addColumnIfMissing(db, 'challenge_state', 'normalization_version', "TEXT NOT NULL DEFAULT 'nfkc-lower-v1'");
  addColumnIfMissing(db, 'challenge_state', 'question_set_version', "TEXT NOT NULL DEFAULT 'legacy'");

  addColumnIfMissing(db, 'answer_digest_set', 'normalization_version', "TEXT NOT NULL DEFAULT 'nfkc-lower-v1'");
  addColumnIfMissing(db, 'answer_digest_set', 'question_set_version', "TEXT NOT NULL DEFAULT 'legacy'");

  addColumnIfMissing(db, 'audit_log', 'redaction_level', 'TEXT');
  addColumnIfMissing(db, 'audit_log', 'has_high_sensitivity_fields', 'INTEGER NOT NULL DEFAULT 0');
  addColumnIfMissing(db, 'audit_log', 'error_code', 'TEXT');
  addColumnIfMissing(db, 'audit_log', 'meta_json', 'TEXT');
}

export async function openStoryLockDatabase(dbPath) {
  const sqlite = await import('node:sqlite');
  const db = new sqlite.DatabaseSync(dbPath);
  db.exec(loadSqliteSchema());
  migrateSqliteSchema(db);
  return db;
}
