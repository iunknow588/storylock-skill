# StoryLock Migration Notes

## Overview

This package is a migrated skill-layer bundle, not a full rewrite of `story-lock`.

## Migration targets

1. keep the same high-level skill names
2. keep input validation behavior
3. keep demo paths runnable inside the new package
4. keep Rust/WASM packaging visible but separated from the JS skill-layer

## Current status

1. JS skill-layer is locally runnable
2. demo and selftest are package-local
3. Rust/WASM artifacts are buildable and self-testable
4. full in-package runtime replacement is still incomplete

## Notes for review

Use this file when checking whether a change is packaging-only or behavior-changing.
