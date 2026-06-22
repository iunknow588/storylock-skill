import { validateArray, validateObject, validateOptionalString, validateRequiredString } from "./validation.js";

export const STORY_NODE_COUNT = 24;

export function validateAuthorDraft(draft) {
  const issues = [];
  if (!validateObject(draft, "$", issues)) {
    return { valid: false, errors: issues, warnings: [], infos: [] };
  }
  validateOptionalString(draft.version, "$.version", issues);
  validateRequiredString(draft.storyTitle, "$.storyTitle", issues);
  validateOptionalString(draft.summary, "$.summary", issues);
  validateArray(draft.memoryAnchors ?? [], "$.memoryAnchors", issues);
  validateArray(draft.elementGroups ?? [], "$.elementGroups", issues);

  if (validateArray(draft.nodes, "$.nodes", issues, { minItems: STORY_NODE_COUNT, code: "SL_PKG_AUTHOR_DRAFT_NODE_COUNT" }) && draft.nodes.length !== STORY_NODE_COUNT) {
    issues.push({
      code: "SL_PKG_AUTHOR_DRAFT_NODE_COUNT",
      level: "error",
      severity: "blocking",
      path: "$.nodes",
      message: `Story author draft must contain exactly ${STORY_NODE_COUNT} nodes.`,
      suggestion: "Create one node for each StoryLock story position.",
    });
  }

  for (let index = 0; index < (draft.nodes ?? []).length; index += 1) {
    const node = draft.nodes[index];
    const path = `$.nodes[${index}]`;
    if (!validateObject(node, path, issues)) {
      continue;
    }
    validateRequiredString(node.nodeId, `${path}.nodeId`, issues);
    validateRequiredString(node.title, `${path}.title`, issues);
    validateRequiredString(node.elementId, `${path}.elementId`, issues);
    validateRequiredString(node.question, `${path}.question`, issues);
    validateOptionalString(node.recommendedSelectionMode, `${path}.recommendedSelectionMode`, issues);
    validateOptionalString(node.verifyPolicy, `${path}.verifyPolicy`, issues);
    validateOptionalString(node.editorNotes, `${path}.editorNotes`, issues);
    if (node.answerOptionsLocalOnly != null) {
      if (validateArray(node.answerOptionsLocalOnly, `${path}.answerOptionsLocalOnly`, issues, { minItems: 9, code: "SL_PKG_AUTHOR_DRAFT_ANSWER_OPTIONS" }) && node.answerOptionsLocalOnly.length !== 9) {
        issues.push({
          code: "SL_PKG_AUTHOR_DRAFT_ANSWER_OPTIONS",
          level: "error",
          severity: "blocking",
          path: `${path}.answerOptionsLocalOnly`,
          message: "Each StoryLock node must contain exactly 9 local answer options when answer options are configured.",
          suggestion: "Provide 9 candidate answers and mark each one as correct or incorrect.",
        });
      }
      for (let optionIndex = 0; optionIndex < (node.answerOptionsLocalOnly ?? []).length; optionIndex += 1) {
        const option = node.answerOptionsLocalOnly[optionIndex];
        const optionPath = `${path}.answerOptionsLocalOnly[${optionIndex}]`;
        if (!validateObject(option, optionPath, issues)) {
          continue;
        }
        validateRequiredString(option.text, `${optionPath}.text`, issues, "SL_PKG_AUTHOR_DRAFT_ANSWER_OPTION_TEXT");
        if (typeof option.isCorrect !== "boolean") {
          issues.push({
            code: "SL_PKG_AUTHOR_DRAFT_ANSWER_OPTION_FLAG",
            level: "error",
            severity: "blocking",
            path: `${optionPath}.isCorrect`,
            message: "Answer option correctness must be a boolean.",
            suggestion: "Use true for correct options and false for distractors.",
          });
        }
      }
    }
  }

  return {
    valid: issues.length === 0,
    errors: issues,
    warnings: [],
    infos: [],
  };
}
