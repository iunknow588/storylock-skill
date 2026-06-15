# StoryLock Rust / WASM

## Overview

This package now includes a Rust/WASM build and validation path, but it still does not expose a fully migrated WASM host layer inside the JS skill flow.

## Current commands

1. `npm run build:wasm`
2. `npm run selftest:wasm`

## What is validated

1. wasm artifacts can be produced
2. generated exports can be loaded
3. dist-level self-test passes

## What is not yet complete

1. JS skill-layer does not yet call a local WASM host as its only runtime path
2. the package is not yet a fully independent Rust distribution
