---
name: storylock-skill-engine
description: Package and reuse the StoryLock skill-layer from the story-lock project without changing its existing behavior. Use when Codex needs a reusable StoryLock skill version for demo, documentation, packaging, invocation guidance, or migration into a Pharos-style skill bundle with SKILL.md, references, and assets.
---

# StoryLock Skill Engine

Use this package as the reusable skill-layer bundle migrated from `E:/2026OPC大赛/story-lock`.

## Scope

This bundle covers the existing StoryLock high-level interfaces only:

1. story drafting and refining helpers
2. question-set strength review
3. local password-fill authorization packaging
4. challenge-signing authorization packaging
5. the current agent-style orchestration example above those skills

Keep behavior aligned with the source project. Do not redefine StoryLock core security semantics here.

Current packaging status:

1. the JS skill-layer in this package is locally runnable
2. the package has its own demo and selftest entry
3. Rust / WASM core packaging is not finished yet in this directory

## Source mapping

Treat these source paths in `story-lock` as the authority for the migrated version:

1. `src/adapters/skills/`
2. `src/adapters/agent/`
3. `examples/04-skill-layer-demo.mjs`
4. `src/adapters/skills/README.md`
5. `src/adapters/agent/README.md`

## Capability Index

| User Need | Capability | Detailed Instructions |
| --- | --- | --- |
| Explain StoryLock skill-layer boundaries | skill boundary and scope | `references/boundary.md` |
| Show available StoryLock skills | skill surface and exports | `references/invocation.md` |
| Run a minimal StoryLock skill demo | skill-layer runnable example | `references/demo.md` |
| Build or verify Rust/WASM packaging | wasm build and dist validation | `references/rust-wasm.md` |
| Package StoryLock into a reusable skill bundle | migrated package structure | `references/migration.md` |
| Show the current agent-style orchestration example | `VideoPublishAgentDemo` mapping | `references/agent.md` |

## Working rules

1. Prefer the migrated files in `assets/migrated/` for packaging and review.
2. Prefer the original `story-lock` repo when behavior needs verification.
3. Keep the distinction between core authorization and skill packaging explicit.
4. Describe this bundle as a reusable interface layer, not as the security boundary.
5. When the user asks for product-facing modes, highlight:
   - `LocalPasswordFillSkill`
   - `ChallengeSigningAuthorizationSkill`
6. Use the `assets/` schemas and template files when you need a stable input/output shape.

## Verification path

For a minimal runnable example, use the demo entry described in `references/demo.md`.

Current local commands:

1. `npm run demo`
2. `npm run selftest`
3. `npm run build:wasm`
4. `npm run selftest:wasm`
