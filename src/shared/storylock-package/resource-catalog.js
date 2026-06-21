import {
  isPlainObject,
  uniqueBy,
  validateArray,
  validateObject,
  validateObjectId,
  validateOptionalString,
  validateRequiredString,
} from "./validation.js";

export function validateResourceCatalog(catalog) {
  const issues = [];
  if (!validateObject(catalog, "$", issues)) {
    return { valid: false, errors: issues, warnings: [], infos: [] };
  }

  validateOptionalString(catalog.version, "$.version", issues);
  if (validateArray(catalog.resources, "$.resources", issues, { code: "SL_CATALOG_MISSING_RESOURCES" })) {
    uniqueBy(
      catalog.resources,
      (item) => item?.resourceId,
      "$.resources",
      "resourceId",
      issues,
      "SL_CATALOG_DUPLICATE_RESOURCE_ID",
    );

    for (let resourceIndex = 0; resourceIndex < catalog.resources.length; resourceIndex += 1) {
      const resource = catalog.resources[resourceIndex];
      const resourcePath = `$.resources[${resourceIndex}]`;
      if (!validateObject(resource, resourcePath, issues)) {
        continue;
      }
      validateRequiredString(resource.resourceId, `${resourcePath}.resourceId`, issues);
      validateRequiredString(resource.resourceKind, `${resourcePath}.resourceKind`, issues);
      validateRequiredString(resource.providerId, `${resourcePath}.providerId`, issues);
      validateOptionalString(resource.displayName, `${resourcePath}.displayName`, issues);

      if (validateArray(resource.bindings, `${resourcePath}.bindings`, issues, { minItems: 1 })) {
        uniqueBy(
          resource.bindings,
          (item) => item?.role,
          `${resourcePath}.bindings`,
          "role",
          issues,
        );
        for (let bindingIndex = 0; bindingIndex < resource.bindings.length; bindingIndex += 1) {
          const binding = resource.bindings[bindingIndex];
          const bindingPath = `${resourcePath}.bindings[${bindingIndex}]`;
          if (!validateObject(binding, bindingPath, issues)) {
            continue;
          }
          validateRequiredString(binding.role, `${bindingPath}.role`, issues);
          validateObjectId(binding.objectId, `${bindingPath}.objectId`, issues, "SL_CATALOG_INVALID_OBJECT_ID");
          if (binding.objectMeta != null && validateObject(binding.objectMeta, `${bindingPath}.objectMeta`, issues)) {
            validateRequiredString(binding.objectMeta.objectKind, `${bindingPath}.objectMeta.objectKind`, issues);
            validateOptionalString(binding.objectMeta.encoding, `${bindingPath}.objectMeta.encoding`, issues);
            validateOptionalString(binding.objectMeta.sensitivity, `${bindingPath}.objectMeta.sensitivity`, issues);
          }
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

export function normalizeResourceCatalog(catalog) {
  const result = validateResourceCatalog(catalog);
  if (!result.valid) {
    return { catalog: null, ...result };
  }
  const resources = catalog.resources.map((resource) => ({
    ...resource,
    bindings: resource.bindings.map((binding) => ({ ...binding })),
  }));
  return {
    valid: true,
    errors: [],
    warnings: [],
    infos: [],
    catalog: {
      version: catalog.version ?? "1",
      resources,
    },
  };
}

export function buildResourceRoleIndex(catalog) {
  const index = new Map();
  for (const resource of catalog?.resources ?? []) {
    const roleIndex = new Map();
    for (const binding of resource.bindings ?? []) {
      roleIndex.set(binding.role, binding);
    }
    index.set(resource.resourceId, { ...resource, roleIndex });
  }
  return index;
}

export function derivePermissionSummary(catalog) {
  if (!isPlainObject(catalog)) {
    return [];
  }
  const summary = [];
  for (const resource of catalog.resources ?? []) {
    for (const binding of resource.bindings ?? []) {
      summary.push({
        resourceId: resource.resourceId,
        resourceKind: resource.resourceKind,
        providerId: resource.providerId,
        displayName: resource.displayName ?? resource.resourceId,
        role: binding.role,
        objectId: binding.objectId,
        objectKind: binding.objectMeta?.objectKind ?? "secret",
        sensitivity: binding.objectMeta?.sensitivity ?? "private",
      });
    }
  }
  return summary;
}
