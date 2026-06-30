import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { DatabaseSync } from 'node:sqlite';
import { MemorySecretStore } from '../../../shared/secret-store.js';
import {
  GridChallengeSkill,
  LocalAuthorizationSkill,
  ObjectStrengthPolicySkill,
} from '../../local-story-access/index.js';
import { StoryLockRemoteGateway } from '../index.js';

function tempDbPath() {
  return join(tmpdir(), `storylock_e2e_${randomUUID().replaceAll('-', '')}.db`);
}

function cleanup(path) {
  for (const suffix of ['', '-wal', '-shm']) {
    rmSync(`${path}${suffix}`, { force: true });
  }
}

function sampleQuestions(count = 9) {
  return Array.from({ length: count }, (_, index) => ({
    questionId: `q-${index + 1}`,
    versionTag: 'v1',
    promptRef: `prompt-${index + 1}`,
    promptText: `Question ${index + 1}`,
    options: [`answer-${index + 1}`, `alt-${index + 1}`],
    answer: `answer-${index + 1}`,
    status: 'active',
  }));
}

function answerForQuestionId(questionId) {
  return `answer-${String(questionId).replace(/^q-/, '')}`;
}

function answersForGrid(grid) {
  return grid.cells.map((cell) => ({
    cellId: cell.cellId,
    answer: answerForQuestionId(cell.questionId),
  }));
}

const dbPath = tempDbPath();
const secretStore = new MemorySecretStore({ developmentMode: true, suppressWarning: true });
const policy = new ObjectStrengthPolicySkill({ dbPath, secretStore });
const grid = new GridChallengeSkill({ host: policy.host });
const auth = new LocalAuthorizationSkill({ host: policy.host });

try {
  policy.host.enrollQuestionSet('id-e2e', sampleQuestions(12));

  const gateway = new StoryLockRemoteGateway({
    transport(request) {
      return request;
    },
    async signatureExecutor(request) {
      const policyResult = await policy.run({
        identityId: request.payload.identityId,
        objectRef: request.payload.keyId,
        objectType: 'signature_key',
        requestedAction: 'signature',
        policyHints: {
          requiredStrength: 'high',
        },
        requestId: `${request.requestId}:policy`,
      });
      assert.equal(policyResult.status, 'success');
      assert.equal(policyResult.result.requiredStrength, 'high');

      const verification = await grid.run({
        identityId: request.payload.identityId,
        objectRef: request.payload.keyId,
        requiredStrength: policyResult.result.requiredStrength,
        requestId: `${request.requestId}:grid`,
        nonce: `${request.nonce}:grid`,
        expiry: request.expiry,
      });
      assert.equal(verification.status, 'success');
      assert.equal(verification.result.grid.requiredCells, 12);

      const authorization = await auth.run({
        identityId: request.payload.identityId,
        objectRef: request.payload.keyId,
        verificationId: verification.result.verificationId,
        allowedAction: 'signature',
        answers: answersForGrid(verification.result.grid),
        requestId: `${request.requestId}:auth`,
      });
      assert.equal(authorization.status, 'success');

      const signatureHash = 'e2e-signature-hash';
      policy.host.recordAudit('signature_authorized', {
        identityId: request.payload.identityId,
        storyObjectId: request.payload.keyId,
        requestId: request.requestId,
        result: 'success',
        redactionLevel: 'result_only',
        hasHighSensitivityFields: true,
        meta: {
          authorizationId: authorization.result.authorizationId,
          signatureHash,
        },
      });

      return {
        requestId: request.requestId,
        capability: request.capability,
        result: {
          authorizationId: authorization.result.authorizationId,
          signature: 'sig-e2e-local',
          signatureHash,
          signingKeyBytes: [1, 2, 3],
          privateKey: 'must-not-leak',
        },
        auditMeta: {
          authorizationId: authorization.result.authorizationId,
        },
      };
    },
  });

  const response = await gateway.requestSignature({
    requestId: 'req-e2e-sign',
    nonce: '10001',
    eip712Nonce: '10001',
    expiry: Date.now() + 60_000,
    identityId: 'id-e2e',
    keyId: 'wallet/main/private_key',
    algorithm: 'ed25519',
    payload: 'sign this payload',
    resourceId: 'wallet/main/private_key',
  });

  assert.equal(response.capability, 'requestSignature');
  assert.equal(response.result.signature, 'sig-e2e-local');
  assert.equal(response.result.signingKeyBytes, '[redacted]');
  assert.equal(response.result.privateKey, '[redacted]');
  assert.match(response.result.authorizationId, /^ses-/);

  const db = new DatabaseSync(dbPath);
  const row = db.prepare(
    `SELECT event_type, identity_id, story_object_id, request_id, result, meta_json
     FROM audit_log
     WHERE event_type = ?
     ORDER BY audit_id DESC
     LIMIT 1`
  ).get('signature_authorized');
  db.close();

  assert.equal(row.identity_id, 'id-e2e');
  assert.equal(row.story_object_id, 'wallet/main/private_key');
  assert.equal(row.request_id, 'req-e2e-sign');
  assert.equal(row.result, 'success');
  assert.equal(JSON.parse(row.meta_json).signatureHash, 'e2e-signature-hash');

  console.log('StoryLock three-layer e2e selftest passed.');
} finally {
  policy.host.close?.();
  cleanup(dbPath);
}
