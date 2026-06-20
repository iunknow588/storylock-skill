import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { DatabaseSync } from 'node:sqlite';
import {
  createAccessHost,
  GridChallengeSkill,
  LocalAuthorizationSkill,
  LocalRevocationSkill,
  ObjectStrengthPolicySkill,
  startAccessHostCleanup,
} from '../index.js';
import { SignatureAuthorizationSkill } from '../../../engine/assets/migrated/skills/authorization-skills.js';
import { MemorySecretStore, createPlatformSecretStore } from '../../../shared/secret-store.js';

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

function cleanupDir(path) {
  rmSync(path, { force: true, recursive: true });
}

function sampleQuestions(count = 9) {
  return Array.from({ length: count }, (_, index) => ({
    questionId: `q-${index + 1}`,
    versionTag: 'v1',
    promptRef: `prompt-${index + 1}`,
    promptText: `Question ${index + 1}`,
    options: [`answer-${index + 1}`, `alt-${index + 1}`],
    answer: `answer-${index + 1}`,
    status: 'active',
  }));
}

function answersFor(count) {
  return Array.from({ length: count }, (_, index) => ({
    cellId: `cell-${index + 1}`,
    answer: ` answer-${index + 1} `,
  }));
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
  grid.host.enrollQuestionSet('id-grid', sampleQuestions(9));
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
  assert.equal(result.result.grid.cells[0].questionId, 'q-1');
  assert.equal(result.result.grid.cells[0].versionTag, 'v1');
  assert.equal(result.result.grid.cells[0].optionDigest.length, 64);
  pass('grid-verification-generated');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-versioned-grid', sampleQuestions(9), {
    questionSetVersion: 'set-v1',
  });
  grid.host.enrollQuestionSet('id-versioned-grid', sampleQuestions(9).map((question, index) => ({
    ...question,
    questionId: `v2-q-${index + 1}`,
    versionTag: 'v2',
    promptRef: `v2-prompt-${index + 1}`,
    answer: `v2-answer-${index + 1}`,
  })), {
    questionSetVersion: 'set-v2',
  });

  const rows = grid.host.db.prepare(
    `SELECT question_set_version, status, COUNT(*) AS count
     FROM question_set_item
     WHERE identity_id = ?
     GROUP BY question_set_version, status
     ORDER BY question_set_version, status`
  ).all('id-versioned-grid').map((row) => ({ ...row }));
  assert.deepEqual(rows, [
    { question_set_version: 'set-v1', status: 'deprecated', count: 9 },
    { question_set_version: 'set-v2', status: 'active', count: 9 },
  ]);

  const result = await grid.run({
    identityId: 'id-versioned-grid',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-versioned-grid',
    nonce: 'nonce-versioned-grid',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.result.grid.questionSetVersion, 'set-v2');
  assert.deepEqual(result.result.grid.questionSetVersions, ['set-v2']);
  assert.equal(result.result.grid.cells[0].questionId, 'v2-q-1');

  const deprecated = await grid.run({
    identityId: 'id-versioned-grid',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    questionSetVersion: 'set-v1',
    requestId: 'req-deprecated-grid',
    nonce: 'nonce-deprecated-grid',
    expiry: Date.now() + 10_000,
  });
  assert.equal(deprecated.status, 'error');
  assert.equal(deprecated.error.code, 'SLG-010');
  pass('question-set-version-rotation');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-pending-grid', sampleQuestions(9), {
    questionSetVersion: 'active-set-v1',
  });
  grid.host.enrollQuestionSet('id-pending-grid', sampleQuestions(9).map((question, index) => ({
    ...question,
    questionId: `pending-q-${index + 1}`,
    versionTag: 'pending-v2',
    promptRef: `pending-prompt-${index + 1}`,
    answer: `pending-answer-${index + 1}`,
    status: 'pending',
  })), {
    questionSetVersion: 'pending-set-v2',
    status: 'pending',
    replacePreviousActive: false,
  });

  const automatic = await grid.run({
    identityId: 'id-pending-grid',
    objectRef: 'wallet-key-main',
    requiredStrength: 'high',
    requestId: 'req-pending-automatic',
    nonce: 'nonce-pending-automatic',
    expiry: Date.now() + 10_000,
  });
  assert.equal(automatic.status, 'success');
  assert.equal(automatic.result.grid.questionSetVersion, 'active-set-v1');
  assert.equal(automatic.result.grid.cells.some((cell) => cell.questionId.startsWith('pending-')), false);

  const explicitPending = await grid.run({
    identityId: 'id-pending-grid',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    questionSetVersion: 'pending-set-v2',
    requestId: 'req-pending-explicit',
    nonce: 'nonce-pending-explicit',
    expiry: Date.now() + 10_000,
  });
  assert.equal(explicitPending.status, 'error');
  assert.equal(explicitPending.error.code, 'SLG-010');
  pass('pending-question-set-is-not-selectable');
});

{
  const dir = mkdtempSync(join(tmpdir(), 'storylock_import_selftest_'));
  const dbPath = join(dir, 'question-set.db');
  const inputPath = join(dir, 'question-set.json');
  try {
    writeFileSync(inputPath, JSON.stringify({
      identityId: 'id-imported-grid',
      questionSetVersion: 'import-set-v1',
      normalizationVersion: 'nfkc-lower-v1',
      status: 'active',
      questions: sampleQuestions(9),
    }, null, 2));
    const dryRunOutput = execFileSync(process.execPath, [
      fileURLToPath(new URL('./import-question-set.mjs', import.meta.url)),
      '--input',
      inputPath,
      '--db',
      dbPath,
      '--development-memory-secret-store',
      '--dry-run',
      '--require-min-active',
      '9',
    ], {
      encoding: 'utf8',
      windowsHide: true,
    });
    const dryRun = JSON.parse(dryRunOutput);
    assert.equal(dryRun.status, 'validated');
    assert.equal(dryRun.dryRun, true);
    assert.equal(dryRun.questionCount, 9);
    assert.equal(dryRun.activeQuestionCount, 9);
    assert.equal(dryRun.requiredActiveQuestions, 9);

    assert.throws(
      () => execFileSync(process.execPath, [
        fileURLToPath(new URL('./import-question-set.mjs', import.meta.url)),
        '--input',
        inputPath,
        '--db',
        dbPath,
        '--development-memory-secret-store',
      ], {
        encoding: 'utf8',
        windowsHide: true,
        stdio: 'pipe',
      }),
      /Persistent question-set import cannot use --development-memory-secret-store/,
    );
    assert.throws(
      () => execFileSync(process.execPath, [
        fileURLToPath(new URL('./import-question-set.mjs', import.meta.url)),
        '--input',
        inputPath,
        '--db',
        dbPath,
        '--use-platform-secret-store',
        '--dry-run',
      ], {
        encoding: 'utf8',
        windowsHide: true,
        stdio: 'pipe',
      }),
      /at least 24 active questions/,
    );
    pass('question-set-import-command');
  } finally {
    cleanup(dbPath);
    cleanupDir(dir);
  }
}

{
  const dir = mkdtempSync(join(tmpdir(), 'storylock_template_selftest_'));
  const outputPath = join(dir, 'generated-question-set.json');
  try {
    const templateOutput = execFileSync(process.execPath, [
      fileURLToPath(new URL('./generate-question-set-template.mjs', import.meta.url)),
      '--output',
      outputPath,
      '--identity-id',
      'id-template',
      '--question-set-version',
      'template-v1',
    ], {
      encoding: 'utf8',
      windowsHide: true,
    });
    const templateResult = JSON.parse(templateOutput);
    const generated = JSON.parse(readFileSync(outputPath, 'utf8'));
    assert.equal(templateResult.status, 'success');
    assert.equal(generated.template, true);
    assert.equal(generated.questions.length, 24);
    assert.equal(generated.questionSetVersion, 'template-v1');
    assert.equal(generated.questions[0].answer, 'replace-answer-1');
    pass('question-set-template-generator');
  } finally {
    cleanupDir(dir);
  }
}

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-replay', sampleQuestions(9));
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
  grid.host.enrollQuestionSet('id-replay-conflict', sampleQuestions(9));
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
  grid.host.enrollQuestionSet('id-auth', sampleQuestions(9));
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
    answers: answersFor(6),
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
  const auth = new LocalAuthorizationSkill({ host: grid.host });
  const revoke = new LocalRevocationSkill({ host: grid.host });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-revoke-challenge', sampleQuestions(9));
  const verification = await grid.run({
    identityId: 'id-revoke-challenge',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-revoke-challenge-grid',
    nonce: 'nonce-revoke-challenge-grid',
    expiry: Date.now() + 10_000,
  });
  const revoked = await revoke.run({
    identityId: 'id-revoke-challenge',
    verificationId: verification.result.verificationId,
    reason: 'selftest',
    requestId: 'req-revoke-challenge',
  });
  assert.equal(revoked.status, 'success');
  assert.equal(revoked.result.targetType, 'challenge');
  assert.equal(revoked.result.status, 'revoked');

  const afterRevoke = await auth.run({
    identityId: 'id-revoke-challenge',
    objectRef: 'wallet-key-main',
    verificationId: verification.result.verificationId,
    allowedAction: 'signature',
    answers: answersFor(6),
    requestId: 'req-revoke-challenge-submit',
  });
  assert.equal(afterRevoke.status, 'error');
  assert.equal(afterRevoke.error.code, 'SLG-003');

  const audit = grid.host.db.prepare(
    `SELECT event_type, identity_id, result, meta_json
     FROM audit_log
     WHERE event_type = ?
     ORDER BY audit_id DESC
     LIMIT 1`
  ).get('challenge_revoked');
  assert.equal(audit.identity_id, 'id-revoke-challenge');
  assert.equal(audit.result, 'success');
  assert.equal(JSON.parse(audit.meta_json).reason, 'selftest');
  pass('challenge-revocation');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  const auth = new LocalAuthorizationSkill({ host: grid.host });
  const revoke = new LocalRevocationSkill({ host: grid.host });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-revoke-session', sampleQuestions(9));
  const verification = await grid.run({
    identityId: 'id-revoke-session',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-revoke-session-grid',
    nonce: 'nonce-revoke-session-grid',
    expiry: Date.now() + 10_000,
  });
  const authorization = await auth.run({
    identityId: 'id-revoke-session',
    objectRef: 'wallet-key-main',
    verificationId: verification.result.verificationId,
    allowedAction: 'signature',
    answers: answersFor(6),
    requestId: 'req-revoke-session-auth',
  });
  const revoked = await revoke.run({
    identityId: 'id-revoke-session',
    authorizationId: authorization.result.authorizationId,
    reason: 'selftest',
    requestId: 'req-revoke-session',
  });
  assert.equal(revoked.status, 'success');
  assert.equal(revoked.result.targetType, 'session');
  assert.equal(revoked.result.status, 'session_revoked');
  const session = grid.host.db.prepare('SELECT status FROM session_store WHERE session_id = ?').get(authorization.result.authorizationId);
  assert.equal(session.status, 'session_revoked');
  const audit = grid.host.db.prepare(
    `SELECT event_type, identity_id, result, meta_json
     FROM audit_log
     WHERE event_type = ?
     ORDER BY audit_id DESC
     LIMIT 1`
  ).get('session_revoked');
  assert.equal(audit.identity_id, 'id-revoke-session');
  assert.equal(audit.result, 'success');
  assert.equal(JSON.parse(audit.meta_json).reason, 'selftest');
  pass('session-revocation');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  const auth = new LocalAuthorizationSkill({ host: grid.host });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-auth-wrong', sampleQuestions(9));
  const verification = await grid.run({
    identityId: 'id-auth-wrong',
    objectRef: 'wallet-key-main',
    requiredStrength: 'medium',
    requestId: 'req-auth-wrong-grid',
    nonce: 'nonce-auth-wrong-grid',
    expiry: Date.now() + 10_000,
  });
  const result = await auth.run({
    identityId: 'id-auth-wrong',
    objectRef: 'wallet-key-main',
    verificationId: verification.result.verificationId,
    allowedAction: 'signature',
    answers: [
      { cellId: 'cell-1', answer: 'answer-2' },
      { cellId: 'cell-2', answer: 'answer-1' },
      ...answersFor(4).slice(2),
    ],
    requestId: 'req-auth-wrong-submit',
  });
  assert.equal(result.status, 'error');
  assert.equal(result.error.code, 'SLG-003');
  pass('local-authorization-rejects-wrong-cell-binding');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  const auth = new LocalAuthorizationSkill({ host: grid.host });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-auth-partial', sampleQuestions(9));
  const verification = await grid.run({
    identityId: 'id-auth-partial',
    objectRef: 'wallet-key-main',
    requiredStrength: 'high',
    requestId: 'req-auth-partial-grid',
    nonce: 'nonce-auth-partial-grid',
    expiry: Date.now() + 10_000,
  });
  const result = await auth.run({
    identityId: 'id-auth-partial',
    objectRef: 'wallet-key-main',
    verificationId: verification.result.verificationId,
    allowedAction: 'signature',
    answers: answersFor(8),
    requestId: 'req-auth-partial-submit',
  });
  assert.equal(result.status, 'error');
  assert.equal(result.error.code, 'SLG-003');
  pass('high-strength-requires-nine-cells');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-lock', sampleQuestions(9));
  for (let i = 0; i < 3; i += 1) {
    const challenge = grid.host.createChallenge('id-lock', 'grid_medium', { requiredCells: 6 });
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
  grid.host.enrollQuestionSet('id-unlock', sampleQuestions(9));
  for (let i = 0; i < 3; i += 1) {
    const challenge = grid.host.createChallenge('id-unlock', 'grid_medium', { requiredCells: 6 });
    grid.host.submitChallengeAnswers('id-unlock', challenge.challengeId, [{ answer: 'wrong' }]);
  }
  const db = new DatabaseSync(dbPath);
  db.prepare('UPDATE failure_window SET locked_until = ? WHERE identity_id = ?').run(Date.now() - 1, 'id-unlock');
  db.prepare('UPDATE challenge_state SET lock_until = ? WHERE identity_id = ? AND status = ?').run(Date.now() - 1, 'id-unlock', 'locked');
  db.close();
  const challenge = grid.host.createChallenge('id-unlock', 'grid_medium', { requiredCells: 6 });
  const result = grid.host.submitChallengeAnswers('id-unlock', challenge.challengeId, answersFor(6));
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
     (challenge_id, identity_id, scope, status, expected_answer_digests_json, challenge_manifest_json, required_threshold, normalization_version, question_set_version, failure_count, lock_until, created_at, expires_at)
     VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, 0, 0, ?, ?)`
  ).run('chl-old-cleanup', 'id-cleanup', 'grid_low', 'challenge_created', '[]', '{}', 'nfkc-lower-v1', 'legacy', now - 60_000, now - 60_000);
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

{
  const calls = [];
  let scheduledCallback = null;
  let clearedTimer = null;
  const cleanup = startAccessHostCleanup({
    cleanupExpired(now, options) {
      calls.push({ now, options });
      return { requestRows: 0, nonceRows: 0, sessionRows: 0, challengeRows: 0 };
    },
  }, {
    intervalMs: 5000,
    batchSize: 7,
    now: () => 12345,
    setIntervalFn(callback, intervalMs) {
      scheduledCallback = callback;
      return { intervalMs, unref() {} };
    },
    clearIntervalFn(timer) {
      clearedTimer = timer;
    },
  });
  assert.equal(typeof scheduledCallback, 'function');
  assert.equal(cleanup.runNow().requestRows, 0);
  scheduledCallback();
  assert.deepEqual(calls, [
    { now: 12345, options: { batchSize: 7 } },
    { now: 12345, options: { batchSize: 7 } },
  ]);
  cleanup.stop();
  assert.equal(clearedTimer.intervalMs, 5000);
  pass('scheduled-cleanup-controller');
}

assert.throws(
  () => startAccessHostCleanup({ cleanupExpired() {} }, { intervalMs: 0 }),
  /cleanup intervalMs must be a positive number/,
);
pass('scheduled-cleanup-validates-interval');

{
  let callback = null;
  const host = createAccessHost({
    developmentMode: true,
    cleanupIntervalMs: 2500,
    cleanupBatchSize: 5,
    databaseFactory(path) {
      return new DatabaseSync(path);
    },
    onCleanupError(error) {
      throw error;
    },
  });
  try {
    host.cleanupController?.stop();
  } finally {
    host.close?.();
  }

  const wiredHost = createAccessHost({
    developmentMode: true,
    cleanupIntervalMs: 2500,
    cleanupBatchSize: 5,
    cleanupSetIntervalFn(callbackFn) {
      callback = callbackFn;
      return { unref() {} };
    },
    cleanupClearIntervalFn() {},
  });
  try {
    assert.equal(typeof callback, 'function');
    assert.equal(typeof wiredHost.cleanupController?.runNow, 'function');
  } finally {
    wiredHost.cleanupController?.stop();
    wiredHost.close?.();
  }
  pass('create-access-host-auto-cleanup');
}

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-production-no-fallback', ['alpha', 'bravo', 'charlie']);
  const result = await grid.run({
    identityId: 'id-production-no-fallback',
    objectRef: 'wallet-key-main',
    requiredStrength: 'low',
    requestId: 'req-production-no-fallback',
    nonce: 'nonce-production-no-fallback',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'error');
  assert.equal(result.error.code, 'SLG-010');
  assert.equal(result.error.type, 'question_set_unavailable');
  assert.match(result.error.suggestedAction, /Enroll a production question set/);
  pass('production-rejects-legacy-fallback');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore, allowLegacyFallback: true });
  ctx.hosts = [grid.host];
  grid.host.enrollAnswers('id-demo-legacy-fallback', ['alpha', 'bravo', 'charlie']);
  const result = await grid.run({
    identityId: 'id-demo-legacy-fallback',
    objectRef: 'wallet-key-main',
    requiredStrength: 'low',
    requestId: 'req-demo-legacy-fallback',
    nonce: 'nonce-demo-legacy-fallback',
    expiry: Date.now() + 10_000,
  });
  assert.equal(result.status, 'success');
  assert.equal(result.result.grid.questionSetVersion, 'legacy');
  assert.equal(result.result.grid.cells[0].legacy, undefined);
  const challenge = grid.host.db.prepare(
    'SELECT expected_answer_digests_json FROM challenge_state WHERE challenge_id = ?'
  ).get(result.result.verificationId);
  assert.equal(JSON.parse(challenge.expected_answer_digests_json)[0].legacy, true);
  pass('demo-can-explicitly-enable-legacy-fallback');
});

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const grid = new GridChallengeSkill({ dbPath, secretStore });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-sign-audit', sampleQuestions(9));
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
    answers: answersFor(9),
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

await withDb(async (ctx) => {
  const { dbPath, secretStore } = ctx;
  const paths = [];
  const grid = new GridChallengeSkill({
    dbPath,
    secretStore,
    databaseFactory(path) {
      paths.push(path);
      return new DatabaseSync(path);
    },
  });
  ctx.hosts = [grid.host];
  grid.host.enrollQuestionSet('id-database-factory', sampleQuestions(9));
  const result = await grid.run({
    identityId: 'id-database-factory',
    objectRef: 'wallet-key-main',
    requiredStrength: 'low',
    requestId: 'req-database-factory',
    nonce: 'nonce-database-factory',
    expiry: Date.now() + 10_000,
  });
  assert.equal(paths.length, 1);
  assert.equal(paths[0], dbPath);
  assert.equal(result.status, 'success');
  assert.equal(result.result.grid.cells.length, 3);
  pass('database-factory-injection');
});

assert.throws(
  () => new GridChallengeSkill({
    secretStore: new MemorySecretStore({ developmentMode: true, suppressWarning: true }),
    databaseFactory() {
      return {};
    },
  }),
  /databaseFactory must return a DatabaseSync-compatible object/,
);
pass('database-factory-contract-enforced');

assert.throws(
  () => new GridChallengeSkill({ dbPath: tempDbPath() }),
  /createAccessHost requires secretStore, usePlatformSecretStore=true, or developmentMode=true/,
);
pass('persistent-db-requires-secret-store');

assert.throws(
  () => new GridChallengeSkill({ dbPath: tempDbPath(), secretStore: new MemorySecretStore() }),
  /MemorySecretStore requires developmentMode=true/,
);
pass('memory-secret-store-requires-development-mode');

{
  const skill = new GridChallengeSkill({ developmentMode: true });
  try {
    assert.equal(skill.host.secretStore.kind, 'memory');
    assert.equal(skill.host.secretStore.developmentMode, true);
  } finally {
    skill.host.close?.();
  }
}
pass('explicit-development-mode-allows-memory-fallback');

assert.equal(createPlatformSecretStore({ platform: 'win32' }).constructor.name, 'WindowsCredentialSecretStore');
assert.equal(createPlatformSecretStore({ platform: 'linux' }).constructor.name, 'LinuxSecretServiceStore');
assert.equal(createPlatformSecretStore({ platform: 'darwin' }).constructor.name, 'MacOSKeychainSecretStore');
assert.equal(
  createPlatformSecretStore({ platform: 'darwin', allowMemoryFallback: true }).constructor.name,
  'MacOSKeychainSecretStore',
);
pass('platform-secret-store-factory');

{
  const dbPath = tempDbPath();
  let calls = 0;
  const secretStore = {
    kind: 'platform_stub',
    checkAvailable() {
      calls += 1;
      return true;
    },
    getSecret() {
      return null;
    },
    setSecret() {},
    deleteSecret() {},
  };
  const host = createAccessHost({
    dbPath,
    secretStore,
  });
  try {
    assert.equal(calls, 1);
  } finally {
    host.close?.();
    cleanup(dbPath);
  }
}
pass('platform-secret-store-startup-check');

{
  const dbPath = tempDbPath();
  assert.throws(
    () => createAccessHost({
      dbPath,
      secretStore: {
        kind: 'platform_stub',
        checkAvailable() {
          throw new Error('CredentialManager PowerShell module is required');
        },
        getSecret() {
          return null;
        },
        setSecret() {},
        deleteSecret() {},
      },
    }),
    /Platform SecretStore is unavailable: CredentialManager PowerShell module is required/,
  );
  cleanup(dbPath);
}
pass('platform-secret-store-startup-check-fails-fast');

console.log('StoryLock local story access selftest passed.');
console.log(JSON.stringify(report));
