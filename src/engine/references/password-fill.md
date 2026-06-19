# Password Fill

## Overview

Use this capability to create an authorization challenge, submit answers, open a temporary authorization session, and package login fields for a target site.

Implemented skill classes:

1. `LoginAuthorizationSkill`
2. `LocalPasswordFillSkill`

Primary implementation source:

1. `assets/migrated/skills/authorization-skills.js`

## Command Template

```js
import { LocalPasswordFillSkill } from "../assets/migrated/skills/authorization-skills.js";

const skill = new LocalPasswordFillSkill({ host });

const result = await skill.run({
  identityId: "identity-001",
  siteId: "generic_username_password",
  bindings: [
    { fieldName: "username", secretObjectId: "credential/generic/main/username" },
    { fieldName: "password", secretObjectId: "credential/generic/main/password" },
  ],
  answers: ["answer-1", "answer-2"],
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `identityId` | `string` | yes | none | Must be a non-empty string. |
| `siteId` | `string` | yes | none | Target login site or template id. |
| `resourceId` | `string \| null` | no | `null` | Used with `resourceCatalog` or template resolution. |
| `resourceCatalog` | `object \| null` | no | `null` | Resource map for role-to-object resolution. |
| `bindings` | `object[]` | no | `[]` | Manual or override field bindings. |
| `bindingMode` | `string` | no | `template_with_overrides` | One of the exported `LOGIN_BINDING_MODE` values. |
| `answers` | `array` | no | `[]` | Challenge answers submitted to the host. |

Host requirements:

1. `host.createChallenge(identityId, scope)`
2. `host.submitChallengeAnswers(identityId, challengeId, answers)`
3. `host.readSecretObject(identityId, sessionId, secretObjectId)`

Schemas:

1. `assets/schemas/password-fill-input.schema.json`
2. `assets/schemas/password-fill-output.schema.json`

## Output Parsing

The return object includes:

1. `mode`: always `local_password_fill`
2. `siteId`
3. `scope`
4. `challenge`
5. `authorization`
6. `fields`

Each `fields[]` entry includes:

1. `fieldName`
2. `value`
3. `secretObjectId`

## Error Handling

| Error Code | Trigger | Fix |
| --- | --- | --- |
| `VALIDATION_ERROR` | `identityId` or `siteId` is empty | Provide valid non-empty identifiers. |
| `VALIDATION_ERROR` | required host methods are missing | Inject a host with all three required methods. |
| `VALIDATION_ERROR` | binding resolution fails | Supply `secretObjectId` directly or provide a valid `resourceCatalog` and `resourceId + role`. |
| `VALIDATION_ERROR` | authorization result does not include `sessionId` | Fix the host authorization response contract. |
| host-raised error | challenge creation, answer submission, or secret read fails | Inspect host state, challenge freshness, answers, and scope. |

## Agent Guidelines

1. Confirm the user is asking for login-field packaging, not generic signing.
2. Run all write-operation pre-checks from `SKILL.md` before invoking.
3. Resolve whether bindings come from a template, manual bindings, or template overrides.
4. Never assume secrets can be read until `authorization.sessionId` is present.
5. Return `fields` as structured data and avoid logging sensitive values in commentary.
