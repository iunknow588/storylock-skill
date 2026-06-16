import assert from 'node:assert/strict';
import { rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
import { StoryReadAccessSkill, StoryWriteAccessSkill } from '../index.js';
import { MemorySecretStore, createPlatformSecretStore } from '../../shared/secret-store.js';

const report = {
  package: 'storylock-local-story-access-skill',
  status: 'passed',
  checks: [],
};

function pass(id) {
  report.checks.push({ id, status: 'passed' });
}

function tempDbPath() {
  return join(tmpdir(), `storylock_selftest_${randomUUID().replaceAll('-', '')}.db`);
}

function cleanup(path) {
  for (const suffix of ['', '-wal', '-shm']) {
    rmSync(`${path}${suffix}`, { force: true });
  }
}

async function withDb(fn) {
  const dbPath = tempDbPath();
  const secretStore = new MemorySecretStore();
  let context;
  try {
    context = { dbPath, secretStore };
    await fn(context);
  } finally {
    context?.hosts?.forEach((host) => host.close?.());
    cleanup(dbPath);
  }
}

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  read.host.enrollAnswers('id-1', ['correct answer']);
  const result = await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: ' Correct   Answer ' }],
    requestId: 'req-success',
    nonce: 'nonce-success',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.redactionLevel, 'partial');
  assert.equal(result.result.storyObject.content, undefined);
  assert.match(result.result.storyObject.contentSummary, /^\[redacted:/);
  pass('read-success-partial-redaction');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  read.host.enrollAnswers('id-1', ['correct answer']);
  const first = await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-replay',
    nonce: 'nonce-replay-a',
    expiry: Date.now() + 10_000,
  });
  const replay = await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-replay-b',
    nonce: 'nonce-replay-a',
    expiry: Date.now() + 10_000,
  });
  assert.equal(first.status, 'success');
  assert.equal(replay.status, 'error');
  assert.equal(replay.error.code, 'SLG-008');
  pass('replay-rejected');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  read.host.enrollAnswers('id-1', ['correct answer']);
  const first = await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-idempotent',
    nonce: 'nonce-idempotent-a',
    expiry: Date.now() + 10_000,
  });
  const replay = await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-idempotent',
    nonce: 'nonce-idempotent-b',
    expiry: Date.now() + 10_000,
  });
  assert.equal(first.status, 'success');
  assert.deepEqual(replay, first);
  pass('request-id-idempotent-replay');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  read.host.enrollAnswers('id-lock', ['correct']);
  const codes = [];
  for (let i = 0; i < 4; i += 1) {
    const result = await read.run({
      identityId: 'id-lock',
      storyObjectId: 'story-001',
      answers: [{ answer: 'wrong' }],
      requestId: `req-lock-${i}`,
      nonce: `nonce-lock-${i}`,
      expiry: Date.now() + 10_000,
    });
    codes.push(result.error.code);
  }
  assert.deepEqual(codes, ['SLG-003', 'SLG-003', 'SLG-003', 'SLG-004']);
  pass('identity-failure-window-lock');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const write = new StoryWriteAccessSkill({ dbPath, secretStore });
  ctx.hosts = [write.host];
  write.host.enrollAnswers('id-1', ['correct answer']);
  const result = await write.run({
    identityId: 'id-1',
    storyObjectId: 'story-selftest-write',
    content: { title: 'Title', content: 'Body' },
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-write',
    nonce: 'nonce-write',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.result.writeResult.content, undefined);
  pass('write-success-redacted');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const write = new StoryWriteAccessSkill({ dbPath, secretStore });
  ctx.hosts = [write.host];
  write.host.enrollAnswers('id-1', ['correct answer']);
  const result = await write.run({
    identityId: 'id-1',
    storyObjectId: 'story-sensitive-write',
    content: {
      title: 'Sensitive title',
      content: 'Body',
      email: 'reader@example.com',
    },
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-sensitive-write',
    nonce: 'nonce-sensitive-write',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.auditMeta.hasHighSensitivityFields, true);
  assert.equal(result.result.writeResult.title, undefined);
  pass('write-high-sensitivity-redaction');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  read.host.enrollAnswers('id-1', ['correct answer']);
  const result = await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    redactionLevel: 'full',
    requestId: 'req-full-redaction',
    nonce: 'nonce-full-redaction',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.result.storyObject.title, '[redacted]');
  assert.equal(result.result.storyObject.contentSummary, '[redacted]');
  assert.equal(result.auditMeta.redactionLevel, 'full');
  pass('read-full-redaction');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  read.host.enrollAnswers('id-1', ['correct answer']);
  await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-audit',
    nonce: 'nonce-audit',
    expiry: Date.now() + 10_000,
  });
  const db = new DatabaseSync(dbPath);
  const rows = db.prepare('SELECT event_type, result, redaction_level, has_high_sensitivity_fields FROM audit_log ORDER BY audit_id').all();
  db.close();
  assert.deepEqual(rows.map((row) => row.event_type), ['replay_registered', 'challenge_verified', 'story_read', 'story_read_redaction_applied']);
  assert.equal(rows.at(-1).redaction_level, 'partial');
  assert.equal(rows.at(-1).has_high_sensitivity_fields, 0);
  pass('audit-log-written');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const legacy = new DatabaseSync(dbPath);
  legacy.exec(`
    CREATE TABLE request_store (
      request_id TEXT PRIMARY KEY,
      nonce TEXT NOT NULL,
      expiry INTEGER NOT NULL,
      created_at INTEGER NOT NULL
    );
    CREATE TABLE audit_log (
      audit_id INTEGER PRIMARY KEY AUTOINCREMENT,
      event_type TEXT NOT NULL,
      identity_id TEXT,
      story_object_id TEXT,
      request_id TEXT,
      result TEXT,
      created_at INTEGER NOT NULL
    );
  `);
  legacy.close();

  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  const db = new DatabaseSync(dbPath);
  const requestColumns = new Set(db.prepare('PRAGMA table_info(request_store)').all().map((row) => row.name));
  const auditColumns = new Set(db.prepare('PRAGMA table_info(audit_log)').all().map((row) => row.name));
  db.close();
  assert.equal(requestColumns.has('request_hash'), true);
  assert.equal(requestColumns.has('response_json'), true);
  assert.equal(auditColumns.has('redaction_level'), true);
  assert.equal(auditColumns.has('meta_json'), true);
  pass('sqlite-legacy-schema-migrated');
});

assert.throws(
  () => new StoryReadAccessSkill({ dbPath: tempDbPath() }),
  /Persistent SQLite host requires secretStore/,
);
pass('persistent-db-requires-secret-store');

assert.equal(createPlatformSecretStore({ platform: 'win32' }).constructor.name, 'WindowsCredentialSecretStore');
assert.equal(createPlatformSecretStore({ platform: 'linux' }).constructor.name, 'LinuxSecretServiceStore');
pass('platform-secret-store-factory');

console.log('StoryLock local story access selftest passed.');
console.log(JSON.stringify(report));
