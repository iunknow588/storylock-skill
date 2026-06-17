import { createHash, randomBytes } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
import { deriveHkdfSha256, hmacSha256Hex } from '../shared/crypto.js';
import { MemorySecretStore, createPlatformSecretStore } from '../shared/secret-store.js';
import { loadSqliteSchema, migrateSqliteSchema } from '../shared/sqlite.js';

const DEFAULT_DB_PATH = ':memory:';
const DEFAULT_SECRET_STORE = new MemorySecretStore({ developmentMode: true, suppressWarning: true });

const REPLAY_DRIFT_MS = 30_000;
const REPLAY_WINDOW_MS = 24 * 60 * 60 * 1000;
const FAILURE_WINDOW_MS = 24 * 60 * 60 * 1000;
const FAILURE_LOCK_MS = 15 * 60 * 1000;
const MAX_FAILURES_PER_WINDOW = 3;
const DEFAULT_CLEANUP_BATCH_SIZE = 1000;

function nowMs() {
  return Date.now();
}

function makeId(prefix) {
  return `${prefix}-${randomBytes(16).toString('hex')}`;
}

function normalizeAnswerValue(value) {
  const raw = typeof value === 'string' ? value : value?.answer;
  if (typeof raw !== 'string' || raw.trim().length === 0) {
    return '';
  }
  return raw.normalize('NFKC').trim().replace(/\s+/g, ' ').toLowerCase();
}

function serializeJson(value) {
  return JSON.stringify(value);
}

function stableStringify(value) {
  if (Array.isArray(value)) {
    return `[${value.map(stableStringify).join(',')}]`;
  }
  if (value && typeof value === 'object') {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

function hashPayload(value) {
  return createHash('sha256').update(stableStringify(value)).digest('hex');
}

function deriveRootKey(masterSalt) {
  return deriveHkdfSha256(masterSalt, {
    salt: Buffer.from('storylock:v1:root:salt'),
    info: Buffer.from('storylock:v1:root'),
  });
}

function deriveWorkKey(masterSalt, purpose) {
  return deriveHkdfSha256(deriveRootKey(masterSalt), {
    salt: Buffer.from(`storylock:v1:work:${purpose}:salt`),
    info: Buffer.from(`storylock:v1:work:${purpose}`),
  });
}

function deriveIdentityAnswerKey(masterSalt, identityId) {
  return deriveHkdfSha256(deriveWorkKey(masterSalt, 'identity-answer-digest'), {
    salt: Buffer.from(`storylock:v1:identity:${identityId}:salt`),
    info: Buffer.from(`storylock:v1:identity:${identityId}`),
  });
}

function parseJson(value, fallback) {
  if (typeof value !== 'string' || value.length === 0) {
    return fallback;
  }
  try {
    return JSON.parse(value);
  } catch {
    return fallback;
  }
}

class SqliteStore {
  constructor(dbPath = DEFAULT_DB_PATH, secretStore = new MemorySecretStore(), { persistent = false } = {}) {
    this.db = new DatabaseSync(dbPath);
    this.db.exec(loadSqliteSchema());
    migrateSqliteSchema(this.db);
    this.secretStore = secretStore;
    this.persistent = persistent;
    this.ready = false;
  }

  ensureSeeded() {
    if (this.ready) {
      return;
    }
    this.ready = true;
    this.db.exec('BEGIN IMMEDIATE;');
    try {
      this.ensureMasterSalt();
      this.db.exec('COMMIT;');
    } catch (error) {
      this.db.exec('ROLLBACK;');
      throw error;
    }
  }

  ensureMasterSalt() {
    const existing = this.secretStore.getSecret('storylock/masterSalt');
    const value = existing || randomBytes(32);
    if (!existing && this.secretStore.setSecret) {
      this.secretStore.setSecret('storylock/masterSalt', value);
    }
    return value;
  }

  get masterSalt() {
    return this.ensureMasterSalt();
  }

  close() {
    this.db.close();
  }

  cleanupExpired(now = nowMs(), { batchSize = DEFAULT_CLEANUP_BATCH_SIZE } = {}) {
    const replayCutoff = now - REPLAY_WINDOW_MS;
    const limit = Math.max(1, Math.min(Number(batchSize) || DEFAULT_CLEANUP_BATCH_SIZE, DEFAULT_CLEANUP_BATCH_SIZE));
    this.db.exec('BEGIN IMMEDIATE;');
    try {
      const requestResult = this.db.prepare(
        `DELETE FROM request_store
         WHERE rowid IN (
           SELECT rowid FROM request_store
           WHERE created_at < ? OR expiry <= ?
           LIMIT ?
         )`
      ).run(replayCutoff, now - REPLAY_DRIFT_MS, limit);
      const nonceResult = this.db.prepare(
        `DELETE FROM nonce_store
         WHERE rowid IN (
           SELECT rowid FROM nonce_store
           WHERE created_at < ? OR expiry <= ?
           LIMIT ?
         )`
      ).run(replayCutoff, now - REPLAY_DRIFT_MS, limit);
      const sessionResult = this.db.prepare(
        `UPDATE session_store
         SET status = ?
         WHERE session_id IN (
           SELECT session_id FROM session_store
           WHERE status = ? AND expires_at <= ?
           LIMIT ?
         )`
      ).run('session_expired', 'session_active', now, limit);
      const challengeResult = this.db.prepare(
        `UPDATE challenge_state
         SET status = ?
         WHERE challenge_id IN (
           SELECT challenge_id FROM challenge_state
           WHERE status = ? AND expires_at <= ?
           LIMIT ?
         )`
      ).run('expired', 'challenge_created', now - REPLAY_DRIFT_MS, limit);
      const result = {
        requestRows: requestResult.changes,
        nonceRows: nonceResult.changes,
        sessionRows: sessionResult.changes,
        challengeRows: challengeResult.changes,
      };
      this.db.exec('COMMIT;');
      return result;
    } catch (error) {
      this.db.exec('ROLLBACK;');
      throw error;
    }
  }

  recordAudit(eventType, {
    identityId = null,
    storyObjectId = null,
    requestId = null,
    result = null,
    redactionLevel = null,
    hasHighSensitivityFields = false,
    errorCode = null,
    meta = null,
  } = {}) {
    this.db.prepare(
      `INSERT INTO audit_log
       (event_type, identity_id, story_object_id, request_id, result, redaction_level, has_high_sensitivity_fields, error_code, meta_json, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
    ).run(
      eventType,
      identityId,
      storyObjectId,
      requestId,
      result,
      redactionLevel,
      hasHighSensitivityFields ? 1 : 0,
      errorCode,
      meta ? serializeJson(meta) : null,
      nowMs(),
    );
  }

  ensureReplaySafe(requestId, nonce, expiry, payload = {}) {
    const requestHash = hashPayload(payload);
    const cutoff = nowMs() - REPLAY_WINDOW_MS;
    this.db.exec('BEGIN IMMEDIATE;');
    try {
      this.db.prepare('DELETE FROM request_store WHERE created_at < ? OR expiry <= ?').run(cutoff, nowMs() - REPLAY_DRIFT_MS);
      this.db.prepare('DELETE FROM nonce_store WHERE created_at < ? OR expiry <= ?').run(cutoff, nowMs() - REPLAY_DRIFT_MS);
      const existingRequest = this.db.prepare('SELECT request_id, request_hash, response_json FROM request_store WHERE request_id = ?').get(requestId);
      const existingNonce = this.db.prepare('SELECT nonce FROM nonce_store WHERE nonce = ?').get(nonce);
      if (existingRequest?.response_json && existingRequest.request_hash === requestHash) {
        this.db.exec('COMMIT;');
        return { replayed: true, response: parseJson(existingRequest.response_json, null) };
      }
      if (existingRequest || existingNonce) {
        const err = new Error('requestId or nonce was already used');
        err.code = 'SLG-013';
        err.type = 'replay_detected';
        err.retryable = false;
        this.recordAudit('replay_rejected', { requestId, result: 'error' });
        throw err;
      }
      this.db.prepare('INSERT INTO request_store (request_id, nonce, expiry, request_hash, created_at) VALUES (?, ?, ?, ?, ?)').run(requestId, nonce, expiry, requestHash, nowMs());
      this.db.prepare('INSERT INTO nonce_store (nonce, request_id, expiry, created_at) VALUES (?, ?, ?, ?)').run(nonce, requestId, expiry, nowMs());
      this.recordAudit('replay_registered', { requestId, result: 'success' });
      this.db.exec('COMMIT;');
      return { replayed: false };
    } catch (error) {
      this.db.exec('ROLLBACK;');
      throw error;
    }
  }

  storeRequestResponse(requestId, response) {
    this.db.prepare(
      'UPDATE request_store SET response_json = ?, response_status = ?, response_created_at = ? WHERE request_id = ?'
    ).run(serializeJson(response), response?.status ?? null, nowMs(), requestId);
  }

  enrollAnswers(identityId, answers) {
    const identityKey = deriveIdentityAnswerKey(this.masterSalt, identityId);
    const digests = answers.map((answer) => hmacSha256Hex(identityKey, normalizeAnswerValue(answer))).filter(Boolean);
    this.db.exec('BEGIN IMMEDIATE;');
    try {
      this.db.prepare('DELETE FROM answer_digest_set WHERE identity_id = ?').run(identityId);
      const insert = this.db.prepare('INSERT OR IGNORE INTO answer_digest_set (identity_id, answer_digest, created_at) VALUES (?, ?, ?)');
      for (const digest of digests) {
        insert.run(identityId, digest, nowMs());
      }
      this.db.exec('COMMIT;');
      return digests;
    } catch (error) {
      this.db.exec('ROLLBACK;');
      throw error;
    }
  }

  getFailureWindow(identityId) {
    const row = this.db.prepare('SELECT identity_id, window_start, failure_count, locked_until FROM failure_window WHERE identity_id = ?').get(identityId);
    const now = nowMs();
    if (!row || row.window_start + FAILURE_WINDOW_MS <= now) {
      this.db.prepare(
        'INSERT INTO failure_window (identity_id, window_start, failure_count, locked_until) VALUES (?, ?, 0, 0) ON CONFLICT(identity_id) DO UPDATE SET window_start = excluded.window_start, failure_count = 0, locked_until = 0'
      ).run(identityId, now);
      this.db.prepare(
        `UPDATE challenge_state
         SET status = ?, failure_count = 0, lock_until = 0
         WHERE identity_id = ? AND status = ?`
      ).run('idle', identityId, 'locked');
      return { windowStart: now, failureCount: 0, lockedUntil: 0 };
    }
    if (row.locked_until > 0 && row.locked_until <= now) {
      this.db.prepare('UPDATE failure_window SET failure_count = 0, locked_until = 0 WHERE identity_id = ?').run(identityId);
      this.db.prepare(
        `UPDATE challenge_state
         SET status = ?, failure_count = 0, lock_until = 0
         WHERE identity_id = ? AND status = ? AND lock_until <= ?`
      ).run('idle', identityId, 'locked', now);
      return { windowStart: row.window_start, failureCount: 0, lockedUntil: 0 };
    }
    return { windowStart: row.window_start, failureCount: row.failure_count, lockedUntil: row.locked_until };
  }

  createChallenge(identityId, scope) {
    const window = this.getFailureWindow(identityId);
    if (window.lockedUntil > nowMs()) {
      const err = new Error('challenge is locked');
      err.key = 'CHALLENGE_LOCKED';
      err.retryAfter = window.lockedUntil;
      throw err;
    }
    const expected = this.db.prepare('SELECT answer_digest FROM answer_digest_set WHERE identity_id = ?').all(identityId);
    if (!expected.length) {
      const err = new Error('answerDigestSet is not enrolled for identity');
      err.key = 'SECRET_UNAVAILABLE';
      throw err;
    }
    const challengeId = makeId('chl');
    this.db.prepare(
      `INSERT INTO challenge_state
       (challenge_id, identity_id, scope, status, expected_answer_digests_json, failure_count, lock_until, created_at, expires_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`
    ).run(challengeId, identityId, scope, 'challenge_created', serializeJson(expected.map((row) => row.answer_digest)), window.failureCount, window.lockedUntil, nowMs(), nowMs() + 5 * 60 * 1000);
    return {
      challengeId,
      identityId,
      scope,
      status: 'challenge_created',
      failureCount: window.failureCount,
      maxRetryCount: MAX_FAILURES_PER_WINDOW,
      lockUntil: window.lockedUntil,
      expectedAnswerDigests: new Set(expected.map((row) => row.answer_digest)),
      createdAt: nowMs(),
      expiresAt: nowMs() + 5 * 60 * 1000,
    };
  }

  submitChallengeAnswers(identityId, challengeId, answers) {
    const challenge = this.db.prepare('SELECT * FROM challenge_state WHERE challenge_id = ?').get(challengeId);
    if (!challenge) {
      const err = new Error('challenge verification failed');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    if (challenge.identity_id !== identityId) {
      const err = new Error('challenge verification failed');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    if (challenge.status !== 'challenge_created') {
      const err = new Error('challenge verification failed');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    if (challenge.expires_at + REPLAY_DRIFT_MS <= nowMs()) {
      const err = new Error('challenge verification failed');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    const window = this.getFailureWindow(identityId);
    if (window.lockedUntil > nowMs()) {
      const err = new Error('challenge is locked');
      err.key = 'CHALLENGE_LOCKED';
      err.retryAfter = window.lockedUntil;
      throw err;
    }
    const stateUpdate = this.db.prepare(
      'UPDATE challenge_state SET status = ? WHERE challenge_id = ? AND status = ?'
    ).run('answers_submitted', challengeId, 'challenge_created');
    if (stateUpdate.changes !== 1) {
      const err = new Error('challenge verification failed');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    const normalizedAnswers = answers.map(normalizeAnswerValue).filter(Boolean);
    const identityKey = deriveIdentityAnswerKey(this.masterSalt, identityId);
    const answerDigests = normalizedAnswers.map((answer) => hmacSha256Hex(identityKey, answer));
    const expectedDigests = new Set(parseJson(challenge.expected_answer_digests_json, []));
    const accepted = answerDigests.some((digest) => expectedDigests.has(digest));
    if (!accepted) {
      const nextCount = window.failureCount + 1;
      const lockedUntil = nextCount >= MAX_FAILURES_PER_WINDOW ? nowMs() + FAILURE_LOCK_MS : 0;
      this.db.prepare(
        'UPDATE failure_window SET failure_count = ?, locked_until = ? WHERE identity_id = ?'
      ).run(nextCount, lockedUntil, identityId);
      this.db.prepare(
        'UPDATE challenge_state SET status = ?, failure_count = ?, lock_until = ? WHERE challenge_id = ?'
      ).run(lockedUntil ? 'locked' : 'failed', nextCount, lockedUntil, challengeId);
      this.recordAudit('challenge_failed', { identityId, result: lockedUntil ? 'locked' : 'failed' });
      return { approved: false, challenge: { ...challenge, status: lockedUntil ? 'locked' : 'failed', failure_count: nextCount, lock_until: lockedUntil }, retryAfter: lockedUntil || null };
    }
    this.db.prepare('UPDATE failure_window SET failure_count = 0, locked_until = 0 WHERE identity_id = ?').run(identityId);
    this.db.prepare('UPDATE challenge_state SET status = ?, failure_count = 0, lock_until = 0 WHERE challenge_id = ?').run('verified', challengeId);
    this.recordAudit('challenge_verified', { identityId, result: 'success' });
    return { approved: true, challenge: { ...challenge, status: 'verified' } };
  }

  issueSession(identityId, challenge, scope, resourceScope, budgets = {}) {
    const sessionId = makeId('ses');
    const ttlMs = budgets.ttlMs ?? 3 * 60 * 1000;
    this.db.prepare(
      `INSERT INTO session_store
       (session_id, challenge_id, identity_id, scope, resource_scope_json, session_type, read_budget, write_budget, status, issued_at, expires_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
    ).run(sessionId, challenge.challengeId, identityId, scope, serializeJson(resourceScope), budgets.sessionType ?? 'one_shot', budgets.readBudget ?? 0, budgets.writeBudget ?? 0, 'session_active', nowMs(), nowMs() + ttlMs);
    return {
      sessionId,
      challengeId: challenge.challengeId,
      identityId,
      scope,
      resourceScope,
      sessionType: budgets.sessionType ?? 'one_shot',
      readBudget: budgets.readBudget ?? 0,
      writeBudget: budgets.writeBudget ?? 0,
      issuedAt: nowMs(),
      expiresAt: nowMs() + ttlMs,
      status: 'session_active',
    };
  }

}

export function createAccessHost({ dbPath = DEFAULT_DB_PATH, secretStore, usePlatformSecretStore = false } = {}) {
  const persistent = dbPath !== ':memory:';
  if (persistent && !secretStore && !usePlatformSecretStore) {
    throw new Error('Persistent SQLite host requires secretStore or usePlatformSecretStore=true');
  }
  const resolvedSecretStore = secretStore ?? (usePlatformSecretStore ? createPlatformSecretStore() : DEFAULT_SECRET_STORE);
  if (persistent && resolvedSecretStore?.kind === 'memory' && !resolvedSecretStore.developmentMode) {
    throw new Error('Persistent SQLite host must not use MemorySecretStore unless developmentMode=true. Use a platform SecretStore in production.');
  }
  const store = new SqliteStore(dbPath, resolvedSecretStore, { persistent });
  store.ensureSeeded();
  store.cleanupExpired();
  return store;
}
