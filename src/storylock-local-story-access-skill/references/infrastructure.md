# Access Infrastructure

## Command Template

```js
import { GridChallengeSkill } from "../index.js";

const skill = new GridChallengeSkill({
  dbPath: "storylock_vault.db",
  usePlatformSecretStore: true,
});
```

For development-only in-memory runs:

```js
const skill = new GridChallengeSkill();
```

For development or demo compatibility with old answer digests:

```js
const skill = new GridChallengeSkill({
  dbPath: "storylock_demo.db",
  secretStore: developmentSecretStore,
  allowLegacyFallback: true,
});
```

Production persistent hosts should leave `allowLegacyFallback` disabled. If the active question set does not contain enough cells for the requested strength, grid creation fails with `SLG-010` and `question_set_unavailable` instead of silently using legacy answer digests.

Revoking a pending verification or active authorization:

```js
import { LocalRevocationSkill } from "../index.js";

const revoke = new LocalRevocationSkill({ host });

await revoke.run({
  identityId: "identity-001",
  authorizationId: "ses-example",
  reason: "user_requested",
  requestId: "req-revoke-001",
});
```

## Storage Contract

The default host uses SQLite through Node.js `node:sqlite`.

Default construction uses an in-memory SQLite database and in-memory `SecretStore`. A persistent `dbPath` requires either `usePlatformSecretStore: true` or an injected `secretStore`. This prevents a persistent encrypted vault from being paired with an ephemeral `masterSalt`.

Hosts that need a different SQLite binding can inject `databaseFactory`:

```js
const skill = new GridChallengeSkill({
  dbPath: "storylock_vault.db",
  secretStore,
  databaseFactory(path) {
    return new CompatibleDatabase(path);
  },
});
```

The returned object must implement the small `DatabaseSync`-compatible surface used by this package:

```ts
interface StoryLockDatabase {
  exec(sql: string): void;
  prepare(sql: string): {
    run(...params: unknown[]): { changes?: number };
    get(...params: unknown[]): unknown;
    all(...params: unknown[]): unknown[];
  };
  close(): void;
}
```

The package still runs its schema bootstrap and migration against the injected database.

Tables:

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `audit_log`

Revocation state:

1. A pending grid verification can be revoked by `LocalRevocationSkill` with `verificationId`; the host sets `challenge_state.status` to `revoked`.
2. An active local authorization can be revoked by `LocalRevocationSkill` with `authorizationId`; the host sets `session_store.status` to `session_revoked`.
3. Revocation writes `challenge_revoked` or `session_revoked` audit events with the local `identityId` and reason metadata.

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
3. Local authorization sessions are issued only after challenge verification.
4. `node:sqlite` is experimental in the current Node runtime; inject `databaseFactory` if a stable SQLite binding is required.
5. Platform SecretStore adapters fail closed when their OS integration is unavailable; they do not silently downgrade to plaintext files.
6. Persistent SQLite databases fail closed unless backed by a persistent SecretStore.
7. Revoked challenges and sessions cannot be reused for authorization.

## Verification

```bash
npm run selftest
npm run check:secret-store
```

`selftest` is non-destructive and uses temporary SQLite files with `MemorySecretStore`.
It prints a final JSON report matching `assets/schemas/selftest-report.schema.json`.

`check:secret-store` is non-destructive and prints a structured JSON report with platform, adapter, status, and failure reason:

1. Windows: checks for PowerShell CredentialManager cmdlets.
2. Linux: checks for `secret-tool`.
3. Unsupported platforms report `unsupported` with the platform name.
