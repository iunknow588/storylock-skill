# StoryLock Three-Package Interface Contract

## Current Baseline

This document defines the minimum interface contracts between the three runtime packages based on the current `skill/src` code:

1. `storylock-local-story-processing-skill`
2. `storylock-local-story-access-skill`
3. `storylock-remote-gateway-skill`

Old review references to story read/write, `requestChallengeSign`, capability status queries, and other interfaces are no longer part of the current design baseline.

## Three-Package Responsibilities

| Package | Layer | Current Responsibilities |
| --- | --- | --- |
| `storylock-local-story-processing-skill` | Layer 1 | Local text processing, story draft/refinement, 24-question set strength assessment |
| `storylock-local-story-access-skill` | Layer 2 | Object strength determination, grid challenge verification, local answer verification, short-term authorization results and auditing |
| `storylock-remote-gateway-skill` | Layer 3 | Remote request packaging, field validation, EIP-712 request expression, optional local executor invocation, result redaction |

## Current Main Capabilities

### Layer 1

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

Layer 1 does not create challenges, issue sessions, read secrets, or directly expose sensitive input to third parties.

### Layer 2

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`

Layer 2 no longer assumes story object reading or writing responsibilities. The current SQLite host only stores challenges, sessions, requests, nonces, failure windows, answer digests, and audit logs.

### Layer 3

1. `requestSignature`
2. `requestPasswordFill`

Layer 3 does not expose Layer 2 internal capabilities, nor does it return grid challenge answers, private keys, passwords, signing key bytes, or long-term session material.

## Call Chain

Recommended chain:

1. Layer 3 receives remote structured requests.
2. Layer 3 validates `requestId`, `nonce`, `expiry`, `capability`, and field shapes.
3. Layer 3 may hand off to the local Host via `transport`, or invoke optional local executors (`signatureExecutor` / `passwordFillExecutor`).
4. Layer 2 determines strength by object and action, generates grid challenge verification, and completes local authorization.
5. The local Host executes signature or password fill after authorization passes.
6. Layer 3 uniformly redacts and returns results.

## Layer 1 Strength Assessment Contract

Request:

```json
{
  "questions": [
    {
      "nodeId": "q-001",
      "validAnswers": ["answer"],
      "distractors": ["d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8"]
    }
  ]
}
```

Constraints:

1. `questions` must contain 24 nodes.
2. Each question must be able to form 9 unique candidate options.
3. Output only contains strength assessment, question count, recommended actions, and boundary markers.

## Layer 2 Authorization Contract

### Object Strength Determination

Request:

```json
{
  "identityId": "identity-001",
  "objectRef": "wallet-key-main",
  "objectType": "signature_key",
  "requestedAction": "signature"
}
```

Response core fields:

```json
{
  "requiredStrength": "high",
  "gridPolicy": {
    "gridSize": 9,
    "requiredCells": 9
  }
}
```

### Grid Challenge Generation

Request must contain:

1. `identityId`
2. `objectRef`
3. `requiredStrength`
4. `requestId`
5. `nonce`
6. `expiry`

Response contains `verificationId`, grid cell descriptions, and expiry time. Cells must not contain answer plaintext.

### Local Authorization

Request:

```json
{
  "identityId": "identity-001",
  "objectRef": "wallet-key-main",
  "verificationId": "chl-001",
  "allowedAction": "signature",
  "answers": [
    {
      "cellId": "cell-1",
      "answer": "normalized answer"
    }
  ]
}
```

Success response contains:

1. `approved`
2. `authorizationId`
3. `identityId`
4. `objectRef`
5. `allowedAction`
6. `expiresAt`

Budget rules:

1. `signature` and `password_fill` default to `readBudget=1, writeBudget=0`.
2. Normal `authorize` defaults to `readBudget=0, writeBudget=0`.
3. Session only represents a short-term authorization result, not a long-term capability.

## Layer 3 Signature Request Contract

`requestSignature` input:

```json
{
  "requestId": "req-sign-001",
  "nonce": "nonce-001",
  "expiry": 1760000300,
  "identityId": "identity-001",
  "keyId": "wallet-key",
  "algorithm": "ed25519",
  "payload": "sign this payload",
  "resourceId": "wallet-main",
  "primaryRole": "private_key",
  "eip712Nonce": "1001"
}
```

Layer 3 generates a `StoryLockSignatureRequest` EIP-712 structure and hands it to the local execution environment via `transport` or `signatureExecutor`.

## Layer 3 Password Fill Contract

`requestPasswordFill` input:

```json
{
  "requestId": "req-fill-001",
  "nonce": "nonce-002",
  "expiry": 1760000300,
  "identityId": "identity-001",
  "credentialRef": "cred-example",
  "targetOrigin": "https://example.com"
}
```

Layer 3 default settings:

1. `capability=requestPasswordFill`
2. `scope=password_fill_basic`
3. `requestedRetention=audit_meta_only`
4. `policyHints.noRemoteSecretReturn=true`

## Unified Error Codes

Current code error codes use the `SLG` prefix:

| Error Code | Type | Meaning |
| --- | --- | --- |
| `SLG-001` | `validation_error` | Request fields are invalid |
| `SLG-002` | `replay_rejected` | Request replay was rejected |
| `SLG-003` | `challenge_failed` | Grid challenge answers did not pass |
| `SLG-004` | `challenge_locked` | Locked after consecutive failures |
| `SLG-005` | `session_invalid` | Session is invalid |
| `SLG-006` | `budget_exhausted` | Budget exhausted |
| `SLG-007` | `object_not_found` | Object does not exist |
| `SLG-008` | `redaction_required` | Result must be redacted |
| `SLG-009` | `scope_insufficient` | Insufficient scope |
| `SLG-010` | `secret_unavailable` | SecretStore is unavailable |
| `SLG-011` | `request_expired` | Request expired |
| `SLG-012` | `internal_error` | Internal error |
| `SLG-013` | `replay_detected` | requestId or nonce conflict |

## Prohibition Rules

1. Layer 3 must not directly expose `createChallenge`, `submitChallengeAnswers`, `issueSession`.
2. Layer 3 must not return private keys, passwords, grid challenge answers, or signing key bytes.
3. Layer 2 must not assume story reading/writing responsibilities.
4. Layer 1 must not bypass Layer 2 to access protected objects.
5. `storylock-skill-engine` is a compatibility and demonstration package, not a new fourth security boundary.