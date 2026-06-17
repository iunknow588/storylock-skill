import assert from 'node:assert/strict';
import { rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
import {
  GridChallengeSkill,
  LocalAuthorizationSkill,
  ObjectStrengthPolicySkill,
} from '../index.js';
import { SignatureAuthorizationSkill } from '../../storylock-skill-engine/assets/migrated/skills/authorization-skills.js';
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
  const secretStore = new MemorySecretStore({ developmentMode: true, suppressWarning: true });
  let context;
  try {
    context = { dbPath, secretStore, hosts: [] };
    await fn(context);
  } finally {
    context?.hosts?.forEach((host) => host.close?.());
    cleanup(dbPath);
  }
}

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const policy = new ObjectStrengthPolicySkill({ dbPath, secretStore });
  ctx.hosts = [policy.host];
  const signaturePolicy = await policy.run({
    identityId: 'id-policy',
    objectRef: 'wallet-key-main',
    objectType: 'signature_key',
    requestedAction: 'signature',
    requestId: 'req-policy-signature',
  });
  assert.equal(signaturePolicy.status, 'success');
  assert.equal(signaturePolicy.result.requiredStrength, 'high');
  assert.equal(signaturePolicy.result.gridPolicy.requiredCells, 9);
  const credentialPolicy = await policy.run({
    identityId: 'id-policy',
    objectRef: 'cred-main',
    objectType: 'credential',
    requestedAction: 'password_fill',
    requestId: 'req-policy-credential',
  });
  assert.equal(credentialPolicy.result.requiredStrength, 'medium');
  assert.equal(credentialPolicy.result.gridPolicy.requiredCells, 6);
  pass('object-strength-policy');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-grid', ['correct grid answer']);
  const result = await grid.run({
    identityId: 'id-grid',
    objectRef: 'wallet-key-main',
    requiredStrength: 'high',
    requestId: 'req-grid',
    nonce: 'nonce-grid',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.result.grid.requiredCells, 9);
  assert.equal(result.result.grid.cells.length, 9);
  assert.equal(result.result.grid.cells[0].answer, undefined);
  pass('grid-verification-generated');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-replay', ['correct']);
  const first = await grid.run({
    identityId: 'id-replay',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-idempotent',
    nonce: 'nonce-idempotent-a',
    expiry: Date.now() + 10_000,
  });
  const replay = await grid.run({
    identityId: 'id-replay',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
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
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-replay-conflict', ['correct']);
  const first = await grid.run({
    identityId: 'id-replay-conflict',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-conflict-a',
    nonce: 'nonce-conflict',
    expiry: Date.now() + 10_000,
  });
  const second = await grid.run({
    identityId: 'id-replay-conflict',
    objectRef: 'wallet-key-main',
    requiredStrength: 'high',
    requestId: 'req-conflict-b',
    nonce: 'nonce-conflict',
    expiry: Date.now() + 10_000,
  });
  assert.equal(first.status, 'success');
  assert.equal(second.status, 'error');
  assert.equal(second.error.code, 'SLG-013');
  pass('replay-conflict-error-code');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  const auth = new LocalAuthorizationSkill({ host: grid.host });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-auth', ['correct grid answer']);
  const verification = await grid.run({
    identityId: 'id-auth',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-auth-grid',
    nonce: 'nonce-auth-grid',
    expiry: Date.now() + 10_000,
  });
  const result = await auth.run({
    identityId: 'id-auth',
    objectRef: 'wallet-key-main',
    verificationId: verification.result.verificationId,
    allowedAction: 'signature',
    answers: [{ cellId: 'cell-1', answer: ' Correct Grid Answer ' }],
    requestId: 'req-auth-submit',
  });
  assert.equal(result.status, 'success');
  assert.equal(result.result.approved, true);
  assert.equal(result.result.allowedAction, 'signature');
  assert.match(result.result.authorizationId, /^ses-/);
  const session = grid.host.db.prepare('SELECT read_budget, write_budget FROM session_store WHERE session_id = ?').get(result.result.authorizationId);
  assert.equal(session.read_budget, 1);
  assert.equal(session.write_budget, 0);
  pass('local-authorization-approved');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-lock', ['correct']);
  for (let i = 0; i < 3; i += 1) {
    const challenge = grid.host.createChallenge('id-lock', 'grid_medium');
    const result = grid.host.submitChallengeAnswers('id-lock', challenge.challengeId, [{ answer: 'wrong' }]);
    assert.equal(result.approved, false);
  }
  assert.throws(
    () => grid.host.createChallenge('id-lock', 'grid_medium'),
    /challenge is locked/,
  );
  pass('identity-failure-window-lock');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-unlock', ['correct']);
  for (let i = 0; i < 3; i += 1) {
    const challenge = grid.host.createChallenge('id-unlock', 'grid_medium');
    grid.host.submitChallengeAnswers('id-unlock', challenge.challengeId, [{ answer: 'wrong' }]);
  }
  const db = new DatabaseSync(dbPath);
  db.prepare('UPDATE failure_window SET locked_until = ? WHERE identity_id = ?').run(Date.now() - 1, 'id-unlock');
  db.prepare('UPDATE challenge_state SET lock_until = ? WHERE identity_id = ? AND status = ?').run(Date.now() - 1, 'id-unlock', 'locked');
  db.close();
  const challenge = grid.host.createChallenge('id-unlock', 'grid_medium');
  const result = grid.host.submitChallengeAnswers('id-unlock', challenge.challengeId, [{ answer: 'correct' }]);
  assert.equal(result.approved, true);
  pass('identity-lock-auto-unlock');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  const db = new DatabaseSync(dbPath);
  const now = Date.now();
  db.prepare('INSERT INTO request_store (request_id, nonce, expiry, request_hash, created_at) VALUES (?, ?, ?, ?, ?)').run('req-old-cleanup', 'nonce-old-cleanup', now - 60_000, 'hash', now - 60_000);
  db.prepare('INSERT INTO nonce_store (nonce, request_id, expiry, created_at) VALUES (?, ?, ?, ?)').run('nonce-old-cleanup', 'req-old-cleanup', now - 60_000, now - 60_000);
  db.prepare(
    `INSERT INTO challenge_state
     (challenge_id, identity_id, scope, status, expected_answer_digests_json, failure_count, lock_until, created_at, expires_at)
     VALUES (?, ?, ?, ?, ?, 0, 0, ?, ?)`
  ).run('chl-old-cleanup', 'id-cleanup', 'grid_low', 'challenge_created', '[]', now - 60_000, now - 60_000);
  db.prepare(
    `INSERT INTO session_store
     (session_id, challenge_id, identity_id, scope, resource_scope_json, session_type, read_budget, write_budget, status, issued_at, expires_at)
     VALUES (?, ?, ?, ?, ?, ?, 0, 0, ?, ?, ?)`
  ).run('ses-old-cleanup', 'chl-old-cleanup', 'id-cleanup', 'signature', '[]', 'authorization_only', 'session_active', now - 60_000, now - 60_000);
  db.close();

  const result = grid.host.cleanupExpired(now);
  assert.equal(result.requestRows, 1);
  assert.equal(result.nonceRows, 1);
  assert.equal(result.sessionRows, 1);
  assert.equal(result.challengeRows, 1);
  pass('expired-records-cleaned');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  const db = new DatabaseSync(dbPath);
  const now = Date.now();
  for (let i = 0; i < 3; i += 1) {
    db.prepare('INSERT INTO request_store (request_id, nonce, expiry, request_hash, created_at) VALUES (?, ?, ?, ?, ?)').run(`req-batch-${i}`, `nonce-batch-${i}`, now - 60_000, 'hash', now - 60_000);
    db.prepare('INSERT INTO nonce_store (nonce, request_id, expiry, created_at) VALUES (?, ?, ?, ?)').run(`nonce-batch-${i}`, `req-batch-${i}`, now - 60_000, now - 60_000);
  }
  db.close();
  const result = grid.host.cleanupExpired(now, { batchSize: 2 });
  assert.equal(result.requestRows, 2);
  assert.equal(result.nonceRows, 2);
  const verify = new DatabaseSync(dbPath);
  assert.equal(verify.prepare('SELECT COUNT(*) AS count FROM request_store WHERE request_id LIKE ?').get('req-batch-%').count, 1);
  assert.equal(verify.prepare('SELECT COUNT(*) AS count FROM nonce_store WHERE nonce LIKE ?').get('nonce-batch-%').count, 1);
  verify.close();
  pass('expired-cleanup-batch-limited');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-sign-audit', ['correct grid answer']);
  const challenges = new Map();
  const host = {
    createChallenge(identityId, scope) {
      const challenge = grid.host.createChallenge(identityId, scope);
      challenges.set(challenge.challengeId, challenge);
      return challenge;
    },
    submitChallengeAnswers(identityId, challengeId, answers) {
      const verification = grid.host.submitChallengeAnswers(identityId, challengeId, answers);
      if (!verification.approved) {
        return verification;
      }
      const challenge = challenges.get(challengeId) ?? verification.challenge;
      return grid.host.issueSession(identityId, challenge, challenge.scope, ['wallet/main/private_key'], {
        sessionType: 'authorization_only',
        readBudget: 1,
        writeBudget: 0,
      });
    },
    recordAudit: grid.host.recordAudit.bind(grid.host),
    readSecretObject(_identityId, _sessionId, secretObjectId) {
      return new TextEncoder().encode(`secret:${secretObjectId}`);
    },
  };
  const skill = new SignatureAuthorizationSkill({
    host,
    signer({ keyId, algorithm, payload, secretReference }) {
      return {
        keyId,
        algorithm,
        payload: Array.from(payload),
        secretReference,
        signature: 'selftest-signature',
      };
    },
  });
  const result = await skill.run({
    identityId: 'id-sign-audit',
    keyId: 'wallet-key-main',
    algorithm: 'ed25519',
    payload: 'sign me',
    secretObjectId: 'wallet/main/private_key',
    answers: [{ answer: 'correct grid answer' }],
  });
  const db = new DatabaseSync(dbPath);
  const row = db.prepare(
    `SELECT event_type, identity_id, story_object_id, result, meta_json
     FROM audit_log
     WHERE event_type = ?
     ORDER BY audit_id DESC
     LIMIT 1`
  ).get('signature_authorized');
  db.close();
  const meta = JSON.parse(row.meta_json);
  assert.equal(row.event_type, 'signature_authorized');
  assert.equal(row.identity_id, 'id-sign-audit');
  assert.equal(row.story_object_id, 'wallet/main/private_key');
  assert.equal(row.result, 'success');
  assert.equal(meta.authorizationId, result.authorizationId);
  assert.equal(meta.signatureHash, result.signatureHash);
  pass('signature-audit-persisted');
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

  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  const db = new DatabaseSync(dbPath);
  const requestColumns = new Set(db.prepare('PRAGMA table_info(request_store)').all().map((row) => row.name));
  const auditColumns = new Set(db.prepare('PRAGMA table_info(audit_log)').all().map((row) => row.name));
  const storyObjectTable = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?").get('protected_story_objects');
  db.close();
  assert.equal(requestColumns.has('request_hash'), true);
  assert.equal(requestColumns.has('response_json'), true);
  assert.equal(auditColumns.has('meta_json'), true);
  assert.equal(storyObjectTable, undefined);
  pass('sqlite-legacy-schema-migrated');
});

assert.throws(
  () => new GridChallengeSkill({ dbPath: tempDbPath() }),
  /Persistent SQLite host requires secretStore/,
);
pass('persistent-db-requires-secret-store');

assert.throws(
  () => new GridChallengeSkill({ dbPath: tempDbPath(), secretStore: new MemorySecretStore() }),
  /must not use MemorySecretStore/,
);
pass('persistent-db-rejects-production-memory-secret-store');

assert.equal(createPlatformSecretStore({ platform: 'win32' }).constructor.name, 'WindowsCredentialSecretStore');
assert.equal(createPlatformSecretStore({ platform: 'linux' }).constructor.name, 'LinuxSecretServiceStore');
pass('platform-secret-store-factory');

console.log('StoryLock local story access selftest passed.');
console.log(JSON.stringify(report));
