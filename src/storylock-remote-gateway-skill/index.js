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

function buildEip712Request(payload) {
  return {
    domain: {
      name: 'StoryLock',
      version: '1-placeholder',
      chainId: payload.chainId ?? 0,
      verifyingContract: payload.verifyingContract ?? '0x0000000000000000000000000000000000000000',
    },
    types: {
      ChallengeSignRequest: [
        { name: 'identityId', type: 'string' },
        { name: 'keyId', type: 'string' },
        { name: 'algorithm', type: 'string' },
        { name: 'payload', type: 'bytes' },
        { name: 'resourceId', type: 'string' },
        { name: 'primaryRole', type: 'string' },
      ],
    },
    value: {
      identityId: ensureString(payload.identityId, 'identityId'),
      keyId: ensureString(payload.keyId, 'keyId'),
      algorithm: ensureAllowedAlgorithm(payload.algorithm),
      payload: typeof payload.payload === 'string' ? `0x${Buffer.from(payload.payload, 'utf8').toString('hex')}` : payload.payload,
      resourceId: payload.resourceId ?? '',
      primaryRole: payload.primaryRole ?? '',
    },
  };
}

export class StoryLockRemoteGateway {
  constructor({ transport }) {
    this.transport = ensureFunction(transport, 'transport');
  }

  async invoke(request) {
    return Promise.resolve(this.transport(request));
  }

  async requestStoryRead(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { redactionPreferred: true },
    });
    return this.invoke({
      requestId: envelope.requestId,
      capability: 'requestStoryRead',
      scope: 'story_read_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        storyObjectId: ensureString(payload.storyObjectId, 'storyObjectId'),
        answers: Array.isArray(payload.answers) ? payload.answers : [],
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    });
  }

  async requestStoryWrite(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { writeReason: 'remote_write_request' },
    });
    return this.invoke({
      requestId: envelope.requestId,
      capability: 'requestStoryWrite',
      scope: 'story_write_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        storyObjectId: ensureString(payload.storyObjectId, 'storyObjectId'),
        content: payload.content,
        answers: Array.isArray(payload.answers) ? payload.answers : [],
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    });
  }

  async requestChallengeSign(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { minAccessLevel: 'L4' },
    });
    return this.invoke({
      requestId: envelope.requestId,
      capability: 'requestChallengeSign',
      scope: 'challenge_sign',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        keyId: ensureString(payload.keyId, 'keyId'),
        algorithm: ensureAllowedAlgorithm(payload.algorithm),
        payload: payload.payload,
        resourceId: payload.resourceId ?? null,
        primaryRole: payload.primaryRole ?? null,
        eip712: buildEip712Request(payload),
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    });
  }

  async requestCapabilityStatus(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { statusOnly: true },
    });
    return this.invoke({
      requestId: envelope.requestId,
      capability: 'requestCapabilityStatus',
      scope: 'capability_status_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        capability: ensureString(payload.capability, 'capability'),
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    });
  }

  async requestPasswordFill(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'audit_meta_only',
      policyHints: { minAccessLevel: 'L4', noRemoteSecretReturn: true },
    });
    return this.invoke({
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
    });
  }

  async requestLocalStoryAssist(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { localProcessingOnly: true },
    });
    return this.invoke({
      requestId: envelope.requestId,
      capability: 'requestLocalStoryAssist',
      scope: 'story_assist_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        storyObjectId: payload.storyObjectId ? ensureString(payload.storyObjectId, 'storyObjectId') : null,
        assistType: ensureString(payload.assistType, 'assistType'),
        prompt: ensureString(payload.prompt, 'prompt'),
        context: payload.context ?? {},
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    });
  }

  async queryStoryMetadata(payload) {
    const envelope = normalizeEnvelope(payload, {
      requestedRetention: 'result_only',
      policyHints: { metadataOnly: true },
    });
    return this.invoke({
      requestId: envelope.requestId,
      capability: 'queryStoryMetadata',
      scope: 'story_metadata_basic',
      payload: {
        identityId: ensureString(payload.identityId, 'identityId'),
        storyObjectId: ensureString(payload.storyObjectId, 'storyObjectId'),
      },
      policyHints: envelope.policyHints,
      requestedRetention: envelope.requestedRetention,
      nonce: envelope.nonce,
      expiry: ensureExpiry(envelope.expiry),
    });
  }
}

export class DelegatedChallengeSignSkill {
  constructor({ gateway }) {
    this.gateway = gateway;
  }

  skillId() {
    return 'delegated_challenge_sign';
  }

  async run({ identityId, keyId, algorithm, payload, resourceId = null, primaryRole = null, requestId, nonce, expiry }) {
    return this.gateway.requestChallengeSign({
      identityId: ensureString(identityId, 'identityId'),
      keyId: ensureString(keyId, 'keyId'),
      algorithm: ensureAllowedAlgorithm(algorithm),
      payload,
      resourceId,
      primaryRole,
      requestId,
      nonce,
      expiry,
    });
  }
}
