import { createIssue } from "./errors.js";
import { validateArray, validateObject, validateOptionalString, validateRequiredString } from "./validation.js";

export const REQUIRED_PACKAGE_FILES = Object.freeze([
  "package-manifest.json",
  "resource-catalog.json",
  "author-draft.json",
  "templates/login-sites.json",
  "templates/signing-actions.json",
  "templates/agent-tasks.json",
]);

export function validatePackageManifest(manifest) {
  const issues = [];
  if (!validateObject(manifest, "$", issues)) {
    return { valid: false, errors: issues, warnings: [], infos: [] };
  }
  validateRequiredString(manifest.packageId, "$.packageId", issues);
  validateRequiredString(manifest.version, "$.version", issues);
  validateRequiredString(manifest.createdAt, "$.createdAt", issues);
  validateOptionalString(manifest.description, "$.description", issues);
  if (validateArray(manifest.files, "$.files", issues, { minItems: 1 })) {
    for (const requiredFile of REQUIRED_PACKAGE_FILES) {
      if (!manifest.files.includes(requiredFile)) {
        issues.push(createIssue({
          code: "SLP-601",
          path: "$.files",
          message: `package-manifest.json must list ${requiredFile}.`,
          suggestion: "Add every required StoryLock package file before export.",
        }));
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
