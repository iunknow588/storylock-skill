# StoryLock Object Access Policy

## Purpose

This document defines how StoryLock should classify objects, determine access strength, and decide whether access can be reused remotely.

It complements:

1. `storylock_skill_positioning_and_boundary.md`
2. `src/storylock-skill-engine/SKILL.md`

## Policy Goal

The local runtime should be able to answer:

1. what object is being accessed
2. how much challenge strength is required
3. whether access is one-time or reusable
4. whether a remote agent may retain any capability after access

## Object Classes

| Class | Example | Sensitivity | Default Runtime |
| --- | --- | --- | --- |
| `story_public` | generic story template, non-private draft | low | remote-safe |
| `story_private` | user memory, personal narrative, private draft | medium to high | local-first |
| `auth_login` | username/password field package | high | local-only |
| `auth_signing` | signing key or signing authority material | very high | local-only |
| `review_data` | 24-question set and readiness metadata | medium | local or remote depending on contents |

## Access Strength Levels

The system can expose multiple access thresholds.

| Level | Example Threshold | Typical Use |
| --- | --- | --- |
| `L1` | weak or no challenge | public templates, low-risk metadata |
| `L2` | 6-of-24 | low-risk read access or limited review data |
| `L3` | 12-of-24 | moderate secret access or short-lived field retrieval |
| `L4` | 22-of-24 | signing-related or other high-trust operations |
| `L5` | host-determined maximum | root-level or especially sensitive operations |

The exact mapping is policy-driven. The current StoryLock constants already imply:

1. basic read threshold: `6`
2. batch read threshold: `12`
3. modify threshold: `22`

## Session Model

Access should be explicit about duration and reuse.

| Session Type | Lifetime | Reads | Reuse | Remote Retention |
| --- | --- | --- | --- | --- |
| `one_shot` | single call | 1 | no | no |
| `short_session` | short TTL | limited | limited | only as opaque handle if allowed |
| `batch_session` | medium TTL | multiple | limited | only if policy permits |
| `privileged_session` | strict TTL | minimal | no | never |

Current implementation constants that can inform policy:

1. challenge TTL
2. session TTL
3. basic session reads
4. batch session reads

## Remote Capability Retention Rule

Remote agents should not retain long-lived secret access.

Allowed to retain:

1. structured results
2. redacted summaries
3. opaque short-lived handles when explicitly approved

Not allowed to retain:

1. raw passwords
2. private keys
3. challenge answers
4. reusable secret-read tokens with long lifetime

## Access Decision Table

| Object Class | Strength | Session Type | Can Remote Cache Result | Can Remote Keep Access Ability |
| --- | --- | --- | --- | --- |
| `story_public` | L1 | one_shot | yes | yes |
| `story_private` | L2-L3 | short_session | redacted only | no |
| `auth_login` | L3 | short_session | no | no |
| `auth_signing` | L4-L5 | privileged_session | no | no |
| `review_data` | L2-L3 | short_session or batch_session | yes, if sanitized | no |

## Operational Rule

If access requires:

1. challenge answers
2. session material
3. secret-object reads
4. signing-key handling

then the access must happen in the local runtime.

## Recommended Implementation Shape

The local runtime should expose a small policy gateway that returns:

1. required threshold
2. approved session type
3. read budget
4. cacheability flag
5. redaction requirement

The remote agent should consume only:

1. capability name
2. result payload
3. redacted metadata
4. opaque short-lived handles, if explicitly permitted

## Conclusion

The correct model is not "everything can be reused everywhere".

The correct model is:

1. access is classified
2. sessions are bounded
3. remote reuse is restricted
4. sensitive capability supply should not be preserved long-term on the remote side
