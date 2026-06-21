import {
  uniqueBy,
  validateArray,
  validateObject,
  validateOptionalString,
  validateRequiredString,
} from "./validation.js";

const TEMPLATE_TYPES = new Set(["login-sites", "signing-actions", "agent-tasks"]);

function validateTemplateItem(item, path, issues) {
  if (!validateObject(item, path, issues)) {
    return;
  }
  validateRequiredString(item.templateId, `${path}.templateId`, issues);
  validateRequiredString(item.resourceId, `${path}.resourceId`, issues);
  validateOptionalString(item.displayName, `${path}.displayName`, issues);

  if (validateArray(item.bindings, `${path}.bindings`, issues, { minItems: 1 })) {
    uniqueBy(item.bindings, (binding) => binding?.fieldName ?? binding?.role, `${path}.bindings`, "fieldName or role", issues);
    for (let index = 0; index < item.bindings.length; index += 1) {
      const binding = item.bindings[index];
      const bindingPath = `${path}.bindings[${index}]`;
      if (!validateObject(binding, bindingPath, issues)) {
        continue;
      }
      validateOptionalString(binding.fieldName, `${bindingPath}.fieldName`, issues);
      validateRequiredString(binding.role, `${bindingPath}.role`, issues);
    }
  }
}

export function validateTemplateBundle(bundle) {
  const issues = [];
  if (!validateObject(bundle, "$", issues)) {
    return { valid: false, errors: issues, warnings: [], infos: [] };
  }
  validateOptionalString(bundle.version, "$.version", issues);
  validateRequiredString(bundle.templateType, "$.templateType", issues);
  if (bundle.templateType && !TEMPLATE_TYPES.has(bundle.templateType)) {
    issues.push({
      code: "SLP-301",
      level: "error",
      path: "$.templateType",
      message: `Unsupported templateType: ${bundle.templateType}.`,
      suggestion: "Use login-sites, signing-actions, or agent-tasks.",
    });
  }
  if (validateArray(bundle.items, "$.items", issues)) {
    uniqueBy(bundle.items, (item) => item?.templateId, "$.items", "templateId", issues);
    for (let index = 0; index < bundle.items.length; index += 1) {
      validateTemplateItem(bundle.items[index], `$.items[${index}]`, issues);
    }
  }
  return {
    valid: issues.length === 0,
    errors: issues,
    warnings: [],
    infos: [],
  };
}
