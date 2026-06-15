import { mkdirSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const skillRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const workspaceRoot = resolve(skillRoot, "../../..");
const rustRepoRoot = resolve(workspaceRoot, "story-lock");
const outDir = resolve(skillRoot, "dist/wasm");

mkdirSync(outDir, { recursive: true });

const args = [
  "build",
  "--target",
  "web",
  "--out-dir",
  outDir,
  "--out-name",
  "story_lock_vault",
  "--features",
  "wasm-bindings",
];

const result = spawnSync("wasm-pack", args, {
  cwd: rustRepoRoot,
  stdio: "inherit",
  shell: true,
});

if (result.error) {
  console.error("Failed to start wasm-pack.");
  console.error("Install wasm-pack first, then rerun `npm run build:wasm`.");
  process.exit(1);
}

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

console.log(`StoryLock skill-engine WASM artifacts written to ${outDir}`);
