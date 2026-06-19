---
name: storylock-local-story-access-skill
description: Local-only StoryLock skill package for object strength policy, grid verification, local authorization, replay protection, and audit state.
---

# StoryLock Local Authorization Skill

This package defines the second layer of StoryLock: object strength policy, grid verification, and local authorization.

## Capability Index

| User Intent | Capability | Reference File | Key Parameters |
| --- | --- | --- | --- |
| "判断对象密码强度" | object strength policy | `references/infrastructure.md` | `identityId`, `objectRef`, `objectType`, `requestedAction` |
| "生成九宫格验证" | grid verification | `references/infrastructure.md` | `identityId`, `objectRef`, `requiredStrength`, `requestId`, `nonce`, `expiry` |
| "提交本地授权答案" | local authorization | `references/infrastructure.md` | `identityId`, `verificationId`, `answers` |
| "导入和治理题集" | question set operations | `references/question-set-operations.md` | `questionSetVersion`, `normalizationVersion`, `status` |
| "查看访问层基础设施" | access infrastructure | `references/infrastructure.md` | `dbPath`, `secretStore` |
| "解释访问层边界" | access boundary | `references/boundary.md` | none |

## Working Rules

1. This package is local-only.
2. It is the only approved object-strength and local-authorization layer.
3. Remote callers should not bypass this package to reach local authorization decisions directly.
4. It must enforce object strength policy, grid verification, session, scope, and replay checks before any protected execution.
5. It must support explicit revocation for pending verifications and active local authorization sessions.
6. It must not return raw answers, long-lived session material, or raw secret-store material.
7. High-sensitivity signing requests should be authorized by this package before any remote-facing wrapper returns a result.
8. Local tests may omit replay fields and let the package generate them, but remote callers must pass explicit `requestId`, `nonce`, and `expiry`.
9. The host uses SQLite plus AES-256-GCM encrypted protected objects and HKDF/HMAC answer digests.
10. Node.js `node:sqlite` is currently experimental; hosts may inject a replacement storage adapter with the same method surface.
11. Persistent `dbPath` usage requires `usePlatformSecretStore: true` or an injected persistent `secretStore`.
12. Production persistent hosts require an active question set with enough cells; legacy answer fallback is only available when `allowLegacyFallback: true` is explicitly enabled for development or demo mode.

## Revocation Capability

`LocalRevocationSkill` revokes either a pending `verificationId` or an active `authorizationId`. Challenge revocation changes `challenge_state.status` to `revoked`; session revocation changes `session_store.status` to `session_revoked`. Both paths write audit events.

## Infrastructure

- `../shared/crypto.js`: AES-256-GCM, HKDF-SHA256, HMAC-SHA256 helpers.
- `../shared/secret-store.js`: `SecretStore` compatible memory adapter, Windows Credential Manager adapter, Linux Secret Service adapter, and `masterSalt` bootstrap.
- `../shared/sqlite-schema.sql`: local vault schema for verification, session, replay, answer digest, protected object, and audit tables.
- `access-host.js`: SQLite-backed access host used by the local authorization skills.

macOS Keychain is intentionally out of scope for this phase.
