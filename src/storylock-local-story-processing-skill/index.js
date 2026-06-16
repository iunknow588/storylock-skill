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

export const StoryDraftAssistSkill = StoryDraftSkill;
export const StoryRefineAssistSkill = StoryRefineSkill;
export const StoryAssistSkill = StoryProcessingSkill;
