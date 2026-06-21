import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { inspectStoryLockPackage, loadStoryLockPackage, validateStoryLockPackage } from "../../src/shared/storylock-package/index.js";

const root = fileURLToPath(new URL("../../", import.meta.url));
const validFixture = fileURLToPath(new URL("./fixtures/storylock-package/valid/", import.meta.url));
const invalidFixture = fileURLToPath(new URL("./fixtures/storylock-package/invalid/", import.meta.url));

const packageData = await loadStoryLockPackage(validFixture);
const validation = validateStoryLockPackage(packageData);
assert.equal(validation.valid, true);
assert.equal(validation.errors.length, 0);

const inspection = inspectStoryLockPackage(packageData);
assert.equal(inspection.summary.resources, 2);
assert.equal(inspection.summary.permissionObjects, 4);
assert.equal(inspection.summary.storyNodes, 24);
assert.equal(inspection.summary.templates, 3);
assert.equal(inspection.summary.permissionSummary.items.length, 4);
assert.ok(
  inspection.summary.permissionSummary.items.every(
    (item) =>
      item.action &&
      item.challengePolicy &&
      item.requiredGridCount &&
      item.password === undefined &&
      item.privateKey === undefined &&
      item.signingKeyBytes === undefined,
  ),
);
assert.equal(
  inspection.summary.permissionSummary.items.find((item) => item.objectKind === "password").action,
  "password_fill",
);
assert.equal(
  inspection.summary.permissionSummary.items.find((item) => item.objectKind === "private_key").action,
  "sign",
);
assert.equal(
  inspection.summary.permissionSummary.items.find((item) => item.sensitivity === "secret").requiredGridCount,
  12,
);

function runScript(script, input) {
  const result = spawnSync(process.execPath, [script, "--input", input], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.stderr, "");
  return {
    status: result.status,
    output: JSON.parse(result.stdout),
  };
}

const validCli = runScript("scripts/storylock-package/validate-package.mjs", validFixture);
assert.equal(validCli.status, 0);
assert.equal(validCli.output.status, "ok");

const inspectCli = runScript("scripts/storylock-package/inspect-package.mjs", validFixture);
assert.equal(inspectCli.status, 0);
assert.equal(inspectCli.output.summary.permissionObjects, 4);
assert.equal(inspectCli.output.summary.permissionSummary.items.length, 4);

const invalidCli = runScript("scripts/storylock-package/validate-package.mjs", invalidFixture);
assert.notEqual(invalidCli.status, 0);
assert.equal(invalidCli.output.status, "failed");
assert.ok(invalidCli.output.errors.some((item) => item.code === "SL_CATALOG_INVALID_OBJECT_ID"));
assert.ok(invalidCli.output.errors.some((item) => item.code === "SL_PKG_MISSING_MANIFEST"));
assert.ok(invalidCli.output.errors.every((item) => item.severity));

const leakedPackageData = structuredClone(packageData);
leakedPackageData.templates.loginSites.items[0].bindings[0].canonicalAnswer = "do-not-export";
const leakedValidation = validateStoryLockPackage(leakedPackageData);
assert.equal(leakedValidation.valid, false);
assert.ok(leakedValidation.errors.some((item) => item.code === "SL_PKG_HOST_READABLE_SECRET_FIELD"));

console.log(JSON.stringify({
  status: "passed",
  validFixture,
  invalidFixture,
}, null, 2));
