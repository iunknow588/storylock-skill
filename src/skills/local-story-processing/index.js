function ensureString(value, fieldName) {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return value.trim();
}

function ensureArray(value, fieldName) {
  if (value === undefined || value === null) {
    return [];
  }
  if (!Array.isArray(value)) {
    throw new Error(`${fieldName} must be an array`);
  }
  return value;
}

function ensureStoryDraft(value, fieldName) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${fieldName} must be an object`);
  }
  return {
    title: typeof value.title === "string" && value.title.trim().length > 0
      ? value.title.trim()
      : "Story Draft",
    content: ensureString(value.content, `${fieldName}.content`),
    cues: ensureArray(value.cues, `${fieldName}.cues`).map((cue, index) =>
      ensureString(cue, `${fieldName}.cues[${index}]`)
    ),
    metadata: value.metadata && typeof value.metadata === "object" && !Array.isArray(value.metadata)
      ? value.metadata
      : {},
  };
}

function normalizeNotes(value) {
  return ensureArray(value, "notes").map((note, index) => ensureString(note, `notes[${index}]`));
}

const STORY_NODE_COUNT = 24;
const BASIC_GRID_DISPLAY_SLOTS = 9;
const BASIC_READ_THRESHOLD = 6;
const BASIC_MIN_DIFFICULTY_BITS = 8;
const BASIC_MIN_CHALLENGE_BITS = 30;
const DEFAULT_VERIFY_POLICY = Object.freeze({
  caseSensitive: false,
  ignoreLeadingTrailingSpaces: true,
  ignoreAllSpaces: false,
});

function defaultDraftGenerator({ objective, audience, tone, constraints }) {
  const constraintText = constraints.length > 0 ? ` Constraints: ${constraints.join("; ")}.` : "";
  return {
    title: "Story Draft",
    content: `${objective} Audience: ${audience}. Tone: ${tone}.${constraintText}`,
    cues: constraints,
    metadata: {
      generatedBy: "storylock-local-story-processing-skill",
      processingMode: "local_rule_template",
    },
  };
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

function uniqueNormalizedAnswers(answers, verifyPolicy) {
  return [...new Set((answers ?? []).map((answer) => normalizeAnswer(answer, verifyPolicy)))].filter(Boolean);
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

function analyzeQuestion(question, index) {
  if (!question || typeof question !== "object" || Array.isArray(question)) {
    throw new Error(`questions[${index}] must be an object`);
  }
  const nodeId = question.nodeId ?? `story_${String(index + 1).padStart(2, "0")}`;
  const verifyPolicy = {
    ...DEFAULT_VERIFY_POLICY,
    ...(question.verifyPolicy ?? {}),
  };
  const validAnswers = uniqueNormalizedAnswers(question.validAnswers, verifyPolicy);
  if (validAnswers.length === 0) {
    throw new Error(`Question ${nodeId} has no valid answers`);
  }
  const displayCandidates = [
    ...new Set([...(question.validAnswers ?? []), ...(question.distractors ?? [])].map((item) => String(item ?? "").trim()).filter(Boolean)),
  ];
  if (displayCandidates.length !== BASIC_GRID_DISPLAY_SLOTS) {
    throw new Error(`Question ${nodeId} must contain exactly ${BASIC_GRID_DISPLAY_SLOTS} unique display candidates`);
  }
  const difficultyBits = estimateDifficultyBits(displayCandidates.length, 1);
  return {
    nodeId,
    formalEligible: (question.distractors ?? []).length >= 5 && difficultyBits >= BASIC_MIN_DIFFICULTY_BITS,
    difficultyBits,
  };
}

function analyzeQuestionSet(questions) {
  if (!Array.isArray(questions) || questions.length !== STORY_NODE_COUNT) {
    throw new Error(`questions must contain exactly ${STORY_NODE_COUNT} nodes`);
  }
  const analyzedQuestions = questions.map(analyzeQuestion);
  const formalEligibleQuestionCount = analyzedQuestions.filter((question) => question.formalEligible).length;
  const strongestBasicChallengeBits = analyzedQuestions
    .filter((question) => question.formalEligible)
    .map((question) => question.difficultyBits)
    .sort((left, right) => right - left)
    .slice(0, BASIC_READ_THRESHOLD)
    .reduce((sum, bits) => sum + bits, 0);
  const strongestBasicChallengeMeetsMinimum =
    formalEligibleQuestionCount >= BASIC_READ_THRESHOLD &&
    strongestBasicChallengeBits >= BASIC_MIN_CHALLENGE_BITS;
  const issues = [];
  if (formalEligibleQuestionCount < BASIC_READ_THRESHOLD) {
    issues.push(`not enough formal-eligible questions for basic read: need at least ${BASIC_READ_THRESHOLD}, got ${formalEligibleQuestionCount}`);
  }
  if (formalEligibleQuestionCount >= BASIC_READ_THRESHOLD && strongestBasicChallengeBits < BASIC_MIN_CHALLENGE_BITS) {
    issues.push(`strongest basic challenge is too weak: need at least ${BASIC_MIN_CHALLENGE_BITS} bits, got ${strongestBasicChallengeBits}`);
  }
  return {
    productionReady: issues.length === 0,
    formalEligibleQuestionCount,
    requiredBasicQuestionCount: BASIC_READ_THRESHOLD,
    basicMinChallengeBits: BASIC_MIN_CHALLENGE_BITS,
    strongestBasicChallengeBits,
    strongestBasicChallengeMeetsMinimum,
    questionSetReady: formalEligibleQuestionCount >= BASIC_READ_THRESHOLD && strongestBasicChallengeMeetsMinimum,
    issues,
  };
}

function buildRecommendedActions(profile) {
  const actions = [];
  if (profile.formalEligibleQuestionCount < profile.requiredBasicQuestionCount) {
    actions.push(`promote at least ${profile.requiredBasicQuestionCount - profile.formalEligibleQuestionCount} more questions into the formal-eligible set`);
  }
  if (!profile.strongestBasicChallengeMeetsMinimum) {
    actions.push(`raise the strongest basic challenge from ${profile.strongestBasicChallengeBits} bits to at least ${profile.basicMinChallengeBits} bits`);
  }
  if (actions.length === 0) {
    actions.push("keep the current question set and runtime profile");
  }
  return actions;
}

function defaultRefiner({ storyDraft, goals, hintStyle }) {
  const goalText = goals.length > 0 ? ` Goals: ${goals.join("; ")}.` : "";
  return {
    ...storyDraft,
    content: `${storyDraft.content}${goalText} Hint style: ${hintStyle}.`,
    metadata: {
      ...storyDraft.metadata,
      refinedBy: "storylock-local-story-processing-skill",
      processingMode: "local_rule_template",
    },
  };
}

export class StoryProcessingSkill {
  skillId() {
    return "story_processing";
  }
}

export class StoryDraftSkill extends StoryProcessingSkill {
  constructor({ generator = defaultDraftGenerator } = {}) {
    super();
    if (typeof generator !== "function") {
      throw new Error("generator must be a function");
    }
    this.generator = generator;
  }

  skillId() {
    return "story_draft";
  }

  async run(input = {}) {
    const objective = ensureString(input.objective, "objective");
    const audience = ensureString(input.audience ?? "self", "audience");
    const tone = ensureString(input.tone ?? "neutral", "tone");
    const constraints = ensureArray(input.constraints, "constraints").map((constraint, index) =>
      ensureString(constraint, `constraints[${index}]`)
    );
    const source = input.source ?? "approved_local_input";
    if (!["approved_local_input", "manual_local_input", "template_only"].includes(source)) {
      throw new Error("source must be approved_local_input, manual_local_input, or template_only");
    }

    const draft = await this.generator({ objective, audience, tone, constraints, source });
    return {
      mode: "story_draft",
      processingLayer: "local_story_processing",
      source,
      draft: ensureStoryDraft(draft, "draft"),
      notes: normalizeNotes(input.notes),
      boundary: {
        challengeCreated: false,
        sessionIssued: false,
        protectedObjectRead: false,
        remoteRetentionGranted: false,
      },
    };
  }
}

export class StoryRefineSkill extends StoryProcessingSkill {
  constructor({ refiner = defaultRefiner } = {}) {
    super();
    if (typeof refiner !== "function") {
      throw new Error("refiner must be a function");
    }
    this.refiner = refiner;
  }

  skillId() {
    return "story_refine";
  }

  async run(input = {}) {
    const storyDraft = ensureStoryDraft(input.storyDraft, "storyDraft");
    const goals = ensureArray(input.goals, "goals").map((goal, index) =>
      ensureString(goal, `goals[${index}]`)
    );
    const hintStyle = ensureString(input.hintStyle ?? "plain", "hintStyle");
    const source = input.source ?? "approved_local_input";
    if (!["approved_local_input", "manual_local_input"].includes(source)) {
      throw new Error("source must be approved_local_input or manual_local_input");
    }

    const refinedDraft = await this.refiner({ storyDraft, goals, hintStyle, source });
    return {
      mode: "story_refine",
      processingLayer: "local_story_processing",
      source,
      refinedDraft: ensureStoryDraft(refinedDraft, "refinedDraft"),
      notes: normalizeNotes(input.notes),
      boundary: {
        challengeCreated: false,
        sessionIssued: false,
        protectedObjectRead: false,
        remoteRetentionGranted: false,
      },
    };
  }
}

export class StrengthReviewSkill extends StoryProcessingSkill {
  constructor({ analyzer = analyzeQuestionSet } = {}) {
    super();
    if (typeof analyzer !== "function") {
      throw new Error("analyzer must be a function");
    }
    this.analyzer = analyzer;
  }

  skillId() {
    return "strength_review";
  }

  async run(input = {}) {
    const profile = await this.analyzer(ensureArray(input.questions, "questions"));
    return {
      mode: "strength_review",
      processingLayer: "local_story_processing",
      ...profile,
      recommendedActions: buildRecommendedActions(profile),
      boundary: {
        challengeCreated: false,
        sessionIssued: false,
        protectedObjectRead: false,
        remoteRetentionGranted: false,
      },
    };
  }
}

export const StoryDraftAssistSkill = StoryDraftSkill;
export const StoryRefineAssistSkill = StoryRefineSkill;
export const StoryAssistSkill = StoryProcessingSkill;
