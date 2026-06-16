---
name: storylock-remote-gateway-skill
description: Remote StoryLock skill package for third-party access, delegated requests, and local capability routing.
---

# StoryLock Remote Gateway Skill

This package defines the third layer of StoryLock: remote request packaging, delegation, and gateway routing.

## Capability Index

| User Intent | Capability | Reference File | Key Parameters |
| --- | --- | --- | --- |
| "远程请求读取故事对象" | delegated story read | `references/delegated-story-read.md` | `identityId`, `storyObjectId`, `requestId`, `nonce`, `expiry` |
| "远程请求写回故事对象" | delegated story write | `references/delegated-story-write.md` | `identityId`, `storyObjectId`, `content`, `requestId`, `nonce`, `expiry` |
| "远程请求本地签名" | delegated challenge sign | `references/delegated-sign.md` | `identityId`, `keyId`, `algorithm`, `payload`, `requestId`, `nonce`, `expiry` |
| "查询对象元信息" | delegated metadata query | `references/boundary.md` | `identityId`, `storyObjectId`, `requestId`, `nonce`, `expiry` |
| "查询能力状态" | delegated capability status | `references/boundary.md` | `identityId`, `capability`, `requestId`, `nonce`, `expiry` |
| "解释远程网关边界" | gateway boundary | `references/boundary.md` | none |

| "Remote password fill request" | delegated password fill | `references/boundary.md` | `identityId`, `credentialRef`, `targetOrigin`, `requestId`, `nonce`, `expiry` |
| "Remote local story assist request" | delegated local story assist | `references/boundary.md` | `identityId`, `assistType`, `prompt`, `requestId`, `nonce`, `expiry` |

## Working Rules

1. This package is remote-facing.
2. It should not hold raw secrets or direct local secret-store access.
3. It must delegate local-sensitive work to the local story access package.
4. It must only expose approved white-list capabilities to third parties.
5. It must not expose createChallenge, submitChallengeAnswers, readSecretObject, deriveKey, or raw session internals.
6. It must pass replay-protection fields such as requestId, nonce, and expiry into the local validation chain.
7. It must reject missing or expired replay-protection fields before invoking transport.
