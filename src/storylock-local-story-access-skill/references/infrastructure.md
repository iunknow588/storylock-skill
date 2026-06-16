# Access Infrastructure

## Command Template

```js
import { StoryReadAccessSkill } from "../index.js";

const skill = new StoryReadAccessSkill({
  dbPath: "storylock_vault.db",
  usePlatformSecretStore: true,
});
```

For development-only in-memory runs:

```js
const skill = new StoryReadAccessSkill();
```

## Storage Contract

The host uses SQLite through Node.js `node:sqlite`.

Default construction uses an in-memory SQLite database and in-memory `SecretStore`. A persistent `dbPath` requires either `usePlatformSecretStore: true` or an injected `secretStore`. This prevents a persistent encrypted vault from being paired with an ephemeral `masterSalt`.

Tables:

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `protected_story_objects`
8. `audit_log`

Protected story objects are stored as AES-256-GCM envelopes. The object key is derived with HKDF-SHA256 from `masterSalt` and the object id.

## Secret Contract

`../shared/secret-store.js` defines the current `SecretStore` compatible surface:

```ts
interface SecretStore {
  getSecret(key: string): Buffer | null;
  setSecret(key: string, value: Buffer | Uint8Array | string): void;
  deleteSecret(key: string): void;
  listKeys(prefix?: string): string[];
}
```

The bundled `MemorySecretStore` is a development adapter. Production hosts should replace it with Windows Credential Manager or Linux Secret Service.

Current platform adapters:

1. `WindowsCredentialSecretStore`
   - Uses PowerShell CredentialManager cmdlets.
   - Requires the `CredentialManager` PowerShell module.
   - Stores base64-encoded secret bytes under `storylock/<key>`.

2. `LinuxSecretServiceStore`
   - Uses `secret-tool`.
   - Requires Secret Service / libsecret to be available in the user session.
   - Stores base64-encoded secret bytes with attributes `service=storylock` and `key=<key>`.

macOS Keychain is intentionally not implemented in the current phase.

## Security Notes

1. Challenge answers are normalized and stored only as HMAC-SHA256 digests.
2. Replay checks persist `requestId` and `nonce` in SQLite.
3. Session budget consumption and object read happen in a `BEGIN IMMEDIATE` SQLite transaction.
4. Default read/write output uses `partial` redaction.
5. `node:sqlite` is experimental in the current Node runtime; replace the adapter if a stable SQLite binding is required.
6. Platform SecretStore adapters fail closed when their OS integration is unavailable; they do not silently downgrade to plaintext files.
7. Persistent SQLite databases fail closed unless backed by a persistent SecretStore.

## Verification

```bash
npm run selftest
npm run check:secret-store
```

`selftest` is non-destructive and uses temporary SQLite files with `MemorySecretStore`.
It prints a final JSON report matching `assets/schemas/selftest-report.schema.json`.

`check:secret-store` is non-destructive and only verifies whether the current platform adapter is available:

1. Windows: checks for PowerShell CredentialManager cmdlets.
2. Linux: checks for `secret-tool`.
