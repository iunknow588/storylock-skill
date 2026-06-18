# StoryLock Security Specification

| Item | Content |
| --- | --- |
| Document Version | v1.0 |
| Date | 2026-06-16 |
| Status | Adopted at current stage |
| Applicable Scope | Shared security constraints for Layer 1, Layer 2, and Layer 3 |

## Purpose

This document centralizes the security implementation standards that must be frozen at the current stage, avoiding omissions caused by scattering them across multiple design documents.

## Current Stage Minimum Security Baseline

The current stage explicitly adopts:

1. Root key derivation and answer digest handled separately
2. Question set master file and challenge runtime state stored separately
3. Symmetric encryption uniformly uses AEAD
4. Long-term sensitive material prioritized into the operating system keychain

## Algorithm Baseline

| Target | Current Stage Recommendation |
| --- | --- |
| Symmetric encryption | `AES-256-GCM` |
| Digest / HMAC | `SHA-256` / `HMAC-SHA256` |
| Working key derivation | `HKDF-SHA256` |
| Local random numbers | Cryptographically secure random numbers, length at least 32 bytes |

## Key Hierarchy Suggestions

The current stage recommends the following hierarchy:

1. `masterSalt`
2. `rootKey`
3. `workKey`
4. `objectKey`

## HKDF Derivation Suggestions

Working key derivation recommendations:

1. Algorithm: `HKDF-SHA256`
2. `salt`: Fixed to a salt value related to the current identity context
3. `info`: Must explicitly include purpose

Example:

```text
info = "storylock/workkey/" + identityId + "/" + purpose
```

## Question Set Master File Encryption Specification

Question set master file current stage recommendations:

1. Explicitly treated as a medium-sensitivity object
2. Use `AES-256-GCM` before writing to disk
3. Encryption key derived from `rootKey` or its derived working key
4. Plaintext long-term writing to disk is not allowed

## GCM Nonce Management

Current stage unified requirements:

1. Each encryption independently generates a 96-bit random nonce
2. Nonce must not be reused under the same key
3. Nonce must be stored together with ciphertext

## GCM Nonce Storage Format

Current stage recommends fixing two acceptable formats to avoid different implementations writing their own.

### Option A: Binary Concatenation Format

Recommended format:

```text
nonce(12 bytes) || ciphertext(n bytes) || tag(16 bytes)
```

Applicable to:

1. Local binary files
2. SQLite BLOB fields
3. Scenarios requiring compact storage

### Option B: Structured JSON Format

Recommended format:

```json
{
  "algorithm": "AES-256-GCM",
  "nonce": "base64...",
  "ciphertext": "base64...",
  "tag": "base64..."
}
```

Applicable to:

1. Debugging and audit-friendly scenarios
2. Scenarios requiring cross-language serialization

Current stage recommendations:

1. Prefer Option A for on-disk main format
2. Prefer Option B for cross-interface or documentation examples

## masterSalt Specification

`masterSalt` recommendations:

1. Under the single-story lifetime model, it should be treated as a **locally controlled representation of a stable root context**
2. Length at least 32 bytes
3. Long-term storage in the operating system keychain
4. Plaintext writing to ordinary configuration files is not allowed

Notes:

1. At the current stage, it is not recommended to simply hash the raw story text and directly use it as all keys
2. A more reasonable approach is to first form a stable root context, then use `masterSalt` as a locally controlled root parameter

Identity salt suggestion:

```text
identitySalt = HMAC-SHA256(masterSalt, identityId)
```

## Answer Digest Specification

Challenge answers current stage recommendations:

1. First perform local normalization
2. Then use `identitySalt`
3. Finally compute `HMAC-SHA256(identitySalt, normalizedAnswer)`

## Long-Term Key Storage Principles

Long-term sensitive material includes:

1. `masterSalt`
2. Signing root key
3. Long-term keys required for recovery material packaging

Current stage requirements:

1. Prioritize the operating system keychain
2. Do not directly enter SQLite ordinary tables
3. Do not directly enter `.env` plaintext files

## Key Rotation Strategy (User-Confirmed Version)

User-confirmed: StoryLock adopts a **single-story lifetime** key management strategy. Core principles are as follows:

### Core Principles

1. **Single-story binding**: Each user maintains only one story root context, all keys are derived from this story
2. **Root default stable**: Before high-risk factors appear, do not proactively rotate the story root or masterSalt
3. **Story optimization rather than replacement**: When updates are needed, prefer precise optimization on the original story
4. **Zero user memory burden**: Users only need to remember one story to manage all keys
5. **Routine rotation does not equal root replacement**: Question sets, derived contexts, and object packaging layers can be rotated, but do not default to triggering root-level reconstruction

### Risk Trigger Conditions (satisfy any one to prompt optimization or reconstruction)

| Risk Level | Trigger Condition | Response Method |
|----------|---------|----------|
| High | Detected underlying key brute-force cracking attempts (such as abnormally surging challenge failure counts) | Mandatory prompt, suggest immediate story optimization; if root context is already leaked, enter root-level reconstruction |
| High | User actively reports story may be leaked (such as known by others) | Mandatory prompt, suggest immediate story optimization; enter root-level reconstruction if necessary |
| Medium | Long-term unused (over 12 months) and question set strength assessment declines | Suggestive prompt, recommend story optimization |
| Low | System detects new security standards or algorithm obsolescence | Informational prompt, optional optimization |

### Story Optimization Flow (Non-Replacement)

When routine risks are triggered, execute **incremental optimization** rather than complete replacement:

1. **Retain core story skeleton**: The main content of the user's original story remains unchanged
2. **Add precise details**: Guide users to supplement more specific details (such as time, place, sensory memories)
3. **Expand question set questions**: Generate new questions based on the optimized story, add them to the original 24-question set
4. **Smooth transition**: New question set and old question set exist in parallel for a period, users can choose to answer either question set
5. **Gradual phasing out**: After confirming the new question set is stable, mark the old question set as deprecated, no longer used for new challenges

### Root-Level Reconstruction Boundary

Only enter root-level reconstruction in the following situations:

1. Story root is confirmed leaked
2. Existing story root cannot restore credibility through routine optimization
3. User actively requests complete reconstruction

During root-level reconstruction, the following must be simultaneously reconstructed:

1. Story root context
2. masterSalt
3. identitySalt
4. Question set master file
5. Object packaging layer

Root-level reconstruction is not routine rotation and must not be triggered frequently.

### Rotation Objects Under Single-Story Mode

Under this mode, "rotation" is not by default understood as "changing the story".

Current stage recommendations for handling by the following levels:

1. **Story root context**
   - Default unchanged
   - Only reconstructed during disaster-level risks
2. **Question set version**
   - Can be expanded
   - Can be marked `active` / `deprecated` / `pending`
3. **Derived context version**
   - Can be adjusted via `versionTag`
   - Does not require user awareness
4. **Object packaging layer**
   - Can be re-encrypted
   - Can switch to new objectKey / workKey derived contexts

### Key Hierarchy and Story Relationship

```
User Story (unique, lifetime)
    │
    ├── masterSalt (derived from story, fixed and unchanged)
    │       └── Derive each layer key through HKDF
    │
    ├── Question set master file (24 questions, expandable to 24+N questions)
    │       └── When new questions are added, old questions are marked deprecated
    │
    └── identitySalt (derived by identity, fixed and unchanged)
            └── Used for answer digest and identity isolation
```

### Relationship with Original Key Hierarchy

The original four-layer key hierarchy (masterSalt → rootKey → workKey → objectKey) remains unchanged, but:

- **masterSalt**: Derived from user story once, lifetime unchanged
- **rootKey / workKey / objectKey**: Derived on demand through HKDF, derivation parameters contain version identifiers
- **Version identifiers**: Only used to distinguish derivation contexts of different optimization stages of the same story, do not force user awareness

### Implementation Constraints

1. **masterSalt in SecretStore must not be deleted**: As a lifetime root key, it must be permanently retained
2. **Question set master file supports version marking**: Each question is marked with `active` / `deprecated` / `pending` status
3. **Challenge creation prioritizes active questions**: Ensures new challenges do not use deprecated questions
4. **Answer verification is compatible with multiple versions**: Historical answer digests of the same identity are stored in isolation by version
5. **Object packaging supports version marking**: Question set master files and secret objects should have packaging versions or derived context marks
6. **Root-level reconstruction belongs to disaster recovery**: Must not be triggered frequently as a routine rotation path
7. **Compatibility windows must be explicitly configured**: Old question set digests, old object ciphertexts, and old sessions should all have explicit expiration times

### User Interaction Design

- **Normal situation**: Users are completely unaware, only need to remember one story
- **Risk prompt**: Prompt copy emphasizes "optimize story" rather than "replace story", reducing user anxiety
- **Optimization guidance**: Guide users to supplement details through conversational guidance, rather than requiring re-conception of a new story

## Relationship with Existing Documents

This specification is used in conjunction with the following documents:

1. `challenge_answer_storage.md`
2. `session_and_replay_protection.md`
3. `proxy_signature_protocol.md`
4. `local_agent_gateway.md`
5. `eip712_minimal_request.md`

## Conclusion

At the current stage, the most important thing is not to spread security functions very wide, but to first get the following five things right:

1. Key layering
2. Question set encryption
3. Nonce uniqueness
4. Answer digest salt isolation
5. Long-term sensitive material not on ordinary disk plaintext

**Supplementary Principles (Single-Story Lifetime)**:

6. Users only need to remember one story, all keys are derived from this story
7. Do not proactively rotate, only prompt story optimization when risks are triggered
8. Story optimization is an incremental process, not a complete replacement
9. Question sets can be expanded, old questions are gradually phased out rather than immediately deleted
