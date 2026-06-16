import { createAccessHost } from './access-host.js';
import { buildErrorPayload } from './errors.js';

const REPLAY_DRIFT_MS = 30_000;

function nowMs() {
  return Date.now();
}

function normalizeString(value, fieldName) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return value.trim();
}

function normalizeArray(value) {
  if (value === undefined || value === null) {
    return [];
  }
  if (!Array.isArray(value)) {
    throw new Error('answers must be an array');
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
  const requestId = normalizeString(input?.requestId ?? fallbackRequestId, 'requestId');
  const nonce = normalizeString(input?.nonce ?? `nonce-${Date.now().toString(16)}`, 'nonce');
  const expiry = Number(input?.expiry ?? (nowMs() + 60_000));
  if (!Number.isFinite(expiry)) {
    throw new Error('expiry must be a valid number');
  }
  if (expiry + REPLAY_DRIFT_MS <= nowMs()) {
    throw new Error('REQUEST_EXPIRED');
  }
  return { requestId, nonce, expiry };
}

function toErrorResponse({ requestId, capability, error }) {
  return {
    requestId,
    status: 'error',
    capability,
    executionLocation: 'local',
    result: null,
    redactionLevel: 'full',
    retentionGranted: 'audit_meta_only',
    auditMeta: {},
    error: buildErrorPayload(error),
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
      const identityId = normalizeString(input.identityId, 'identityId');
      const storyObjectId = normalizeString(input.storyObjectId, 'storyObjectId');
      const answers = normalizeArray(input.answers);
      const redactionLevel = normalizeRedactionLevel(input);
      this.host.ensureSeeded?.();
      this.host.ensureReplaySafe?.(requestId, nonce, expiry);
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
      const { storyObject } = this.host.readStoryObjectWithBudget(identityId, session.sessionId, storyObjectId);
      return {
        requestId,
        status: 'success',
        capability: 'requestStoryRead',
        executionLocation: 'local',
        result: {
          mode: 'story_read_access',
          storyObjectId,
          storyObject: redactionLevel === 'none' ? storyObject : {
            storyObjectId: storyObject.storyObjectId,
            title: storyObject.title,
            version: storyObject.version,
            sensitivity: storyObject.sensitivity,
            contentSummary: typeof storyObject.content === 'string' ? `[redacted:${storyObject.content.length} chars]` : '[redacted]',
          },
        },
        redactionLevel,
        retentionGranted: 'result_only',
        auditMeta: {
          challengeId: challenge.challengeId,
          sessionId: session.sessionId,
        },
        error: null,
      };
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
      const identityId = normalizeString(input.identityId, 'identityId');
      const storyObjectId = normalizeString(input.storyObjectId, 'storyObjectId');
      const content = input.content;
      if (!content || typeof content !== 'object' || Array.isArray(content)) {
        throw new Error('content must be an object');
      }
      const answers = normalizeArray(input.answers);
      const redactionLevel = normalizeRedactionLevel(input);
      this.host.ensureSeeded?.();
      this.host.ensureReplaySafe?.(requestId, nonce, expiry);
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
      const writeResult = this.host.writeStoryObject(identityId, session.sessionId, storyObjectId, content);
      return {
        requestId,
        status: 'success',
        capability: 'requestStoryWrite',
        executionLocation: 'local',
        result: {
          mode: 'story_write_access',
          storyObjectId,
          writeResult: redactionLevel === 'none' ? writeResult : {
            storyObjectId: writeResult.storyObjectId,
            version: writeResult.version,
            sensitivity: writeResult.sensitivity,
          },
        },
        redactionLevel,
        retentionGranted: 'result_only',
        auditMeta: {
          challengeId: challenge.challengeId,
          sessionId: session.sessionId,
        },
        error: null,
      };
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'requestStoryWrite', error });
    }
  }
}
