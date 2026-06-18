import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  createDemoEip712Domain,
  createEip712DomainFromEnv,
  createProductionEip712Domain,
  StoryLockRemoteGateway,
} from '../index.js';

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
assert.equal(sign.payload.eip712.domain.environment, 'demo');
assert.equal(sign.payload.eip712.types.StoryLockSignatureRequest[0].name, 'action');
assert.equal(sign.payload.eip712.value.action, 'request_signature');
assert.equal(sign.payload.eip712.value.resource, 'resource-1');
assert.equal(sign.payload.eip712.value.nonce, '1001');

const productionSign = await gateway.requestSignature({
  requestId: 'req-sign-prod',
  nonce: 'nonce-sign-prod',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
  eip712Domain: {
    name: 'StoryLock',
    version: '1.0.0',
    chainId: 11155111,
    verifyingContract: '0x0000000000000000000000000000000000000123',
    environment: 'production',
  },
});

assert.equal(productionSign.payload.eip712.domain.version, '1.0.0');
assert.equal(productionSign.payload.eip712.domain.environment, 'production');

const productionGateway = new StoryLockRemoteGateway({
  transport(request) {
    return request;
  },
  eip712Domain: {
    name: 'StoryLock',
    version: '1.1.0',
    chainId: 11155111,
    verifyingContract: '0x0000000000000000000000000000000000000456',
    environment: 'production',
  },
});

const defaultProductionSign = await productionGateway.requestSignature({
  requestId: 'req-sign-prod-default',
  nonce: 'nonce-sign-prod-default',
  expiry: Date.now() + 10_000,
  identityId: 'id-1',
  keyId: 'key-1',
  algorithm: 'ed25519',
  payload: 'hello',
});

assert.equal(defaultProductionSign.payload.eip712.domain.version, '1.1.0');
assert.equal(defaultProductionSign.payload.eip712.domain.chainId, 11155111);
assert.equal(defaultProductionSign.payload.eip712.domain.verifyingContract, '0x0000000000000000000000000000000000000456');
assert.equal(defaultProductionSign.payload.eip712.domain.environment, 'production');

await assert.rejects(
  () => gateway.requestSignature({
    requestId: 'req-prod-placeholder',
    nonce: 'nonce-prod-placeholder',
    expiry: Date.now() + 10_000,
    identityId: 'id-1',
    keyId: 'key-1',
    algorithm: 'ed25519',
    payload: 'hello',
    eip712Domain: {
      version: '1-placeholder',
      chainId: 11155111,
      verifyingContract: '0x0000000000000000000000000000000000000123',
      environment: 'production',
    },
  }),
  /production EIP-712 domain must not use a placeholder version/,
);

await assert.rejects(
  () => gateway.requestSignature({
    requestId: 'req-prod-zero-chain',
    nonce: 'nonce-prod-zero-chain',
    expiry: Date.now() + 10_000,
    identityId: 'id-1',
    keyId: 'key-1',
    algorithm: 'ed25519',
    payload: 'hello',
    eip712Domain: {
      version: '1.0.0',
      chainId: 0,
      verifyingContract: '0x0000000000000000000000000000000000000123',
      environment: 'production',
    },
  }),
  /production EIP-712 domain must use a non-zero chainId/,
);

await assert.rejects(
  () => gateway.requestSignature({
    requestId: 'req-prod-zero-contract',
    nonce: 'nonce-prod-zero-contract',
    expiry: Date.now() + 10_000,
    identityId: 'id-1',
    keyId: 'key-1',
    algorithm: 'ed25519',
    payload: 'hello',
    eip712Domain: {
      version: '1.0.0',
      chainId: 11155111,
      verifyingContract: '0x0000000000000000000000000000000000000000',
      environment: 'production',
    },
  }),
  /production EIP-712 domain must not use the zero verifyingContract/,
);

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

const demoDomain = createDemoEip712Domain();
assert.equal(demoDomain.environment, 'demo');
assert.equal(demoDomain.version, '1-placeholder');

const productionDomain = createProductionEip712Domain({
  version: '1.2.3',
  chainId: 11155111,
  verifyingContract: '0x0000000000000000000000000000000000000123',
});
assert.equal(productionDomain.environment, 'production');
assert.equal(productionDomain.chainId, 11155111);
assert.equal(productionDomain.verifyingContract, '0x0000000000000000000000000000000000000123');

const envProductionDomain = createEip712DomainFromEnv({
  STORYLOCK_EIP712_ENVIRONMENT: 'production',
  STORYLOCK_EIP712_NAME: 'StoryLock Env',
  STORYLOCK_EIP712_VERSION: '2.0.0',
  STORYLOCK_EIP712_CHAIN_ID: '11155111',
  STORYLOCK_EIP712_VERIFYING_CONTRACT: '0x0000000000000000000000000000000000000789',
});
assert.equal(envProductionDomain.name, 'StoryLock Env');
assert.equal(envProductionDomain.environment, 'production');
assert.equal(envProductionDomain.version, '2.0.0');
assert.equal(envProductionDomain.chainId, 11155111);
assert.equal(envProductionDomain.verifyingContract, '0x0000000000000000000000000000000000000789');

assert.throws(
  () => createEip712DomainFromEnv({
    STORYLOCK_EIP712_ENVIRONMENT: 'production',
    STORYLOCK_EIP712_VERSION: '2.0.0',
  }),
  /production EIP-712 config requires STORYLOCK_EIP712_CHAIN_ID, STORYLOCK_EIP712_VERIFYING_CONTRACT/,
);

const checkScript = fileURLToPath(new URL('./check-eip712-config.mjs', import.meta.url));
const demoCheck = JSON.parse(execFileSync(process.execPath, [checkScript], {
  encoding: 'utf8',
  windowsHide: true,
}));
assert.equal(demoCheck.status, 'success');
assert.equal(demoCheck.productionReady, false);
assert.equal(demoCheck.domain.environment, 'demo');

assert.throws(
  () => execFileSync(process.execPath, [checkScript, '--environment', 'production'], {
    encoding: 'utf8',
    windowsHide: true,
    stdio: 'pipe',
  }),
  /production EIP-712 config requires/,
);

const productionCheck = JSON.parse(execFileSync(process.execPath, [checkScript, '--environment', 'production'], {
  env: {
    ...process.env,
    STORYLOCK_EIP712_VERSION: '3.0.0',
    STORYLOCK_EIP712_CHAIN_ID: '11155111',
    STORYLOCK_EIP712_VERIFYING_CONTRACT: '0x0000000000000000000000000000000000000999',
  },
  encoding: 'utf8',
  windowsHide: true,
}));
assert.equal(productionCheck.status, 'success');
assert.equal(productionCheck.productionReady, true);
assert.equal(productionCheck.domain.environment, 'production');
assert.equal(productionCheck.domain.version, '3.0.0');

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

assert.equal(calls.length, 3);
console.log('StoryLock remote gateway selftest passed.');
