import { createHash, randomBytes } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
import { deriveHkdfSha256, hmacSha256Hex } from '../../shared/crypto.js';
import { MemorySecretStore, createPlatformSecretStore } from '../../shared/secret-store.js';
import { loadSqliteSchema, migrateSqliteSchema } from '../../shared/sqlite.js';

const DEFAULT_DB_PATH = ':memory:';
const DEFAULT_SECRET_STORE = new MemorySecretStore({ developmentMode: true, suppressWarning: true });

const REPLAY_DRIFT_MS = 30_000;
const REPLAY_WINDOW_MS = 24 * 60 * 60 * 1000;
const FAILURE_WINDOW_MS = 24 * 60 * 60 * 1000;
const FAILURE_LOCK_MS = 15 * 60 * 1000;
const MAX_FAILURES_PER_WINDOW = 3;
const DEFAULT_CLEANUP_BATCH_SIZE = 1000;
const NORMALIZATION_VERSION = 'nfkc-lower-v1';
const QUESTION_SET_VERSION = 'storylock-local-v1';

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

function normalizeQuestionStatus(value = 'active') {
  if (!['active', 'deprecated', 'pending'].includes(value)) {
    throw new Error('question status must be active, deprecated, or pending');
  }
  return value;
}

function uniqueOrderedStrings(values, fallback = []) {
  const seen = new Set();
  const result = [];
  for (const value of values) {
    const text = String(value ?? '').trim();
    if (!text || seen.has(text)) {
      continue;
    }
    seen.add(text);
    result.push(text);
  }
  return result.length > 0 ? result : fallback;
}

function summarizeVersionSet(values, fallback = 'legacy') {
  const unique = uniqueOrderedStrings(values);
  if (unique.length === 0) {
    return fallback;
  }
  if (unique.length === 1) {
    return unique[0];
  }
  return `mixed:${unique.join('+')}`;
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

function answerDigestForQuestion(identityKey, questionId, answer) {
  const normalized = normalizeAnswerValue(answer);
  if (!normalized) {
    return '';
  }
  return hmacSha256Hex(identityKey, `${questionId}:${normalized}`);
}

function answerDigestForLegacy(identityKey, answer) {
  const normalized = normalizeAnswerValue(answer);
  if (!normalized) {
    return '';
  }
  return hmacSha256Hex(identityKey, normalized);
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

function normalizeQuestionSetVersion(value, fieldName = 'questionSetVersion') {
  const version = String(value ?? '').trim();
  if (!version) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  if (version.length > 128) {
    throw new Error(`${fieldName} must be 128 characters or less`);
  }
  return version;
}

class SqliteStore {
  constructor(dbPath = DEFAULT_DB_PATH, secretStore = new MemorySecretStore({ developmentMode: true, suppressWarning: true }), {
    persistent = false,
    allowLegacyFallback = false,
    databaseFactory = (path) => new DatabaseSync(path),
  } = {}) {
    this.db = databaseFactory(dbPath);
    if (!this.db || typeof this.db.exec !== 'function' || typeof this.db.prepare !== 'function' || typeof this.db.close !== 'function') {
      throw new Error('databaseFactory must return a DatabaseSync-compatible object with exec, prepare, and close methods');
    }
    this.db.exec(loadSqliteSchema());
    migrateSqliteSchema(this.db);
    this.secretStore = secretStore;
    this.persistent = persistent;
    this.allowLegacyFallback = allowLegacyFallback;
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
    const digests = answers.map((answer) => answerDigestForLegacy(identityKey, answer)).filter(Boolean);
    this.db.exec('BEGIN IMMEDIATE;');
    try {
      this.db.prepare('DELETE FROM answer_digest_set WHERE identity_id = ?').run(identityId);
      const insert = this.db.prepare(
        `INSERT OR IGNORE INTO answer_digest_set
         (identity_id, answer_digest, normalization_version, question_set_version, created_at)
         VALUES (?, ?, ?, ?, ?)`
      );
      for (const digest of digests) {
        insert.run(identityId, digest, NORMALIZATION_VERSION, 'legacy', nowMs());
      }
      this.db.exec('COMMIT;');
      return digests;
    } catch (error) {
      this.db.exec('ROLLBACK;');
      throw error;
    }
  }

  enrollQuestionSet(identityId, questions, {
    questionSetVersion = QUESTION_SET_VERSION,
    normalizationVersion = NORMALIZATION_VERSION,
    status = 'active',
    replacePreviousActive = true,
  } = {}) {
    if (!Array.isArray(questions) || questions.length === 0) {
      throw new Error('questions must be a non-empty array');
    }
    const normalizedQuestionSetVersion = normalizeQuestionSetVersion(questionSetVersion);
    const normalizedNormalizationVersion = normalizeQuestionSetVersion(normalizationVersion, 'normalizationVersion');
    const normalizedStatus = normalizeQuestionStatus(status);
    const identityKey = deriveIdentityAnswerKey(this.masterSalt, identityId);
    const now = nowMs();
    const rows = questions.map((question, index) => {
      const questionId = String(question.questionId ?? `q-${index + 1}`).trim();
      const versionTag = String(question.versionTag ?? normalizedQuestionSetVersion).trim();
      const promptRef = String(question.promptRef ?? questionId).trim();
      if (!questionId || !versionTag || !promptRef) {
        throw new Error('questionId, versionTag, and promptRef must be non-empty');
      }
      const answerDigest = answerDigestForQuestion(identityKey, questionId, question.answer);
      if (!answerDigest) {
        throw new Error(`question ${questionId} must include a non-empty answer`);
      }
      return {
        questionId,
        versionTag,
        promptRef,
        promptText: question.promptText ?? null,
        optionDigest: question.optionDigest ?? (question.options ? hashPayload(question.options) : null),
        answerDigest,
        normalizationVersion: normalizeQuestionSetVersion(question.normalizationVersion ?? normalizedNormalizationVersion, 'normalizationVersion'),
        questionSetVersion: normalizeQuestionSetVersion(question.questionSetVersion ?? normalizedQuestionSetVersion),
        status: normalizeQuestionStatus(question.status ?? normalizedStatus),
      };
    });
    this.db.exec('BEGIN IMMEDIATE;');
    try {
      if (normalizedStatus === 'active' && replacePreviousActive) {
        this.db.prepare(
          `UPDATE question_set_item
           SET status = ?, updated_at = ?
           WHERE identity_id = ? AND question_set_version != ? AND status = ?`
        ).run('deprecated', now, identityId, normalizedQuestionSetVersion, 'active');
      }
      const insert = this.db.prepare(
        `INSERT INTO question_set_item
         (identity_id, question_id, version_tag, prompt_ref, prompt_text, option_digest, answer_digest, normalization_version, question_set_version, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(identity_id, question_id, version_tag) DO UPDATE SET
           prompt_ref = excluded.prompt_ref,
           prompt_text = excluded.prompt_text,
           option_digest = excluded.option_digest,
           answer_digest = excluded.answer_digest,
           normalization_version = excluded.normalization_version,
           question_set_version = excluded.question_set_version,
           status = excluded.status,
           updated_at = excluded.updated_at`
      );
      for (const row of rows) {
        insert.run(
          identityId,
          row.questionId,
          row.versionTag,
          row.promptRef,
          row.promptText,
          row.optionDigest,
          row.answerDigest,
          row.normalizationVersion,
          row.questionSetVersion,
          row.status,
          now,
          now,
        );
      }
      this.db.exec('COMMIT;');
      return rows.map(({ answerDigest, ...row }) => row);
    } catch (error) {
      this.db.exec('ROLLBACK;');
      throw error;
    }
  }

  setQuestionSetVersionStatus(identityId, questionSetVersion, status) {
    const normalizedQuestionSetVersion = normalizeQuestionSetVersion(questionSetVersion);
    const normalizedStatus = normalizeQuestionStatus(status);
    return this.db.prepare(
      `UPDATE question_set_item
       SET status = ?, updated_at = ?
       WHERE identity_id = ? AND question_set_version = ?`
    ).run(normalizedStatus, nowMs(), identityId, normalizedQuestionSetVersion);
  }

  selectQuestionCells(identityId, requiredCells, { questionSetVersion = null } = {}) {
    const preferredVersion = questionSetVersion
      ? normalizeQuestionSetVersion(questionSetVersion)
      : this.db.prepare(
        `SELECT question_set_version
         FROM question_set_item
         WHERE identity_id = ? AND status = ?
         ORDER BY updated_at DESC, created_at DESC, question_set_version DESC
         LIMIT 1`
      ).get(identityId, 'active')?.question_set_version ?? null;
    const rows = preferredVersion
      ? this.db.prepare(
        `SELECT question_id, version_tag, prompt_ref, prompt_text, option_digest, answer_digest, normalization_version, question_set_version
         FROM question_set_item
         WHERE identity_id = ? AND status = ? AND question_set_version = ?
         ORDER BY question_id, version_tag
         LIMIT ?`
      ).all(identityId, 'active', preferredVersion, requiredCells)
      : this.db.prepare(
        `SELECT question_id, version_tag, prompt_ref, prompt_text, option_digest, answer_digest, normalization_version, question_set_version
         FROM question_set_item
         WHERE identity_id = ? AND status = ?
         ORDER BY question_id, version_tag
         LIMIT ?`
      ).all(identityId, 'active', requiredCells);
    if (rows.length < requiredCells) {
      return null;
    }
    return rows.map((row, index) => ({
      cellId: `cell-${index + 1}`,
      position: index + 1,
      questionId: row.question_id,
      versionTag: row.version_tag,
      promptRef: row.prompt_ref,
      promptText: row.prompt_text,
      optionDigest: row.option_digest,
      expectedDigest: row.answer_digest,
      normalizationVersion: row.normalization_version,
      questionSetVersion: row.question_set_version,
    }));
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

  createChallenge(identityId, scope, { requiredCells = 1, questionSetVersion = null } = {}) {
    const window = this.getFailureWindow(identityId);
    if (window.lockedUntil > nowMs()) {
      const err = new Error('challenge is locked');
      err.key = 'CHALLENGE_LOCKED';
      err.retryAfter = window.lockedUntil;
      throw err;
    }
    let cells = this.selectQuestionCells(identityId, requiredCells, { questionSetVersion });
    if (!cells) {
      if (!this.allowLegacyFallback) {
        const err = new Error('active question set does not contain enough cells for this challenge');
        err.key = 'SECRET_UNAVAILABLE';
        err.type = 'question_set_unavailable';
        err.retryable = false;
        err.suggestedAction = 'Enroll a production question set or enable allowLegacyFallback only for development/demo mode.';
        throw err;
      }
      const expected = this.db.prepare('SELECT answer_digest FROM answer_digest_set WHERE identity_id = ?').all(identityId);
      if (expected.length >= requiredCells) {
        cells = expected.slice(0, requiredCells).map((row, index) => ({
          cellId: `cell-${index + 1}`,
          position: index + 1,
          questionId: `legacy-${index + 1}`,
          versionTag: 'legacy',
          promptRef: `legacy-cue-${index + 1}`,
          promptText: null,
          optionDigest: null,
          expectedDigest: row.answer_digest,
          normalizationVersion: NORMALIZATION_VERSION,
          questionSetVersion: 'legacy',
          legacy: true,
        }));
      }
    }
    if (!cells) {
      const err = new Error('answerDigestSet is not enrolled for identity');
      err.key = 'SECRET_UNAVAILABLE';
      throw err;
    }
    const questionSetVersions = uniqueOrderedStrings(cells.map((cell) => cell.questionSetVersion), ['legacy']);
    const normalizationVersions = uniqueOrderedStrings(cells.map((cell) => cell.normalizationVersion), [NORMALIZATION_VERSION]);
    const selectedQuestionSetVersion = summarizeVersionSet(questionSetVersions, 'legacy');
    const selectedNormalizationVersion = summarizeVersionSet(normalizationVersions, NORMALIZATION_VERSION);
    const challengeId = makeId('chl');
    const manifest = {
      cells: cells.map(({ expectedDigest, ...cell }) => cell),
      requiredCells,
      requiredThreshold: requiredCells,
      questionSetVersions,
      normalizationVersions,
      questionSetVersion: selectedQuestionSetVersion,
      normalizationVersion: selectedNormalizationVersion,
    };
    this.db.prepare(
      `INSERT INTO challenge_state
       (challenge_id, identity_id, scope, status, expected_answer_digests_json, challenge_manifest_json, required_threshold, normalization_version, question_set_version, failure_count, lock_until, created_at, expires_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
    ).run(
      challengeId,
      identityId,
      scope,
      'challenge_created',
      serializeJson(cells.map(({ cellId, questionId, expectedDigest, legacy = false }) => ({ cellId, questionId, digest: expectedDigest, legacy }))),
      serializeJson(manifest),
      requiredCells,
      selectedNormalizationVersion,
      selectedQuestionSetVersion,
      window.failureCount,
      window.lockedUntil,
      nowMs(),
      nowMs() + 5 * 60 * 1000,
    );
    return {
      challengeId,
      identityId,
      scope,
      status: 'challenge_created',
      failureCount: window.failureCount,
      maxRetryCount: MAX_FAILURES_PER_WINDOW,
      lockUntil: window.lockedUntil,
      cells: manifest.cells,
      requiredThreshold: requiredCells,
      expectedAnswerDigests: new Set(cells.map((row) => row.expectedDigest)),
      questionSetVersions,
      normalizationVersions,
      questionSetVersion: selectedQuestionSetVersion,
      normalizationVersion: selectedNormalizationVersion,
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
    const submittedByCell = new Map();
    for (const answer of answers) {
      const cellId = typeof answer === 'string' ? `cell-${submittedByCell.size + 1}` : answer?.cellId;
      if (typeof cellId === 'string' && cellId.trim()) {
        submittedByCell.set(cellId.trim(), answer);
      }
    }
    const identityKey = deriveIdentityAnswerKey(this.masterSalt, identityId);
    const expectedCells = parseJson(challenge.expected_answer_digests_json, []);
    let matchedCount = 0;
    for (const expected of expectedCells) {
      const submitted = submittedByCell.get(expected.cellId);
      if (!submitted) {
        continue;
      }
      const digest = expected.legacy
        ? answerDigestForLegacy(identityKey, submitted)
        : answerDigestForQuestion(identityKey, expected.questionId, submitted);
      if (digest && digest === expected.digest) {
        matchedCount += 1;
      }
    }
    const accepted = matchedCount >= challenge.required_threshold;
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
      return { approved: false, matchedCount, challenge: { ...challenge, status: lockedUntil ? 'locked' : 'failed', failure_count: nextCount, lock_until: lockedUntil }, retryAfter: lockedUntil || null };
    }
    this.db.prepare('UPDATE failure_window SET failure_count = 0, locked_until = 0 WHERE identity_id = ?').run(identityId);
    this.db.prepare('UPDATE challenge_state SET status = ?, failure_count = 0, lock_until = 0 WHERE challenge_id = ?').run('verified', challengeId);
    this.recordAudit('challenge_verified', { identityId, result: 'success' });
    return { approved: true, matchedCount, challenge: { ...challenge, status: 'verified' } };
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

  revokeChallenge(identityId, challengeId, { reason = 'manual_revocation' } = {}) {
    const row = this.db.prepare('SELECT challenge_id, identity_id, status FROM challenge_state WHERE challenge_id = ?').get(challengeId);
    if (!row || row.identity_id !== identityId || row.status !== 'challenge_created') {
      const err = new Error('challenge cannot be revoked');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    const result = this.db.prepare(
      'UPDATE challenge_state SET status = ? WHERE challenge_id = ? AND status = ?'
    ).run('revoked', challengeId, 'challenge_created');
    if (result.changes !== 1) {
      const err = new Error('challenge cannot be revoked');
      err.key = 'CHALLENGE_FAILED';
      throw err;
    }
    this.recordAudit('challenge_revoked', {
      identityId,
      result: 'success',
      meta: {
        challengeId,
        reason,
      },
    });
    return {
      revoked: true,
      targetType: 'challenge',
      challengeId,
      status: 'revoked',
      reason,
    };
  }

  revokeSession(identityId, sessionId, { reason = 'manual_revocation' } = {}) {
    const row = this.db.prepare('SELECT session_id, identity_id, status FROM session_store WHERE session_id = ?').get(sessionId);
    if (!row || row.identity_id !== identityId || row.status !== 'session_active') {
      const err = new Error('session cannot be revoked');
      err.key = 'SESSION_INVALID';
      throw err;
    }
    const result = this.db.prepare(
      'UPDATE session_store SET status = ? WHERE session_id = ? AND status = ?'
    ).run('session_revoked', sessionId, 'session_active');
    if (result.changes !== 1) {
      const err = new Error('session cannot be revoked');
      err.key = 'SESSION_INVALID';
      throw err;
    }
    this.recordAudit('session_revoked', {
      identityId,
      result: 'success',
      meta: {
        sessionId,
        reason,
      },
    });
    return {
      revoked: true,
      targetType: 'session',
      sessionId,
      status: 'session_revoked',
      reason,
    };
  }

}

export function createAccessHost({
  dbPath = DEFAULT_DB_PATH,
  secretStore,
  usePlatformSecretStore = false,
  allowLegacyFallback,
  databaseFactory,
} = {}) {
  const persistent = dbPath !== ':memory:';
  if (persistent && !secretStore && !usePlatformSecretStore) {
    throw new Error('Persistent SQLite host requires secretStore or usePlatformSecretStore=true');
  }
  const resolvedSecretStore = secretStore ?? (usePlatformSecretStore ? createPlatformSecretStore() : DEFAULT_SECRET_STORE);
  if (persistent && resolvedSecretStore?.kind === 'memory' && !resolvedSecretStore.developmentMode) {
    throw new Error('Persistent SQLite host must not use MemorySecretStore unless developmentMode=true. Use a platform SecretStore in production.');
  }
  const resolvedAllowLegacyFallback = allowLegacyFallback ?? !persistent;
  const store = new SqliteStore(dbPath, resolvedSecretStore, {
    persistent,
    allowLegacyFallback: resolvedAllowLegacyFallback,
    databaseFactory: databaseFactory ?? ((path) => new DatabaseSync(path)),
  });
  store.ensureSeeded();
  store.cleanupExpired();
  return store;
}

export function startAccessHostCleanup(host, {
  intervalMs,
  batchSize = DEFAULT_CLEANUP_BATCH_SIZE,
  now = nowMs,
  setIntervalFn = setInterval,
  clearIntervalFn = clearInterval,
  onError = null,
} = {}) {
  const normalizedIntervalMs = Number(intervalMs);
  if (!Number.isFinite(normalizedIntervalMs) || normalizedIntervalMs <= 0) {
    throw new Error('cleanup intervalMs must be a positive number');
  }
  if (!host || typeof host.cleanupExpired !== 'function') {
    throw new Error('host must provide cleanupExpired');
  }
  const runCleanup = () => {
    try {
      return host.cleanupExpired(now(), { batchSize });
    } catch (error) {
      if (onError) {
        onError(error);
        return null;
      }
      throw error;
    }
  };
  const timer = setIntervalFn(runCleanup, normalizedIntervalMs);
  if (timer && typeof timer.unref === 'function') {
    timer.unref();
  }
  return {
    runNow: runCleanup,
    stop() {
      clearIntervalFn(timer);
    },
  };
}
