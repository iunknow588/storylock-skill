# Challenge Sign

## Overview

Use this capability to authorize access to signing material and package a challenge-signing result for a target key and payload.

Implemented skill classes:

1. `SigningAuthorizationSkill`
2. `ChallengeSigningAuthorizationSkill`

Primary implementation source:

1. `assets/migrated/skills/authorization-skills.js`

## Command Template

```js
import { ChallengeSigningAuthorizationSkill } from "../assets/migrated/skills/authorization-skills.js";

const skill = new ChallengeSigningAuthorizationSkill({
  host,
  signer,
});

const result = await skill.run({
  identityId: "identity-001",
  keyId: "demo-key",
  algorithm: "ed25519",
  payload: "sign this payload",
  secretObjectId: "keys/demo/main/private",
  answers: ["answer-1", "answer-2"],
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `identityId` | `string` | yes | none | Must be a non-empty string. |
| `keyId` | `string` | yes | none | Logical key identifier. |
| `algorithm` | `string` | yes | none | Signing algorithm label. |
| `payload` | `Uint8Array \| string \| number[]` | yes | none | Converted into binary payload before signing. |
| `secretObjectId` | `string` | conditional | none | Direct signing secret reference. |
| `resourceId` | `string \| null` | no | `null` | Used with `primaryRole` or attachment role lookup. |
| `primaryRole` | `string \| null` | no | `null` | Role used when `secretObjectId` is omitted. |
| `resourceCatalog` | `object \| null` | no | `null` | Resource map for role-based resolution. |
| `includeKeyMaterial` | `boolean` | no | `false` | Expands requested scope when true. |
| `attachments` | `object[]` | no | `[]` | Optional extra secret references. |
| `answers` | `array` | no | `[]` | Challenge answers submitted to the host. |

Host requirements:

1. `host.createChallenge(identityId, scope)`
2. `host.submitChallengeAnswers(identityId, challengeId, answers)`
3. `host.readSecretObject(identityId, sessionId, secretObjectId)`

Signer requirements:

1. a signing function
2. or an object exposing `sign()`

Input schema:

1. `assets/schemas/challenge-sign-input.schema.json`

## Output Parsing

The signing result is assembled by the skill implementation and includes authorization state plus signing output. Agents should preserve at least:

1. `mode` or capability label when present
2. `keyId`
3. `algorithm`
4. `scope`
5. `challenge`
6. `authorization`
7. signature-related fields returned by the signer

## Error Handling

| Error Code | Trigger | Fix |
| --- | --- | --- |
| `VALIDATION_ERROR` | `identityId`, `keyId`, or `algorithm` is empty | Provide valid non-empty identifiers. |
| `VALIDATION_ERROR` | `payload` cannot be normalized | Supply a `Uint8Array`, string, or byte array. |
| `VALIDATION_ERROR` | `signer` is missing or invalid | Inject a function or object with `sign()`. |
| `VALIDATION_ERROR` | secret reference cannot be resolved | Provide `secretObjectId` directly or a valid `resourceCatalog` path. |
| host-raised error | challenge, authorization, or secret reads fail | Verify challenge freshness, answers, scope, and object references. |

## Agent Guidelines

1. Confirm the request is for signing rather than login-field retrieval.
2. Run all write-operation pre-checks from `SKILL.md` before invoking.
3. Resolve the signing secret source before starting the challenge flow.
4. Normalize `payload` deliberately and keep binary handling explicit.
5. Preserve structured signing output and avoid collapsing it into prose.
