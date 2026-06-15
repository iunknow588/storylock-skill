# StoryLock Skill Demo

## 1. Package-local demo

Current preferred commands:

```powershell
cd E:\2026OPC大赛\skill\src\storylock-skill-engine
npm run demo
npm run selftest
```

These validate the migrated package directly.

What they show:

1. story draft output
2. password-fill output
3. challenge-signing output
4. the package can run as a self-contained JS skill-layer bundle

## 2. Rust / WASM packaging path

Current commands:

```powershell
cd E:\2026OPC大赛\skill\src\storylock-skill-engine
npm run build:wasm
npm run selftest:wasm
```

What this proves:

1. the migrated package can trigger the Rust/WASM build path
2. the generated `dist/wasm` artifacts exist
3. exported bindings are loadable

What this does not yet prove:

1. that all JS flows run through a local WASM host layer
2. that the package is already a fully independent Rust distribution

## 3. Reporting guidance

When summarizing results:

1. say the migrated package already has a runnable JS skill-layer
2. say Rust/WASM build and dist validation are wired in
3. do not say the entire runtime has been fully re-hosted
