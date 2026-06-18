# StoryLock Three Skill Package Split Strategy

This document explains why StoryLock is split into three main packages, and what each package actually does in the current code.

## Conclusion

The current mainline uses three Skill packages:

1. `storylock-local-story-processing-skill`
2. `storylock-local-story-access-skill`
3. `storylock-remote-gateway-skill`

Additionally, `storylock-skill-engine` is a compatibility and demonstration package, used to retain local password fill and signature authorization examples, and is not a new security layer.

## Package 1: Local Story Processing Skill Package

Path:

`src/storylock-local-story-processing-skill`

Current capabilities:

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

Responsibilities:

1. Processing story draft generation.
2. Processing story refinement.
3. Evaluating the strength of question sets or problem collections.

Boundaries:

1. Does not read protected objects.
2. Does not issue authorization.
3. Does not use signing keys or password credentials.

## Package 2: Local Access Authorization Skill Package

Path:

`src/storylock-local-story-access-skill`

Current capabilities:

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`

Responsibilities:

1. Determining required strength based on the object to be accessed or used.
2. Generating corresponding grid challenge verification.
3. Verifying answers and issuing short-term local authorization.
4. Writing sessions, anti-replay, failure windows, answer digests, and audit logs into SQLite.

Boundaries:

1. No longer provides story object reading interfaces.
2. No longer provides story object writing interfaces.
3. Does not directly execute remote requests.
4. Does not return long-term secret material.

## Package 3: Remote Gateway Skill Package

Path:

`src/storylock-remote-gateway-skill`

Current interfaces:

1. `requestSignature`
2. `requestPasswordFill`

Responsibilities:

1. Packaging third-party or remote Agent requests.
2. Using `StoryLockSignatureRequest` structures for signature requests.
3. Calling optional local executors.
4. Performing recursive redaction on results.
5. Returning structured requests, responses, and audit metadata.

Boundaries:

1. Does not read story content.
2. Does not directly hold private keys or passwords.
3. Does not expose `requestChallengeSign` and other old interfaces.
4. Does not write the remote gateway as a story reading/writing entry point.

## Calling Relationship

The recommended mainline calling relationship is:

1. Layer 3 receives remote signature or password fill requests.
2. Layer 2 completes object strength determination, grid challenge verification, and short-term authorization.
3. Local executors complete signature or password fill after authorization.
4. Layer 3 returns redacted structured results.

There is no mandatory direct calling relationship between Layer 1 and Layer 2. Layer 1 is responsible for text and question set quality, while Layer 2 is responsible for access authorization.

## About Pharos Skill Engine

The current project can reference the Pharos Skill Engine's Skill package organization method, but local security access capabilities should be implemented by StoryLock itself.

More accurate division is:

1. Layer 1 and Layer 2: Implemented by StoryLock itself, focusing on protecting local data and authorization boundaries.
2. Layer 3: Can interface with Pharos or other Agent ecosystems for adaptation.
3. Compatibility/Demo Package: Retains runnable examples to help verify local password fill and signature authorization chains.

## Currently Deprecated Old Statements

The following old statements are no longer suitable as current documentation standards:

1. "Package 2 is responsible for protected story object reading/writing."
2. "Package 3 calls Package 2 which then calls Package 1 to read stories."
3. "The signature interface is called deprecated old interface `requestChallengeSign`."
4. "Currently, `SigningAuthorizationSkill` can be referenced as the mainline signature interface."

The current mainline signature standard is unified as `requestSignature` and `SignatureAuthorizationSkill`.
