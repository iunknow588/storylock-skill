# Access Boundary

## Command Template

Use this reference when deciding whether a task belongs in the local story access package.

## Parameters

No runtime parameters.

## Output Parsing

This reference is descriptive. It does not return a runtime payload.

## Error Handling

This reference defines boundary rules rather than executable errors.

## Agent Guidelines

1. This package is the only approved protected-object access layer.
2. It must enforce challenge, session, scope, replay, and budget checks.
3. It must not expose raw challenge answers, raw secret-store material, or long-lived session internals.
4. Third-party or remote-facing packages must call this package rather than bypass it.
5. Production persistent hosts must use active question-set cells for grid verification; legacy answer fallback is allowed only through an explicit development/demo `allowLegacyFallback` switch.
