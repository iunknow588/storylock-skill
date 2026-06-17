# StoryLock Redaction Specification

| Item | Content |
| --- | --- |
| Document Version | v1.0 |
| Date | 2026-06-16 |
| Status | Adopted at current stage |
| Applicable Layers | Layer 1, Layer 2, Layer 3 |

## Purpose

This document clarifies how StoryLock performs redaction in the "local processing, remote packaging, third-party integration" chain.

At the current stage, it must answer:

1. Which fields can be transmitted remotely
2. Which fields must be deleted or replaced
3. How return results are graded

## Core Conclusion

Redaction is not as simple as "deleting obviously secret stuff", but rather:

1. Performing field-level minimum exposure before cross-layer, cross-Agent transmission
2. Letting Layer 3 and third-party Skills only see the information necessary to complete the task
3. Letting audit logs record "redaction was performed" rather than "original text was saved"

## Three Redaction Levels

| Level | Meaning | Applicable Scenarios |
| --- | --- | --- |
| `none` | No redaction | Between Layer 1 and Layer 2, local-only processing |
| `partial` | Partial redaction | Layer 2 returning to Layer 3, Layer 3 returning to trusted callers |
| `full` | Full redaction / summary only | Third parties only need conclusions, not content plaintext |

## Default Rules

### Layer 1 and Layer 2

Defaults:

1. Can use `none`
2. But only within the same local trust boundary

### Layer 2 and Layer 3

Defaults:

1. Prefer `partial`
2. `none` is prohibited by default for high-sensitivity objects

### Layer 3 and Third-Party Skills

Defaults:

1. Prefer `partial` or `full`
2. Must not return challenge answers, private keys, or long-term session capabilities

## High-Sensitivity Field List

The following fields are treated as high-sensitivity by default:

1. Real name
2. Phone number
3. Email address
4. ID number / passport number / document number
5. Home address
6. Precise geographic location
7. User account name and password combination
8. Private key / mnemonic / seed / signing key bytes
9. Challenge answers plaintext
10. Unredacted private story paragraphs

## Minimum Redaction Checklist

At the current stage, before sending content to a remote LLM or third-party Skill, at least check the following items:

1. Does it contain a real name?
2. Does it contain a phone number?
3. Does it contain an email address?
4. Does it contain an ID number, passport number, or other document number?
5. Does it contain a home address or precise geographic location?
6. Does it contain an account name and password combination?
7. Does it contain a private key, seed, mnemonic, or key bytes?
8. Does it contain challenge answers plaintext?
9. Does it contain unredacted private story content?
10. Does it contain combination clues that can uniquely locate a user's identity, such as organization name, school name, hospital name, specific house number, etc.?

Recommended execution order:

1. First perform structured field checks
2. Then perform regex matching
3. Finally perform manual confirmation or local rule review

Only after all checks pass is it allowed to enter the remote calling chain.

## Field Processing Rules

| Field Type | Default Action | Example |
| --- | --- | --- |
| Identifier | Hash or partial display | `identity-001` can be retained, phone number changed to `138****0000` |
| Text content | Summarized or partially masked | Private stories only return summaries |
| Key material | Never return | Private keys, seeds, key bytes are all prohibited from external transmission |
| Challenge material | Never return | Question answers, comparison clues are all prohibited from external transmission |
| Audit fields | Minimal metadata can be retained | requestId, capability, status |

## Recommended Redaction Actions

### 1. Partial Masking

Applicable to:

1. Phone numbers
2. Emails
3. Usernames

Example:

```json
{
  "phone": "138****0000",
  "email": "st***@example.com"
}
```

### 2. Summary Replacement

Applicable to:

1. Private story content
2. Evaluation raw data
3. Long text materials

Example:

```json
{
  "storySummary": "A summary of a private narrative about childhood memories and identity."
}
```

### 3. Structure Retained, Value Cleared

Applicable to:

1. Returning object structures to third-party Skills
2. But not wanting to expose field values

Example:

```json
{
  "result": {
    "storyObjectId": "story-001",
    "title": "[redacted]",
    "content": "[redacted]"
  }
}
```

## Default Redaction Levels by Object Category

| Object Category | Layer 2 -> Layer 3 | Layer 3 -> Third Party |
| --- | --- | --- |
| `story_public` | `none` or `partial` | `partial` |
| `story_private` | `partial` | `full` or controlled `partial` |
| `auth_login` | `partial` | `full` |
| `auth_signing` | `full` | `full` |
| `review_data` | `partial` | `partial` or `full` |

## Redaction Expression in Response Structures

All cross-package responses should explicitly return:

1. `redactionLevel`
2. `retentionGranted`
3. `result`

For example:

```json
{
  "requestId": "req-001",
  "status": "success",
  "redactionLevel": "partial",
  "retentionGranted": "result_only",
  "result": {
    "storySummary": "Redacted summary"
  }
}
```

## Audit Requirements

Audit logs should record:

1. Original object category
2. Applied redaction level
3. Whether high-sensitivity fields exist
4. Whether returning original text was refused

But must not record:

1. Original content of deleted fields
2. Private key material
3. Challenge answers plaintext

## Relationship with Three-Package Boundaries

Package 1 is responsible for content processing, but does not determine remote visibility scope.

Package 2 is responsible for:

1. Deciding whether redaction is mandatory based on object policy
2. Enforcing redaction on cross-boundary results

Package 3 is responsible for:

1. Not relaxing redaction results already imposed by Package 2
2. Further restricting caching and forwarding according to `retentionGranted`

## Current Stage Minimal Implementation Suggestions

It is recommended to first support the following three minimal rules:

1. High-sensitivity field blacklist deletion
2. Private story summarization
3. Login/signature results only return success status and necessary results, not underlying secrets
4. Execute the minimum redaction checklist before calling a remote LLM

## Conclusion

StoryLock's redaction specification is essentially the output constraint of the three-layer Skill boundary.

It guarantees:

1. Local can process complete data
2. Remote can only obtain minimal results
3. Third-party Skills will not reverse-expand data exposure because integration is convenient
