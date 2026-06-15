---
name: storylock-skill-guide
description: Explain, package, invoke, demo, or review the StoryLock skill-layer and the migrated `storylock-skill-engine` package. Use when Codex needs to describe StoryLock skill boundaries, show runnable invocation examples, explain current JS skill-layer packaging status, or clarify what is and is not already integrated from the Rust/WASM core.
---

# StoryLock Skill Guide

This guide is the technical appendix for the StoryLock skill package and its competition-facing documentation.

## Scope

Use this guide for:

1. StoryLock skill-layer boundaries
2. StoryLock skill invocation examples
3. StoryLock migrated package demo paths
4. current Rust / WASM packaging status

Do not use this guide to claim that StoryLock core security semantics were rewritten here.

## Current package status

The migrated package at `skill/src/storylock-skill-engine/` now has:

1. a locally runnable JS skill-layer
2. local `demo` and `selftest` commands
3. a Rust / WASM build path and dist self-test

It still does not yet have:

1. a fully independent Rust source package
2. a complete in-package host/runtime layer replacing every original `story-lock` path

## Capability Index

| User Need | Capability | Detailed Instructions |
| --- | --- | --- |
| Explain StoryLock skill boundaries | boundary and responsibility split | `references/boundary.md` |
| Show how to call StoryLock skills | package exports and invocation map | `references/invocation.md` |
| Draft or refine a StoryLock story | story assist workflow | `references/story-assist.md` |
| Review whether a question set is strong enough | strength review workflow | `references/strength-review.md` |
| Produce authorized login fields | password-fill workflow | `references/password-fill.md` |
| Produce or explain challenge signing | challenge-sign workflow | `references/challenge-sign.md` |
| Show runnable demo paths | package demo / selftest / browser demo | `references/demo.md` |

## Working rules

1. Keep core authorization and skill packaging distinct.
2. Prefer the migrated package path when the user asks for the current reusable version:
   - `skill/src/storylock-skill-engine/`
3. Say clearly when a capability is JS-local only versus Rust/WASM-backed.
4. Do not describe the current package as fully Rust-complete.
5. Use the capability-specific reference page before improvising parameter or output claims.
