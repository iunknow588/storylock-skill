# StoryLock Project Brief

Version: 2026-06-17  
Scope: `skill/`

## 1. Positioning

StoryLock is a local-first authorization Skill project. It separates story-memory cues, object-strength policy, grid verification, short-lived local authorization, and remote request wrapping into three layers. A remote Agent can request local signing or Web2 password filling, but it does not directly hold long-term secrets.

The current focus is a runnable, auditable, redaction-aware local authorization chain. It does not claim to be a complete multi-chain wallet, multi-platform account manager, or full business orchestration system.

In one sentence:

> StoryLock connects local story-memory verification with remote delegated requests through a three-layer Skill structure, keeping sensitive signing and password-filling operations inside the local authorization boundary.

## 2. Current Three-Layer Structure

| Layer | Package | Current Capabilities |
| --- | --- | --- |
| Layer 1: Story processing and strength review | `src/storylock-local-story-processing-skill` | `StoryDraftSkill`, `StoryRefineSkill`, `StrengthReviewSkill` |
| Layer 2: Local access authorization | `src/storylock-local-story-access-skill` | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill` |
| Layer 3: Remote gateway | `src/storylock-remote-gateway-skill` | `requestSignature`, `requestPasswordFill` |
| Compatibility demo package | `src/storylock-skill-engine` | Local password-fill and signature-authorization examples |

The current mainline remote surface focuses on two entries: `requestSignature` and `requestPasswordFill`. Local authorization is handled by Layer 2, while the remote gateway handles request wrapping, delegated execution, and redacted returns.

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

## 5. Run and Verification

Run the following commands from the `skill/` directory.

### 5.0 One-Command Verification

```powershell
npm run test
```

### 5.1 Layer 1 Self-Test

```powershell
Push-Location src/storylock-local-story-processing-skill
npm run selftest
Pop-Location
```

### 5.2 Layer 2 Self-Test

```powershell
Push-Location src/storylock-local-story-access-skill
npm run selftest
Pop-Location
```

This covers object strength, grid verification, replay protection, local authorization, lockout, cleanup, SQLite audit, and SecretStore constraints.

### 5.3 Layer 3 Self-Test

```powershell
Push-Location src/storylock-remote-gateway-skill
npm run selftest
Pop-Location
```

This covers `requestSignature`, `requestPasswordFill`, EIP-712 wrapping, and recursive redaction.

### 5.4 Three-Layer E2E Signature Demo

```powershell
Push-Location src/storylock-remote-gateway-skill
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
Push-Location src/storylock-skill-engine
npm run selftest
Pop-Location
```

### 5.6 SQLite Cleanup Command

```powershell
Push-Location src/storylock-local-story-access-skill
npm run cleanup -- 2 --development-memory-secret-store
Pop-Location
```

## 6. Documentation Entry Points

| Document | Path |
| --- | --- |
| Chinese design entry | `docs/design/cn/README.md` |
| Submission reference docs | `docs/ref/README.md` |
| Test plan | `docs/test/StoryLock测试方案_v1.0.md` |
| Project completeness analysis | `docs/management/StoryLock项目完善度分析_20260617.md` |
| Development improvement plan | `docs/management/开发完善实施计划_20260617.md` |
| Submission brief | `docs/usecase/00-参赛说明文档.md` |

## 7. Current Completion Status

Completed:

1. Three main packages and capability boundaries.
2. Self-tests for all four packages.
3. Three-layer E2E signature demo.
4. SQLite audit, replay protection, failure lockout, and cleanup.
5. SecretStore production constraints.
6. Current design, test, and submission-reference documentation alignment.

Still to improve:

1. Grid cells are still generated from placeholder seeds; future work should connect real question sets or object policy.
2. Active challenge/session revocation can be expanded.
3. HTTP or host integration remains future work.
4. Multi-chain, multi-platform, and multi-account scenarios are application exploration directions, not current implemented capabilities.

## 8. Recommended External Description

Recommended:

> StoryLock is a local-first authorization Skill project. Through story processing, local access authorization, and a remote gateway, it packages grid verification, short-lived authorization, signature requests, and Web2 password filling as callable capabilities while keeping long-term secrets and authorization decisions inside the local boundary.

Avoid:

1. “It already supports a production-grade multi-chain wallet system.”
2. “The remote gateway directly handles local plaintext sensitive content.”
3. “The compatibility demo package is the full production security boundary.”

## 9. Summary

StoryLock currently provides a verifiable three-layer Skill baseline:

1. Layer 1 handles story processing and question-set strength.
2. Layer 2 handles local verification, authorization, and audit.
3. Layer 3 handles remote request wrapping, delegated execution, and redacted returns.

It is suitable for demonstrating an Agent security model where remote systems can request capabilities, local systems authorize sensitive actions, results are auditable, and long-term secrets do not leave the local boundary.
