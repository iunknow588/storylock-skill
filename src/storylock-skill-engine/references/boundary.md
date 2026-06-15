# StoryLock Skill Boundary

This migrated bundle preserves the same boundary as the source `story-lock` project.

## What this layer is

The skill layer is the high-level interface layer above StoryLock core.

It is responsible for:

1. drafting and refining story material
2. reviewing question-set readiness
3. packaging authorized login-fill output
4. packaging authorized signing output
5. providing stable demo and integration-facing invocation surfaces

## What this layer is not

It does not:

1. redefine `6-of-24`, `12-of-24`, or `22-of-24`
2. change root-key derivation or recovery semantics
3. replace host UI, deployment policy, or production credential handling
4. bypass challenge verification or scope enforcement

## Required host surface

Authorization-oriented skills still depend on the host:

1. `createChallenge(identityId, scope)`
2. `submitChallengeAnswers(identityId, challengeId, answers)`
3. `readSecretObject(identityId, sessionId, secretObjectId)`

So this layer is orchestration and packaging, not the core authorization engine.
