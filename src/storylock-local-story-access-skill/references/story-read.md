# Story Read Access

## Command Template

```js
import { StoryReadAccessSkill } from "../index.js";

const skill = new StoryReadAccessSkill({ host });

const result = await skill.run({
  identityId: "identity-001",
  storyObjectId: "story-001",
  answers: [
    { questionId: "q-001", answer: "normalized answer", answerFormat: "text" }
  ],
  requestId: "req-story-read-001",
  nonce: "nonce-001",
  expiry: Date.now() + 60_000,
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `identityId` | string | yes | none | Identity requesting access. |
| `storyObjectId` | string | yes | none | Protected story object id. |
| `answers` | array | no | `[]` | Challenge answer payloads. |
| `requestId` | string | no | auto | Request id for replay protection. |
| `nonce` | string | no | auto | Request nonce for replay protection. |
| `expiry` | number | no | derived | Expiry timestamp in ms. |
| `redactionLevel` | string | no | `partial` | `none`, `partial`, or `full`. Remote-facing calls should use `partial` or `full`. |
| `policyHints` | object | no | `{}` | May include `redactionLevel`. |

## Output Parsing

```json
{
  "requestId": "req-story-read-001",
  "status": "success",
  "capability": "requestStoryRead",
  "executionLocation": "local",
  "result": {
    "mode": "story_read_access",
    "storyObjectId": "story-001",
    "storyObject": {}
  },
  "redactionLevel": "partial",
  "retentionGranted": "result_only",
  "auditMeta": {},
  "error": null
}
```

Fields:

1. `result.mode`: fixed local access path.
2. `result.storyObject`: authorized story object payload.
3. `auditMeta`: challenge/session identifiers only, not session material.

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `SLG-001` | Invalid request shape | Fix the request fields and retry. |
| `SLG-002` | Request exceeded expiry | Reissue the request with a fresh expiry. |
| `SLG-003` | Challenge answers did not match | Retry with correct challenge answers. |
| `SLG-004` | Identity is locked after repeated failures | Wait until `retryAfter`. |
| `SLG-008` | Reused requestId or nonce | Generate a new nonce and requestId. |

## Agent Guidelines

1. Read access must go through the local access package.
2. Treat `answers` as local-only material.
3. Do not return raw challenge answers or local session internals to remote callers.
4. Apply replay checks before reading the protected object.
5. Remote callers should provide explicit replay fields even if local harnesses may auto-fill them.
6. Challenge answers are normalized with NFKC, trimmed, collapsed whitespace, lowercased, and compared by HMAC-SHA256 digest.
7. Failure counting is keyed by `identityId` over a 24-hour window with a 15-minute lock after 3 failures.
