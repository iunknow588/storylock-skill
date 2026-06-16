# Delegated Story Read

## Command Template

```js
import { StoryLockRemoteGateway } from "../index.js";

const gateway = new StoryLockRemoteGateway({ transport });

const response = await gateway.requestStoryRead({
  requestId: "req-story-read-001",
  identityId: "identity-001",
  storyObjectId: "story-001",
  answers: [],
  nonce: "nonce-001",
  expiry: Date.now() + 60_000,
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `requestId` | string | yes | none | Remote request id. |
| `identityId` | string | yes | none | Requesting identity. |
| `storyObjectId` | string | yes | none | Protected object id. |
| `answers` | array | no | `[]` | Passed through to local validation only. |
| `nonce` | string | yes | none | Replay-protection nonce. |
| `expiry` | number | yes | none | Request expiry timestamp. |
| `requestedRetention` | string | no | `result_only` | Retention requested from the gateway. |
| `policyHints` | object | no | `{}` | Route and redaction hints. |

## Output Parsing

The remote gateway returns the structured outer wrapper from the local access package.

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `CAPABILITY_NOT_AVAILABLE` | Gateway capability missing | Check gateway configuration. |
| `NONCE_REPLAY_DETECTED` | Nonce replayed | Generate a new nonce. |
| `REQUEST_EXPIRED` | Request too old | Recreate request with fresh expiry. |

## Agent Guidelines

1. The remote gateway may package the request, but local validation still happens in the second package.
2. Do not persist raw challenge answers remotely.
3. Do not treat a successful read as reusable long-lived access.
4. Do not omit replay fields when calling the remote gateway.
