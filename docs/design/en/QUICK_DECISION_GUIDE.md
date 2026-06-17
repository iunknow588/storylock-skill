# StoryLock Quick Decision Guide

This document is used for quickly determining which layer a capability should be placed in, and whether it must be executed locally.

## One-Page Judgment

### Step 1: Does it touch sensitive material?

If a capability will touch any of the following, it cannot be directly placed in the remote layer:

1. Grid challenge answers.
2. Session or short-term authorization material.
3. Passwords, private keys, signers, or equivalent credentials.
4. Raw private story content.
5. Unredacted local audit context.

### Step 2: Is it processing content, controlling authorization, or packaging requests?

If it is responsible for drafts, refinement, or question set strength assessment, it belongs to Layer 1.

If it is responsible for object strength determination, grid challenge verification, sessions, anti-replay, and short-term authorization, it belongs to Layer 2.

If it is responsible for receiving remote requests, packaging requests, calling local executors, and returning redacted results, it belongs to Layer 3.

## Current Three Layers

| Layer | Current Positioning | Representative Capabilities |
| --- | --- | --- |
| Layer 1 | Local story processing and strength analysis | `StoryDraftSkill`, `StoryRefineSkill`, `StrengthReviewSkill` |
| Layer 2 | Object strength, grid challenge verification, and local authorization | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill` |
| Layer 3 | Remote request packaging and proxy authorization entry | `requestSignature`, `requestPasswordFill` |

## Common Scenario Reference

| Scenario | Suggested Ownership | Description |
| --- | --- | --- |
| Generate story draft based on private memories | Layer 1 | Raw private input should not be handed to the remote side |
| Refine already-redacted drafts | Layer 1 or remote text capability | Depends on whether input is already redacted |
| Determine what verification strength a signature object needs | Layer 2 | Belongs to object access policy |
| Generate grid challenge and verify answers | Layer 2 | Answers are only processed locally |
| Third-party requests signature | Layer 3 entry, local execution | Use `requestSignature` to package requests |
| Third-party requests login form password fill | Layer 3 entry, local execution | Use `requestPasswordFill` to package requests |

## Anti-Patterns

Do not do this:

1. Let the remote gateway read story content.
2. Let the remote gateway hold private keys or passwords.
3. Let the story processing layer bypass Layer 2 authorization.
4. Write Layer 2 as a business content reading/writing layer.
5. Name the signature interface as `requestChallengeSign` which only fits challenge scenarios.

## Current Stage Recommendations

Current stage priority guarantees:

1. Layer 1 text processing capability is runnable.
2. Layer 2 object strength, grid challenge verification, and authorization chain are clear.
3. Layer 3 only exposes `requestSignature` and `requestPasswordFill`.
4. All remote returns comply with redaction and minimization principles.
