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
  validateRequiredString(manifest.packageId, "$.packageId", issues, "SL_MANIFEST_MISSING_PACKAGE_ID");
  validateRequiredString(manifest.version, "$.version", issues, "SL_MANIFEST_UNSUPPORTED_VERSION");
  validateRequiredString(manifest.createdAt, "$.createdAt", issues);
  validateOptionalString(manifest.description, "$.description", issues);
  if (validateArray(manifest.files, "$.files", issues, { minItems: 1, code: "SL_MANIFEST_MISSING_CATALOG_FILE" })) {
    for (const requiredFile of REQUIRED_PACKAGE_FILES) {
      if (!manifest.files.includes(requiredFile)) {
        const code = requiredFile === "package-manifest.json"
          ? "SL_PKG_MISSING_MANIFEST"
          : requiredFile === "resource-catalog.json"
            ? "SL_MANIFEST_MISSING_CATALOG_FILE"
            : requiredFile === "vault.stlk"
              ? "SL_MANIFEST_MISSING_VAULT_FILE"
              : "SL_PKG_OPTIONAL_FILE_MISSING";
        issues.push(createIssue({
          code,
          level: code === "SL_PKG_OPTIONAL_FILE_MISSING" ? "info" : "error",
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
