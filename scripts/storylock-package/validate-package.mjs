import { validateStoryLockPackage } from "../../src/shared/storylock-package/index.js";
import { withLoadedPackage } from "./cli-common.mjs";

await withLoadedPackage("validate-package", async (packageData) => validateStoryLockPackage(packageData));
