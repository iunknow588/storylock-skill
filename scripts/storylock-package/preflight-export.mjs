import { REQUIRED_PACKAGE_FILES, inspectStoryLockPackage } from "../../src/shared/storylock-package/index.js";
import { withLoadedPackage } from "./cli-common.mjs";

await withLoadedPackage("preflight-export", async (packageData) => {
  const result = inspectStoryLockPackage(packageData);
  return {
    ...result,
    summary: {
      ...result.summary,
      exportReady: result.errors.length === 0,
      files: ["vault.stlk", ...REQUIRED_PACKAGE_FILES],
    },
  };
});
