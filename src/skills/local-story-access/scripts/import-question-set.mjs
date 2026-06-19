import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { MemorySecretStore, createPlatformSecretStore } from '../../../shared/secret-store.js';
import { createAccessHost } from '../access-host.js';

function readArg(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) {
    return null;
  }
  return process.argv[index + 1] ?? null;
}

function hasFlag(name) {
  return process.argv.includes(name);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function requireString(value, fieldName, maxLength = 128) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  const normalized = value.trim();
  if (normalized.length > maxLength) {
    throw new Error(`${fieldName} must be ${maxLength} characters or less`);
  }
  return normalized;
}

function requireStatus(value, fieldName) {
  const status = requireString(value, fieldName, 32);
  if (!['active', 'deprecated', 'pending'].includes(status)) {
    throw new Error(`${fieldName} must be active, deprecated, or pending`);
  }
  return status;
}

function normalizeQuestion(question, index, defaults) {
  const normalized = {
    questionId: requireString(question.questionId ?? `q-${index + 1}`, `questions[${index}].questionId`),
    versionTag: requireString(question.versionTag ?? defaults.questionSetVersion, `questions[${index}].versionTag`),
    promptRef: requireString(question.promptRef ?? question.questionId ?? `q-${index + 1}`, `questions[${index}].promptRef`, 256),
    answer: requireString(question.answer, `questions[${index}].answer`, 512),
    status: requireStatus(question.status ?? defaults.status, `questions[${index}].status`),
    questionSetVersion: requireString(question.questionSetVersion ?? defaults.questionSetVersion, `questions[${index}].questionSetVersion`),
    normalizationVersion: requireString(question.normalizationVersion ?? defaults.normalizationVersion, `questions[${index}].normalizationVersion`),
  };
  if (question.promptText !== undefined) {
    normalized.promptText = requireString(question.promptText, `questions[${index}].promptText`, 4096);
  }
  if (question.optionDigest !== undefined) {
    normalized.optionDigest = requireString(question.optionDigest, `questions[${index}].optionDigest`, 256);
  }
  if (question.options !== undefined) {
    if (!Array.isArray(question.options) || question.options.length === 0) {
      throw new Error(`questions[${index}].options must be a non-empty array when provided`);
    }
    normalized.options = question.options.map((option, optionIndex) => (
      requireString(option, `questions[${index}].options[${optionIndex}]`, 512)
    ));
  }
  if (!normalized.optionDigest && !normalized.options) {
    throw new Error(`questions[${index}] must include options or optionDigest`);
  }
  return normalized;
}

function normalizeMaster(input) {
  const defaults = {
    identityId: requireString(input.identityId, 'identityId'),
    questionSetVersion: requireString(input.questionSetVersion, 'questionSetVersion'),
    normalizationVersion: requireString(input.normalizationVersion, 'normalizationVersion'),
    status: requireStatus(input.status, 'status'),
  };
  if (!Array.isArray(input.questions) || input.questions.length === 0) {
    throw new Error('questions must be a non-empty array');
  }
  const questions = input.questions.map((question, index) => normalizeQuestion(question, index, defaults));
  const keys = new Set();
  for (const question of questions) {
    const key = `${question.questionId}\u0000${question.versionTag}`;
    if (keys.has(key)) {
      throw new Error(`duplicate questionId/versionTag pair: ${question.questionId}/${question.versionTag}`);
    }
    keys.add(key);
  }
  return {
    ...defaults,
    replacePreviousActive: input.replacePreviousActive ?? true,
    questions,
  };
}

const inputPath = readArg('--input') ?? process.argv.slice(2).find((arg) => !arg.startsWith('--'));
if (!inputPath) {
  fail('Usage: npm run import:question-set -- --input assets/question-set-master.sample.json --db storylock.db --use-platform-secret-store');
}

const dbPath = readArg('--db') ?? ':memory:';
const usePlatformSecretStore = hasFlag('--use-platform-secret-store');
const useDevelopmentMemoryStore = hasFlag('--development-memory-secret-store');
const dryRun = hasFlag('--dry-run');
const requiredActiveQuestions = Number(readArg('--require-min-active') ?? (dbPath === ':memory:' ? 9 : 24));

if (dbPath !== ':memory:' && !usePlatformSecretStore && !useDevelopmentMemoryStore) {
  fail('Persistent import requires --use-platform-secret-store or --development-memory-secret-store.');
}

if (dbPath !== ':memory:' && useDevelopmentMemoryStore && !dryRun) {
  fail('Persistent question-set import cannot use --development-memory-secret-store because answer digests depend on a stable masterSalt. Use --use-platform-secret-store, or run --dry-run for validation.');
}

let master;
try {
  master = normalizeMaster(readJson(resolve(inputPath)));
} catch (error) {
  fail(`Invalid question set: ${error instanceof Error ? error.message : String(error)}`);
}

if (!Number.isInteger(requiredActiveQuestions) || requiredActiveQuestions < 9) {
  fail('--require-min-active must be an integer greater than or equal to 9.');
}

const activeQuestionCount = master.questions.filter((question) => question.status === 'active').length;
if (master.status === 'active' && activeQuestionCount < requiredActiveQuestions) {
  fail(`Active question set imports require at least ${requiredActiveQuestions} active questions. Use --require-min-active 9 only for demo or compatibility validation.`);
}

if (dryRun) {
  console.log(JSON.stringify({
    status: 'validated',
    dryRun: true,
    dbPath,
    identityId: master.identityId,
    questionSetVersion: master.questionSetVersion,
    normalizationVersion: master.normalizationVersion,
    questionStatus: master.status,
    questionCount: master.questions.length,
    activeQuestionCount,
    requiredActiveQuestions,
  }, null, 2));
  process.exit(0);
}

const secretStore = usePlatformSecretStore
  ? createPlatformSecretStore()
  : new MemorySecretStore({ developmentMode: true, suppressWarning: true });

const host = createAccessHost({
  dbPath,
  secretStore,
});

try {
  const imported = host.enrollQuestionSet(master.identityId, master.questions, {
    questionSetVersion: master.questionSetVersion,
    normalizationVersion: master.normalizationVersion,
    status: master.status,
    replacePreviousActive: master.replacePreviousActive,
  });
  const byStatus = imported.reduce((counts, question) => {
    counts[question.status] = (counts[question.status] ?? 0) + 1;
    return counts;
  }, {});
  console.log(JSON.stringify({
    status: 'success',
    dbPath,
    identityId: master.identityId,
    questionSetVersion: master.questionSetVersion,
    normalizationVersion: master.normalizationVersion,
    importedQuestionCount: imported.length,
    questionStatusCounts: byStatus,
    requiredActiveQuestions,
    replacePreviousActive: master.replacePreviousActive,
  }, null, 2));
} finally {
  host.close?.();
}
