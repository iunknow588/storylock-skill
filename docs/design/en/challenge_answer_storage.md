# StoryLock Challenge Answer Storage Strategy

| Item | Content |
| --- | --- |
| Document Version | v1.0 |
| Date | 2026-06-16 |
| Status | Adopted at current stage |
| Applicable Layer | Layer 2 `storylock-local-story-access-skill` |

## Purpose

This document clarifies how Layer 2 local access Skills handle challenge answers.

What needs to be solved is not "whether answers can be stored", but rather:

1. Which data can be long-term saved
2. Which data can only short-term exist in memory
3. How to avoid answers themselves becoming new high-sensitivity leak points

## Core Conclusion

The current stage adopts the following principles:

1. Raw challenge answers are not long-term persisted
2. Raw answers are only allowed to exist in local short-term memory
3. The persistence layer only saves irreversible or derived materials required for verification
4. Remote Agents, third-party Skills, and Layer 3 remote gateways must not touch answer plaintext

## Data Classification

| Data Item | Sensitivity | Allow Persistence | Description |
| --- | --- | --- | --- |
| `raw_answers` | High | No | User's raw submitted answers this time, only short-term in memory |
| `answer_digest` | Medium to high | Yes | Digest computed after normalizing answers, used for comparison or auditing |
| `challenge_manifest` | Medium | Yes | This challenge's questions, thresholds, TTL, scope, etc. |
| `verification_result` | Medium | Yes | Whether passed, pass time, failure count |
| `session_binding` | Medium | Yes | Session metadata derived from challenge, does not contain answer plaintext |

## Current Stage Storage Principles

### 1. Raw Answers Only Exist in Memory

After the local Agent / Host receives answers:

1. Placed in short-term memory before entering the verification process
2. Immediately cleaned after verification completes
3. Not written to logs
4. Not written to remote cache
5. Not written to general database plaintext fields

⚠️ Current stage special attention:

1. JavaScript `String` is an immutable object
2. Even if a string variable is set to `null`, it does not mean the underlying memory is immediately cleared
3. For extremely high security scenarios, sensitive answers should prefer `Buffer` over `String`

Node.js implementation suggestions:

```js
function clearSensitiveBuffer(buf) {
  if (Buffer.isBuffer(buf)) {
    buf.fill(0);
  }
}
```

More strict overwrite example:

```js
import { randomFillSync } from "node:crypto";

function wipeBuffer(buf) {
  if (!Buffer.isBuffer(buf)) return;
  randomFillSync(buf);
  buf.fill(0);
}
```

If answers exist as string arrays, it is recommended to:

1. Convert to short-lifetime structures as soon as possible
2. After verification completes, overwrite array elements with empty values
3. Then set the reference to `null`

Example:

```js
function clearRawAnswers(rawAnswers) {
  if (!Array.isArray(rawAnswers)) return null;
  for (let i = 0; i < rawAnswers.length; i += 1) {
    rawAnswers[i] = "";
  }
  return null;
}
```

Recommended actual order:

1. If possible, first convert user input to `Buffer`
2. After verification completes, prioritize zeroing `Buffer`
3. Then clear string references
4. Finally remove object references and wait for GC

### 2. Persistence Only Saves Derived Results

Data recommended for persistence:

1. `challengeId`
2. `identityId`
3. `scope`
4. `requiredThreshold`
5. `submittedAt`
6. `verificationPassed`
7. `failureCount`
8. `answerDigestSet`

Where `answerDigestSet` should be:

1. A set of normalized answer digests
2. Irreversible
3. Insufficient to recover original answers

Digest recommendations:

1. Use identity-isolated salt value digests
2. For example `salt = HMAC(masterSalt, identityId)`
3. Then compute digest on normalized answers, reducing rainbow table attack risk

### 3. Audit Logs Only Record Process, Not Answers

Audit logs can record:

1. Whether challenge was created
2. Whether answers were submitted
3. Whether verification passed
4. Which scope was used
5. What session was ultimately issued

Audit logs must not record:

1. Raw answer text
2. Reversibly encrypted answer copies
3. Challenge material that can be directly used for replay

## Recommended Data Structures

### Challenge Manifest

```json
{
  "challengeId": "chl-001",
  "identityId": "identity-001",
  "scope": "story_read_basic",
  "requiredThreshold": 6,
  "questionIds": ["q1", "q2", "q3"],
  "createdAt": 1760000000,
  "expiresAt": 1760000300
}
```

### Verification Record

```json
{
  "challengeId": "chl-001",
  "submittedAt": 1760000100,
  "verificationPassed": true,
  "matchedCount": 6,
  "failureCount": 0,
  "answerDigestSet": [
    "sha256:...",
    "sha256:..."
  ],
  "normalizationVersion": "v1"
}
```

## Answer Normalization Suggestions

To reduce misjudgment and digest splitting, it is recommended to perform unified normalization before local verification:

1. Remove leading and trailing whitespace
2. Unify full-width and half-width characters
3. Unify case strategy
4. Clarify whether to retain punctuation
5. Clarify whether to allow synonym answer mapping

Note:

1. Normalization process must be completed locally
2. Normalization rules should be fixed versions
3. When rules are upgraded, compatibility strategies should be retained

It is recommended to explicitly record:

1. `normalizationVersion`
2. `digestAlgorithm`
3. `saltStrategyVersion`

## Normalization Version Compatibility Strategy

When `normalizationVersion` is upgraded, current stage recommendations:

1. Support dual-version verification within the migration window
2. New writes uniformly use the new version
3. After verification passes, old records can be asynchronously migrated to new version digests

This avoids:

1. All old challenges immediately becoming invalid after upgrade
2. Users being forced to rebuild all question sets without awareness

## Relationship with Question Set Storage

Question set answer bodies should not be mixed with challenge submitted answers.

The current implementation separates them:

1. `question_set_item` stores the question-set master record: `questionId`, `versionTag`, `promptRef`, display prompt, option digest, answer digest, `normalizationVersion`, `questionSetVersion`, and `active/deprecated/pending` status.
2. `challenge_state.challenge_manifest_json` stores the challenge instance: grid cells, question references, versions, threshold, and expiry.
3. `challenge_state.expected_answer_digests_json` stores only the per-cell digests required for verification, not answer plaintext.
4. `GridChallengeSkill` returns cells with question references, prompt text, versions, and option digests, but never answer plaintext.
5. `LocalAuthorizationSkill` verifies answers by `cellId -> questionId`; high strength requires 9 cells, medium requires 6 cells, and low requires 3 cells.

It is recommended to split into two types of storage:

1. **Question Set Master File**
   - Questions, answer digests, version numbers, rotation status
2. **Challenge Instance File**
   - Which questions were selected this challenge, what the threshold is, whether it passed

The reason is:

1. Question set master file is a long-term policy object
2. Challenge instance file is a short-lifetime runtime object
3. Separating them is more conducive to minimum permissions and rotation management

Question set master file recommendations:

1. Treated as medium-sensitivity objects
2. Default static encrypted storage
3. At least avoid plaintext writing to disk

## Lifecycle Suggestions

| Data Item | Lifecycle | Default Handling |
| --- | --- | --- |
| `raw_answers` | Seconds-level | Immediately cleaned after verification completes |
| `challenge_manifest` | Minutes to hours-level | After expiry, can be archived with streamlined metadata |
| `verification_record` | Days to months-level | Only retain minimum fields required for auditing |
| `session_binding` | Session-level | Expires upon expiry, retain digest audit if necessary |

## Relationship with Three-Package Boundaries

Package 1 must not touch challenge answers.

Package 2 is responsible for:

1. Creating challenges
2. Receiving answers
3. Local verification
4. Deriving sessions

Package 3 can only see:

1. Whether challenge is required
2. Whether verification passed
3. Whether session was obtained
4. Minimum audit metadata

## Current Stage Minimal Implementation Suggestions

The current stage can first adopt:

1. Raw answers only saved in memory
2. Local JSON / SQLite saves challenge manifest and verification record
3. Only record digests, not original text
4. Challenge expired then clean runtime data
5. Explicitly execute memory cleanup logic after verification completes

## Conclusion

StoryLock's challenge answers should not be treated as "ordinary business input", but as high-sensitivity authorization material.

Therefore the most appropriate strategy at the current stage is:

1. Original text is not persisted
2. Derived values are minimally persisted
3. Auditing only records process, not secrets
