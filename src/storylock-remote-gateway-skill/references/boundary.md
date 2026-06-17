# Gateway Boundary

## Command Template

Use this reference when deciding whether a task belongs in the remote gateway package.

## Parameters

No runtime parameters.

## Output Parsing

This reference is descriptive. It does not return a runtime payload.

## Error Handling

This reference defines boundary rules rather than executable errors.

## Agent Guidelines

1. This package is remote-facing and packaging-only.
2. Its main white-list capabilities are `requestSignature` and `requestPasswordFill`.
3. Historical story and challenge-named wrappers may exist for compatibility, but they are not the current primary gateway surface.
4. It must not expose internal access-layer operations such as grid verification creation, answer submission, or raw secret-object reads.
5. It must forward replay-protection fields into the local validation chain.
6. It must reject requests without `requestId`, `nonce`, or `expiry`.
