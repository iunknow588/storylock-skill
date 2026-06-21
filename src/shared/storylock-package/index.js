import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { validateAuthorDraft } from "./author-draft.js";
import { REQUIRED_PACKAGE_FILES, validatePackageManifest } from "./manifest.js";
import { createPermissionSummary, validatePermissionSummary } from "./permission-summary.js";
import { normalizeResourceCatalog, validateResourceCatalog } from "./resource-catalog.js";
import { validateTemplateBundle } from "./templates.js";

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function mergeResults(results) {
  const errors = results.flatMap((item) => item.errors ?? []);
  const warnings = results.flatMap((item) => item.warnings ?? []);
  const infos = results.flatMap((item) => item.infos ?? []);
  return {
    valid: errors.length === 0,
    errors,
    warnings,
    infos,
  };
}

export {
  REQUIRED_PACKAGE_FILES,
  createPermissionSummary,
  normalizeResourceCatalog,
  validateAuthorDraft,
  validatePackageManifest,
  validatePermissionSummary,
  validateResourceCatalog,
  validateTemplateBundle,
};

export async function loadStoryLockPackage(packageDir) {
  const manifest = await readJson(join(packageDir, "package-manifest.json"));
  const catalog = await readJson(join(packageDir, "resource-catalog.json"));
  const authorDraft = await readJson(join(packageDir, "author-draft.json"));
  const templates = {
    loginSites: await readJson(join(packageDir, "templates", "login-sites.json")),
    signingActions: await readJson(join(packageDir, "templates", "signing-actions.json")),
    agentTasks: await readJson(join(packageDir, "templates", "agent-tasks.json")),
  };
  return {
    manifest,
    catalog,
    authorDraft,
    templates,
  };
}

export function validateStoryLockPackage(packageData) {
  const templateResults = Object.values(packageData.templates ?? {}).map((bundle) =>
    validateTemplateBundle(bundle),
  );
  const permissionSummary = createPermissionSummary(packageData.catalog);
  return mergeResults([
    validatePackageManifest(packageData.manifest),
    validateResourceCatalog(packageData.catalog),
    validateAuthorDraft(packageData.authorDraft),
    validatePermissionSummary(permissionSummary),
    validateTemplateResourceReferences(packageData),
    ...templateResults,
  ]);
}

export function inspectStoryLockPackage(packageData) {
  const validation = validateStoryLockPackage(packageData);
  const permissionSummary = createPermissionSummary(packageData.catalog);
  return {
    ...validation,
    summary: {
      packageId: packageData.manifest?.packageId ?? null,
      resources: packageData.catalog?.resources?.length ?? 0,
      permissionObjects: permissionSummary.items.length,
      storyNodes: packageData.authorDraft?.nodes?.length ?? 0,
      templates: Object.values(packageData.templates ?? {}).reduce(
        (total, bundle) => total + (bundle?.items?.length ?? 0),
        0,
      ),
      requiredFiles: REQUIRED_PACKAGE_FILES,
      permissionSummary,
    },
  };
}

function validateTemplateResourceReferences(packageData) {
  const resourceIds = new Set((packageData.catalog?.resources ?? []).map((resource) => resource.resourceId));
  const roleIndex = new Map();
  for (const resource of packageData.catalog?.resources ?? []) {
    roleIndex.set(resource.resourceId, new Set((resource.bindings ?? []).map((binding) => binding.role)));
  }

  const errors = [];
  for (const [bundleName, bundle] of Object.entries(packageData.templates ?? {})) {
    for (let itemIndex = 0; itemIndex < (bundle?.items ?? []).length; itemIndex += 1) {
      const item = bundle.items[itemIndex];
      const itemPath = `$.templates.${bundleName}.items[${itemIndex}]`;
      if (!resourceIds.has(item.resourceId)) {
        errors.push({
          code: "SLP-701",
          level: "error",
          path: `${itemPath}.resourceId`,
          message: `Template references unknown resourceId: ${item.resourceId}.`,
          suggestion: "Add the resource to resource-catalog.json or update the template resourceId.",
        });
        continue;
      }
      const roles = roleIndex.get(item.resourceId) ?? new Set();
      for (let bindingIndex = 0; bindingIndex < (item.bindings ?? []).length; bindingIndex += 1) {
        const binding = item.bindings[bindingIndex];
        if (!roles.has(binding.role)) {
          errors.push({
            code: "SLP-702",
            level: "error",
            path: `${itemPath}.bindings[${bindingIndex}].role`,
            message: `Template role ${binding.role} is not defined under resourceId ${item.resourceId}.`,
            suggestion: "Use a role declared in resource-catalog.json bindings.",
          });
        }
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
