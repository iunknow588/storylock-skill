import { ValidationError } from "../runtime/errors.js";
import { analyzeIdentityConfig } from "../runtime/strength-analyzer.js";

function ensureQuestions(questions) {
  if (!Array.isArray(questions) || questions.length === 0) {
    throw new ValidationError("questions must be a non-empty array");
  }
  return questions;
}

function buildRecommendedActions(profile) {
  const actions = [];
  if (profile.formalEligibleQuestionCount < profile.requiredBasicQuestionCount) {
    actions.push(
      `promote at least ${profile.requiredBasicQuestionCount - profile.formalEligibleQuestionCount} more questions into the formal-eligible set`,
    );
  }
  if (!profile.strongestBasicChallengeMeetsMinimum) {
    actions.push(
      `raise the strongest basic challenge from ${profile.strongestBasicChallengeBits} bits to at least ${profile.basicMinChallengeBits} bits`,
    );
  }
  if (!profile.rootKdfMeetsProductionMinimum) {
    actions.push("raise the root-key KDF profile to the documented production minimum");
  }
  if (!profile.answerKdfMeetsProductionMinimum) {
    actions.push("raise the answer KDF profile to the documented production minimum");
  }
  if (actions.length === 0) {
    actions.push("keep the current question set and runtime profile");
  }
  return actions;
}

async function defaultAnalyzeStrength({ questions }) {
  return analyzeIdentityConfig({ questions: ensureQuestions(questions) });
}

export class StrengthReviewSkill {
  constructor({ analyzer = defaultAnalyzeStrength } = {}) {
    if (typeof analyzer !== "function") {
      throw new ValidationError("analyzer must be a function");
    }
    this.analyzer = analyzer;
  }

  skillId() {
    return "strength_review";
  }

  async run({ questions }) {
    const profile = await this.analyzer({ questions: ensureQuestions(questions) });
    const questionSetReady =
      profile.formalEligibleQuestionCount >= profile.requiredBasicQuestionCount &&
      profile.strongestBasicChallengeMeetsMinimum;

    return {
      ...profile,
      questionSetReady,
      recommendedActions: buildRecommendedActions(profile),
    };
  }
}
