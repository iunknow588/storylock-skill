# Story Write Access

## Command Template

```js
import { StoryWriteAccessSkill } from "../index.js";

const skill = new StoryWriteAccessSkill({ host });

const result = await skill.run({
  identityId: "identity-001",
  storyObjectId: "story-001",
  content: {
    title: "Updated title",
    content: "Updated private story content"
  },
  answers: [
    { questionId: "q-001", answer: "normalized answer", answerFormat: "text" }
  ],
  requestId: "req-story-write-001",
  nonce: "nonce-002",
  expiry: Date.now() + 60_000,
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `identityId` | string | yes | none | Identity requesting write access. |
| `storyObjectId` | string | yes | none | Protected story object id. |
| `content` | object | yes | none | New object content. |
| `answers` | array | no | `[]` | Challenge answer payloads. |
| `requestId` | string | no | auto | Request id for replay protection. |
| `nonce` | string | no | auto | Request nonce for replay protection. |
| `expiry` | number | no | derived | Expiry timestamp in ms. |
| `redactionLevel` | string | no | `partial` | `none`, `partial`, or `full`. Remote-facing calls should use `partial` or `full`. |
| `policyHints` | object | no | `{}` | May include `redactionLevel`. |

## Output Parsing

```json
{
  "requestId": "req-story-write-001",
  "status": "success",
  "capability": "requestStoryWrite",
  "executionLocation": "local",
  "result": {
    "mode": "story_write_access",
    "storyObjectId": "story-001",
    "writeResult": {}
  },
  "redactionLevel": "partial",
  "retentionGranted": "result_only",
  "auditMeta": {},
  "error": null
}
```

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `SLG-001` | Invalid request shape | Fix the request fields and retry. |
| `SLG-002` | Expired request | Reissue with new expiry. |
| `SLG-003` | Challenge answers did not match | Retry with correct challenge answers. |
| `SLG-004` | Identity is locked after repeated failures | Wait until `retryAfter`. |
| `SLG-008` | Reused requestId or nonce | Generate a fresh nonce. |

## Agent Guidelines

1. Story writes must be challenge-gated and session-gated.
2. Write access must consume authorized write budget.
3. Keep challenge answers local and short-lived.
4. Only return structured write results to remote wrappers.
5. Remote callers should provide explicit replay fields even if local harnesses may auto-fill them.
6. Challenge answers are normalized with NFKC, trimmed, collapsed whitespace, lowercased, and compared by HMAC-SHA256 digest.
7. Failure counting is keyed by `identityId` over a 24-hour window with a 15-minute lock after 3 failures.
