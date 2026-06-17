# StoryLock Platform Key Storage Adaptation Guide

| Item | Content |
| --- | --- |
| Document Version | v1.0 |
| Date | 2026-06-16 |
| Status | Adopted at current stage |
| Applicable Scope | Local Host / Layer 2 / Layer 3 local gateway |

## Purpose

This document clarifies how long-term sensitive material should be saved on different platforms.

## Protection Objects

Objects recommended for platform secure storage at the current stage:

1. `masterSalt`
2. Local signing root key
3. Recovery material packaging key
4. Platform-level credential aliases and handles

## Windows

Recommended priority:

1. System protected storage capabilities
2. Credential management capabilities

## macOS

Current stage recommendations:

1. Prefer using Keychain
2. Save key aliases using service name + account name
3. Only put truly long-term material into Keychain, runtime sessions do not enter Keychain

## Linux

Current stage recommendations:

1. Prefer using Secret Service / libsecret compatible capabilities
2. If the target environment does not have a desktop keyring, an explicit downgrade prompt must be given
3. Silent downgrade to plaintext files is not recommended

## Unified Adaptation Interface Suggestions

It is recommended to abstract a unified interface in the code:

```ts
interface SecretStore {
  getSecret(key: string): Promise<Uint8Array | null>;
  setSecret(key: string, value: Uint8Array): Promise<void>;
  deleteSecret(key: string): Promise<void>;
  listKeys?(prefix: string): Promise<string[]>;
}
```

## Naming Suggestions

Recommended unified key naming:

1. `storylock/masterSalt`
2. `storylock/signingRoot/<identityId>`
3. `storylock/recoveryKey/<identityId>`

## Objects Not to be Put into Platform Key Storage

The following objects are not recommended for platform key storage at the current stage:

1. `requestId` deduplication records
2. `nonce` deduplication records
3. `session` runtime records
4. Large-volume question set ciphertext bodies

## Downgrade Principles

If platform secure storage is unavailable:

1. Return a clear error by default
2. Let the Host decide whether to allow entering "development mode"
3. Development mode must have explicit risk warnings

## Development Mode Security Boundaries

If development mode is enabled at the current stage, it is recommended to simultaneously satisfy:

1. Only for local development environments
2. Explicitly confirmed by users or developers
3. Output clear warnings at startup
4. Record `developmentMode = true` in audit logs

Development mode should still comply with:

1. Not allowing default silent writing of plaintext long-term private keys
2. Not allowing removal of challenge / session / scope verification
3. Not allowing direct exposure of local sensitive results to Layer 3

Recommended risk warnings should at least include:

1. Currently not using formal platform key storage
2. Current environment is only suitable for local development or testing
3. Not to be regarded as production security configuration

## Current Stage Conclusion

At the current stage of implementation, only two things need to be done:

1. Long-term sensitive material goes through unified `SecretStore`
2. Ordinary runtime data goes through SQLite
