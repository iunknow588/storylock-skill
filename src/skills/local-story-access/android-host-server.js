import { createHmac, randomUUID } from 'node:crypto';
import { createServer } from 'node:http';
import { MemorySecretStore } from '../../shared/secret-store.js';
import {
  GridChallengeSkill,
  LocalAuthorizationSkill,
  ObjectStrengthPolicySkill,
} from './index.js';
import { StrengthReviewSkill } from '../local-story-processing/index.js';

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

function secretKey(...parts) {
  return parts.map((part) => encodeURIComponent(String(part))).join('/');
}

function buildAndroidProcessingQuestions(count = 24) {
  return Array.from({ length: count }, (_, index) => {
    const n = index + 1;
    return {
      nodeId: `story_${String(n).padStart(2, '0')}`,
      validAnswers: [`answer-${n}`],
      distractors: Array.from({ length: 8 }, (_, offset) => `noise-${n}-${offset + 1}`),
    };
  });
}

function buildAndroidAccessQuestions(count = 24) {
  return Array.from({ length: count }, (_, index) => {
    const n = index + 1;
    return {
      questionId: `q-${n}`,
      versionTag: 'v1',
      promptRef: `prompt-${n}`,
      promptText: `Question ${n}`,
      options: [`answer-${n}`, `alt-${n}-1`, `alt-${n}-2`, `alt-${n}-3`],
      answer: `answer-${n}`,
      status: 'active',
    };
  });
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

function json(res, statusCode, value) {
  res.statusCode = statusCode;
  res.setHeader('content-type', 'application/json; charset=utf-8');
  res.end(JSON.stringify(value, null, 2));
}

function demoSignature(keyMaterial, request) {
  return createHmac('sha256', keyMaterial)
    .update(JSON.stringify({ payload: request.payload.payload, eip712: request.payload.eip712 }))
    .digest('hex');
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
  request,
}) {
  const policyResult = await policy.run({
    identityId,
    objectRef,
    objectType,
    requestedAction,
    requestId: `${request.requestId}:policy`,
  });
  if (policyResult.status !== 'success') {
    return policyResult;
  }
  const verification = await grid.run({
    identityId,
    objectRef,
    requiredStrength: policyResult.result.requiredStrength,
    requestId: `${request.requestId}:grid`,
    nonce: `${request.nonce}:grid`,
    expiry: request.expiry,
  });
  if (verification.status !== 'success') {
    return verification;
  }
  const authorization = await auth.run({
    identityId,
    objectRef,
    verificationId: verification.result.verificationId,
    allowedAction: requestedAction,
    answers: answersForGrid(verification.result.grid, answerMap),
    requestId: `${request.requestId}:auth`,
  });
  if (authorization.status !== 'success') {
    return authorization;
  }
  return { policyResult, verification, authorization };
}

export function createAndroidStoryLockHostServer({
  host = '127.0.0.1',
  port = 0,
  identityId = 'android-demo-001',
  sharedSecret = '',
} = {}) {
  const processingQuestions = buildAndroidProcessingQuestions(24);
  const accessQuestions = buildAndroidAccessQuestions(24);
  const answerMap = answerMapFor(accessQuestions);
  const secretStore = new MemorySecretStore({ developmentMode: true, suppressWarning: true });
  const policy = new ObjectStrengthPolicySkill({ secretStore });
  const grid = new GridChallengeSkill({ host: policy.host });
  const auth = new LocalAuthorizationSkill({ host: policy.host });
  const strengthReview = new StrengthReviewSkill();

  policy.host.enrollQuestionSet(identityId, accessQuestions, {
    questionSetVersion: 'android-v1',
    normalizationVersion: 'nfkc-lower-v1',
    status: 'active',
  });

  const credential = {
    credentialRef: 'site/example',
    targetOrigin: 'https://example.com',
    username: 'android-user',
    password: 'android-password-123',
  };
  const signatureKey = {
    keyId: 'wallet/main/private_key',
    algorithm: 'ed25519',
    keyMaterial: 'android-signing-key-material',
  };
  setJsonSecret(secretStore, secretKey('credential', identityId, credential.credentialRef), credential);
  setJsonSecret(secretStore, secretKey('signatureKey', identityId, signatureKey.keyId), signatureKey);

  const state = {
    processingQuestions: processingQuestions.length,
    accessQuestions: accessQuestions.length,
    strengthReview: null,
    requestCount: 0,
  };

  const server = createServer(async (req, res) => {
    try {
      if (sharedSecret) {
        const received = req.headers['x-storylock-shared-secret'];
        if (received !== sharedSecret) {
          json(res, 401, { status: 'error', message: 'unauthorized' });
          return;
        }
      }

      const url = new URL(req.url ?? '/', `http://${req.headers.host ?? `${host}:${port}`}`);
      if (req.method === 'GET' && url.pathname === '/health') {
        json(res, 200, {
          status: 'ok',
          layer1: {
            mode: 'local_story_processing',
            questionSetReady: state.strengthReview?.questionSetReady ?? false,
            strongestBasicChallengeBits: state.strengthReview?.strongestBasicChallengeBits ?? 0,
          },
          layer2: {
            identityId,
            questionSetVersion: 'android-v1',
            activeQuestionCount: accessQuestions.length,
          },
          stats: {
            requestCount: state.requestCount,
          },
        });
        return;
      }
      if (req.method !== 'POST' || url.pathname !== '/execute') {
        json(res, 404, { status: 'error', message: 'not found' });
        return;
      }

      let body = '';
      for await (const chunk of req) {
        body += chunk;
      }
      const request = JSON.parse(body);
      state.requestCount += 1;

      if (request.capability === 'requestSignature') {
        const flow = await authorizeObject({
          policy,
          grid,
          auth,
          answerMap,
          identityId: request.payload.identityId,
          objectRef: request.payload.keyId,
          objectType: 'signature_key',
          requestedAction: 'signature',
          request,
        });
        if (flow.status === 'error') {
          json(res, 400, flow);
          return;
        }
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
        json(res, 200, {
          requestId: request.requestId,
          status: 'success',
          capability: request.capability,
          executionLocation: 'local',
          result: {
            authorizationId: flow.authorization.result.authorizationId,
            signature,
            privateKey: storedKey.keyMaterial,
            signingKeyBytes: [1, 2, 3, 4],
          },
          redactionLevel: 'none',
          retentionGranted: 'result_only',
          auditMeta: {
            authorizationId: flow.authorization.result.authorizationId,
          },
          error: null,
        });
        return;
      }

      if (request.capability === 'requestPasswordFill') {
        const flow = await authorizeObject({
          policy,
          grid,
          auth,
          answerMap,
          identityId: request.payload.identityId,
          objectRef: request.payload.credentialRef,
          objectType: 'credential',
          requestedAction: 'password_fill',
          request,
        });
        if (flow.status === 'error') {
          json(res, 400, flow);
          return;
        }
        const storedCredential = getJsonSecret(secretStore, secretKey('credential', request.payload.identityId, request.payload.credentialRef));
        json(res, 200, {
          requestId: request.requestId,
          status: 'success',
          capability: request.capability,
          executionLocation: 'local',
          result: {
            authorizationId: flow.authorization.result.authorizationId,
            username: storedCredential.username,
            password: storedCredential.password,
            targetOrigin: storedCredential.targetOrigin,
          },
          redactionLevel: 'none',
          retentionGranted: 'audit_meta_only',
          auditMeta: {
            authorizationId: flow.authorization.result.authorizationId,
          },
          error: null,
        });
        return;
      }

      json(res, 400, {
        requestId: request.requestId ?? `req-${randomUUID()}`,
        status: 'error',
        capability: request.capability ?? 'requestSignature',
        executionLocation: 'local',
        result: null,
        redactionLevel: 'full',
        retentionGranted: 'audit_meta_only',
        auditMeta: {
          timestamp: new Date().toISOString(),
        },
        error: {
          code: 'SLG-001',
          type: 'validation_error',
          message: 'unsupported capability',
          suggestedAction: 'Use requestSignature or requestPasswordFill.',
          retryable: false,
        },
      });
    } catch (error) {
      json(res, 500, { status: 'error', message: error.message });
    }
  });

  return {
    async start() {
      state.strengthReview = await strengthReview.run({ questions: processingQuestions });
      if (!state.strengthReview.questionSetReady) {
        throw new Error('android layer 1 strength review did not reach questionSetReady');
      }
      await new Promise((resolve) => server.listen(port, host, resolve));
      const address = server.address();
      const resolvedPort = typeof address === 'object' && address ? address.port : port;
      return {
        url: `http://${host}:${resolvedPort}`,
        healthUrl: `http://${host}:${resolvedPort}/health`,
        executeUrl: `http://${host}:${resolvedPort}/execute`,
        state,
      };
    },
    async stop() {
      await new Promise((resolve, reject) => server.close((error) => (error ? reject(error) : resolve())));
      policy.host.close?.();
    },
  };
}
