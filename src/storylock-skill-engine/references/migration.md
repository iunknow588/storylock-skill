# StoryLock Migration Notes

## Overview

This page records the migrated package boundary and current packaging status.

## Migration Targets

1. `src/adapters/skills/`
2. `src/adapters/agent/`
3. `examples/04-skill-layer-demo.mjs`

## Current Status

1. the JS skill layer is runnable
2. demo and self-test commands are present
3. Rust/WASM build and dist validation are wired in
4. full WASM-host replacement is not complete

## Agent Guidelines

1. Use this page for status reporting only.
2. Keep migration status separate from capability behavior.
