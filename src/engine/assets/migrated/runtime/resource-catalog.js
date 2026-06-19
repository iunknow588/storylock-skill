import { ValidationError } from "./errors.js";

const STRUCTURED_OBJECT_ID_PATTERN =
  /^[a-z0-9][a-z0-9_-]*(?:\/[a-z0-9][a-z0-9_-]*){3}$/;

function ensureNonEmptyString(value, fieldName) {
  if (!value || typeof value !== "string") {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

function ensureOptionalString(value, fieldName) {
  if (value == null) {
    return null;
  }
  return ensureNonEmptyString(value, fieldName);
}

function ensureArray(value, fieldName) {
  if (!Array.isArray(value)) {
    throw new ValidationError(`${fieldName} must be an array`);
  }
  return value;
}

export function isStructuredObjectId(objectId) {
  return typeof objectId === "string" && STRUCTURED_OBJECT_ID_PATTERN.test(objectId);
}

export function normalizeResourceCatalog(resourceCatalog) {
  if (resourceCatalog == null) {
    return null;
  }
  if (
    resourceCatalog &&
    Array.isArray(resourceCatalog.resources) &&
    resourceCatalog.resourceIndex instanceof Map
  ) {
    return resourceCatalog;
  }
  if (typeof resourceCatalog !== "object") {
    throw new ValidationError("resourceCatalog must be an object");
  }

  const resourceIndex = new Map();
  const normalizedResources = ensureArray(
    resourceCatalog.resources,
    "resourceCatalog.resources",
  ).map((resource, resourceIndexPosition) => {
    if (!resource || typeof resource !== "object") {
      throw new ValidationError(
        `resourceCatalog.resources[${resourceIndexPosition}] must be an object`,
      );
    }

    const resourceId = ensureNonEmptyString(
      resource.resourceId,
      `resourceCatalog.resources[${resourceIndexPosition}].resourceId`,
    );
    if (resourceIndex.has(resourceId)) {
      throw new ValidationError(`resourceCatalog contains duplicate resourceId: ${resourceId}`);
    }

    const roleIndex = new Map();
    const bindings = ensureArray(
      resource.bindings,
      `resourceCatalog.resources[${resourceIndexPosition}].bindings`,
    ).map((binding, bindingIndex) => {
      if (!binding || typeof binding !== "object") {
        throw new ValidationError(
          `resourceCatalog.resources[${resourceIndexPosition}].bindings[${bindingIndex}] must be an object`,
        );
      }

      const role = ensureNonEmptyString(
        binding.role,
        `resourceCatalog.resources[${resourceIndexPosition}].bindings[${bindingIndex}].role`,
      );
      if (roleIndex.has(role)) {
        throw new ValidationError(
          `resource ${resourceId} contains duplicate role binding: ${role}`,
        );
      }

      const objectId = ensureNonEmptyString(
        binding.objectId,
        `resourceCatalog.resources[${resourceIndexPosition}].bindings[${bindingIndex}].objectId`,
      );
      if (!isStructuredObjectId(objectId)) {
        throw new ValidationError(
          `resource ${resourceId} role ${role} must use a four-segment objectId`,
        );
      }

      const normalizedBinding = {
        role,
        objectId,
      };
      roleIndex.set(role, normalizedBinding);
      return normalizedBinding;
    });

    const normalizedResource = {
      resourceId,
      resourceKind: ensureOptionalString(
        resource.resourceKind,
        `resourceCatalog.resources[${resourceIndexPosition}].resourceKind`,
      ),
      providerId: ensureOptionalString(
        resource.providerId,
        `resourceCatalog.resources[${resourceIndexPosition}].providerId`,
      ),
      displayName: ensureOptionalString(
        resource.displayName,
        `resourceCatalog.resources[${resourceIndexPosition}].displayName`,
      ),
      bindings,
      roleIndex,
    };
    resourceIndex.set(resourceId, normalizedResource);
    return normalizedResource;
  });

  return {
    version: resourceCatalog.version ?? 1,
    resources: normalizedResources,
    resourceIndex,
  };
}

export function getResourceCatalogEntry(resourceCatalog, resourceId) {
  const normalizedCatalog = normalizeResourceCatalog(resourceCatalog);
  const normalizedResourceId = ensureNonEmptyString(resourceId, "resourceId");
  if (!normalizedCatalog) {
    throw new ValidationError(
      `resourceCatalog is required to resolve resourceId ${normalizedResourceId}`,
    );
  }

  const entry = normalizedCatalog.resourceIndex.get(normalizedResourceId);
  if (!entry) {
    throw new ValidationError(`resourceId not found in resourceCatalog: ${normalizedResourceId}`);
  }
  return entry;
}

export function resolveResourceObjectId({ resourceCatalog, resourceId, role }) {
  const resourceEntry = getResourceCatalogEntry(resourceCatalog, resourceId);
  const normalizedRole = ensureNonEmptyString(role, "role");
  const binding = resourceEntry.roleIndex.get(normalizedRole);
  if (!binding) {
    const knownRoles = resourceEntry.bindings.map((item) => item.role).join(", ");
    throw new ValidationError(
      `role ${normalizedRole} was not found under resourceId ${resourceEntry.resourceId}` +
        (knownRoles ? `; known roles: ${knownRoles}` : ""),
    );
  }
  return binding.objectId;
}

export function resolveSecretReference({
  resourceCatalog = null,
  resourceId = null,
  role = null,
  secretObjectId = null,
  objectId = null,
  fieldName = "secretReference",
}) {
  const directObjectId = secretObjectId ?? objectId;
  if (directObjectId != null) {
    return ensureNonEmptyString(directObjectId, `${fieldName}.secretObjectId`);
  }

  if (resourceId == null && role == null) {
    throw new ValidationError(
      `${fieldName} must provide either secretObjectId or resourceId + role`,
    );
  }
  if (resourceId == null || role == null) {
    throw new ValidationError(
      `${fieldName} must provide both resourceId and role when secretObjectId is omitted`,
    );
  }

  return resolveResourceObjectId({
    resourceCatalog,
    resourceId,
    role,
  });
}
