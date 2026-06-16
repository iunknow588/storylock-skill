import assert from 'node:assert/strict';
import { StoryLockRemoteGateway } from '../index.js';

const calls = [];
const gateway = new StoryLockRemoteGateway({
  transport(request) {
    calls.push(request);
    return request;
  },
});

const sign = await gateway.requestChallengeSign({
  requestId: 'req-sign',
  nonce: 'nonce-sign',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
  resourceId: 'resource-1',
  primaryRole: 'private_key',
});

assert.equal(sign.capability, 'requestChallengeSign');
assert.equal(sign.payload.eip712.domain.name, 'StoryLock');
assert.equal(sign.payload.eip712.value.payload, '0x68656c6c6f');

const status = await gateway.requestCapabilityStatus({
  requestId: 'req-status',
  nonce: 'nonce-status',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  capability: 'requestStoryRead',
});

assert.equal(status.capability, 'requestCapabilityStatus');
assert.equal(status.scope, 'capability_status_basic');

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

const assist = await gateway.requestLocalStoryAssist({
  requestId: 'req-assist',
  nonce: 'nonce-assist',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  storyObjectId: 'story-001',
  assistType: 'summarize',
  prompt: 'Summarize locally',
});

assert.equal(assist.capability, 'requestLocalStoryAssist');
assert.equal(assist.scope, 'story_assist_basic');
assert.equal(assist.policyHints.localProcessingOnly, true);

await assert.rejects(
  () => gateway.requestChallengeSign({
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

await assert.rejects(
  () => gateway.requestStoryRead({
    identityId: 'id-1',
    storyObjectId: 'story-001',
  }),
  /requestId must be a non-empty string/,
);

assert.equal(calls.length, 4);
console.log('StoryLock remote gateway selftest passed.');
