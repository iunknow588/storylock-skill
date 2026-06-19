# StoryLock Project Brief

Version: 2026-06-18  
Scope: `skill/`

## 1. Positioning

StoryLock is a local-first authorization Skill project. It separates story-memory cues, object-strength policy, grid verification, short-lived local authorization, and remote request wrapping into three layers. A remote Agent can request local signing or Web2 password filling, but it does not directly hold long-term secrets.

The current focus is a runnable, auditable, redaction-aware local authorization chain. It does not claim to be a complete multi-chain wallet, multi-platform account manager, or full business orchestration system.

In one sentence:

> StoryLock connects local story-memory verification with remote delegated requests through a three-layer Skill structure, keeping sensitive signing and password-filling operations inside the local authorization boundary.

## 2. Current Three-Layer Structure

| Layer | Package | Current Capabilities |
| --- | --- | --- |
| Layer 1: Story processing and strength review | `src/skills/local-story-processing` | `StoryDraftSkill`, `StoryRefineSkill`, `StrengthReviewSkill` |
| Layer 2: Local access authorization | `src/skills/local-story-access` | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill` |
| Layer 3: Remote gateway | `src/skills/remote-gateway` | `requestSignature`, `requestPasswordFill` |
| Compatibility demo package | `src/engine` | Local password-fill and signature-authorization examples |

The current mainline remote surface focuses on two entries: `requestSignature` and `requestPasswordFill`. Local authorization is handled by Layer 2, while the remote gateway handles request wrapping, delegated execution, and redacted returns.

Current deployment direction:

1. Layer 3 can run behind the Web API entry at `web-api/storylock-gateway.mjs`.
2. Layers 1 and 2 can remain on an Android local host.
3. The repo currently verifies that split with a local Android-host mock, not a full Android app project.
4. Layer 3 can now expose Android app distribution metadata and second-layer connection metadata so the full system can be routed and downloaded from the server side.
5. Yian is the public website layer for this split: it presents the project, exposes APK download and first binding entries, and displays gateway / APK / host status.
6. PHAROS is an optional anchoring and trusted collaboration layer, not the local execution layer.

## 3. Core Capabilities

### 3.1 Layer 1

Layer 1 handles story content and question-set quality:

1. Generate story drafts.
2. Refine and organize story text.
3. Review question-set strength.

It does not issue authorization, read long-term secrets, or serve as a remote gateway.

### 3.2 Layer 2

Layer 2 handles local authorization control:

1. Resolve target-object strength: `low`, `medium`, or `high`.
2. Generate grid verification.
3. Verify local answers.
4. Issue short-lived sessions or authorizations.
5. Maintain requestId, nonce, failure windows, and audit logs.

Current SQLite state includes:

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `audit_log`

Answers are stored as digests, not plaintext.

### 3.3 Layer 3

Layer 3 is the unified entry point for remote Agents:

1. `requestSignature`: wraps signature requests using a `StoryLockSignatureRequest` EIP-712 style structure.
2. `requestPasswordFill`: wraps Web2 password-fill requests and defaults to audit metadata only.
3. Calls optional local executors: `signatureExecutor` and `passwordFillExecutor`.
4. Recursively redacts sensitive fields from returned results.

Layer 3 does not store private keys, passwords, grid answers, or plaintext story content.

## 4. Security Mechanisms

| Mechanism | Current Status |
| --- | --- |
| Node.js runtime | Requires Node.js 22 or above |
| SQLite state storage | Implemented |
| requestId/nonce replay protection | Implemented, conflicts use `SLG-013` |
| Expiry checks | Implemented, expired requests use `SLG-011` |
| Grid failure lockout | Implemented |
| Expired-state cleanup | Implemented via `npm run cleanup` |
| Answer digest storage | HMAC-SHA256 digest storage |
| SecretStore | Development memory store and platform-store factory |
| Remote result redaction | Recursive redaction implemented |

Persistent SQLite hosts must not silently use a plain `MemorySecretStore`. Development tests may explicitly use `developmentMode=true`; production should use a platform SecretStore or equivalent secure storage.

Production persistent hosts do not enable legacy answer fallback by default. Grid verification must use enough active question-set cells; insufficient question-set coverage returns `SLG-010` / `question_set_unavailable`. Only development or demo compatibility flows should explicitly pass `allowLegacyFallback: true`.

## 5. Run and Verification

Run the following commands from the `skill/` directory.

### 5.0 One-Command Verification

```powershell
npm run test
```

### 5.1 Layer 1 Self-Test

```powershell
Push-Location src/skills/local-story-processing
npm run selftest
Pop-Location
```

### 5.2 Layer 2 Self-Test

```powershell
Push-Location src/skills/local-story-access
npm run selftest
Pop-Location
```

This covers object strength, grid verification, replay protection, local authorization, lockout, cleanup, SQLite audit, and SecretStore constraints.

### 5.3 Layer 3 Self-Test

```powershell
Push-Location src/skills/remote-gateway
npm run selftest
Pop-Location
```

This covers `requestSignature`, `requestPasswordFill`, EIP-712 wrapping, and recursive redaction.

### 5.4 Three-Layer E2E Signature Demo

```powershell
Push-Location src/skills/remote-gateway
npm run selftest:e2e
Pop-Location
```

The script connects:

1. Layer 3 `requestSignature`.
2. Layer 2 object-strength policy.
3. Layer 2 grid verification.
4. Layer 2 local authorization.
5. A local signature executor.
6. Layer 3 redacted return.
7. SQLite audit logging.

### 5.5 Compatibility Demo Package Self-Test

```powershell
Push-Location src/engine
npm run selftest
Pop-Location
```

### 5.6 Vercel + Android-Host Split Self-Test

```powershell
Push-Location src/skills/remote-gateway
npm run selftest:web-api-android
Pop-Location
```

This starts:

1. A Vercel-style Layer 3 gateway entry.
2. An Android-host mock that simulates Layer 1 story-strength review, Layer 2 local authorization, and local executors.
3. End-to-end request routing plus redaction verification.

### 5.7 Yian Website Self-Test

```powershell
Push-Location src/ui
npm run selftest
Pop-Location
```

This verifies the homepage, locale controls, static assets, runtime status endpoint, and APK metadata rendering path.

### 5.8 Local Vercel-Style Gateway Development

```powershell
scripts\vercel\dev_local.cmd
```

This script reads `scripts/vercel/.env` when present and uses the same entry file as the Vercel deployment target.

### 5.9 Link Local Repo To Vercel Project

```powershell
scripts\vercel\link_project.cmd
```

Set `VERCEL_PROJECT_NAME` in `scripts/vercel/.env` before linking.

### 5.10 SQLite Cleanup Command

```powershell
Push-Location src/skills/local-story-access
npm run cleanup -- 2 --development-memory-secret-store
Pop-Location
```

## 6. Documentation Entry Points

| Document | Path |
| --- | --- |
| Workspace root entry | `README.md` |
| Chinese design entry | `docs/design/cn/README.md` |
| Submission reference docs | `docs/ref/README.md` |
| Test plan | `docs/test/StoryLock娴嬭瘯鏂规_v1.0.md` |
| Development progress | `docs/design/cn/寮€鍙戣惤鍦拌矾绾夸笌褰撳墠杩涘睍.md` |
| Review demo script | `docs/ref/06-璇勫璁茶В涓庢紨绀鸿鏄?md` |
| APK distribution | `docs/ref/07-APK鍒嗗彂涓庡畨瑁呰鏄?md` |
| Yian deployment | `docs/ref/08-鏄撳畨閮ㄧ讲涓庡煙鍚嶈鏄?md` |
| Layer terminology | `docs/ref/09-涓夊眰鏈涓嶱HAROS瀹氫綅.md` |
| Submission overview | `docs/ref/01-鍙傝禌姒傝.md` |

## 7. Current Completion Status

Completed:

1. Three main packages and capability boundaries.
2. Self-tests for all four packages.
3. Three-layer E2E signature demo.
4. SQLite audit, replay protection, failure lockout, and cleanup.
5. SecretStore production constraints.
6. Current design, test, and submission-reference documentation alignment.

Still to improve:

1. Production question sets now have an import dry-run path and schema; fuller release, migration, and rollback operation notes are still needed.
2. A real Android app host and Android Keystore integration are still future work; the current repo proves the split with an Android-host mock plus Vercel-style gateway entry.
3. Multi-chain, multi-platform, and multi-account scenarios are application exploration directions, not current implemented capabilities.

## 8. Recommended External Description

Recommended:

> StoryLock is a local-first authorization Skill project. Through story processing, local access authorization, and a remote gateway, it packages grid verification, short-lived authorization, signature requests, and Web2 password filling as callable capabilities while keeping long-term secrets and authorization decisions inside the local boundary.

Avoid:

1. 鈥淚t already supports a production-grade multi-chain wallet system.鈥?2. 鈥淭he remote gateway directly handles local plaintext sensitive content.鈥?3. 鈥淭he compatibility demo package is the full production security boundary.鈥?
## 9. Summary

StoryLock currently provides a verifiable three-layer Skill baseline:

1. Layer 1 handles story processing and question-set strength.
2. Layer 2 handles local verification, authorization, and audit.
3. Layer 3 handles remote request wrapping, delegated execution, and redacted returns.

It is suitable for demonstrating an Agent security model where remote systems can request capabilities, local systems authorize sensitive actions, results are auditable, and long-term secrets do not leave the local boundary.


