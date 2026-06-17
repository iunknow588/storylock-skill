# StoryLock Challenge State Machine

This document defines the minimum state machine for challenge, session, and authorization in Layer 2 local access authorization. It serves `storylock-local-story-access-skill` and no longer describes story object reading or writing flows.

## Current Responsibilities

Layer 2 is responsible for:

1. Creating grid challenge verification based on object policy.
2. Receiving and verifying local answers.
3. Recording failure windows and answer digests.
4. Creating short-term sessions or authorization after verification passes.
5. Writing local audit logs.

Layer 2 is not responsible for:

1. Reading story content.
2. Writing back story content.
3. Directly executing signatures or password fills.
4. Returning long-term secrets to the remote side.

## Minimum State Set

| State | Meaning |
| --- | --- |
| `created` | Challenge has been created, waiting for answer submission |
| `verified` | Answer verification passed, short-term authorization can be issued |
| `failed` | Answer verification failed |
| `expired` | Challenge has expired |
| `locked` | Failure count or policy conditions reached threshold, temporarily locked |

Session or authorization states should be recorded separately:

| State | Meaning |
| --- | --- |
| `active` | Short-term authorization is valid |
| `expired` | TTL expired or budget exhausted |
| `revoked` | Actively revoked by user or Host |

## State Transitions

### Challenge Transitions

1. `created -> verified`: Answers meet policy requirements.
2. `created -> failed`: Answers do not meet policy requirements.
3. `created -> expired`: Exceeds challenge TTL.
4. `failed -> created`: Retry is allowed and lockout threshold is not exceeded.
5. `failed -> locked`: Reaches failure count threshold.
6. `locked -> created`: Lockout window ends and challenge is recreated.

### Authorization Transitions

1. `verified -> active`: Local policy allows issuing short-term authorization.
2. `active -> expired`: TTL expired, read/write budget exhausted, or nonce/requestId invalid.
3. `active -> revoked`: Actively revoked by user or Host.

## Session Activation Conditions

Short-term session or authorization can only be issued when all of the following conditions are simultaneously satisfied:

1. Challenge has not expired.
2. Answer verification passed.
3. Scope matches target object.
4. requestId and nonce have not been used.
5. Local policy allows this capability to be requested.

## Failure and Lockout Strategy

Current recommendations:

1. Failure count is statistics by `identityId + time window`.
2. Default window is 24 hours.
3. Default maximum failure count is 3.
4. Enter `locked` after reaching threshold.
5. Continuous failure count can be reset after successful verification.

During lockout:

1. Reject new answer submissions.
2. Return clear errors.
3. Response contains `retryAfter`.
4. Do not leak which specific question was wrong.

## SQLite Persistence

Current implementation uses SQLite to save access authorization related states, mainly including:

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `audit_log`

Layer 2 no longer saves `protected_story_objects`.

## Concurrency Control

Same `challengeId`, `requestId`, or `nonce` should guarantee serialized state transitions.

Recommendations:

1. Use SQLite transactions to protect state migrations.
2. Validate whether current state matches expectations before writing.
3. Return concurrency or replay errors when state has already changed.
4. Replay conflicts uniformly use `SLG-013`.

## Default Duration Suggestions

| Item | Default Value |
| --- | --- |
| challenge TTL | 5 minutes |
| one-shot authorization TTL | 3 minutes |
| short session TTL | 10 minutes |
| locked window | 15 minutes |

These are conservative defaults, subsequently can be overridden by Host policy.

## Conclusion

The purpose of the challenge state machine is not to support story reading/writing, but to form a local access authorization closed loop:

1. Create grid challenge verification.
2. Verify answers.
3. Prevent replay and brute-force attempts.
4. Issue short-term authorization.
5. Record audit.
