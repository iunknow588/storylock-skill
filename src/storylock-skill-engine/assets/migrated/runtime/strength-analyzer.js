import {
  BASIC_GRID_DISPLAY_SLOTS,
  BASIC_MIN_CHALLENGE_BITS,
  BASIC_MIN_DIFFICULTY_BITS,
  BASIC_READ_THRESHOLD,
  DEFAULT_VERIFY_POLICY,
  PROD_QUESTION_MIN_ITERATIONS,
  PROD_QUESTION_MIN_MEMORY_KIB,
  PROD_ROOT_MIN_ITERATIONS,
  PROD_ROOT_MIN_MEMORY_KIB,
  QUESTION_KEY_KDF,
  ROOT_KEY_KDF,
} from "./constants.js";
import { ValidationError } from "./errors.js";

function assert(condition, message) {
  if (!condition) {
    throw new ValidationError(message);
  }
}

function normalizeAnswer(answer, policy) {
  let value = String(answer ?? "");
  if (!policy.caseSensitive) {
    value = value.toLowerCase();
  }
  if (policy.ignoreLeadingTrailingSpaces) {
    value = value.trim();
  }
  if (policy.ignoreAllSpaces) {
    value = value.replace(/\s+/g, "");
  }
  return value;
}

function dedupeNormalizedAnswers(answers, verifyPolicy) {
  return [...new Set((answers ?? []).map((answer) => normalizeAnswer(answer, verifyPolicy)))].filter(
    Boolean,
  );
}

function buildDisplayCandidates(nodeId, validAnswers, distractors) {
  const candidates = [];
  for (const value of [...validAnswers, ...distractors]) {
    const text = String(value ?? "").trim();
    if (!text || candidates.includes(text)) {
      continue;
    }
    candidates.push(text);
  }

  assert(
    candidates.length === BASIC_GRID_DISPLAY_SLOTS,
    `Question ${nodeId} must contain exactly ${BASIC_GRID_DISPLAY_SLOTS} unique display candidates`,
  );
  return candidates;
}

function estimateDifficultyBits(candidatePoolSize, acceptedAnswerSetCount) {
  if (candidatePoolSize <= 0 || acceptedAnswerSetCount <= 0) {
    return 0;
  }
  const totalNonEmpty = (2 ** candidatePoolSize) - 1;
  if (totalNonEmpty <= acceptedAnswerSetCount) {
    return 0;
  }
  return Math.floor(Math.log2(totalNonEmpty / acceptedAnswerSetCount));
}

function prepareQuestion(question, index) {
  const nodeId = question.nodeId ?? `story_${String(index + 1).padStart(2, "0")}`;
  const verifyPolicy = {
    ...DEFAULT_VERIFY_POLICY,
    ...(question.verifyPolicy ?? {}),
  };
  const normalizedValidAnswers = dedupeNormalizedAnswers(
    question.validAnswers ?? [],
    verifyPolicy,
  );
  const distractors = [...(question.distractors ?? [])];
  const displayCandidates = buildDisplayCandidates(nodeId, question.validAnswers ?? [], distractors);
  const difficultyBits = estimateDifficultyBits(displayCandidates.length, 1);
  const formalEligible =
    displayCandidates.length === BASIC_GRID_DISPLAY_SLOTS &&
    distractors.length >= 5 &&
    difficultyBits >= BASIC_MIN_DIFFICULTY_BITS;

  assert(normalizedValidAnswers.length > 0, `Question ${nodeId} has no valid answers`);

  return {
    nodeId,
    formalEligible,
    difficultyBits,
  };
}

export function analyzeIdentityConfig({ questions }) {
  assert(Array.isArray(questions), "questions must be an array");
  assert(
    questions.length === 24,
    "questions must contain exactly 24 nodes",
  );

  const preparedQuestions = questions.map((question, index) => prepareQuestion(question, index));
  const formalEligibleQuestionCount = preparedQuestions.filter(
    (question) => question.formalEligible,
  ).length;
  const strongestBasicChallengeBits = preparedQuestions
    .filter((question) => question.formalEligible)
    .map((question) => question.difficultyBits ?? 0)
    .sort((left, right) => right - left)
    .slice(0, BASIC_READ_THRESHOLD)
    .reduce((sum, bits) => sum + bits, 0);

  const rootKdfMeetsProductionMinimum =
    ROOT_KEY_KDF.memorySize >= PROD_ROOT_MIN_MEMORY_KIB &&
    ROOT_KEY_KDF.iterations >= PROD_ROOT_MIN_ITERATIONS;
  const answerKdfMeetsProductionMinimum =
    QUESTION_KEY_KDF.memorySize >= PROD_QUESTION_MIN_MEMORY_KIB &&
    QUESTION_KEY_KDF.iterations >= PROD_QUESTION_MIN_ITERATIONS;
  const strongestBasicChallengeMeetsMinimum =
    formalEligibleQuestionCount >= BASIC_READ_THRESHOLD &&
    strongestBasicChallengeBits >= BASIC_MIN_CHALLENGE_BITS;

  const issues = [];
  if (!rootKdfMeetsProductionMinimum) {
    issues.push(
      `root_kdf below production minimum: need memory >= ${PROD_ROOT_MIN_MEMORY_KIB} KiB and iterations >= ${PROD_ROOT_MIN_ITERATIONS}`,
    );
  }
  if (!answerKdfMeetsProductionMinimum) {
    issues.push(
      `answer_kdf below production minimum: need memory >= ${PROD_QUESTION_MIN_MEMORY_KIB} KiB and iterations >= ${PROD_QUESTION_MIN_ITERATIONS}`,
    );
  }
  if (formalEligibleQuestionCount < BASIC_READ_THRESHOLD) {
    issues.push(
      `not enough formal-eligible questions for basic read: need at least ${BASIC_READ_THRESHOLD}, got ${formalEligibleQuestionCount}`,
    );
  }
  if (
    formalEligibleQuestionCount >= BASIC_READ_THRESHOLD &&
    strongestBasicChallengeBits < BASIC_MIN_CHALLENGE_BITS
  ) {
    issues.push(
      `strongest basic challenge is too weak: need at least ${BASIC_MIN_CHALLENGE_BITS} bits, got ${strongestBasicChallengeBits}`,
    );
  }

  return {
    productionReady: issues.length === 0,
    rootKdfMeetsProductionMinimum,
    answerKdfMeetsProductionMinimum,
    formalEligibleQuestionCount,
    requiredBasicQuestionCount: BASIC_READ_THRESHOLD,
    basicMinChallengeBits: BASIC_MIN_CHALLENGE_BITS,
    strongestBasicChallengeBits,
    strongestBasicChallengeMeetsMinimum,
    issues,
  };
}
