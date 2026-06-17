# StoryLock System Skill Table and Capability Boundaries

## Current Code Baseline

This document is organized according to the current `skill/src` code and no longer follows historical interface standards from old reviews.

## Layer 1: Local Story Processing

Package: `storylock-local-story-processing-skill`

| Capability | Class | Current Status | Boundary |
| --- | --- | --- | --- |
| Story Draft | `StoryDraftSkill` | Implemented and self-tested | Only processes local input, does not create authorization |
| Story Refinement | `StoryRefineSkill` | Implemented and self-tested | Only processes local input, does not read secrets |
| Question Set Strength Assessment | `StrengthReviewSkill` | Implemented and self-tested | Evaluates 24-question sets, does not issue sessions |

## Layer 2: Local Authorization

Package: `storylock-local-story-access-skill`

| Capability | Class | Current Status | Boundary |
| --- | --- | --- | --- |
| Object Strength Policy | `ObjectStrengthPolicySkill` | Implemented and self-tested | Determines low/medium/high and grid strategy |
| Grid Challenge | `GridChallengeSkill` | Implemented and self-tested | Generates verification, records requestId/nonce, anti-replay |
| Local Authorization | `LocalAuthorizationSkill` | Implemented and self-tested | Verifies answers and returns short-term authorization results |

Layer 2 no longer provides story reading, story writing, or protected content persistence interfaces.

## Layer 3: Remote Gateway

Package: `storylock-remote-gateway-skill`

| Capability | Interface/Class | Current Status | Boundary |
| --- | --- | --- | --- |
| Remote Signature Request | `requestSignature` | Implemented and self-tested | Packages EIP-712 requests, optionally calls `signatureExecutor` |
| Web2 Password Fill Request | `requestPasswordFill` | Implemented and self-tested | Packages local password fill requests, does not return plaintext passwords by default |
| Delegated Signature Skill | `DelegatedSignatureSkill` | Implemented | Requests signatures through the gateway |

Layer 3 uniformly redacts return values, not exposing private keys, passwords, answers, or signing key bytes.

## Compatibility Package

Package: `storylock-skill-engine`

Currently used as a compatibility and demonstration package, not a new security layer. Top-level exports only retain:

1. `LocalPasswordFillSkill`
2. `LOGIN_BINDING_MODE`
3. `SignatureAuthorizationSkill`

Files under `assets/migrated/` may still retain old implementation files or old class names, but they are not current main entry points.

## Current External Main Interfaces

1. `requestSignature`
2. `requestPasswordFill`

## Current Internal Main Capabilities

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`
4. `ObjectStrengthPolicySkill`
5. `GridChallengeSkill`
6. `LocalAuthorizationSkill`

## Capabilities No Longer in Current Mainline

1. `requestStoryRead`
2. `requestStoryWrite`
3. `requestChallengeSign`
4. `requestCapabilityStatus`
5. `requestLocalStoryAssist`
6. `queryStoryMetadata`
7. Layer 2 story object reading/writing

## Subsequent Design Principles

1. New documents and code must no longer treat story reading/writing as Layer 2 responsibilities.
2. New gateway capabilities must enter the whitelist before being exposed externally.
3. Signature capabilities uniformly use the `requestSignature` and `SignatureAuthorizationSkill` standards.
4. Question set strength assessment belongs to Layer 1.
5. Single-story lifetime, question set version management, and real grid question selection are subsequent enhancements and should not be disguised as completed capabilities.