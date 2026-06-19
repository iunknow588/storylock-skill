import { ValidationError } from "../runtime/errors.js";

export class StoryAssistSkill {
  skillId() {
    throw new Error("StoryAssistSkill.skillId() must be implemented");
  }

  async run(_input) {
    throw new Error("StoryAssistSkill.run() must be implemented");
  }
}

function ensurePrompt(value, fieldName) {
  if (!value || typeof value !== "string") {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

export class StoryDraftAssistSkill extends StoryAssistSkill {
  constructor({ generator }) {
    super();
    if (typeof generator !== "function") {
      throw new ValidationError("generator must be a function");
    }
    this.generator = generator;
  }

  skillId() {
    return "story_draft_assist";
  }

  async run(input) {
    return this.generator({
      objective: ensurePrompt(input.objective, "objective"),
      audience: input.audience ?? "individual founder",
      tone: input.tone ?? "memorable and concrete",
      constraints: Array.isArray(input.constraints) ? input.constraints : [],
    });
  }
}

export class StoryRefineAssistSkill extends StoryAssistSkill {
  constructor({ refiner }) {
    super();
    if (typeof refiner !== "function") {
      throw new ValidationError("refiner must be a function");
    }
    this.refiner = refiner;
  }

  skillId() {
    return "story_refine_assist";
  }

  async run(input) {
    return this.refiner({
      storyDraft: ensurePrompt(input.storyDraft, "storyDraft"),
      goals: Array.isArray(input.goals) ? input.goals : [],
      hintStyle: input.hintStyle ?? "short mnemonic cues",
    });
  }
}
