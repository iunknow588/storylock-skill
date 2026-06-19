import {
  createDemoEip712Domain,
  createEip712DomainFromEnv,
  createProductionEip712Domain,
  normalizeEip712Domain,
} from './eip712-domain.js';

function ensureFunction(value, fieldName) {
  if (typeof value !== 'function') {
    throw new Error(`${fieldName} must be a function`);
  }
  return value;
}

function ensureString(value, fieldName) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return value.trim();
}

function normalizeEnvelope(payload = {}, defaults = {}) {
  return {
    requestId: ensureString(payload.requestId ?? defaults.requestId, 'requestId'),
    nonce: ensureString(payload.nonce ?? defaults.nonce, 'nonce'),
    expiry: Number(payload.expiry ?? defaults.expiry),
    requestedRetention: payload.requestedRetention ?? defaults.requestedRetention ?? 'result_only',
    policyHints: payload.policyHints ?? defaults.policyHints ?? {},
  };
}

function ensureExpiry(value) {
  if (!Number.isFinite(value)) {
    throw new Error('expiry must be a valid number');
  }
  if (value <= Date.now()) {
    throw new Error('REQUEST_EXPIRED');
  }
  return value;
}

function ensureAllowedAlgorithm(value) {
  const algorithm = ensureString(value, 'algorithm');
  if (!['ed25519', 'secp256k1'].includes(algorithm)) {
    throw new Error('algorithm must be ed25519 or secp256k1');
  }
  return algorithm;
}

function ensureNonceUint256(value) {
  const nonceText = ensureString(String(value), 'nonce');
  if (!/^\d+$/.test(nonceText)) {
    throw new Error('nonce must be a uint256-compatible decimal string');
  }
  return nonceText;
}

function buildEip712Request(payload) {
  const eip712Nonce = payload.eip712Nonce ?? payload.signingNonce ?? '0';
  const domain = normalizeEip712Domain(payload.eip712Domain, payload.eip712DomainDefaults);
  return {
    domain,
    types: {
      StoryLockSignatureRequest: [
        { name: 'action', type: 'string' },
        { name: 'resource', type: 'string' },
        { name: 'scope', type: 'string' },
        { name: 'expiry', type: 'uint256' },
        { name: 'nonce', type: 'uint256' },
        { name: 'requestedBy', type: 'string' },
        { name: 'delegationContext', type: 'string' },
      ],
    },
    value: {
      action: payload.action ?? 'request_signature',
      resource: payload.resource ?? payload.resourceId ?? ensureString(payload.keyId, 'keyId'),
      scope: payload.scope ?? 'signature_basic',
      expiry: String(ensureExpiry(payload.expiry)),
      nonce: ensureNonceUint256(eip712Nonce),
      requestedBy: payload.requestedBy ?? 'remote-agent',
      delegationContext: payload.delegationContext ?? `identity:${ensureString(payload.identityId, 'identityId')}/key:${ensureString(payload.keyId, 'keyId')}`,
    },
  };
}

function redactRemoteValue(value) {
  if (Array.isArray(value)) {
    return value.map(redactRemoteValue);
  }
  if (!value || typeof value !== 'object') {
    return value;
  }
  const redacted = {};
  for (const [key, nested] of Object.entries(value)) {
    if (sensitiveRemoteKey(key)) {
      redacted[key] = '[redacted]';
      continue;
    }
    redacted[key] = redactRemoteValue(nested);
  }
  return redacted;
}

function sensitiveRemoteKey(key) {
  return /^(answers|signingKey|signingKeyBytes|secretBytes|secretValue|password|privateKey|mnemonic|seed|rawSecret|keyMaterial|token|accessToken|refreshToken|apiKey)$/i.test(key);
}

function assertRedactedRemoteValue(value, path = 'response') {
  if (Array.isArray(value)) {
    value.forEach((item, index) => assertRedactedRemoteValue(item, `${path}[${index}]`));
    return;
  }
  if (!value || typeof value !== 'object') {
    return;
  }
  for (const [key, nested] of Object.entries(value)) {
    const nextPath = `${path}.${key}`;
    if (sensitiveRemoteKey(key) && nested !== '[redacted]') {
      const error = new Error(`remote response contains unredacted sensitive field at ${nextPath}`);
      error.code = 'SLG-008';
      error.type = 'redaction_required';
      error.retryable = false;
      throw error;
    }
    assertRedactedRemoteValue(nested, nextPath);
  }
}

function redactRemoteResponse(response) {
  const redacted = redactRemoteValue(response);
  assertRedactedRemoteValue(redacted);
  return redacted;
}

export class StoryLockRemoteGateway {
  constructor({
    transport,
    signatureExecutor = null,
    passwordFillExecutor = null,
    eip712Domain = null,
  }) {
    this.transport = ensureFunction(transport, 'transport');
    this.signatureExecutor = signatureExecutor
      ? ensureFunction(signatureExecutor, 'signatureExecutor')
      : null;
    this.passwordFillExecutor = passwordFillExecutor
      ? ensureFunction(passwordFillExecutor, 'passwordFillExecutor')
      : null;
    this.eip712Domain = eip712Domain;
  }

  async invoke(request) {
    const response = await Promise.resolve(this.transport(request));
    return redactRemoteResponse(response);
  }

  async executeLocal(request, executor) {
    if (!executor) {
      return this.invoke(request);
    }
    const response = await Promise.resolve(executor(request));
    return redactRemoteResponse(response);
  }

  async requestSignature(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { minAccessLevel: 'high' },
    });
    const eip712Domain = payload.eip712Domain ?? this.eip712Domain ?? null;
    return this.executeLocal({
      requestId: envelope.requestId,
      capability: 'requestSignature',
      scope: 'signature_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        keyId: ensureString(payload.keyId, 'keyId'),
        algorithm: ensureAllowedAlgorithm(payload.algorithm),
        payload: payload.payload,
        resourceId: payload.resourceId ?? null,
        primaryRole: payload.primaryRole ?? null,
        eip712: buildEip712Request({
          ...payload,
          eip712Domain,
        }),
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    }, this.signatureExecutor);
  }

  async requestPasswordFill(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'audit_meta_only',
      policyHints: { minAccessLevel: 'L4', noRemoteSecretReturn: true },
    });
    return this.executeLocal({
      requestId: envelope.requestId,
      capability: 'requestPasswordFill',
      scope: 'password_fill_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        credentialRef: ensureString(payload.credentialRef, 'credentialRef'),
        targetOrigin: ensureString(payload.targetOrigin, 'targetOrigin'),
        purpose: payload.purpose ?? 'remote_password_fill',
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    }, this.passwordFillExecutor);
  }

}

export class DelegatedSignatureSkill {
  constructor({ gateway }) {
    this.gateway = gateway;
  }

  skillId() {
    return 'delegated_signature';
  }

  async run({
    identityId,
    keyId,
    algorithm,
    payload,
    resourceId = null,
    primaryRole = null,
    requestId,
    nonce,
    expiry,
    eip712Domain = null,
  }) {
    return this.gateway.requestSignature({
      identityId: ensureString(identityId, 'identityId'),
      keyId: ensureString(keyId, 'keyId'),
      algorithm: ensureAllowedAlgorithm(algorithm),
      payload,
      resourceId,
      primaryRole,
      requestId,
      nonce,
      expiry,
      eip712Domain,
    });
  }
}

export { createHttpRemoteTransport } from './http-transport.js';
export {
  createDemoEip712Domain,
  createEip712DomainFromEnv,
  createProductionEip712Domain,
  normalizeEip712Domain,
} from './eip712-domain.js';
