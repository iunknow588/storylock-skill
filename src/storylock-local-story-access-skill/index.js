import { createAccessHost } from './access-host.js';
import { buildErrorPayload } from './errors.js';

const REPLAY_DRIFT_MS = 30_000;
const MAX_REQUEST_ID_LENGTH = 128;
const MAX_NONCE_LENGTH = 128;
const MAX_ID_LENGTH = 128;
const MAX_ANSWERS = 10;
const MAX_ANSWER_LENGTH = 512;
const MAX_CONTENT_JSON_BYTES = 64 * 1024;
const HIGH_SENSITIVITY_FIELD_PATTERN = /(password|secret|token|privateKey|mnemonic|seed|phone|email|idCard|credential)/i;
const HIGH_SENSITIVITY_VALUE_PATTERNS = [
  /\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b/i,
  /\b1[3-9]\d{9}\b/,
  /\b\d{15}(\d{2}[0-9Xx])?\b/,
  /\b(?:0x)?[a-f0-9]{64}\b/i,
];

function nowMs() {
  return Date.now();
}

function normalizeString(value, fieldName, { maxLength = MAX_ID_LENGTH } = {}) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  const normalized = value.trim();
  if (normalized.length > maxLength) {
    throw new Error(`${fieldName} must be ${maxLength} characters or less`);
  }
  return normalized;
}

function normalizeArray(value) {
  if (value === undefined || value === null) {
    return [];
  }
  if (!Array.isArray(value)) {
    throw new Error('answers must be an array');
  }
  if (value.length > MAX_ANSWERS) {
    throw new Error(`answers must contain ${MAX_ANSWERS} items or fewer`);
  }
  for (const item of value) {
    const answer = typeof item === 'string' ? item : item?.answer;
    if (typeof answer !== 'string') {
      throw new Error('answers must contain strings or objects with an answer string');
    }
    if (answer.length > MAX_ANSWER_LENGTH) {
      throw new Error(`answer must be ${MAX_ANSWER_LENGTH} characters or less`);
    }
  }
  return value;
}

function normalizeContent(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error('content must be an object');
  }
  const encoded = Buffer.byteLength(JSON.stringify(value), 'utf8');
  if (encoded > MAX_CONTENT_JSON_BYTES) {
    throw new Error(`content must be ${MAX_CONTENT_JSON_BYTES} bytes or less when JSON encoded`);
  }
  return value;
}

function normalizeRedactionLevel(input) {
  const level = input?.redactionLevel ?? input?.policyHints?.redactionLevel ?? 'partial';
  if (!['none', 'partial', 'full'].includes(level)) {
    throw new Error('redactionLevel must be none, partial, or full');
  }
  return level;
}

function normalizeRequestEnvelope(input, fallbackRequestId) {
  const requestId = normalizeString(input?.requestId ?? fallbackRequestId, 'requestId', { maxLength: MAX_REQUEST_ID_LENGTH });
  const nonce = normalizeString(input?.nonce ?? `nonce-${Date.now().toString(16)}`, 'nonce', { maxLength: MAX_NONCE_LENGTH });
  const expiry = Number(input?.expiry ?? (nowMs() + 60_000));
  if (!Number.isFinite(expiry)) {
    throw new Error('expiry must be a valid number');
  }
  if (expiry + REPLAY_DRIFT_MS <= nowMs()) {
    throw new Error('REQUEST_EXPIRED');
  }
  return { requestId, nonce, expiry };
}

function collectSensitiveSignals(value, path = '', signals = new Set()) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => collectSensitiveSignals(item, `${path}[${index}]`, signals));
    return signals;
  }
  if (value && typeof value === 'object') {
    for (const [key, nested] of Object.entries(value)) {
      const nextPath = path ? `${path}.${key}` : key;
      if (HIGH_SENSITIVITY_FIELD_PATTERN.test(key)) {
        signals.add(`field:${nextPath}`);
      }
      collectSensitiveSignals(nested, nextPath, signals);
    }
    return signals;
  }
  if (typeof value === 'string') {
    for (const pattern of HIGH_SENSITIVITY_VALUE_PATTERNS) {
      if (pattern.test(value)) {
        signals.add(path ? `value:${path}` : 'value');
        break;
      }
    }
  }
  return signals;
}

function analyzeSensitiveContent(value) {
  const signals = Array.from(collectSensitiveSignals(value));
  return {
    hasHighSensitivityFields: signals.length > 0,
    highSensitivitySignals: signals.slice(0, 20),
  };
}

function redactStoryObject(storyObject, redactionLevel, sensitivityReport) {
  if (redactionLevel === 'none') {
    return storyObject;
  }
  const base = {
    storyObjectId: storyObject.storyObjectId,
    version: storyObject.version,
    sensitivity: storyObject.sensitivity,
  };
  if (redactionLevel === 'full' || sensitivityReport.hasHighSensitivityFields) {
    return {
      ...base,
      title: '[redacted]',
      contentSummary: '[redacted]',
    };
  }
  return {
    ...base,
    title: storyObject.title,
    contentSummary: typeof storyObject.content === 'string' ? `[redacted:${storyObject.content.length} chars]` : '[redacted]',
  };
}

function redactWriteResult(writeResult, redactionLevel, sensitivityReport) {
  if (redactionLevel === 'none') {
    return writeResult;
  }
  const base = {
    storyObjectId: writeResult.storyObjectId,
    version: writeResult.version,
    sensitivity: writeResult.sensitivity,
  };
  if (redactionLevel === 'full' || sensitivityReport.hasHighSensitivityFields) {
    return base;
  }
  return {
    ...base,
    title: writeResult.title,
  };
}

function toErrorResponse({ requestId, capability, error }) {
  const errorPayload = buildErrorPayload(error);
  return {
    requestId,
    status: 'error',
    capability,
    executionLocation: 'local',
    result: null,
    redactionLevel: 'full',
    retentionGranted: 'audit_meta_only',
    auditMeta: {
      timestamp: new Date().toISOString(),
      errorCode: errorPayload.code,
    },
    error: errorPayload,
  };
}

export class StoryReadAccessSkill {
  constructor({ host, dbPath, secretStore, usePlatformSecretStore = false } = {}) {
    this.host = host ?? createAccessHost({ dbPath, secretStore, usePlatformSecretStore });
  }

  skillId() {
    return 'story_read_access';
  }

  async run(input = {}) {
    const fallbackRequestId = input?.requestId ?? `req-${Date.now().toString(16)}`;
    try {
      const { requestId, nonce, expiry } = normalizeRequestEnvelope(input, fallbackRequestId);
      const identityId = normalizeString(input.identityId, 'identityId', { maxLength: MAX_ID_LENGTH });
      const storyObjectId = normalizeString(input.storyObjectId, 'storyObjectId', { maxLength: MAX_ID_LENGTH });
      const answers = normalizeArray(input.answers);
      const redactionLevel = normalizeRedactionLevel(input);
      this.host.ensureSeeded?.();
      const replay = this.host.ensureReplaySafe?.(requestId, nonce, expiry, {
        capability: 'requestStoryRead',
        identityId,
        storyObjectId,
        answers,
        redactionLevel,
      });
      if (replay?.replayed) {
        return replay.response;
      }
      const challenge = this.host.createChallenge(identityId, 'story_read_basic');
      const authorization = this.host.submitChallengeAnswers(identityId, challenge.challengeId, answers);
      if (!authorization.approved) {
        throw Object.assign(new Error('challenge answers did not match'), {
          code: 'SLG-003',
          type: 'challenge_failed',
          retryable: true,
          retryAfter: authorization.retryAfter ?? null,
        });
      }
      const session = this.host.issueSession(identityId, challenge, 'story_read_basic', [storyObjectId], {
        readBudget: 1,
        ttlMs: 3 * 60 * 1000,
        sessionType: 'one_shot',
      });
      const { storyObject } = this.host.readStoryObjectWithBudget(identityId, session.sessionId, storyObjectId, {
        redactionLevel,
      });
      const sensitivityReport = analyzeSensitiveContent(storyObject);
      this.host.recordAudit?.('story_read_redaction_applied', {
        identityId,
        storyObjectId,
        requestId,
        result: 'success',
        redactionLevel,
        hasHighSensitivityFields: sensitivityReport.hasHighSensitivityFields,
        meta: { highSensitivitySignals: sensitivityReport.highSensitivitySignals },
      });
      const response = {
        requestId,
        status: 'success',
        capability: 'requestStoryRead',
        executionLocation: 'local',
        result: {
          mode: 'story_read_access',
          storyObjectId,
          storyObject: redactStoryObject(storyObject, redactionLevel, sensitivityReport),
        },
        redactionLevel,
        retentionGranted: 'result_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
          challengeId: challenge.challengeId,
          sessionId: session.sessionId,
          redactionLevel,
          hasHighSensitivityFields: sensitivityReport.hasHighSensitivityFields,
          highSensitivitySignals: sensitivityReport.highSensitivitySignals,
        },
        error: null,
      };
      this.host.storeRequestResponse?.(requestId, response);
      return response;
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'requestStoryRead', error });
    }
  }
}

export class StoryWriteAccessSkill {
  constructor({ host, dbPath, secretStore, usePlatformSecretStore = false } = {}) {
    this.host = host ?? createAccessHost({ dbPath, secretStore, usePlatformSecretStore });
  }

  skillId() {
    return 'story_write_access';
  }

  async run(input = {}) {
    const fallbackRequestId = input?.requestId ?? `req-${Date.now().toString(16)}`;
    try {
      const { requestId, nonce, expiry } = normalizeRequestEnvelope(input, fallbackRequestId);
      const identityId = normalizeString(input.identityId, 'identityId', { maxLength: MAX_ID_LENGTH });
      const storyObjectId = normalizeString(input.storyObjectId, 'storyObjectId', { maxLength: MAX_ID_LENGTH });
      const content = normalizeContent(input.content);
      const answers = normalizeArray(input.answers);
      const redactionLevel = normalizeRedactionLevel(input);
      this.host.ensureSeeded?.();
      const replay = this.host.ensureReplaySafe?.(requestId, nonce, expiry, {
        capability: 'requestStoryWrite',
        identityId,
        storyObjectId,
        content,
        answers,
        redactionLevel,
      });
      if (replay?.replayed) {
        return replay.response;
      }
      const challenge = this.host.createChallenge(identityId, 'story_write_basic');
      const authorization = this.host.submitChallengeAnswers(identityId, challenge.challengeId, answers);
      if (!authorization.approved) {
        throw Object.assign(new Error('challenge answers did not match'), {
          code: 'SLG-003',
          type: 'challenge_failed',
          retryable: true,
          retryAfter: authorization.retryAfter ?? null,
        });
      }
      const session = this.host.issueSession(identityId, challenge, 'story_write_basic', [storyObjectId], {
        writeBudget: 1,
        ttlMs: 3 * 60 * 1000,
        sessionType: 'one_shot',
      });
      const contentSensitivityReport = analyzeSensitiveContent(content);
      const writeResult = this.host.writeStoryObject(identityId, session.sessionId, storyObjectId, content, {
        redactionLevel,
        hasHighSensitivityFields: contentSensitivityReport.hasHighSensitivityFields,
        meta: { highSensitivitySignals: contentSensitivityReport.highSensitivitySignals },
      });
      const sensitivityReport = analyzeSensitiveContent(writeResult);
      const response = {
        requestId,
        status: 'success',
        capability: 'requestStoryWrite',
        executionLocation: 'local',
        result: {
          mode: 'story_write_access',
          storyObjectId,
          writeResult: redactWriteResult(writeResult, redactionLevel, sensitivityReport),
        },
        redactionLevel,
        retentionGranted: 'result_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
          challengeId: challenge.challengeId,
          sessionId: session.sessionId,
          redactionLevel,
          hasHighSensitivityFields: sensitivityReport.hasHighSensitivityFields,
          highSensitivitySignals: sensitivityReport.highSensitivitySignals,
        },
        error: null,
      };
      this.host.storeRequestResponse?.(requestId, response);
      return response;
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'requestStoryWrite', error });
    }
  }
}
