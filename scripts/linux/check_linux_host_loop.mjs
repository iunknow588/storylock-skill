import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import {
  createLinuxHostRuntime,
  startLinuxHostServer,
} from '../../src/host/linux-host/server.mjs';

function assertSuccess(response, name) {
  assert.equal(response.status, 'success', `${name} must return success: ${JSON.stringify(response)}`);
}

async function getJson(url) {
  const response = await fetch(url);
  const payload = await response.json();
  assert.ok(response.ok, `${url} failed: ${JSON.stringify(payload)}`);
  return payload;
}

async function postJson(url, body) {
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'content-type': 'application/json; charset=utf-8',
    },
    body: JSON.stringify(body),
  });
  const payload = await response.json();
  assert.ok(response.ok, `${url} failed: ${JSON.stringify(payload)}`);
  return payload;
}

function importedQuestionBank() {
  return {
    schemaVersion: 'linux-local-question-bank-v1',
    identityId: 'linux-demo-001',
    questionSetVersion: 'linux-loop-v2',
    normalizationVersion: 'nfkc-lower-v1',
    questions: Array.from({ length: 9 }, (_, index) => ({
      questionId: `linux-loop-q-${index + 1}`,
      promptRef: `linux-loop-prompt-${index + 1}`,
      versionTag: 'v2',
      promptText: `Linux loop question ${index + 1}.`,
      answer: `linux-answer-${index + 1}`,
      status: 'active',
    })),
  };
}

const dataDir = await mkdtemp(join(tmpdir(), 'storylock-linux-host-loop-'));
let server;
let runtime;

try {
  runtime = await createLinuxHostRuntime({
    dataDir,
    port: 0,
    developmentMode: true,
    resetDataDir: true,
  });
  server = await startLinuxHostServer(runtime);
  const address = server.address();
  const baseUrl = `http://127.0.0.1:${address.port}`;

  const health = await getJson(`${baseUrl}/health`);
  assert.equal(health.implementation, 'node-linux-prototype');
  assert.equal(health.questionBank.questionSetVersion, 'linux-local-v1');

  const statusBefore = await getJson(`${baseUrl}/question-bank/status`);
  assertSuccess(statusBefore, 'question-bank-status');
  assert.equal(statusBefore.result.questionCount, 9);

  const importPath = join(dataDir, 'import-question-bank.json');
  await writeFile(importPath, `\uFEFF${JSON.stringify(importedQuestionBank(), null, 2)}`, 'utf8');
  const imported = await postJson(`${baseUrl}/question-bank/import`, {
    requestId: 'req-linux-import',
    sourcePath: importPath,
  });
  assertSuccess(imported, 'question-bank-import');
  assert.equal(imported.result.questionSetVersion, 'linux-loop-v2');
  assert.equal(imported.result.questionCount, 9);

  const verification = await postJson(`${baseUrl}/verify`, {
    requestId: 'req-linux-verify',
    nonce: `nonce-${randomUUID()}`,
    capability: 'requestSignature',
    keyId: 'wallet-linux',
  });
  assertSuccess(verification, 'verify');
  assert.match(verification.result.verificationId, /^chl-/);
  assert.equal(verification.result.grid.cells.length, 9);

  const answers = verification.result.grid.cells.map((cell) => ({
    cellId: cell.cellId,
    answer: `linux-answer-${cell.position}`,
  }));
  const authorization = await postJson(`${baseUrl}/authorize`, {
    requestId: 'req-linux-authorize',
    capability: 'requestSignature',
    keyId: 'wallet-linux',
    verificationId: verification.result.verificationId,
    answers,
  });
  assertSuccess(authorization, 'authorize');
  assert.match(authorization.result.authorizationId, /^ses-/);

  const execution = await postJson(`${baseUrl}/execute`, {
    requestId: 'req-linux-execute',
    capability: 'requestSignature',
    keyId: 'wallet-linux',
    authorizationId: authorization.result.authorizationId,
    payload: 'hello linux host',
  });
  assertSuccess(execution, 'execute');
  assert.match(execution.result.signature, /^sha256:[0-9a-f]{64}$/u);
  assert.equal(execution.result.authorizationId, authorization.result.authorizationId);

  const revoked = await postJson(`${baseUrl}/revoke`, {
    requestId: 'req-linux-revoke',
    authorizationId: authorization.result.authorizationId,
  });
  assertSuccess(revoked, 'revoke');
  assert.equal(revoked.result.status, 'session_revoked');

  console.log(JSON.stringify({
    status: 'passed',
    baseUrl,
    checks: [
      'health',
      'question-bank-status',
      'question-bank-import',
      'verify',
      'authorize',
      'execute',
      'revoke',
    ],
  }, null, 2));
} finally {
  if (server) {
    await new Promise((resolve) => server.close(resolve));
  }
  runtime?.host?.close?.();
  await rm(dataDir, { recursive: true, force: true });
}
