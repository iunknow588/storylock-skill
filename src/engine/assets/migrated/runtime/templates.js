import { STORY_NODE_COUNT, DEFAULT_VERIFY_POLICY } from "./constants.js";
import { ValidationError } from "./errors.js";
import { resolveSecretReference } from "./resource-catalog.js";

export function createDefaultStoryTemplate() {
  return Array.from({ length: STORY_NODE_COUNT }, (_, index) => ({
    nodeId: `story_${String(index + 1).padStart(2, "0")}`,
    question: `Story node ${String(index + 1).padStart(2, "0")}`,
    answerType: "text",
    verifyPolicy: { ...DEFAULT_VERIFY_POLICY },
    trainingHints: [],
  }));
}

function createWebsiteTemplate({ siteId, resourceId, fieldBindings }) {
  return Object.freeze({
    siteId,
    resourceId,
    fieldBindings: Object.freeze(
      fieldBindings.map((binding) =>
        Object.freeze({
          fieldName: binding.fieldName,
          role: binding.role,
          secretObjectId: binding.secretObjectId,
        }),
      ),
    ),
    bindings: Object.freeze(
      fieldBindings.map((binding) =>
        Object.freeze({
          fieldName: binding.fieldName,
          secretObjectId: binding.secretObjectId,
        }),
      ),
    ),
  });
}

const DEFAULT_LOGIN_SITE_TEMPLATES = Object.freeze([
  createWebsiteTemplate({
    siteId: "generic_username_password",
    resourceId: "generic-main",
    fieldBindings: [
      {
        fieldName: "username",
        role: "username",
        secretObjectId: "credential/generic/main/username",
      },
      {
        fieldName: "password",
        role: "password",
        secretObjectId: "credential/generic/main/password",
      },
    ],
  }),
]);

function ensureNonEmptyString(value, fieldName) {
  if (!value || typeof value !== "string") {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

function materializeLoginBinding(binding, defaultResourceId, resourceCatalog) {
  if (!binding || typeof binding !== "object") {
    throw new ValidationError("binding must be an object");
  }

  const fieldName = ensureNonEmptyString(binding.fieldName, "binding.fieldName");
  return {
    fieldName,
    secretObjectId: resolveSecretReference({
      resourceCatalog,
      resourceId: binding.resourceId ?? defaultResourceId ?? null,
      role: binding.role ?? null,
      secretObjectId: binding.secretObjectId ?? binding.objectId ?? null,
      fieldName: `binding ${fieldName}`,
    }),
  };
}

function cloneTemplate(template) {
  return {
    siteId: template.siteId,
    resourceId: template.resourceId ?? null,
    fieldBindings: (template.fieldBindings ?? []).map((binding) => ({ ...binding })),
    bindings: (template.bindings ?? []).map((binding) => ({ ...binding })),
  };
}

export const LOGIN_BINDING_MODE = Object.freeze({
  MANUAL_ONLY: "manual_only",
  TEMPLATE_ONLY: "template_only",
  TEMPLATE_WITH_OVERRIDES: "template_with_overrides",
});

export function createDefaultLoginSiteTemplates() {
  return DEFAULT_LOGIN_SITE_TEMPLATES.map((template) => cloneTemplate(template));
}

export function getDefaultLoginSiteTemplate(siteId) {
  return createDefaultLoginSiteTemplates().find((template) => template.siteId === siteId) ?? null;
}

export function resolveLoginFormBindings({
  siteId,
  resourceId = null,
  resourceCatalog = null,
  bindings = [],
  bindingMode = LOGIN_BINDING_MODE.MANUAL_ONLY,
}) {
  const template = getDefaultLoginSiteTemplate(siteId);
  let resolved;

  switch (bindingMode) {
    case LOGIN_BINDING_MODE.MANUAL_ONLY:
      resolved = bindings.map((binding) =>
        materializeLoginBinding(binding, resourceId, resourceCatalog),
      );
      break;
    case LOGIN_BINDING_MODE.TEMPLATE_ONLY:
      if (!template) {
        throw new Error(`login template not found: ${siteId}`);
      }
      resolved = template.fieldBindings.map((binding) =>
        materializeLoginBinding(binding, resourceId ?? template.resourceId, resourceCatalog),
      );
      break;
    case LOGIN_BINDING_MODE.TEMPLATE_WITH_OVERRIDES: {
      resolved = template
        ? template.fieldBindings.map((binding) =>
            materializeLoginBinding(
              binding,
              resourceId ?? template.resourceId,
              resourceCatalog,
            ),
          )
        : [];
      for (const binding of bindings) {
        const nextBinding = materializeLoginBinding(
          binding,
          resourceId ?? template?.resourceId ?? null,
          resourceCatalog,
        );
        const existing = resolved.findIndex((item) => item.fieldName === nextBinding.fieldName);
        if (existing >= 0) {
          resolved[existing] = nextBinding;
        } else {
          resolved.push(nextBinding);
        }
      }
      break;
    }
    default:
      throw new Error(`unsupported login binding mode: ${bindingMode}`);
  }

  if (resolved.length === 0) {
    throw new Error("login request resolved to zero field bindings");
  }

  return resolved;
}
