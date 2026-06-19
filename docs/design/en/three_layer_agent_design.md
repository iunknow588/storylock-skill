# StoryLock Three-Layer Agent Design Method

## Purpose

This document explains how to present the StoryLock three-layer Skill structure as an Agent-readable, callable, and auditable capability system.

Agent enablement here does not mean letting a remote Agent directly read stories, passwords, private keys, or answers. It means writing each layer's responsibilities, inputs, outputs, forbidden actions, and safety preflight checks as explicit tool contracts.

The Pharos Skill Engine Guide is a useful reference pattern: it organizes underlying tools into Agent-callable capabilities through `SKILL.md`, capability lists, preflight checks, call templates, and safety constraints. StoryLock can use this pattern, especially for the Layer 3 remote gateway, while keeping StoryLock's local authorization boundary intact.

## Three-Layer Agent Principles

1. Layer 1 can help an Agent generate, refine, and evaluate story question sets, but it does not touch secrets.
2. Layer 2 can be called by a local Agent or host for object-strength policy, grid verification, local authorization, and audit.
3. Layer 3 is the only main entry point for remote Agents. It handles request wrapping, delegated execution, and redacted returns.
4. Remote Agents do not directly call Layer 2 internal APIs.
5. No Agent may bypass challenge, session, replay protection, or SecretStore boundaries.

## Layer 1: Story Processing Agent

Layer 1 corresponds to `storylock-local-story-processing-skill`.

The Layer 1 Agent is a story and question-set assistant. It turns user-provided story material into verification question candidates.

Exposable capabilities:

1. `StoryDraftSkill`: generate story drafts.
2. `StoryRefineSkill`: refine and organize story text.
3. `StrengthReviewSkill`: review question or question-set strength.

Forbidden actions:

1. Do not read passwords, private keys, signing keys, or SecretStore values.
2. Do not issue authorization.
3. Do not create challenges.
4. Do not present question review results as successful authorization.

Layer 1 may say "this question is a better high-strength candidate", but it must not say "this object is authorized for access."

## Layer 2: Local Authorization Agent

Layer 2 corresponds to `storylock-local-story-access-skill`.

The Layer 2 Agent is a local authorization control Agent. It may run inside the local host environment, but it should not be the direct entry point for remote Agents.

Exposable capabilities:

1. `ObjectStrengthPolicySkill`: determine strength by object type and action.
2. `GridChallengeSkill`: create grid challenges.
3. `LocalAuthorizationSkill`: verify answers and issue short-lived authorization.
4. `LocalRevocationSkill`: revoke a challenge or authorization.

Local Agent preflight checks:

1. Whether the question set exists and is active.
2. Whether the question set has enough cells for the required strength.
3. Whether `requestId` and `nonce` are replayed.
4. Whether the challenge is expired.
5. Whether the identity is locked by the failure window.
6. Whether SecretStore matches the current runtime mode.

Forbidden actions:

1. Do not return plaintext answers to the remote side.
2. Do not return plaintext secrets from SecretStore to the remote side.
3. Do not describe Layer 2 as a story read/write layer.
4. Do not allow a remote Agent to bypass local user confirmation.

## Layer 3: Remote Gateway Agent

Layer 3 corresponds to `storylock-remote-gateway-skill`.

The Layer 3 Agent is the only main entry point for remote requests entering the local authorization chain. It is the best place to use the Pharos Skill Engine style of organizing capabilities into remote Agent-callable tool contracts.

Exposable capabilities:

1. `requestSignature`
2. `requestPasswordFill`

Current mainline does not expose old interfaces such as `requestStoryRead`, `requestStoryWrite`, `requestChallengeSign`, or `requestCapabilityStatus`.

Layer 3 Agent preflight checks:

1. Whether `capability` is whitelisted.
2. Whether `requestId`, `nonce`, and `expiry` exist.
3. Whether the signing algorithm is allowed.
4. Whether production EIP-712 domains reject placeholders, zero chainId, and zero verifying contracts.
5. Whether `requestedRetention` is allowed.
6. Whether returned results are recursively redacted.

Forbidden actions:

1. Do not hold private keys.
2. Do not hold passwords.
3. Do not hold grid answers.
4. Do not directly read SecretStore.
5. Do not pass sensitive fields from local executors through to the remote Agent.

## Recommended Call Chains

Password fill:

```text
Remote Agent
  -> Layer 3 requestPasswordFill
  -> Layer 2 ObjectStrengthPolicySkill: credential + password_fill => medium
  -> Layer 2 GridChallengeSkill: 6 cells
  -> Local user answers challenge
  -> Layer 2 LocalAuthorizationSkill
  -> Local executor fills password locally
  -> Layer 3 returns audit metadata or redacted result
```

Signature:

```text
Remote Agent
  -> Layer 3 requestSignature
  -> Layer 2 ObjectStrengthPolicySkill: signature_key + signature => high
  -> Layer 2 GridChallengeSkill: 9 cells
  -> Local user answers challenge
  -> Layer 2 LocalAuthorizationSkill
  -> Local executor signs payload locally
  -> Layer 3 returns signature and redacted metadata
```

## Agent Tool Contract Format

Each Agent-facing Skill document should include at least:

1. Capability name.
2. Input fields.
3. Output fields.
4. Preflight checks.
5. Forbidden actions.
6. Error codes.
7. Sensitive-field redaction rules.
8. Example calls.

Example:

```json
{
  "capability": "requestSignature",
  "identityId": "storylock-demo-user",
  "keyId": "wallet/main/private_key",
  "algorithm": "ed25519",
  "payload": "message to sign"
}
```

## Relationship With Pharos Skill Engine

Pharos Skill Engine is valuable as an Agent organization pattern: it turns CLI operations, chain actions, configuration checks, and safety reminders into Agent-callable tools.

StoryLock should reference:

1. `SKILL.md` as an Agent call contract.
2. Capability index.
3. Preflight checks.
4. Operation templates.
5. Sensitive-field non-disclosure rules.

StoryLock should not copy:

1. Letting Agents directly control secrets.
2. Letting Layer 3 directly perform local sensitive reads or writes.
3. Treating a chain executor model as equivalent to a local authorization model.

## Current Implementation Status

Currently implemented: three Skill packages, Layer 3 `requestSignature` and `requestPasswordFill`, Layer 2 object strength/grid verification/local authorization/replay protection/audit, Layer 3 recursive redaction, and the three-layer E2E selftest.

Layer 3 now provides a machine-readable Agent capability manifest at `src/skills/remote-gateway/assets/agent-capabilities.json`. It is checked by `npm run check:agent-capabilities` to keep Agent-facing capabilities aligned with the current `StoryLockRemoteGateway` methods.

Future extensions include an Agent-facing HTTP or MCP host, an Agent workflow for generating 24 questions automatically, and more complete production SecretStore adapters across platforms.
