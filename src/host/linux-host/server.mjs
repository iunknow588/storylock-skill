import { createHash, randomUUID } from 'node:crypto';
import { createServer } from 'node:http';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { existsSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  createAccessHost,
  GridChallengeSkill,
  LocalAuthorizationSkill,
  LocalRevocationSkill,
} from '../../skills/local-story-access/index.js';
import { MemorySecretStore } from '../../shared/secret-store.js';
import {
  inspectStoryLockPackage,
  loadStoryLockPackage,
} from '../../shared/storylock-package/index.js';

const hostRoot = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(hostRoot, '../../..');
const defaultAssetPath = join(hostRoot, 'assets', 'question-bank.json');

function envFlag(name, fallback = false) {
  const value = process.env[name];
  if (value === undefined) {
    return fallback;
  }
  return ['1', 'true', 'yes', 'on'].includes(String(value).trim().toLowerCase());
}

function envString(name, fallback) {
  const value = process.env[name];
  return typeof value === 'string' && value.trim() ? value.trim() : fallback;
}

function parseJsonWithBom(text) {
  return JSON.parse(String(text).trimStart().replace(/^\uFEFF/u, ''));
}

function sha256Hex(value) {
  return createHash('sha256').update(value).digest('hex');
}

function normalizeQuestionBank(raw, fallbackIdentityId) {
  const identityId = String(raw.identityId ?? fallbackIdentityId).trim();
  const questionSetVersion = String(raw.questionSetVersion ?? '').trim();
  const normalizationVersion = String(raw.normalizationVersion ?? 'nfkc-lower-v1').trim();
  const questions = Array.isArray(raw.questions) ? raw.questions : [];
  if (!identityId) {
    throw new Error('question bank identityId must be non-empty');
  }
  if (!questionSetVersion) {
    throw new Error('question bank questionSetVersion must be non-empty');
  }
  if (!normalizationVersion) {
    throw new Error('question bank normalizationVersion must be non-empty');
  }
  if (questions.length < 9) {
    throw new Error('Linux host question bank requires at least 9 questions');
  }
  return {
    identityId,
    questionSetVersion,
    normalizationVersion,
    questions: questions.map((question, index) => {
      const questionId = String(question.questionId ?? `linux-q-${index + 1}`).trim();
      const promptRef = String(question.promptRef ?? questionId).trim();
      const versionTag = String(question.versionTag ?? questionSetVersion).trim();
      const answer = String(question.answer ?? '').trim();
      if (!questionId || !promptRef || !versionTag || !answer) {
        throw new Error(`question ${index + 1} must include questionId, promptRef, versionTag, and answer`);
      }
      return {
        questionId,
        promptRef,
        versionTag,
        promptText: question.promptText ?? null,
        answer,
        status: question.status ?? 'active',
      };
    }),
  };
}

async function readQuestionBank(path, fallbackIdentityId) {
  const text = await readFile(path, 'utf8');
  return normalizeQuestionBank(parseJsonWithBom(text), fallbackIdentityId);
}

function readQuestionBankSync(path, fallbackIdentityId) {
  const text = readFileSync(path, 'utf8');
  return normalizeQuestionBank(parseJsonWithBom(text), fallbackIdentityId);
}

function answersForCells(cells) {
  return cells.map((cell) => ({
    cellId: cell.cellId,
    answer: `linux-answer-${cell.position}`,
  }));
}

function defaultDataDir() {
  return join(tmpdir(), 'storylock-linux-host');
}

export async function createLinuxHostRuntime({
  dataDir = envString('STORYLOCK_LINUX_DATA_DIR', defaultDataDir()),
  identityId = envString('STORYLOCK_LINUX_IDENTITY_ID', 'linux-demo-001'),
  port = Number(envString('STORYLOCK_LINUX_HOST_PORT', '4520')),
  usePlatformSecretStore = envFlag('STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE', false),
  developmentMode = envFlag('STORYLOCK_LINUX_DEVELOPMENT_MODE', true),
  resetDataDir = false,
  questionBankPath = defaultAssetPath,
  storyLockPackageDir = envString('STORYLOCK_LINUX_STORYLOCK_PACKAGE_DIR', ''),
} = {}) {
  const absoluteDataDir = resolve(dataDir);
  if (resetDataDir && existsSync(absoluteDataDir)) {
    await rm(absoluteDataDir, { recursive: true, force: true });
  }
  await mkdir(absoluteDataDir, { recursive: true });
  const dbPath = join(absoluteDataDir, 'linux-host.db');
  const secretStore = usePlatformSecretStore
    ? undefined
    : new MemorySecretStore({ developmentMode: true, suppressWarning: true });
  const host = createAccessHost({
    dbPath,
    secretStore,
    usePlatformSecretStore,
    developmentMode,
    allowLegacyFallback: false,
  });
  const bank = await readQuestionBank(questionBankPath, identityId);
  host.enrollQuestionSet(bank.identityId, bank.questions, {
    questionSetVersion: bank.questionSetVersion,
    normalizationVersion: bank.normalizationVersion,
  });
  const resolvedStoryLockPackageDir = storyLockPackageDir ? resolve(storyLockPackageDir) : null;
  const storyLockPackage = resolvedStoryLockPackageDir
    ? inspectStoryLockPackage(await loadStoryLockPackage(resolvedStoryLockPackageDir))
    : {
        valid: false,
        errors: [],
        warnings: [],
        infos: [{
          code: 'SLP-LINUX-001',
          level: 'info',
          path: '$',
          message: 'Linux host StoryLock package directory is not configured.',
          suggestion: 'Set STORYLOCK_LINUX_STORYLOCK_PACKAGE_DIR to a local StoryLock package directory.',
        }],
        summary: {
          packageId: null,
          resources: 0,
          permissionObjects: 0,
          storyNodes: 0,
          templates: 0,
          permissionSummary: { items: [] },
        },
      };
  return {
    product: 'Yian Linux Host',
    implementation: 'node-linux-prototype',
    version: '0.1.0',
    identityId: bank.identityId,
    port,
    dataDir: absoluteDataDir,
    dbPath,
    questionBankPath,
    questionBank: bank,
    storyLockPackageDir: resolvedStoryLockPackageDir,
    storyLockPackage,
    host,
    grid: new GridChallengeSkill({ host }),
    auth: new LocalAuthorizationSkill({ host }),
    revoke: new LocalRevocationSkill({ host }),
  };
}

function jsonResponse(res, statusCode, value) {
  const body = JSON.stringify(value, null, 2);
  res.writeHead(statusCode, {
    'content-type': 'application/json; charset=utf-8',
    'content-length': Buffer.byteLength(body),
  });
  res.end(body);
}

async function readBody(req) {
  let body = '';
  for await (const chunk of req) {
    body += chunk;
    if (body.length > 1_000_000) {
      throw new Error('request body too large');
    }
  }
  return body.trim() ? parseJsonWithBom(body) : {};
}

function activeQuestionCount(runtime) {
  return runtime.host.db.prepare(
    'SELECT COUNT(*) AS count FROM question_set_item WHERE identity_id = ? AND status = ?',
  ).get(runtime.identityId, 'active').count;
}

function requireSession(runtime, identityId, authorizationId, objectRef) {
  const row = runtime.host.db.prepare(
    'SELECT session_id, identity_id, status, expires_at, resource_scope_json FROM session_store WHERE session_id = ?',
  ).get(authorizationId);
  if (!row || row.identity_id !== identityId || row.status !== 'session_active' || row.expires_at <= Date.now()) {
    throw new Error('authorization session is invalid or expired');
  }
  const resourceScope = JSON.parse(row.resource_scope_json || '[]');
  if (!resourceScope.includes(objectRef)) {
    throw new Error('authorization session does not cover requested objectRef');
  }
  return row;
}

async function handleLinuxHostRequest(runtime, req, res) {
  const url = new URL(req.url ?? '/', `http://127.0.0.1:${runtime.port}`);
  if (req.method === 'GET' && url.pathname === '/health') {
    jsonResponse(res, 200, {
      schemaVersion: 'linux-host-health-v1',
      product: runtime.product,
      implementation: runtime.implementation,
      version: runtime.version,
      identityId: runtime.identityId,
      hostPort: runtime.port,
      status: 'local_core_prototype',
      capabilities: ['health', 'question-bank', 'verify', 'authorize', 'execute', 'revoke'],
      storage: {
        provider: runtime.host.secretStore?.constructor?.name ?? 'unknown',
        dataDir: runtime.dataDir,
        dbPath: runtime.dbPath,
      },
      questionBank: {
        path: runtime.questionBankPath,
        questionSetVersion: runtime.questionBank.questionSetVersion,
        questionCount: activeQuestionCount(runtime),
      },
      storyLockPackage: {
        path: runtime.storyLockPackageDir,
        configured: Boolean(runtime.storyLockPackageDir),
        valid: runtime.storyLockPackage.valid,
        packageId: runtime.storyLockPackage.summary.packageId,
        permissionObjects: runtime.storyLockPackage.summary.permissionObjects,
      },
    });
    return;
  }

  if (req.method === 'GET' && url.pathname === '/permission-summary') {
    jsonResponse(res, 200, {
      requestId: `req-${randomUUID()}`,
      status: runtime.storyLockPackageDir ? 'success' : 'not_configured',
      capability: 'permissionSummary',
      executionLocation: 'local',
      result: {
        packageDir: runtime.storyLockPackageDir,
        packageId: runtime.storyLockPackage.summary.packageId,
        valid: runtime.storyLockPackage.valid,
        resources: runtime.storyLockPackage.summary.resources,
        permissionObjects: runtime.storyLockPackage.summary.permissionObjects,
        permissionSummary: runtime.storyLockPackage.summary.permissionSummary,
        errors: runtime.storyLockPackage.errors,
        warnings: runtime.storyLockPackage.warnings,
        infos: runtime.storyLockPackage.infos,
      },
      redactionLevel: 'audit_meta_only',
      retentionGranted: 'audit_meta_only',
      error: null,
    });
    return;
  }

  if (req.method === 'GET' && url.pathname === '/question-bank/status') {
    jsonResponse(res, 200, {
      requestId: `req-${randomUUID()}`,
      status: 'success',
      capability: 'questionBankStatus',
      executionLocation: 'local',
      result: {
        identityId: runtime.identityId,
        path: runtime.questionBankPath,
        questionSetVersion: runtime.questionBank.questionSetVersion,
        normalizationVersion: runtime.questionBank.normalizationVersion,
        questionCount: activeQuestionCount(runtime),
      },
      error: null,
    });
    return;
  }

  if (req.method !== 'POST') {
    jsonResponse(res, 404, { status: 'error', message: 'not found' });
    return;
  }

  const body = await readBody(req);
  if (url.pathname === '/question-bank/import') {
    const sourcePath = body.sourcePath ? resolve(String(body.sourcePath)) : null;
    if (!sourcePath) {
      throw new Error('sourcePath is required');
    }
    const bank = await readQuestionBank(sourcePath, runtime.identityId);
    runtime.host.enrollQuestionSet(bank.identityId, bank.questions, {
      questionSetVersion: bank.questionSetVersion,
      normalizationVersion: bank.normalizationVersion,
    });
    runtime.identityId = bank.identityId;
    runtime.questionBank = bank;
    runtime.questionBankPath = sourcePath;
    jsonResponse(res, 200, {
      requestId: body.requestId ?? `req-${randomUUID()}`,
      status: 'success',
      capability: 'questionBankImport',
      executionLocation: 'local',
      result: {
        identityId: bank.identityId,
        path: sourcePath,
        questionSetVersion: bank.questionSetVersion,
        normalizationVersion: bank.normalizationVersion,
        questionCount: activeQuestionCount(runtime),
      },
      error: null,
    });
    return;
  }

  if (url.pathname === '/verify') {
    const result = await runtime.grid.run({
      identityId: body.identityId ?? runtime.identityId,
      objectRef: body.objectRef ?? body.keyId ?? body.credentialRef ?? 'linux-object',
      requiredStrength: body.requiredStrength ?? (body.capability === 'requestSignature' ? 'high' : 'medium'),
      requestId: body.requestId ?? `req-${randomUUID()}`,
      nonce: body.nonce ?? `nonce-${randomUUID()}`,
      expiry: body.expiry ?? Date.now() + 60_000,
      questionSetVersion: body.questionSetVersion ?? runtime.questionBank.questionSetVersion,
    });
    jsonResponse(res, result.status === 'success' ? 200 : 400, result);
    return;
  }

  if (url.pathname === '/authorize') {
    const result = await runtime.auth.run({
      identityId: body.identityId ?? runtime.identityId,
      objectRef: body.objectRef ?? body.keyId ?? body.credentialRef ?? 'linux-object',
      verificationId: body.verificationId,
      allowedAction: body.allowedAction ?? (body.capability === 'requestPasswordFill' ? 'password_fill' : 'signature'),
      answers: body.answers ?? [],
      requestId: body.requestId ?? `req-${randomUUID()}`,
    });
    jsonResponse(res, result.status === 'success' ? 200 : 400, result);
    return;
  }

  if (url.pathname === '/execute') {
    const identityId = body.identityId ?? runtime.identityId;
    const objectRef = body.objectRef ?? body.keyId ?? body.credentialRef ?? 'linux-object';
    requireSession(runtime, identityId, body.authorizationId, objectRef);
    const capability = body.capability ?? 'requestSignature';
    const result = capability === 'requestPasswordFill'
      ? {
          username: body.usernameHint ?? 'linux-user',
          password: `linux-password-${sha256Hex(objectRef).slice(0, 12)}`,
          credentialRef: objectRef,
        }
      : {
          signature: `sha256:${sha256Hex(JSON.stringify({ objectRef, payload: body.payload ?? '', authorizationId: body.authorizationId }))}`,
          keyId: objectRef,
          algorithm: 'linux-local-sha256-prototype',
        };
    jsonResponse(res, 200, {
      requestId: body.requestId ?? `req-${randomUUID()}`,
      status: 'success',
      capability,
      executionLocation: 'local',
      result: {
        ...result,
        authorizationId: body.authorizationId,
        coreBoundary: 'storylock_local_core',
      },
      redactionLevel: capability === 'requestPasswordFill' ? 'full' : 'result_only',
      retentionGranted: capability === 'requestPasswordFill' ? 'audit_meta_only' : 'result_only',
      error: null,
    });
    return;
  }

  if (url.pathname === '/revoke') {
    const result = await runtime.revoke.run({
      identityId: body.identityId ?? runtime.identityId,
      authorizationId: body.authorizationId,
      verificationId: body.verificationId,
      reason: body.reason ?? 'linux-host-loop',
      requestId: body.requestId ?? `req-${randomUUID()}`,
    });
    jsonResponse(res, result.status === 'success' ? 200 : 400, result);
    return;
  }

  jsonResponse(res, 404, { status: 'error', message: 'not found' });
}

export function startLinuxHostServer(runtime) {
  const server = createServer((req, res) => {
    handleLinuxHostRequest(runtime, req, res).catch((error) => {
      jsonResponse(res, 500, {
        status: 'error',
        message: error instanceof Error ? error.message : String(error),
      });
    });
  });
  return new Promise((resolvePromise, rejectPromise) => {
    server.once('error', rejectPromise);
    server.listen(runtime.port, '127.0.0.1', () => {
      server.off('error', rejectPromise);
      resolvePromise(server);
    });
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const runtime = await createLinuxHostRuntime();
  const server = await startLinuxHostServer(runtime);
  console.log(JSON.stringify({
    status: 'running',
    url: `http://127.0.0.1:${runtime.port}`,
    dataDir: runtime.dataDir,
    pid: process.pid,
  }, null, 2));
  process.once('SIGINT', () => {
    server.close();
    runtime.host.close?.();
  });
}

export { readQuestionBankSync, answersForCells };
