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
2. It may expose white-list capabilities such as `requestStoryRead`, `requestStoryWrite`, `requestChallengeSign`, `queryStoryMetadata`, and `requestCapabilityStatus`.
3. It must not expose internal access-layer operations such as challenge creation, answer submission, or raw secret-object reads.
4. It must forward replay-protection fields into the local validation chain.
5. It must reject requests without `requestId`, `nonce`, or `expiry`.
