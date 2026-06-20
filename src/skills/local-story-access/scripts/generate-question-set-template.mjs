import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

function readArg(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) {
    return null;
  }
  return process.argv[index + 1] ?? null;
}

function normalizePositiveInteger(value, fallback) {
  const parsed = Number.parseInt(String(value ?? ''), 10);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : fallback;
}

function buildQuestions(count, {
  questionSetVersion,
  normalizationVersion,
} = {}) {
  return Array.from({ length: count }, (_, index) => {
    const n = index + 1;
    return {
      questionId: `q-${n}`,
      versionTag: questionSetVersion,
      promptRef: `prompt-${n}`,
      promptText: `Replace with your real question ${n}`,
      options: [
        `replace-answer-${n}`,
        `replace-distractor-${n}-1`,
        `replace-distractor-${n}-2`,
        `replace-distractor-${n}-3`,
      ],
      answer: `replace-answer-${n}`,
      status: 'active',
      questionSetVersion,
      normalizationVersion,
    };
  });
}

const identityId = readArg('--identity-id') ?? 'replace-identity-id';
const questionSetVersion = readArg('--question-set-version') ?? 'replace-question-set-version';
const normalizationVersion = readArg('--normalization-version') ?? 'nfkc-lower-v1';
const count = normalizePositiveInteger(readArg('--count'), 24);
const outputPath = resolve(readArg('--output') ?? 'assets/question-set-master.sample.json');

const template = {
  template: true,
  usage: 'Copy this file, replace prompts/options/answers with real data, then run npm run validate:question-set or npm run import:question-set.',
  identityId,
  questionSetVersion,
  normalizationVersion,
  status: 'active',
  replacePreviousActive: true,
  questions: buildQuestions(count, {
    questionSetVersion,
    normalizationVersion,
  }),
};

writeFileSync(outputPath, `${JSON.stringify(template, null, 2)}\n`, 'utf8');

console.log(JSON.stringify({
  status: 'success',
  outputPath,
  identityId,
  questionSetVersion,
  normalizationVersion,
  questionCount: count,
}, null, 2));
