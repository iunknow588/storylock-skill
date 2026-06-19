# StoryLock Object Access Policy

## Purpose

This document defines how StoryLock should:

1. Classify objects
2. Determine access strength
3. Determine whether access is one-time or reusable
4. Determine whether a remote Agent can retain any capability or result after access

It is used in conjunction with the following documents:

1. `skill_positioning_and_boundaries.md`
2. `src/engine/SKILL.md`

## Policy Goals

When running locally, it should be able to clearly answer the following questions:

1. What category of object is currently being accessed
2. How high a challenge strength is required
3. Whether access is single-time or session-based
4. Whether a remote Agent can retain any capability after access

## Object Classification

| Object Category | Examples | Sensitivity | Default Runtime Location |
| --- | --- | --- | --- |
| `story_public` | General story templates, non-private drafts | Low | Remote-safe |
| `story_private` | User memories, personal narratives, private drafts | Medium to high | Local-first |
| `auth_login` | Username / password field packages | High | Local-only |
| `auth_signing` | Signing keys, signing authorization material | Very high | Local-only |
| `review_data` | 24-question sets and readiness assessment data | Medium | Depends on content, can be local or remote |

## Access Strength Levels

The system can expose multiple access thresholds.

| Level | Example Threshold | Typical Usage |
| --- | --- | --- |
| `L1` | Weak challenge or no challenge | Public templates, low-risk metadata |
| `L2` | 6-of-24 | Low-risk access or limited assessment data |
| `L3` | 12-of-24 | Medium-sensitive secret access or short-term field access |
| `L4` | 22-of-24 | Signature-class or high-trust operations |
| `L5` | Higher threshold determined by Host | Root-level objects or particularly sensitive operations |

Current StoryLock constants already provide a basic mapping:

1. basic access threshold: `6`
2. batch access threshold: `12`
3. modify threshold: `22`

## Progressive Challenge Strategy

The 24-question set is a unified fallback question set, not three different question sets.

Current stage recommendations:

1. `L2 = 6-of-24`
2. `L3 = 12-of-24`
3. `L4 = 22-of-24`

This reflects:

1. The same question set
2. Different pass thresholds
3. Corresponding to different access strengths

The benefits are:

1. Users only maintain one question set
2. Threshold differences directly reflect permission strength differences
3. System complexity is not wasted on managing multiple question sets

## Session Model

Access must explicitly declare duration and reuse capability.

| Session Type | Lifecycle | Read Count | Reusable | Remote Retention Rule |
| --- | --- | --- | --- | --- |
| `one_shot` | Single call | 1 | No | No |
| `short_session` | Short TTL | Limited | Limited | Only retain opaque handle when permitted |
| `batch_session` | Medium TTL | Multiple | Limited | Can retain only when policy permits |
| `privileged_session` | Strict TTL | Minimal | No | Never retain |

Current implementation constants can be used as policy inputs:

1. challenge TTL
2. session TTL
3. basic session access counts
4. batch session access counts

## Remote Capability Retention Rules

Remote Agents should not long-term retain sensitive access capabilities.

Allowed to retain:

1. Structured results
2. Redacted summaries
3. Short-term opaque handles under explicit approval

Not allowed to retain:

1. Raw passwords
2. Private keys
3. Challenge answers
4. Long-term reusable secret access tokens
5. Layer 2 internal session material

## Local Execution Mandatory Rules

If object access involves any of the following, it must be executed locally:

1. challenge answers
2. session material
3. secret-object access
4. signing-key handling
5. root-level object reconstruction
6. question set answer digest verification

This means:

1. Layer 3 can only initiate requests, not replace local execution
2. Pharos or other remote Skills can only undertake packaging and capability indexing
3. If Layer 1 processes raw private story content, it should also prefer local execution

## Access Decision Table

| Object Category | Recommended Strength | Session Type | Can Remote Cache Results | Can Remote Retain Subsequent Access Capability |
| --- | --- | --- | --- | --- |
| `story_public` | `L1` | `one_shot` | Yes | Yes |
| `story_private` | `L2-L3` | `short_session` | Only after redaction | No |
| `auth_login` | `L3` | `short_session` | No | No |
| `auth_signing` | `L4-L5` | `privileged_session` | No | No |
| `review_data` | `L2-L3` | `short_session` or `batch_session` | After redaction | No |

Supplementary restrictions:

1. `story_private` does not return complete original text to the remote side by default, unless explicit policy permits and results are already redacted
2. `auth_signing` only returns signature results by default, not signature capability handles
3. `review_data` if containing question set structures, answer digests, or highly inferable information, should be elevated to local-first objects

## Challenge Configuration Examples

### L2: Low-Risk Reading

```json
{
  "accessLevel": "L2",
  "threshold": 6,
  "questionPoolSize": 24,
  "sessionType": "short_session",
  "sessionTtlSeconds": 600,
  "maxReads": 3,
  "redactionRequired": true
}
```

### L3: Medium-Sensitivity Reading or Limited Write-Back

```json
{
  "accessLevel": "L3",
  "threshold": 12,
  "questionPoolSize": 24,
  "sessionType": "short_session",
  "sessionTtlSeconds": 600,
  "maxReads": 1,
  "maxWrites": 1,
  "redactionRequired": true
}
```

### L4: High-Sensitivity Signature or High-Trust Write Operation

```json
{
  "accessLevel": "L4",
  "threshold": 22,
  "questionPoolSize": 24,
  "sessionType": "privileged_session",
  "sessionTtlSeconds": 180,
  "maxReads": 0,
  "maxWrites": 1,
  "redactionRequired": true,
  "requiresLocalExecution": true
}
```

## Runtime Rules

If access involves any of the following:

1. challenge answers
2. session material
3. secret-object reads
4. signing-key handling

Then it must be completed within the local runtime.

If access belongs to the following scenarios, it should also not be directly persisted by Layer 3 by default:

1. Intermediate materials other than high-sensitivity signature authorization results
2. Raw question set inputs during question set evaluation
3. Any intermediate states during root-level reconstruction

## Session and Compatibility Window Rules

Under the single-story lifetime model, access policy not only determines "whether access is allowed", but also "how long old capabilities can be retained".

It is recommended to clarify the following:

1. During routine question set optimization, already issued `short_session` can run until TTL ends
2. During routine object packaging migration, old object ciphertexts can retain a short compatibility window
3. During root-level reconstruction, old sessions must immediately expire
4. `privileged_session` regardless of optimization type, does not allow cross-window reuse

## Layer 3 Visible Information Boundary

Layer 3 remote Agents should only see by default:

1. capability name
2. structured results
3. redaction metadata
4. retentionGranted
5. low-sensitivity meta information

Layer 3 should not see by default:

1. challenge creation details
2. answer verification details
3. question set answer digests
4. secret object raw ciphertexts
5. rootKey / workKey / objectKey related information

## Recommended Implementation Form

Local runtime should expose a small policy gateway for returning:

1. Required access threshold
2. Approved session type
3. Read budget
4. Whether cacheable
5. Whether redaction is required

Remote Agents should only consume:

1. capability name
2. result payload
3. redacted metadata
4. Short-term opaque handles under explicit approval

## Conclusion

The correct model is not "any capability can be reused anywhere".

The correct model should be:

1. Access is classified by object
2. Sessions are strictly limited
3. Remote reuse is constrained
4. Sensitive capability supply should not be long-term saved on the remote side
