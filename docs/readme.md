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

## 4. StoryLock Package Workflow

The implementation workspace now includes the first shared StoryLock package validation entry. It covers `package-manifest.json`, `resource-catalog.json`, `templates/*`, `author-draft.json`, and derived permission summaries.

```powershell
npm run test:storylock-package
npm run validate:storylock-package -- scripts\test\fixtures\storylock-package\valid
npm run inspect:storylock-package -- scripts\test\fixtures\storylock-package\valid
```

The host UI must continue to read only derived permission summaries. Story text, challenge answers, passwords, private keys, and `signingKeyBytes` stay inside the local core boundary.

## 5. Question Set Workflow

Layer 2 provides a template, validation, and import path:

```powershell
Push-Location src/skills/local-story-access
npm run generate:question-set-template
npm run validate:question-set -- --input assets/question-set-master.sample.json --require-min-active 24
npm run import:question-set -- --input assets/question-set-master.sample.json --db storylock.db --use-platform-secret-store
Pop-Location
```

Use `--dry-run` validation before importing any persistent question set.

## 6. Verification

Full workspace:

```powershell
node scripts/test/run-selftests.mjs
```

Path consistency:

```powershell
node scripts/verify/path-consistency.mjs
```

## 7. Documentation Entry Points

1. Workspace root: `README.md`
2. Chinese design entry: `docs/design/cn/README.md`
3. StoryLock package and validation CLI: `docs/design/cn/StoryLock数据包与校验CLI说明_20260621.md`
4. Submission references: `docs/ref/README.md`
5. Windows Host menu configuration: `docs/design/cn/YianWindowsHost菜单配置说明_20260621.md`
6. Current work configuration and task plans: `docs/management/`
7. Local-story-access assets and scripts: `src/skills/local-story-access/`
