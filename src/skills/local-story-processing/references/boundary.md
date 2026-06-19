# Processing Boundary

## Command Template

Use this reference when deciding whether a task belongs in the local story processing package.

## Parameters

No runtime parameters.

## Output Parsing

This reference is descriptive. It does not return a runtime payload.

## Error Handling

This reference defines boundary rules rather than executable errors.

## Agent Guidelines

1. This package handles local story drafting and refinement only.
2. It does not create challenges, issue sessions, or read protected objects by itself.
3. Any protected-object input must already be approved and supplied by the local access package.
4. Remote-facing packages must not receive raw private input directly from this layer.
