# Delegated Story Write

## Command Template

```js
import { StoryLockRemoteGateway } from "../index.js";

const gateway = new StoryLockRemoteGateway({ transport });

const response = await gateway.requestStoryWrite({
  requestId: "req-story-write-001",
  identityId: "identity-001",
  storyObjectId: "story-001",
  content: {
    title: "Updated title",
    content: "Updated content"
  },
  answers: [],
  nonce: "nonce-002",
  expiry: Date.now() + 60_000,
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `requestId` | string | yes | none | Remote request id. |
| `identityId` | string | yes | none | Requesting identity. |
| `storyObjectId` | string | yes | none | Protected object id. |
| `content` | object | yes | none | Proposed write payload. |
| `answers` | array | no | `[]` | Challenge answers for local validation only. |
| `nonce` | string | yes | none | Replay-protection nonce. |
| `expiry` | number | yes | none | Request expiry timestamp. |
| `requestedRetention` | string | no | `result_only` | Retention requested from the gateway. |
| `policyHints` | object | no | `{}` | Route and redaction hints. |

## Output Parsing

The remote gateway returns the local access wrapper for write authorization and write result.

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `SCOPE_INSUFFICIENT` | Access level too low | Retry through the correct challenge path. |
| `NONCE_REPLAY_DETECTED` | Nonce replayed | Generate a new nonce. |
| `REQUEST_EXPIRED` | Expired request | Reissue the request. |

## Agent Guidelines

1. The remote gateway packages the request but does not approve the write itself.
2. Protected writes must remain challenge-gated and budget-gated locally.
3. Do not retain reusable local write authority remotely.
4. Do not omit replay fields when calling the remote gateway.
