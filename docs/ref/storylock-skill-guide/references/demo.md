# StoryLock Skill Demo

## 1. Compatibility Package Demo

```powershell
cd E:\2026OPC大赛\skill\src\engine
npm run demo
npm run selftest
```

These validate the migrated compatibility package directly.

## 2. Rust / WASM Packaging Path

```powershell
cd E:\2026OPC大赛\skill\src\engine
npm run build:wasm
npm run selftest:wasm
```

This proves:

1. the compatibility package can trigger the Rust/WASM build path
2. `dist/wasm` artifacts are produced
3. exported bindings can be loaded

## 3. Mainline Runtime Demo

```powershell
cd E:\2026OPC大赛\skill\src\skills\remote-gateway
npm run selftest:e2e
```

Use the `src/skills/*` runtime directories for current end-to-end demos. Use `src/engine` only for compatibility and migrated-package validation.
