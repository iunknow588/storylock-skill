import { derivePermissionSummary } from "./resource-catalog.js";

const FORBIDDEN_SUMMARY_KEYS = new Set([
  "storyText",
  "storyBody",
  "canonicalAnswer",
  "acceptedAnswers",
  "answer",
  "password",
  "privateKey",
  "seed",
  "signingKeyBytes",
  "challengeMaterial",
]);

export function createPermissionSummary(catalog) {
  return {
    items: derivePermissionSummary(catalog).map((item) => ({
      resourceId: item.resourceId,
      resourceKind: item.resourceKind,
      providerId: item.providerId,
      displayName: item.displayName,
      role: item.role,
      objectId: item.objectId,
      objectKind: item.objectKind,
      sensitivity: item.sensitivity,
      action: permissionAction(item.objectKind),
      challengePolicy: permissionChallengePolicy(item.sensitivity),
      requiredGridCount: permissionRequiredGridCount(item.sensitivity),
    })),
  };
}

export function permissionAction(objectKind) {
  if (objectKind === "private_key" || objectKind === "signing_key") {
    return "sign";
  }
  if (objectKind === "password") {
    return "password_fill";
  }
  return "read";
}

export function permissionChallengePolicy(sensitivity) {
  return sensitivity === "secret" || sensitivity === "high" ? "high" : "medium";
}

export function permissionRequiredGridCount(sensitivity) {
  return sensitivity === "secret" || sensitivity === "high" ? 12 : 6;
}

export function validatePermissionSummary(summary) {
  const errors = [];
  for (let index = 0; index < (summary?.items ?? []).length; index += 1) {
    const item = summary.items[index];
    for (const key of Object.keys(item ?? {})) {
      if (FORBIDDEN_SUMMARY_KEYS.has(key)) {
        errors.push({
          code: "SLP-501",
          level: "error",
          path: `$.items[${index}].${key}`,
          message: `Permission summary must not expose ${key}.`,
          suggestion: "Keep private story, answer, password, and key material inside StoryLock Core.",
        });
      }
    }
  }
  return {
    valid: errors.length === 0,
    errors,
    warnings: [],
    infos: [],
  };
}
