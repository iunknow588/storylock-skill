# StoryLock Rust / WASM

## Overview

Use this page when the task is about build status, packaging verification, or truthfully describing the current Rust/WASM integration boundary.

Current state:

1. the package includes a Rust/WASM build path
2. generated `dist/wasm` artifacts can be validated
3. the JS skill layer is still the primary runtime path for the documented capabilities

## Command Template

```powershell
npm run build:wasm
npm run selftest:wasm
```

## Parameters

This reference does not define runtime input parameters. It defines operator commands only.

## Output Parsing

Successful verification means:

1. the build completed without fatal toolchain errors
2. `dist/wasm` artifacts exist
3. exported bindings are loadable by the dist self-test

This does not mean:

1. every skill call already routes through a local WASM host
2. the package is already a fully independent Rust distribution

## Error Handling

| Case | Meaning | Fix |
| --- | --- | --- |
| build command fails | Rust or `wasm-pack` toolchain is unavailable or misconfigured | inspect the local Rust toolchain and build script |
| dist self-test fails | generated exports are missing or incompatible | rebuild and inspect `dist/wasm` artifacts |
| user asks whether Rust migration is complete | current package is being overclaimed | state clearly that full JS-to-WASM host replacement is not finished |

## Agent Guidelines

1. Use this page only for packaging status and build validation.
2. Do not present WASM build success as proof of full runtime migration.
3. Keep the distinction between artifact validation and runtime completeness explicit.
