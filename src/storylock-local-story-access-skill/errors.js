export const ERROR_DEFS = {
  'SLG-001': ['validation_error', 'Fix the request fields and retry.', true],
  'SLG-002': ['replay_rejected', 'Reissue the request with a fresh expiry.', true],
  'SLG-003': ['challenge_failed', 'Retry with correct challenge answers.', true],
  'SLG-004': ['challenge_locked', 'Wait until retryAfter before trying again.', true],
  'SLG-005': ['session_invalid', 'Restart the challenge flow.', true],
  'SLG-006': ['budget_exhausted', 'Create a new authorized session.', true],
  'SLG-007': ['object_not_found', 'Check the storyObjectId.', false],
  'SLG-008': ['redaction_required', 'Apply the required redaction policy before returning data.', false],
  'SLG-009': ['scope_insufficient', 'Use an authorized scope for this operation.', false],
  'SLG-010': ['secret_unavailable', 'Configure SecretStore before using this capability.', false],
  'SLG-011': ['request_expired', 'Reissue the request with a fresh expiry.', false],
  'SLG-012': ['internal_error', 'Check local logs and retry later.', true],
  'SLG-013': ['replay_detected', 'Generate a new requestId and nonce.', false],
};

export const ERROR_KEY_TO_CODE = {
  VALIDATION_ERROR: 'SLG-001',
  REQUEST_EXPIRED: 'SLG-011',
  CHALLENGE_FAILED: 'SLG-003',
  CHALLENGE_LOCKED: 'SLG-004',
  SESSION_INVALID: 'SLG-005',
  BUDGET_EXHAUSTED: 'SLG-006',
  OBJECT_NOT_FOUND: 'SLG-007',
  REPLAY_DETECTED: 'SLG-013',
  REDACTION_REQUIRED: 'SLG-008',
  SCOPE_INSUFFICIENT: 'SLG-009',
  SECRET_UNAVAILABLE: 'SLG-010',
  CAPABILITY_NOT_AVAILABLE: 'SLG-001',
  INTERNAL_ERROR: 'SLG-012',
};

export function normalizeErrorCode(error) {
  if (error?.code && ERROR_DEFS[error.code]) {
    return error.code;
  }
  if (error?.key && ERROR_KEY_TO_CODE[error.key]) {
    return ERROR_KEY_TO_CODE[error.key];
  }
  if (error?.message === 'REQUEST_EXPIRED') {
    return 'SLG-011';
  }
  if (error?.message?.includes('must be')) {
    return 'SLG-001';
  }
  return 'SLG-012';
}

export function buildErrorPayload(error) {
  const code = normalizeErrorCode(error);
  const [type, suggestedAction, retryable] = ERROR_DEFS[code];
  return {
    code,
    type: error?.type ?? type,
    message: error?.message ?? 'internal error',
    suggestedAction: error?.suggestedAction ?? suggestedAction,
    retryable: error?.retryable ?? retryable,
    ...(error?.retryAfter ? { retryAfter: error.retryAfter } : {}),
  };
}
