import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const skillRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const wasmJsPath = resolve(skillRoot, "dist/wasm/story_lock_vault.js");
const wasmBinPath = resolve(skillRoot, "dist/wasm/story_lock_vault_bg.wasm");

async function main() {
  if (!existsSync(wasmJsPath) || !existsSync(wasmBinPath)) {
    console.log(
      "WASM self-test skipped: dist/wasm artifacts are not built yet. Run `npm run build:wasm` first.",
    );
    return;
  }

  const wasmModule = new WebAssembly.Module(readFileSync(wasmBinPath));
  const modulePath = pathToFileURL(wasmJsPath).href;
  const moduleNamespace = await import(modulePath);
  const initFunction = moduleNamespace.default ?? moduleNamespace.initSync;

  if (typeof initFunction === "function") {
    await initFunction({ module_or_path: wasmModule });
  }

  assert(typeof moduleNamespace.createVault === "function", "createVault export missing");
  assert(
    typeof moduleNamespace.analyzeCreateVaultRequest === "function",
    "analyzeCreateVaultRequest export missing",
  );
  assert(
    typeof moduleNamespace.exportIdentityPackage === "function",
    "exportIdentityPackage export missing",
  );
  assert(
    typeof moduleNamespace.importIdentityPackage === "function",
    "importIdentityPackage export missing",
  );
  assert(typeof moduleNamespace.StoryLockRuntime === "function", "StoryLockRuntime export missing");

  console.log(
    "StoryLock skill-engine WASM self-test passed: dist artifacts are loadable and core exports are present.",
  );
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
