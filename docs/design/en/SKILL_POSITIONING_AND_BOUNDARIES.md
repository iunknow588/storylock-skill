# StoryLock Skill Positioning and Boundaries

This document explains how Skills are layered in the current StoryLock code, what responsibility each layer bears, and which capabilities should not cross boundaries.

## Current Implementation Packages

| Package | Layer | Currently Implemented Capabilities |
| --- | --- | --- |
| `storylock-local-story-processing-skill` | Layer 1 | `StoryDraftSkill`, `StoryRefineSkill`, `StrengthReviewSkill` |
| `storylock-local-story-access-skill` | Layer 2 | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill` |
| `storylock-remote-gateway-skill` | Layer 3 | `requestSignature`, `requestPasswordFill` |
| `storylock-skill-engine` | Compatibility/Demo Package | `LocalPasswordFillSkill`, `SignatureAuthorizationSkill` |

`storylock-skill-engine` is used for compatibility and demonstration of local execution paths, and is **not** a fourth security layer.

## Three-Layer Boundaries

### Layer 1: Local Story Processing and Strength Analysis

**Responsible for:**

1. Generating story drafts.
2. Polishing and organizing story content.
3. Evaluating the strength of question sets or problem collections.

**Not responsible for:**

1. Reading protected objects.
2. Using signing keys.
3. Filling Web2 passwords.
4. Issuing local authorization.

### Layer 2: Object Strength, Grid Challenge Verification, and Local Authorization

**Responsible for:**

1. Determining required access strength based on the target object.
2. Generating grid challenges based on strength.
3. Verifying answers and generating short-term authorization results.
4. Recording sessions, anti-replay, failure windows, answer digests, and audit logs.

**Not responsible for:**

1. Story object reading or writing back.
2. Remote request orchestration.
3. Directly returning long-term keys or long-term credentials.

### Layer 3: Remote Request Packaging and Proxy Authorization Entry

**Responsible for:**

1. Receiving remote-side signature or password fill requests.
2. Packaging requests into auditable, redactable, and delegable structures for local execution.
3. Calling optional local executors: `signatureExecutor` or `passwordFillExecutor`.
4. Performing recursive redaction on returned results.

**Current external main interfaces:**

1. `requestSignature`
2. `requestPasswordFill`

**Not responsible for:**

1. Reading story content.
2. Generating story content.
3. Directly saving or using long-term private keys.
4. Bypassing Layer 2 authorization.

## Skill Inventory

| Capability | Current Name | Package | Description |
| --- | --- | --- | --- |
| Story Draft | `StoryDraftSkill` | Layer 1 | Generates drafts based on structured input |
| Story Refinement | `StoryRefineSkill` | Layer 1 | Rewrites and organizes existing drafts |
| Strength Assessment | `StrengthReviewSkill` | Layer 1 | Evaluates question set strength and provides suggestions |
| Object Strength Policy | `ObjectStrengthPolicySkill` | Layer 2 | Determines required verification strength for objects |
| Grid Challenge | `GridChallengeSkill` | Layer 2 | Generates, submits, and verifies grid challenge answers |
| Local Authorization | `LocalAuthorizationSkill` | Layer 2 | Returns short-term authorization without returning long-term secrets |
| Remote Signature Request | `requestSignature` | Layer 3 | Packages EIP-712 style signature requests |
| Remote Password Fill Request | `requestPasswordFill` | Layer 3 | Packages Web2 password fill requests |
| Local Password Fill | `LocalPasswordFillSkill` | Compatibility/Demo | Local execution example |
| Local Signature Authorization | `SignatureAuthorizationSkill` | Compatibility/Demo | Local signature execution example |

## Boundary Judgment Rules

1. Capabilities that only process already-redacted text or non-sensitive structured data can be placed in Layer 1 or on the remote side.
2. Capabilities requiring local answer verification, sessions, nonces, short-term authorization, or audit writing belong to Layer 2.
3. Capabilities that need to expose entry points to third parties or remote Agents belong to Layer 3.
4. Capabilities that actually use passwords, private keys, or signers must remain within the local execution boundary.
5. Layer 3 can request signatures or password fills, but cannot itself save private keys, passwords, or grid challenge answers.

## Typical Combinations

### Story Processing

1. `StoryDraftSkill` generates a draft.
2. `StoryRefineSkill` performs polishing.
3. `StrengthReviewSkill` evaluates question set quality.

### Signature Authorization

1. Layer 3 calls `requestSignature` to package the request.
2. Layer 2 determines target object strength and completes grid challenge verification.
3. Layer 2 issues short-term authorization.
4. The local signature executor completes the signature.
5. Layer 3 returns a redacted structured result.

### Web2 Password Fill

1. Layer 3 calls `requestPasswordFill` to package the request.
2. Layer 2 determines credential object strength and completes grid challenge verification.
3. Layer 2 issues short-term authorization.
4. The local password fill executor completes the fill.
5. Layer 3 returns a minimized audit result.

## Currently Deprecated Mainline Interfaces

The following names may still appear in historical documents or migration code, but are no longer current mainline interfaces:

1. `requestStoryRead`
2. `requestStoryWrite`
3. `requestChallengeSign`
4. `requestCapabilityStatus`
5. `StoryReadAccessSkill`
6. `StoryWriteAccessSkill`
7. `ChallengeSigningAuthorizationSkill`
