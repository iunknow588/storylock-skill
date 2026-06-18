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

function isZeroAddress(value) {
  return /^0x0{40}$/i.test(value);
}

function normalizeEip712Domain(domain = {}, defaults = {}) {
  const input = domain ?? {};
  const fallback = defaults ?? {};
  const version = ensureString(input.version ?? fallback.version ?? '1-placeholder', 'eip712.domain.version');
  const chainId = Number(input.chainId ?? fallback.chainId ?? 1);
  if (!Number.isInteger(chainId) || chainId < 0) {
    throw new Error('eip712.domain.chainId must be a non-negative integer');
  }
  const verifyingContract = ensureString(
    input.verifyingContract ?? fallback.verifyingContract ?? '0x0000000000000000000000000000000000000000',
    'eip712.domain.verifyingContract',
  );
  if (!/^0x[0-9a-fA-F]{40}$/.test(verifyingContract)) {
    throw new Error('eip712.domain.verifyingContract must be a 20-byte hex address');
  }
  const environment = ensureString(
    input.environment ?? fallback.environment ?? (version.includes('placeholder') ? 'demo' : 'production'),
    'eip712.domain.environment',
  );
  if (!['demo', 'test', 'production'].includes(environment)) {
    throw new Error('eip712.domain.environment must be demo, test, or production');
  }
  if (environment === 'production') {
    if (/placeholder/i.test(version)) {
      throw new Error('production EIP-712 domain must not use a placeholder version');
    }
    if (chainId === 0) {
      throw new Error('production EIP-712 domain must use a non-zero chainId');
    }
    if (isZeroAddress(verifyingContract)) {
      throw new Error('production EIP-712 domain must not use the zero verifyingContract');
    }
  }
  return {
    name: ensureString(input.name ?? fallback.name ?? 'StoryLock', 'eip712.domain.name'),
    version,
    chainId,
    verifyingContract,
    environment,
  };
}

export function createEip712DomainFromEnv(env = process.env, {
  prefix = 'STORYLOCK_EIP712_',
  defaultEnvironment = 'demo',
} = {}) {
  const environment = ensureString(
    env[`${prefix}ENVIRONMENT`] ?? env[`${prefix}ENV`] ?? defaultEnvironment,
    `${prefix}ENVIRONMENT`,
  );
  const input = {
    name: env[`${prefix}NAME`] ?? 'StoryLock',
    version: env[`${prefix}VERSION`],
    chainId: env[`${prefix}CHAIN_ID`],
    verifyingContract: env[`${prefix}VERIFYING_CONTRACT`],
    environment,
  };
  if (environment === 'production') {
    const missing = [
      ['VERSION', input.version],
      ['CHAIN_ID', input.chainId],
      ['VERIFYING_CONTRACT', input.verifyingContract],
    ].filter(([, value]) => value === undefined || value === null || String(value).trim() === '');
    if (missing.length > 0) {
      throw new Error(`production EIP-712 config requires ${missing.map(([name]) => `${prefix}${name}`).join(', ')}`);
    }
  }
  return normalizeEip712Domain(input);
}

export function createProductionEip712Domain({
  name = 'StoryLock',
  version,
  chainId,
  verifyingContract,
} = {}) {
  return normalizeEip712Domain(
    {
      name,
      version,
      chainId,
      verifyingContract,
      environment: 'production',
    },
    {
      name,
      version,
      chainId,
      verifyingContract,
      environment: 'production',
    },
  );
}

export function createDemoEip712Domain({
  name = 'StoryLock',
  version = '1-placeholder',
  chainId = 1,
  verifyingContract = '0x0000000000000000000000000000000000000000',
} = {}) {
  return normalizeEip712Domain({
    name,
    version,
    chainId,
    verifyingContract,
    environment: version.includes('placeholder') ? 'demo' : 'test',
  });
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
    if (/^(answers|signingKey|signingKeyBytes|secretBytes|secretValue|password|privateKey|mnemonic|seed|rawSecret|keyMaterial)$/i.test(key)) {
      redacted[key] = '[redacted]';
      continue;
    }
    redacted[key] = redactRemoteValue(nested);
  }
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
    return redactRemoteValue(response);
  }

  async executeLocal(request, executor) {
    if (!executor) {
      return this.invoke(request);
    }
    const response = await Promise.resolve(executor(request));
    return redactRemoteValue(response);
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
