---
name: storylock-local-story-access-skill
description: Local-only StoryLock skill package for protected story object access, including challenge verification, read access, and write access.
---

# StoryLock Local Story Access Skill

This package defines the second layer of StoryLock: protected story object access.

## Capability Index

| User Intent | Capability | Reference File | Key Parameters |
| --- | --- | --- | --- |
| "读取受保护故事对象" | story read access | `references/story-read.md` | `identityId`, `storyObjectId`, `answers`, `requestId`, `nonce`, `expiry` |
| "写回受保护故事对象" | story write access | `references/story-write.md` | `identityId`, `storyObjectId`, `content`, `answers`, `requestId`, `nonce`, `expiry` |
| "查看访问层基础设施" | access infrastructure | `references/infrastructure.md` | `dbPath`, `secretStore` |
| "解释访问层边界" | access boundary | `references/boundary.md` | none |

## Working Rules

1. This package is local-only.
2. It is the only approved story-object access layer.
3. Remote callers should not bypass this package to reach local story processing directly.
4. It must enforce challenge, session, scope, budget, and replay checks before any protected access.
5. It must not return raw challenge answers, long-lived session material, or raw secret-store material.
6. High-sensitivity signing requests should be validated by this package before any remote-facing wrapper returns a result.
7. Local tests may omit replay fields and let the package generate them, but remote callers must pass explicit `requestId`, `nonce`, and `expiry`.
8. The host uses SQLite plus AES-256-GCM encrypted protected objects and HKDF/HMAC answer digests.
9. Node.js `node:sqlite` is currently experimental; hosts may inject a replacement storage adapter with the same method surface.
10. Persistent `dbPath` usage requires `usePlatformSecretStore: true` or an injected persistent `secretStore`.

## Infrastructure

- `../shared/crypto.js`: AES-256-GCM, HKDF-SHA256, HMAC-SHA256 helpers.
- `../shared/secret-store.js`: `SecretStore` compatible memory adapter, Windows Credential Manager adapter, Linux Secret Service adapter, and `masterSalt` bootstrap.
- `../shared/sqlite-schema.sql`: local vault schema for challenge, session, replay, answer digest, protected object, and audit tables.
- `access-host.js`: SQLite-backed access host used by `StoryReadAccessSkill` and `StoryWriteAccessSkill`.

macOS Keychain is intentionally out of scope for this phase.
