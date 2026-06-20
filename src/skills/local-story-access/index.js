import { createAccessHost, startAccessHostCleanup } from './access-host.js';
import { buildErrorPayload } from './errors.js';

const REPLAY_DRIFT_MS = 30_000;
const MAX_REQUEST_ID_LENGTH = 128;
const MAX_NONCE_LENGTH = 128;
const MAX_ID_LENGTH = 128;
const MAX_ANSWERS = 10;
const MAX_ANSWER_LENGTH = 512;

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

function normalizeObjectType(value) {
  const objectType = normalizeString(value ?? 'generic_secret', 'objectType', { maxLength: 64 });
  if (!['generic_secret', 'credential', 'signature_key', 'file_key', 'story_object'].includes(objectType)) {
    throw new Error('objectType must be generic_secret, credential, signature_key, file_key, or story_object');
  }
  return objectType;
}

function normalizeRequestedAction(value) {
  const requestedAction = normalizeString(value ?? 'authorize', 'requestedAction', { maxLength: 64 });
  if (!['authorize', 'password_fill', 'signature', 'local_processing'].includes(requestedAction)) {
    throw new Error('requestedAction must be authorize, password_fill, signature, or local_processing');
  }
  return requestedAction;
}

function normalizeStrength(value, fieldName = 'requiredStrength') {
  const strength = normalizeString(value, fieldName, { maxLength: 32 });
  if (!['low', 'medium', 'high'].includes(strength)) {
    throw new Error(`${fieldName} must be low, medium, or high`);
  }
  return strength;
}

function resolveStrength({ objectType, requestedAction, policyHints = {} }) {
  if (policyHints.requiredStrength) {
    return normalizeStrength(policyHints.requiredStrength, 'policyHints.requiredStrength');
  }
  if (objectType === 'signature_key' || requestedAction === 'signature') {
    return 'high';
  }
  if (objectType === 'credential' || requestedAction === 'password_fill') {
    return 'medium';
  }
  return 'low';
}

function gridPolicyForStrength(requiredStrength) {
  const strength = normalizeStrength(requiredStrength);
  const requiredCells = {
    low: 3,
    medium: 6,
    high: 9,
  }[strength];
  return {
    gridSize: 9,
    requiredCells,
  };
}

function buildGridCells(challenge) {
  if (Array.isArray(challenge?.cells) && challenge.cells.length > 0) {
    return challenge.cells.map((cell) => ({
      cellId: cell.cellId,
      promptRef: cell.promptRef,
      position: cell.position,
      questionId: cell.questionId,
      versionTag: cell.versionTag,
      promptText: cell.promptText ?? undefined,
      optionDigest: cell.optionDigest ?? undefined,
      questionSetVersion: cell.questionSetVersion,
      normalizationVersion: cell.normalizationVersion,
    }));
  }
  return [];
}

function normalizeAuthorizationAnswers(value) {
  const answers = normalizeArray(value);
  return answers.map((item, index) => {
    if (typeof item === 'string') {
      return { cellId: `cell-${index + 1}`, answer: item };
    }
    return {
      cellId: item.cellId ? normalizeString(item.cellId, `answers[${index}].cellId`, { maxLength: 64 }) : `cell-${index + 1}`,
      answer: normalizeString(item.answer, `answers[${index}].answer`, { maxLength: MAX_ANSWER_LENGTH }),
    };
  });
}

function determineAuthorizationBudgets(allowedAction) {
  if (allowedAction === 'signature' || allowedAction === 'password_fill') {
    return {
      readBudget: 1,
      writeBudget: 0,
    };
  }
  return {
    readBudget: 0,
    writeBudget: 0,
  };
}

export class ObjectStrengthPolicySkill {
  constructor({
    host,
    dbPath,
    secretStore,
    usePlatformSecretStore = false,
    developmentMode = false,
    allowLegacyFallback,
    databaseFactory,
    cleanupIntervalMs,
    cleanupBatchSize,
    cleanupOnError,
  } = {}) {
    this.host = host ?? createAccessHost({
      dbPath,
      secretStore,
      usePlatformSecretStore,
      developmentMode,
      allowLegacyFallback,
      databaseFactory,
      cleanupIntervalMs,
      cleanupBatchSize,
      cleanupOnError,
    });
  }

  skillId() {
    return 'object_strength_policy';
  }

  async run(input = {}) {
    const fallbackRequestId = input?.requestId ?? `req-${Date.now().toString(16)}`;
    try {
      const identityId = normalizeString(input.identityId, 'identityId', { maxLength: MAX_ID_LENGTH });
      const objectRef = normalizeString(input.objectRef ?? input.credentialRef ?? input.keyId, 'objectRef', { maxLength: MAX_ID_LENGTH });
      const objectType = normalizeObjectType(input.objectType);
      const requestedAction = normalizeRequestedAction(input.requestedAction);
      const requiredStrength = resolveStrength({
        objectType,
        requestedAction,
        policyHints: input.policyHints ?? {},
      });
      return {
        requestId: fallbackRequestId,
        status: 'success',
        capability: 'resolveObjectStrength',
        executionLocation: 'local',
        result: {
          identityId,
          objectRef,
          objectType,
          requestedAction,
          requiredStrength,
          gridPolicy: gridPolicyForStrength(requiredStrength),
        },
        redactionLevel: 'none',
        retentionGranted: 'audit_meta_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
        },
        error: null,
      };
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'resolveObjectStrength', error });
    }
  }
}

export class GridChallengeSkill {
  constructor({
    host,
    dbPath,
    secretStore,
    usePlatformSecretStore = false,
    developmentMode = false,
    allowLegacyFallback,
    databaseFactory,
    cleanupIntervalMs,
    cleanupBatchSize,
    cleanupOnError,
  } = {}) {
    this.host = host ?? createAccessHost({
      dbPath,
      secretStore,
      usePlatformSecretStore,
      developmentMode,
      allowLegacyFallback,
      databaseFactory,
      cleanupIntervalMs,
      cleanupBatchSize,
      cleanupOnError,
    });
  }

  skillId() {
    return 'grid_challenge';
  }

  async run(input = {}) {
    const fallbackRequestId = input?.requestId ?? `req-${Date.now().toString(16)}`;
    try {
      const { requestId, nonce, expiry } = normalizeRequestEnvelope(input, fallbackRequestId);
      const identityId = normalizeString(input.identityId, 'identityId', { maxLength: MAX_ID_LENGTH });
      const objectRef = normalizeString(input.objectRef ?? input.credentialRef ?? input.keyId, 'objectRef', { maxLength: MAX_ID_LENGTH });
      const requiredStrength = normalizeStrength(input.requiredStrength ?? 'medium');
      const questionSetVersion = input.questionSetVersion
        ? normalizeString(input.questionSetVersion, 'questionSetVersion', { maxLength: 128 })
        : null;
      this.host.ensureSeeded?.();
      const replay = this.host.ensureReplaySafe?.(requestId, nonce, expiry, {
        capability: 'createGridVerification',
        identityId,
        objectRef,
        requiredStrength,
        questionSetVersion,
      });
      if (replay?.replayed) {
        return replay.response;
      }
      const policy = gridPolicyForStrength(requiredStrength);
      const challenge = this.host.createChallenge(identityId, `grid_${requiredStrength}`, {
        requiredCells: policy.requiredCells,
        questionSetVersion,
      });
      const response = {
        requestId,
        status: 'success',
        capability: 'createGridVerification',
        executionLocation: 'local',
        result: {
          verificationId: challenge.challengeId,
          identityId,
          objectRef,
          requiredStrength,
          grid: {
            ...policy,
            questionSetVersion: challenge.questionSetVersion,
            questionSetVersions: challenge.questionSetVersions,
            normalizationVersion: challenge.normalizationVersion,
            normalizationVersions: challenge.normalizationVersions,
            cells: buildGridCells(challenge),
          },
          expiresAt: challenge.expiresAt,
        },
        redactionLevel: 'none',
        retentionGranted: 'audit_meta_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
          verificationId: challenge.challengeId,
        },
        error: null,
      };
      this.host.storeRequestResponse?.(requestId, response);
      return response;
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'createGridVerification', error });
    }
  }
}

export class LocalAuthorizationSkill {
  constructor({
    host,
    dbPath,
    secretStore,
    usePlatformSecretStore = false,
    developmentMode = false,
    allowLegacyFallback,
    databaseFactory,
    cleanupIntervalMs,
    cleanupBatchSize,
    cleanupOnError,
  } = {}) {
    this.host = host ?? createAccessHost({
      dbPath,
      secretStore,
      usePlatformSecretStore,
      developmentMode,
      allowLegacyFallback,
      databaseFactory,
      cleanupIntervalMs,
      cleanupBatchSize,
      cleanupOnError,
    });
  }

  skillId() {
    return 'local_authorization';
  }

  async run(input = {}) {
    const fallbackRequestId = input?.requestId ?? `req-${Date.now().toString(16)}`;
    try {
      const identityId = normalizeString(input.identityId, 'identityId', { maxLength: MAX_ID_LENGTH });
      const verificationId = normalizeString(input.verificationId, 'verificationId', { maxLength: MAX_ID_LENGTH });
      const objectRef = normalizeString(input.objectRef ?? input.credentialRef ?? input.keyId, 'objectRef', { maxLength: MAX_ID_LENGTH });
      const allowedAction = normalizeRequestedAction(input.allowedAction ?? input.requestedAction ?? 'authorize');
      const answers = normalizeAuthorizationAnswers(input.answers);
      const authorization = this.host.submitChallengeAnswers(identityId, verificationId, answers);
      if (!authorization.approved) {
        throw Object.assign(new Error('local authorization answers did not match'), {
          code: 'SLG-003',
          type: 'authorization_failed',
          retryable: true,
          retryAfter: authorization.retryAfter ?? null,
        });
      }
      const budgets = determineAuthorizationBudgets(allowedAction);
      const session = this.host.issueSession(identityId, {
        challengeId: verificationId,
      }, allowedAction, [objectRef], {
        readBudget: budgets.readBudget,
        writeBudget: budgets.writeBudget,
        ttlMs: input.ttlMs ?? 3 * 60 * 1000,
        sessionType: 'authorization_only',
      });
      return {
        requestId: fallbackRequestId,
        status: 'success',
        capability: 'authorizeLocalAction',
        executionLocation: 'local',
        result: {
          approved: true,
          authorizationId: session.sessionId,
          identityId,
          objectRef,
          allowedAction,
          expiresAt: session.expiresAt,
        },
        redactionLevel: 'none',
        retentionGranted: 'audit_meta_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
          verificationId,
          authorizationId: session.sessionId,
        },
        error: null,
      };
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'authorizeLocalAction', error });
    }
  }
}

export class LocalRevocationSkill {
  constructor({
    host,
    dbPath,
    secretStore,
    usePlatformSecretStore = false,
    developmentMode = false,
    allowLegacyFallback,
    databaseFactory,
    cleanupIntervalMs,
    cleanupBatchSize,
    cleanupOnError,
  } = {}) {
    this.host = host ?? createAccessHost({
      dbPath,
      secretStore,
      usePlatformSecretStore,
      developmentMode,
      allowLegacyFallback,
      databaseFactory,
      cleanupIntervalMs,
      cleanupBatchSize,
      cleanupOnError,
    });
  }

  skillId() {
    return 'local_revocation';
  }

  async run(input = {}) {
    const fallbackRequestId = input?.requestId ?? `req-${Date.now().toString(16)}`;
    try {
      const identityId = normalizeString(input.identityId, 'identityId', { maxLength: MAX_ID_LENGTH });
      const reason = input.reason
        ? normalizeString(input.reason, 'reason', { maxLength: 128 })
        : 'manual_revocation';
      let result;
      if (input.authorizationId) {
        const authorizationId = normalizeString(input.authorizationId, 'authorizationId', { maxLength: MAX_ID_LENGTH });
        result = this.host.revokeSession(identityId, authorizationId, { reason });
      } else {
        const verificationId = normalizeString(input.verificationId, 'verificationId', { maxLength: MAX_ID_LENGTH });
        result = this.host.revokeChallenge(identityId, verificationId, { reason });
      }
      return {
        requestId: fallbackRequestId,
        status: 'success',
        capability: 'revokeLocalAuthorization',
        executionLocation: 'local',
        result: {
          identityId,
          ...result,
        },
        redactionLevel: 'none',
        retentionGranted: 'audit_meta_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
          targetType: result.targetType,
          reason,
        },
        error: null,
      };
    } catch (error) {
      return toErrorResponse({ requestId: fallbackRequestId, capability: 'revokeLocalAuthorization', error });
    }
  }
}

export { createAccessHost, startAccessHostCleanup };
