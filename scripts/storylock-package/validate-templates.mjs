import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { validateTemplateBundle } from "../../src/shared/storylock-package/index.js";
import { parseInputDir, printCaught, printResult } from "./cli-common.mjs";

try {
  const input = parseInputDir();
  const templatePaths = [
    join(input, "templates", "login-sites.json"),
    join(input, "templates", "signing-actions.json"),
    join(input, "templates", "agent-tasks.json"),
  ];
  const results = [];
  for (const templatePath of templatePaths) {
    results.push(validateTemplateBundle(JSON.parse(await readFile(templatePath, "utf8"))));
  }
  printResult("validate-templates", {
    errors: results.flatMap((item) => item.errors),
    warnings: results.flatMap((item) => item.warnings),
    infos: results.flatMap((item) => item.infos),
    summary: { bundles: templatePaths.length },
  });
} catch (error) {
  printCaught("validate-templates", error);
}
