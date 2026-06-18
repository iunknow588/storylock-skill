import { createHmac, randomUUID } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { MemorySecretStore } from '../../shared/secret-store.js';
import {
  GridChallengeSkill,
  LocalAuthorizationSkill,
  ObjectStrengthPolicySkill,
} from '../index.js';
import {
  StoryLockRemoteGateway,
  createDemoEip712Domain,
} from '../../storylock-remote-gateway-skill/index.js';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const defaultConfigPath = resolve(scriptDir, '../assets/demo-story-config.json');

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function requireArray(value, fieldName) {
  if (!Array.isArray(value) || value.length === 0) {
    throw new Error(`${fieldName} must be a non-empty array`);
  }
  return value;
}

function secretKey(...parts) {
  return parts.map((part) => encodeURIComponent(String(part))).join('/');
}

function setJsonSecret(secretStore, key, value) {
  secretStore.setSecret(key, Buffer.from(JSON.stringify(value), 'utf8'));
}

function getJsonSecret(secretStore, key) {
  const value = secretStore.getSecret(key);
  if (!value) {
    throw new Error(`secret not found: ${key}`);
  }
  return JSON.parse(value.toString('utf8'));
}

function answerMapFor(questions) {
  return new Map(questions.map((question) => [question.questionId, question.answer]));
}

function answersForGrid(grid, answerMap) {
  return grid.cells.map((cell) => ({
    cellId: cell.cellId,
    answer: answerMap.get(cell.questionId),
  }));
}

function printStep(title) {
  console.log(`\n== ${title} ==`);
}

function printJson(value) {
  console.log(JSON.stringify(value, null, 2));
}

function summarizeChallenge(verification) {
  return {
    verificationId: verification.result.verificationId,
    requiredStrength: verification.result.requiredStrength,
    requiredCells: verification.result.grid.requiredCells,
    questionSetVersion: verification.result.grid.questionSetVersion,
    cells: verification.result.grid.cells.map((cell) => ({
      cellId: cell.cellId,
      questionId: cell.questionId,
      promptText: cell.promptText,
      optionDigest: cell.optionDigest,
    })),
  };
}

function assertSuccess(response, label) {
  if (response.status !== 'success') {
    throw new Error(`${label} failed: ${JSON.stringify(response.error ?? response, null, 2)}`);
  }
  return response;
}

function consumeReadSession(host, identityId, authorizationId, objectRef) {
  const row = host.db.prepare(
    `SELECT session_id, identity_id, resource_scope_json, read_budget, status, expires_at
     FROM session_store
     WHERE session_id = ?`
  ).get(authorizationId);
  if (!row || row.identity_id !== identityId) {
    throw new Error('authorization session does not belong to this identity');
  }
  if (row.status !== 'session_active' || row.expires_at <= Date.now()) {
    throw new Error('authorization session is not active');
  }
  const resourceScope = JSON.parse(row.resource_scope_json);
  if (!resourceScope.includes(objectRef)) {
    throw new Error('authorization session does not cover the requested object');
  }
  if (row.read_budget < 1) {
    throw new Error('authorization session has no read budget left');
  }
  const nextBudget = row.read_budget - 1;
  host.db.prepare(
    `UPDATE session_store
     SET read_budget = ?, status = ?
     WHERE session_id = ?`
  ).run(nextBudget, nextBudget === 0 ? 'session_consumed' : 'session_active', authorizationId);
}

function demoSignature(keyMaterial, request) {
  const payload = JSON.stringify({
    payload: request.payload.payload,
    eip712: request.payload.eip712,
  });
  return createHmac('sha256', keyMaterial).update(payload).digest('hex');
}

async function authorizeObject({
  policy,
  grid,
  auth,
  answerMap,
  identityId,
  objectRef,
  objectType,
  requestedAction,
  requestPrefix,
}) {
  const policyResult = assertSuccess(await policy.run({
    identityId,
    objectRef,
    objectType,
    requestedAction,
    requestId: `${requestPrefix}:policy`,
  }), `${requestPrefix} policy`);

  const verification = assertSuccess(await grid.run({
    identityId,
    objectRef,
    requiredStrength: policyResult.result.requiredStrength,
    requestId: `${requestPrefix}:grid:${randomUUID()}`,
    nonce: `${requestPrefix}:nonce:${randomUUID()}`,
    expiry: Date.now() + 60_000,
  }), `${requestPrefix} grid`);

  printStep(`${requestPrefix} challenge`);
  printJson(summarizeChallenge(verification));

  const answers = answersForGrid(verification.result.grid, answerMap);
  const authorization = assertSuccess(await auth.run({
    identityId,
    objectRef,
    verificationId: verification.result.verificationId,
    allowedAction: requestedAction,
    answers,
    requestId: `${requestPrefix}:auth:${randomUUID()}`,
  }), `${requestPrefix} authorization`);

  return {
    policyResult,
    verification,
    authorization,
    answers,
  };
}

async function main() {
  const configPath = resolve(process.argv[2] ?? defaultConfigPath);
  const config = readJson(configPath);
  const questions = requireArray(config.questions, 'questions');
  if (questions.length < 24) {
    throw new Error('demo config should contain at least 24 questions');
  }

  const identityId = config.identityId;
  const secretStore = new MemorySecretStore({ developmentMode: true, suppressWarning: true });
  const policy = new ObjectStrengthPolicySkill({ secretStore });
  const grid = new GridChallengeSkill({ host: policy.host });
  const auth = new LocalAuthorizationSkill({ host: policy.host });
  const answerMap = answerMapFor(questions);

  printStep('load demo story');
  console.log(`config: ${configPath}`);
  console.log(`identityId: ${identityId}`);
  console.log(`storyId: ${config.storyId ?? '(none)'}`);
  console.log(`questions: ${questions.length}`);

  policy.host.enrollQuestionSet(identityId, questions, {
    questionSetVersion: config.questionSetVersion ?? 'demo-story-v1',
    normalizationVersion: config.normalizationVersion ?? 'nfkc-lower-v1',
    status: 'active',
  });

  for (const credential of requireArray(config.credentials, 'credentials')) {
    setJsonSecret(secretStore, secretKey('credential', identityId, credential.credentialRef), {
      targetOrigin: credential.targetOrigin,
      username: credential.username,
      password: credential.password,
    });
  }
  for (const key of requireArray(config.signatureKeys, 'signatureKeys')) {
    setJsonSecret(secretStore, secretKey('signatureKey', identityId, key.keyId), {
      algorithm: key.algorithm,
      keyMaterial: key.keyMaterial,
    });
  }

  console.log(`credentials stored: ${config.credentials.length}`);
  console.log(`signature keys stored: ${config.signatureKeys.length}`);

  const credential = config.credentials[0];
  const passwordAuthorization = await authorizeObject({
    policy,
    grid,
    auth,
    answerMap,
    identityId,
    objectRef: credential.credentialRef,
    objectType: 'credential',
    requestedAction: 'password_fill',
    requestPrefix: 'password-fill',
  });
  consumeReadSession(policy.host, identityId, passwordAuthorization.authorization.result.authorizationId, credential.credentialRef);
  const realCredential = getJsonSecret(secretStore, secretKey('credential', identityId, credential.credentialRef));

  printStep('password fill local result');
  printJson({
    authorizationId: passwordAuthorization.authorization.result.authorizationId,
    credentialRef: credential.credentialRef,
    targetOrigin: realCredential.targetOrigin,
    username: realCredential.username,
    password: realCredential.password,
  });

  const signatureKey = config.signatureKeys[0];
  const gateway = new StoryLockRemoteGateway({
    transport(request) {
      return request;
    },
    eip712Domain: createDemoEip712Domain(),
    async signatureExecutor(request) {
      const flow = await authorizeObject({
        policy,
        grid,
        auth,
        answerMap,
        identityId: request.payload.identityId,
        objectRef: request.payload.keyId,
        objectType: 'signature_key',
        requestedAction: 'signature',
        requestPrefix: 'signature',
      });
      consumeReadSession(policy.host, request.payload.identityId, flow.authorization.result.authorizationId, request.payload.keyId);
      const storedKey = getJsonSecret(secretStore, secretKey('signatureKey', request.payload.identityId, request.payload.keyId));
      const signature = demoSignature(storedKey.keyMaterial, request);
      policy.host.recordAudit('signature_authorized', {
        identityId: request.payload.identityId,
        storyObjectId: request.payload.keyId,
        requestId: request.requestId,
        result: 'success',
        redactionLevel: 'result_only',
        hasHighSensitivityFields: true,
        meta: {
          authorizationId: flow.authorization.result.authorizationId,
          signatureHash: signature,
        },
      });
      return {
        requestId: request.requestId,
        capability: request.capability,
        result: {
          authorizationId: flow.authorization.result.authorizationId,
          algorithm: storedKey.algorithm,
          signature,
          signingKeyBytes: storedKey.keyMaterial,
        },
        auditMeta: {
          authorizationId: flow.authorization.result.authorizationId,
        },
      };
    },
  });

  const signatureResponse = await gateway.requestSignature({
    requestId: `req-demo-signature-${randomUUID()}`,
    nonce: String(Date.now()),
    eip712Nonce: String(Date.now()),
    expiry: Date.now() + 60_000,
    identityId,
    keyId: signatureKey.keyId,
    algorithm: signatureKey.algorithm,
    payload: 'demo payload to sign',
    resourceId: signatureKey.keyId,
  });

  printStep('signature remote result');
  printJson(signatureResponse);

  printStep('audit summary');
  const auditRows = policy.host.db.prepare(
    `SELECT event_type, identity_id, story_object_id, request_id, result, redaction_level, error_code, meta_json
     FROM audit_log
     ORDER BY audit_id`
  ).all();
  printJson(auditRows.map((row) => ({
    ...row,
    meta_json: row.meta_json ? JSON.parse(row.meta_json) : null,
  })));

  policy.host.close?.();
}

main().catch((error) => {
  console.error(error.stack ?? error.message);
  process.exitCode = 1;
});
