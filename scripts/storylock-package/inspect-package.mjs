import { inspectStoryLockPackage } from "../../src/shared/storylock-package/index.js";
import { withLoadedPackage } from "./cli-common.mjs";

await withLoadedPackage("inspect-package", async (packageData) => inspectStoryLockPackage(packageData));
