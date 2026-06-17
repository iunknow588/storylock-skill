import assert from 'node:assert/strict';
import { StoryLockRemoteGateway } from '../index.js';

const calls = [];
const gateway = new StoryLockRemoteGateway({
  transport(request) {
    calls.push(request);
    return request;
  },
});

const sign = await gateway.requestSignature({
  requestId: 'req-sign',
  nonce: 'nonce-sign',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
  resourceId: 'resource-1',
  primaryRole: 'private_key',
  eip712Nonce: '1001',
});

assert.equal(sign.capability, 'requestSignature');
assert.equal(sign.scope, 'signature_basic');
assert.equal(sign.payload.eip712.domain.name, 'StoryLock');
assert.equal(sign.payload.eip712.types.StoryLockSignatureRequest[0].name, 'action');
assert.equal(sign.payload.eip712.value.action, 'request_signature');
assert.equal(sign.payload.eip712.value.resource, 'resource-1');
assert.equal(sign.payload.eip712.value.nonce, '1001');

const passwordFill = await gateway.requestPasswordFill({
  requestId: 'req-password-fill',
  nonce: 'nonce-password-fill',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  credentialRef: 'cred-1',
  targetOrigin: 'https://example.com',
});

assert.equal(passwordFill.capability, 'requestPasswordFill');
assert.equal(passwordFill.scope, 'password_fill_basic');
assert.equal(passwordFill.requestedRetention, 'audit_meta_only');
assert.equal(passwordFill.policyHints.noRemoteSecretReturn, true);

const redactingGateway = new StoryLockRemoteGateway({
  transport(request) {
    return {
      requestId: request.requestId,
      capability: request.capability,
      result: {
        password: 'plain-password',
        privateKey: 'plain-private-key',
        secretBytes: [1, 2, 3],
        secretReference: 'wallet/main/private_key',
        nested: {
          answers: [{ answer: 'grid answer' }],
        },
      },
    };
  },
});

const redacted = await redactingGateway.requestSignature({
  requestId: 'req-redacted',
  nonce: 'nonce-redacted',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
  eip712Nonce: '1002',
});

assert.equal(redacted.result.password, '[redacted]');
assert.equal(redacted.result.privateKey, '[redacted]');
assert.equal(redacted.result.secretBytes, '[redacted]');
assert.equal(redacted.result.secretReference, 'wallet/main/private_key');
assert.equal(redacted.result.nested.answers, '[redacted]');

const localGateway = new StoryLockRemoteGateway({
  transport(request) {
    return request;
  },
  signatureExecutor(request) {
    return {
      requestId: request.requestId,
      capability: request.capability,
      result: {
        signature: 'sig-from-local-executor',
        signingKeyBytes: [9, 9, 9],
        secretReference: request.payload.resourceId,
      },
    };
  },
});

const localResult = await localGateway.requestSignature({
  requestId: 'req-local-sign',
  nonce: 'nonce-local-sign',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
  resourceId: 'resource-1',
  eip712Nonce: '1003',
});

assert.equal(localResult.result.signature, 'sig-from-local-executor');
assert.equal(localResult.result.signingKeyBytes, '[redacted]');

await assert.rejects(
  () => gateway.requestSignature({
    requestId: 'req-bad-alg',
    nonce: 'nonce-bad-alg',
    expiry: Date.now() + 10_000,
    identityId: 'id-1',
    keyId: 'key-1',
    algorithm: 'md5',
    payload: 'hello',
  }),
  /algorithm must be ed25519 or secp256k1/,
);

await assert.rejects(() => gateway.requestSignature({
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
}), /requestId must be a non-empty string/);

assert.equal(calls.length, 2);
console.log('StoryLock remote gateway selftest passed.');
