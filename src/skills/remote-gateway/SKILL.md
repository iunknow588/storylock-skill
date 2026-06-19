---
name: storylock-remote-gateway-skill
description: Remote StoryLock skill package for third-party access, delegated requests, and local capability routing.
---

# StoryLock Remote Gateway Skill

This package defines the third layer of StoryLock: remote request packaging, delegation, and gateway routing.

## Capability Index

Machine-readable Agent capability manifest: `assets/agent-capabilities.json`.

| User Intent | Capability | Reference File | Key Parameters |
| --- | --- | --- | --- |
| "远程请求本地签名" | delegated signature | `references/delegated-sign.md` | `identityId`, `keyId`, `algorithm`, `payload`, `requestId`, `nonce`, `expiry` |
| "Remote password fill request" | delegated password fill | `references/boundary.md` | `identityId`, `credentialRef`, `targetOrigin`, `requestId`, `nonce`, `expiry` |
| "检查 EIP-712 生产配置" | EIP-712 config check | `references/delegated-sign.md` | `STORYLOCK_EIP712_*` |
| "解释远程网关边界" | gateway boundary | `references/boundary.md` | none |

## Working Rules

1. This package is remote-facing.
2. It should not hold raw secrets or direct local secret-store access.
3. It must delegate local-sensitive work to the local authorization package.
4. Its primary third-party surface is `requestSignature` and `requestPasswordFill`.
5. It must not expose grid verification creation, answer submission, raw credential reads, raw private key reads, deriveKey, or raw session internals.
6. It must pass replay-protection fields such as requestId, nonce, and expiry into the local validation chain.
7. It must reject missing or expired replay-protection fields before invoking transport.
8. Use `createProductionEip712Domain()` or `createEip712DomainFromEnv()` for explicit production EIP-712 presets and `createDemoEip712Domain()` for demo placeholders.
9. Keep `assets/agent-capabilities.json` synchronized with the actual `StoryLockRemoteGateway` request methods.
