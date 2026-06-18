# Question Set Operations

This reference describes the operational path for production question sets used by `GridChallengeSkill`.

## Master File

Use `assets/schemas/question-set-master.schema.json` for the master-file shape.

Required fields:

1. `identityId`
2. `questionSetVersion`
3. `normalizationVersion`
4. `status`: `active`, `deprecated`, or `pending`
5. `questions[]` with `questionId`, `versionTag`, `promptRef`, `answer`, and either `options` or `optionDigest`

For high-strength verification, the active set must contain at least 9 active questions.

## Validate Before Import

```powershell
node scripts/import-question-set.mjs --input assets/question-set-master.sample.json --dry-run
```

Dry-run validates shape and active question count without writing answer digests.

## Production Import

Persistent imports must use a stable platform SecretStore because answer digests depend on the local `masterSalt`.

```powershell
node scripts/import-question-set.mjs --input question-set-v2.json --db storylock.db --use-platform-secret-store
```

Do not use `--development-memory-secret-store` for persistent production imports. The script rejects that combination.

## Release Flow

1. Prepare a new master file with a new `questionSetVersion`.
2. Run dry-run validation.
3. Import with `status: "pending"` for review.
4. Promote the reviewed set by importing the same version with `status: "active"` and `replacePreviousActive: true`.
5. Confirm new high-strength grid creation selects only the active version.
6. Keep old versions as `deprecated` until their sessions and migration windows expire.

## Rollback Flow

1. Mark the faulty active version as `deprecated`.
2. Re-import the previous known-good version with `status: "active"`.
3. Run a medium and high grid self-check for the affected identity.
4. Record the rollback reason in deployment notes or audit metadata outside this package.

## Current Enforcement

The access host only selects `status = "active"` rows when building grid cells. Pending and deprecated question sets are not selectable, even when a caller passes their `questionSetVersion` explicitly.

Legacy answer fallback is disabled for persistent production hosts by default. It is available only through explicit development or demo configuration.
