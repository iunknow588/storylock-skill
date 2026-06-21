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

  if (validateArray(draft.nodes, "$.nodes", issues, { minItems: STORY_NODE_COUNT }) && draft.nodes.length !== STORY_NODE_COUNT) {
    issues.push({
      code: "SLP-401",
      level: "error",
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
  }

  return {
    valid: issues.length === 0,
    errors: issues,
    warnings: [],
    infos: [],
  };
}
