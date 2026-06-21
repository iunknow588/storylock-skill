import { readFile } from "node:fs/promises";
import { createPermissionSummary } from "../../src/shared/storylock-package/index.js";

const inputIndex = process.argv.findIndex((item) => item === "--input");
const input = inputIndex >= 0
  ? process.argv[inputIndex + 1]
  : process.argv.find((item, index) => index > 1 && !item.startsWith("--"));

if (!input) {
  console.error("Usage: node scripts/storylock-package/permission-summary-json.mjs [--input] <resource-catalog.json>");
  process.exit(2);
}

const catalog = JSON.parse(await readFile(input, "utf8"));
process.stdout.write(`${JSON.stringify(createPermissionSummary(catalog), null, 2)}\n`);
