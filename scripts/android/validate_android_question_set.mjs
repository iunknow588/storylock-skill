import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..', '..');
const assetPath = path.join(
  repoRoot,
  'src',
  'host',
  'android-host',
  'app',
  'src',
  'main',
  'assets',
  'storylock-question-set.json',
);

function fail(message) {
  throw new Error(message);
}

function requireNonEmpty(value, label) {
  if (typeof value !== 'string' || value.trim() === '') {
    fail(`${label} must be a non-empty string`);
  }
  return value.trim();
}

function validateQuestionSet(payload) {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    fail('question set payload must be an object');
  }

  const identityId = requireNonEmpty(payload.identityId, 'identityId');
  const questionSetVersion = requireNonEmpty(payload.questionSetVersion, 'questionSetVersion');
  const normalizationVersion = requireNonEmpty(payload.normalizationVersion, 'normalizationVersion');

  if (!Array.isArray(payload.questions)) {
    fail('questions must be an array');
  }

  const seenQuestionIds = new Set();
  let activeQuestionCount = 0;

  payload.questions.forEach((item, index) => {
    if (!item || typeof item !== 'object' || Array.isArray(item)) {
      fail(`questions[${index}] must be an object`);
    }
    if ((item.status ?? 'active') !== 'active') {
      return;
    }
    activeQuestionCount += 1;

    const questionId = requireNonEmpty(item.questionId, `questions[${index}].questionId`);
    const promptText = requireNonEmpty(
      typeof item.promptText === 'string' && item.promptText.trim() !== ''
        ? item.promptText
        : item.promptRef,
      `questions[${index}].promptText`,
    );
    requireNonEmpty(item.answer, `questions[${index}].answer`);

    if (seenQuestionIds.has(questionId)) {
      fail(`duplicate active questionId detected: ${questionId}`);
    }
    seenQuestionIds.add(questionId);
    void promptText;
  });

  if (activeQuestionCount < 24) {
    fail(`active question count must be at least 24, received ${activeQuestionCount}`);
  }

  return {
    identityId,
    questionSetVersion,
    normalizationVersion,
    activeQuestionCount,
    uniqueActiveQuestionCount: seenQuestionIds.size,
  };
}

async function main() {
  const raw = await readFile(assetPath, 'utf8');
  const payload = JSON.parse(raw);
  const result = validateQuestionSet(payload);
  console.log(
    JSON.stringify(
      {
        status: 'passed',
        assetPath: path.relative(repoRoot, assetPath).replaceAll('\\', '/'),
        ...result,
      },
      null,
      2,
    ),
  );
}

main().catch((error) => {
  console.error(
    JSON.stringify(
      {
        status: 'failed',
        assetPath: path.relative(repoRoot, assetPath).replaceAll('\\', '/'),
        reason: error instanceof Error ? error.message : String(error),
      },
      null,
      2,
    ),
  );
  process.exitCode = 1;
});
