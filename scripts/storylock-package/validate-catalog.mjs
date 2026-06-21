import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { validateResourceCatalog } from "../../src/shared/storylock-package/index.js";
import { parseInputDir, printCaught, printResult } from "./cli-common.mjs";

try {
  const input = parseInputDir();
  const catalog = JSON.parse(await readFile(join(input, "resource-catalog.json"), "utf8"));
  printResult("validate-catalog", validateResourceCatalog(catalog));
} catch (error) {
  printCaught("validate-catalog", error);
}
