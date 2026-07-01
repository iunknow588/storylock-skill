# UI Doc Screenshots

This directory groups the internal tooling context for Windows Host / StoryLock UI documentation screenshots.

## Purpose

These files support screenshot generation for:

1. design and deployment feasibility docs
2. flowchart and storyboard materials
3. UI regression spot checks during documentation updates

This is an internal documentation tool area, not part of the Windows Host runtime path.

## Current layout

The executable entry still lives in the Cargo bin target:

- `src/bin/render_storylock_ui_docs.rs`

That entry remains in `src/bin` because it directly depends on the local Slint UI components and currently fits the Cargo multi-bin workflow well.

This `tools/ui-doc-screenshots/` directory is the landing zone for:

1. usage notes
2. fixture notes
3. future helper scripts
4. output conventions

## Related files

1. Rust screenshot renderer:
   - `src/bin/render_storylock_ui_docs.rs`
2. existing capture helper:
   - `docs/management/流程图/tools/capture_storylock_ui.ps1`
3. screenshot output target used by docs:
   - `docs/management/流程图/ui-screenshots`

## Recommended usage

Generate the Slint-based static UI screenshots with:

```powershell
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

If interactive window capture is needed for a live executable flow, use the PowerShell helper in the docs flowchart tools directory.

## Why this is not fully moved yet

The current renderer:

1. imports local Slint components directly
2. is compiled as part of the same crate
3. is referenced in existing documentation commands as a Cargo bin

So the current recommended cleanup is a half-split:

1. keep the executable entry in `src/bin`
2. move the screenshot-tool ownership and documentation into `tools/ui-doc-screenshots/`
3. move any future fixtures and helper scripts here first

If more internal tools accumulate, this directory can become the stable home for all screenshot-related support files before considering a larger Cargo/workspace refactor.
