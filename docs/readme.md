# StoryLock Project Brief

Version: 2026-06-20  
Scope: `skill/`

## 1. Positioning

StoryLock is a local-first authorization Skill project. It separates story-memory cues, local access authorization, and remote request wrapping into three runtime layers. Remote systems can request signatures or Web2 password filling, but long-term secrets and final authorization stay inside the local boundary.

## 2. Current Structure

| Layer | Source directory | Historical package name | Current capabilities |
| --- | --- | --- | --- |
| Layer 1 | `src/skills/local-story-processing` | `storylock-local-story-processing-skill` | `StoryDraftSkill`, `StoryRefineSkill`, `StrengthReviewSkill` |
| Layer 2 | `src/skills/local-story-access` | `storylock-local-story-access-skill` | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill`, `LocalRevocationSkill` |
| Layer 3 | `src/skills/remote-gateway` | `storylock-remote-gateway-skill` | `requestSignature`, `requestPasswordFill`, redaction-aware transport |
| Compatibility/demo | `src/engine` | `storylock-skill-engine` | local password-fill and signature-authorization examples |

The source tree uses the simplified directories above. Older package names remain in some manifests, schemas, and review documents for compatibility.

## 3. Security Notes

1. Persistent SQLite hosts must use an injected SecretStore or `usePlatformSecretStore=true`.
2. `MemorySecretStore` is development-only and now requires explicit `developmentMode=true`.
3. Layer 3 automatically resolves `demo`, `test`, or `production` EIP-712 domain configuration from environment variables when a domain is not passed directly.
4. Production persistent hosts do not enable legacy answer fallback by default.

## 4. Question Set Workflow

Layer 2 provides a template, validation, and import path:

```powershell
Push-Location src/skills/local-story-access
npm run generate:question-set-template
npm run validate:question-set -- --input assets/question-set-master.sample.json --require-min-active 24
npm run import:question-set -- --input assets/question-set-master.sample.json --db storylock.db --use-platform-secret-store
Pop-Location
```

Use `--dry-run` validation before importing any persistent question set.

## 5. Verification

Full workspace:

```powershell
node scripts/test/run-selftests.mjs
```

Path consistency:

```powershell
node scripts/verify/path-consistency.mjs
```

## 6. Documentation Entry Points

1. Workspace root: `README.md`
2. Chinese design entry: `docs/design/cn/README.md`
3. Submission references: `docs/ref/README.md`
4. Management and review notes: `docs/management/`
5. Local-story-access assets and scripts: `src/skills/local-story-access/`
