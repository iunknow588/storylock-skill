import { writeFileSync } from "node:fs";
import { loadStoryLockPackage } from "../../src/shared/storylock-package/index.js";
import { toIssue } from "../../src/shared/storylock-package/errors.js";

export function parseInputDir(argv = process.argv.slice(2)) {
  const index = argv.indexOf("--input");
  if (index >= 0 && argv[index + 1]) {
    return argv[index + 1];
  }
  const positional = argv.find((item) => !item.startsWith("--"));
  if (positional) {
    return positional;
  }
  return ".temp/storylock-package";
}

export function printResult(command, result) {
  const errors = result.errors ?? [];
  const output = {
    status: errors.length === 0 ? "ok" : "failed",
    command,
    errors,
    warnings: result.warnings ?? [],
    infos: result.infos ?? [],
    summary: result.summary ?? {},
  };
  process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
  process.exitCode = errors.length === 0 ? 0 : 1;
}

export function printCaught(command, error) {
  printResult(command, {
    errors: [toIssue(error)],
  });
}

export async function withLoadedPackage(command, callback) {
  try {
    const input = parseInputDir();
    const packageData = await loadStoryLockPackage(input);
    const result = await callback(packageData, input);
    printResult(command, result);
  } catch (error) {
    printCaught(command, error);
  }
}

export function writeJson(path, payload) {
  writeFileSync(path, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
}
