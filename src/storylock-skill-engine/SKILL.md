---
name: storylock-skill-engine
description: StoryLock skill package for AI agents. Use when an agent needs to draft story material, review question-set strength, produce authorized login fields, or produce a challenge-signing authorization result through the migrated StoryLock skill layer.
---

# StoryLock Skill Engine

This package is the migrated compatibility and reference bundle for the earlier StoryLock skill layer.

New development should treat the three current runtime packages as authoritative:

1. `storylock-local-story-processing-skill`
2. `storylock-local-story-access-skill`
3. `storylock-remote-gateway-skill`

This package remains useful for legacy demos, migrated assets, and compatibility schemas, but it is not a fourth runtime security layer.

## Scope

This compatibility package covers four agent-facing capabilities:

1. story draft assistance
2. question-set strength review
3. password-fill authorization packaging
4. challenge-sign authorization packaging

This package does not redefine StoryLock core security semantics. It documents and packages the migrated JS skill layer, plus the current Rust/WASM build path. Security-sensitive access decisions must be delegated to `storylock-local-story-access-skill`.

## Capability Index

| User Intent | Capability | Reference File | Key Parameters |
| --- | --- | --- | --- |
| "Help me draft a story" | story draft generation | `references/story-assist.md` | `objective`, `audience`, `tone`, `constraints` |
| "Check whether these 24 questions are strong enough" | strength review | `references/strength-review.md` | `questions` |
| "Fill login fields for this identity and site" | password fill | `references/password-fill.md` | `identityId`, `siteId`, `bindings`, `answers` |
| "Sign this challenge payload" | challenge sign | `references/challenge-sign.md` | `identityId`, `keyId`, `algorithm`, `payload`, `answers` |
| "Explain boundaries and safety limits" | skill boundary | `references/boundary.md` | none |
| "Run the package demo or self-test" | demo and verification | `references/demo.md` | none |
| "Check current Rust/WASM packaging status" | wasm packaging status | `references/rust-wasm.md` | none |

## Write Operation Pre-checks

Before any authorization-oriented write or signing flow, verify all of the following:

1. Identity context: `identityId` exists and matches the intended user context.
2. Challenge lifecycle: a fresh challenge was created before answers are submitted.
3. Session validity: the returned authorization includes a valid `sessionId` before secrets are read or signing output is assembled.
4. Scope coverage: the requested scope and object references match the target resource, site, key, and attachments.

## Package Layout

Use these directories as part of the runtime contract:

1. `references/` contains machine-readable operator guidance for each capability.
2. `assets/schemas/` contains stable input and output schemas for story drafting, password fill, challenge sign, and strength review.
3. `assets/templates/` contains reusable request templates for story drafting, password fill, and challenge sign.
4. `agents/openai.yaml` contains agent integration metadata.
5. `llms.txt` provides a compact package index for LLM-oriented discovery.

## Working Rules

1. Prefer capability-specific reference files before improvising parameters or output claims.
2. Keep the distinction between legacy skill packaging and StoryLock core authorization explicit.
3. Treat `assets/migrated/` as the packaged implementation source for this bundle.
4. Describe the current runtime honestly: this compatibility layer is runnable, while the three-package runtime is the active implementation surface.

## Verification Path

For local validation, use:

1. `npm run demo`
2. `npm run selftest`
3. `npm run build:wasm`
4. `npm run selftest:wasm`
