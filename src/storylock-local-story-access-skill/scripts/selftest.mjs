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
    requestId: 'req-replay',
    nonce: 'nonce-replay-b',
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
  const read = new StoryReadAccessSkill({ dbPath, secretStore });
  ctx.hosts = [read.host];
  await read.run({
    identityId: 'id-1',
    storyObjectId: 'story-001',
    answers: [{ answer: 'correct answer' }],
    requestId: 'req-audit',
    nonce: 'nonce-audit',
    expiry: Date.now() + 10_000,
  });
  const db = new DatabaseSync(dbPath);
  const rows = db.prepare('SELECT event_type, result FROM audit_log ORDER BY audit_id').all();
  db.close();
  assert.deepEqual(rows.map((row) => row.event_type), ['replay_registered', 'challenge_verified', 'story_read']);
  pass('audit-log-written');
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
