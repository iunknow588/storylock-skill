# StoryLock Session and Anti-Replay Strategy

## Current Implementation Baseline

This document is based on the current `storylock-local-story-access-skill/access-host.js`.

Layer 2 currently uses SQLite to store:

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `audit_log`

Layer 2 no longer stores `protected_story_objects`, nor does it provide story object reading/writing methods.

## Session Purpose

Session is a short-term authorization result after passing the grid challenge, not a long-term login state.

Each session binds:

1. `sessionId`
2. `challengeId`
3. `identityId`
4. `scope`
5. `resourceScope`
6. `sessionType`
7. `readBudget`
8. `writeBudget`
9. `issuedAt`
10. `expiresAt`
11. `status`

Default budget is `0/0`. Budget is only granted when an explicit action requires reading local material, e.g., `signature` and `password_fill` currently use `readBudget=1, writeBudget=0`.

## Anti-Replay Fields

| Field | Purpose |
| --- | --- |
| `requestId` | Identifies a business request, supports idempotent returns |
| `nonce` | Identifies an authorization or signature sequence, prevents replay |
| `expiry` | Limits request validity time |
| `sessionId` | Identifies a local authorization result |

Current implementation:

1. Same `requestId` with identical request hash returns the saved response.
2. Same `requestId` but different request is rejected.
3. Already used `nonce` appearing again is rejected.
4. Expired requests return `SLG-011`.
5. Replay conflicts return `SLG-013`.

## Challenge States

Current code actually uses the following challenge states:

1. `challenge_created`
2. `answers_submitted`
3. `verified`
4. `failed`
5. `locked`
6. `expired`
7. `idle` (used for state recovery after lock release)

The complete 8-state model can be a subsequent enhancement, but current implementation is based on runnable SQLite transaction states.

## Failure Lockout

Current strategy:

1. 24-hour failure window.
2. 3 consecutive failures trigger lockout.
3. Lockout duration is 15 minutes.
4. Automatic recovery after lockout expires.

## Cleanup Strategy

`cleanupExpired(now, { batchSize })` cleans up or marks:

1. Expired `request_store`
2. Expired `nonce_store`
3. Expired active sessions
4. Expired challenges

Default single batch limit is 1000 to avoid blocking the main thread for extended periods.

Currently triggers a cleanup once during `createAccessHost()` initialization. Long-running hosts should subsequently add scheduled calls or CLI cleanup commands.

## Error Codes

| Error Code | Type | Scenario |
| --- | --- | --- |
| `SLG-011` | `request_expired` | Request expired |
| `SLG-013` | `replay_detected` | requestId or nonce conflict |
| `SLG-003` | `challenge_failed` | Answers do not match |
| `SLG-004` | `challenge_locked` | Locked after too many failures |
| `SLG-005` | `session_invalid` | Session does not exist, expired, or status is invalid |

## Subsequent Improvements

1. Add scheduled cleanup mechanism.
2. Clarify multi-device nonce strategy.
3. If object reading/writing is restored later, atomic transactions for budget deduction and object access need to be implemented in an independent persistence layer.