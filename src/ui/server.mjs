import { createHmac, randomUUID } from 'node:crypto';
import { createReadStream, existsSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { MemorySecretStore } from '../shared/secret-store.js';
import {
  GridChallengeSkill,
  LocalAuthorizationSkill,
  ObjectStrengthPolicySkill,
} from '../storylock-local-story-access-skill/index.js';
import {
  StoryLockRemoteGateway,
  createDemoEip712Domain,
} from '../storylock-remote-gateway-skill/index.js';
import { handleStoryLockGatewayRequest } from '../storylock-remote-gateway-skill/vercel-handler.js';

const uiRoot = dirname(fileURLToPath(import.meta.url));
const publicRoot = resolve(uiRoot, '../yian-web/public');
const configPath = resolve(uiRoot, '../storylock-local-story-access-skill/assets/demo-story-config.json');
const port = Number(process.env.PORT || 4174);
const host = String(process.env.HOST || process.env.STORYLOCK_UI_HOST || '127.0.0.1').trim();
const gatewayBaseUrl = String(process.env.STORYLOCK_UI_GATEWAY_BASE_URL || '').trim() || null;

const mimeByExtension = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
};

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function readBody(req) {
  return new Promise((resolveBody, reject) => {
    let body = '';
    req.setEncoding('utf8');
    req.on('data', (chunk) => {
      body += chunk;
      if (body.length > 2_000_000) {
        req.destroy(new Error('request body too large'));
      }
    });
    req.on('end', () => resolveBody(body));
    req.on('error', reject);
  });
}

function sendJson(res, statusCode, value) {
  res.writeHead(statusCode, { 'content-type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify(value, null, 2));
}

function sendError(res, statusCode, error) {
  sendJson(res, statusCode, {
    status: 'error',
    message: error.message,
    stack: process.env.NODE_ENV === 'production' ? undefined : error.stack,
  });
}

function safeSkillPublicPath(urlPath) {
  const safePath = normalize(decodeURIComponent(urlPath)).replace(/^(\.\.[/\\])+/, '');
  const relativePath = safePath === '/'
    || safePath === '\\'
    || safePath === ''
    || safePath === '/skill-demo.html'
    || safePath === '\\skill-demo.html'
    ? 'index.html'
    : safePath.replace(/^[/\\]/, '');
  return join(publicRoot, relativePath);
}

async function proxyJson(pathname) {
  if (!gatewayBaseUrl) {
    throw new Error('STORYLOCK_UI_GATEWAY_BASE_URL is not configured');
  }
  const response = await fetch(new URL(pathname, gatewayBaseUrl));
  const body = await response.json();
  if (!response.ok || body.status === 'error') {
    throw new Error(body.message ?? `proxy request failed: ${response.status}`);
  }
  return body;
}

function validateConfig(config) {
  if (!config || typeof config !== 'object') {
    throw new Error('config must be an object');
  }
  if (!String(config.identityId ?? '').trim()) {
    throw new Error('identityId is required');
  }
  if (!Array.isArray(config.questions) || config.questions.length < 24) {
    throw new Error('questions must contain at least 24 items');
  }
  if (!Array.isArray(config.credentials) || config.credentials.length < 1) {
    throw new Error('credentials must contain at least one item');
  }
  if (!Array.isArray(config.signatureKeys) || config.signatureKeys.length < 1) {
    throw new Error('signatureKeys must contain at least one item');
  }
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
    throw new Error(`${label} failed: ${JSON.stringify(response.error ?? response)}`);
  }
  return response;
}

function consumeReadSession(host, identityId, authorizationId, objectRef) {
  const row = host.db.prepare(
    `SELECT session_id, identity_id, resource_scope_json, read_budget, status, expires_at
     FROM session_store
     WHERE session_id = ?`
  ).get(authorizationId);
  if (!row || row.identity_id !== identityId || row.status !== 'session_active' || row.expires_at <= Date.now()) {
    throw new Error('authorization session is not active');
  }
  const resourceScope = JSON.parse(row.resource_scope_json);
  if (!resourceScope.includes(objectRef) || row.read_budget < 1) {
    throw new Error('authorization session cannot read this object');
  }
  const nextBudget = row.read_budget - 1;
  host.db.prepare('UPDATE session_store SET read_budget = ?, status = ? WHERE session_id = ?')
    .run(nextBudget, nextBudget === 0 ? 'session_consumed' : 'session_active', authorizationId);
}

function demoSignature(keyMaterial, request) {
  return createHmac('sha256', keyMaterial)
    .update(JSON.stringify({ payload: request.payload.payload, eip712: request.payload.eip712 }))
    .digest('hex');
}

async function authorizeObject({ policy, grid, auth, answerMap, identityId, objectRef, objectType, requestedAction, requestPrefix }) {
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
  const authorization = assertSuccess(await auth.run({
    identityId,
    objectRef,
    verificationId: verification.result.verificationId,
    allowedAction: requestedAction,
    answers: answersForGrid(verification.result.grid, answerMap),
    requestId: `${requestPrefix}:auth:${randomUUID()}`,
  }), `${requestPrefix} authorization`);
  return { policyResult, verification, authorization };
}

async function runDemo(config) {
  validateConfig(config);
  const identityId = config.identityId;
  const secretStore = new MemorySecretStore({ developmentMode: true, suppressWarning: true });
  const policy = new ObjectStrengthPolicySkill({ secretStore });
  const grid = new GridChallengeSkill({ host: policy.host });
  const auth = new LocalAuthorizationSkill({ host: policy.host });
  const answerMap = answerMapFor(config.questions);

  try {
    policy.host.enrollQuestionSet(identityId, config.questions, {
      questionSetVersion: config.questionSetVersion ?? 'demo-story-v1',
      normalizationVersion: config.normalizationVersion ?? 'nfkc-lower-v1',
      status: 'active',
    });
    for (const credential of config.credentials) {
      setJsonSecret(secretStore, secretKey('credential', identityId, credential.credentialRef), credential);
    }
    for (const key of config.signatureKeys) {
      setJsonSecret(secretStore, secretKey('signatureKey', identityId, key.keyId), key);
    }

    const credential = config.credentials[0];
    const passwordFlow = await authorizeObject({
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
    consumeReadSession(policy.host, identityId, passwordFlow.authorization.result.authorizationId, credential.credentialRef);
    const realCredential = getJsonSecret(secretStore, secretKey('credential', identityId, credential.credentialRef));

    const signatureKey = config.signatureKeys[0];
    let signatureChallenge = null;
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
        signatureChallenge = summarizeChallenge(flow.verification);
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

    const auditRows = policy.host.db.prepare(
      `SELECT event_type, identity_id, story_object_id, request_id, result, redaction_level, error_code, meta_json
       FROM audit_log
       ORDER BY audit_id`
    ).all().map((row) => ({
      ...row,
      meta_json: row.meta_json ? JSON.parse(row.meta_json) : null,
    }));

    return {
      status: 'success',
      configSummary: {
        identityId,
        storyId: config.storyId,
        questions: config.questions.length,
        credentials: config.credentials.length,
        signatureKeys: config.signatureKeys.length,
      },
      passwordFill: {
        challenge: summarizeChallenge(passwordFlow.verification),
        authorization: passwordFlow.authorization.result,
        result: {
          credentialRef: credential.credentialRef,
          targetOrigin: realCredential.targetOrigin,
          username: realCredential.username,
          password: realCredential.password,
        },
      },
      signature: {
        challenge: signatureChallenge,
        response: signatureResponse,
      },
      auditRows,
    };
  } finally {
    policy.host.close?.();
  }
}

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? '/', `http://${req.headers.host ?? '127.0.0.1'}`);
    if (req.method === 'GET' && url.pathname === '/api/config') {
      sendJson(res, 200, readJson(configPath));
      return;
    }
    if (req.method === 'POST' && url.pathname === '/api/config') {
      const config = JSON.parse(await readBody(req));
      validateConfig(config);
      writeJson(configPath, config);
      sendJson(res, 200, { status: 'saved', path: configPath });
      return;
    }
    if (req.method === 'POST' && url.pathname === '/api/run-demo') {
      const body = await readBody(req);
      const parsed = body.trim() ? JSON.parse(body) : null;
      const config = parsed && Object.keys(parsed).length > 0 ? parsed : readJson(configPath);
      sendJson(res, 200, await runDemo(config));
      return;
    }
    if (
      url.pathname === '/api/storylock-gateway'
      || url.pathname.startsWith('/api/site/')
      || url.pathname === '/app/download'
      || url.pathname === '/app/bind'
      || url.pathname === '/app/registrations'
      || url.pathname === '/download/android-host'
      || url.pathname.startsWith('/android-host/')
    ) {
      await handleStoryLockGatewayRequest(req, res);
      return;
    }

    const targetPath = url.pathname === '/'
      || url.pathname === '/skill-demo.html'
      || url.pathname.startsWith('/skill-ui/')
      ? safeSkillPublicPath(url.pathname.replace(/^\/skill-ui\/?/u, '/'))
      : safeSkillPublicPath(url.pathname);
    if (!existsSync(targetPath) || statSync(targetPath).isDirectory()) {
      res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
      res.end('Not Found');
      return;
    }
    res.writeHead(200, { 'content-type': mimeByExtension[extname(targetPath).toLowerCase()] ?? 'application/octet-stream' });
    createReadStream(targetPath).pipe(res);
  } catch (error) {
    sendError(res, 500, error);
  }
});

server.listen(port, host, () => {
  console.log(`Yian site running at http://${host}:${port}/`);
});
