# StoryLock Design Document Optimization Suggestions

This document is a converged version of historical organization suggestions, only retaining content that is still valuable to the current code. The current implementation baseline is based on the following documents:

1. `readme.md`
2. `system_skill_table.md`
3. `three_package_contract.md`
4. `local_agent_gateway.md`
5. `session_and_replay_protection.md`

## Completed Convergence

1. The three-layer mainline has converged to story processing, local access authorization, and remote gateway packages.
2. Layer 2 has converged from "story reading/writing" to "object strength, grid challenge verification, and local authorization".
3. Layer 3 has converged from `requestChallengeSign` to the more neutral `requestSignature`.
4. Layer 3 currently only retains `requestSignature` and `requestPasswordFill` as main interfaces.
5. Signature audit and authorization status have been written to SQLite.

## Still Valuable Optimization Directions

### 1. Continue Unifying Terminology

It is recommended that all mainline documents uniformly use:

| Recommended Terminology | No Longer as Mainline Terminology |
| --- | --- |
| `requestSignature` | `requestChallengeSign` |
| `SignatureAuthorizationSkill` | `ChallengeSigningAuthorizationSkill` |
| Local access authorization | Local story access |
| Object strength policy | Story reading strength |
| Grid challenge verification | Story reading challenge |

### 2. Documents Continuously Calibrated by Code

After each code change, at least check:

1. `three_package_contract.md`
2. `system_skill_table.md`
3. `local_agent_gateway.md`
4. `session_and_replay_protection.md`
5. `DEVELOPMENT_COMPLETION_PLAN_20260617.md`

### 3. Maintain Restrained Function Descriptions

Document descriptions should distinguish three types of states:

1. **Implemented**: Already exists in code and has self-test coverage.
2. **Planned improvement**: Already has interfaces or prototypes, but still needs to complete production constraints.
3. **Subsequent exploration**: Only application directions, not written as current capabilities.

Especially multi-account transactions, multi-platform publishing, multi-chain wallets, and other directions can only be used as "typical scenarios for application domain exploration", and cannot be written as currently supported.

### 4. Continue Supplementing Acceptance Criteria

When adding new capabilities subsequently, it is recommended to clearly write for each item:

1. Which package is involved.
2. Whether it changes external interfaces.
3. Whether it involves local sensitive material.
4. Whether SQLite needs migration.
5. What is the self-test command.

## Historical Suggestions No Longer Adopted

The following suggestions have become invalid and are no longer used as subsequent development basis:

1. Add story reading and story writing as Layer 2 main interfaces.
2. Retain `requestStoryRead` and `requestStoryWrite` in Layer 3.
3. Do not use deprecated old interface `requestChallengeSign` as the main signature interface.
4. Describe the remote gateway as being able to directly read local stories or secret objects.
5. Place Pharos Skill Engine within the local security access boundary.

## Current Priorities

1. Complete end-to-end demonstration: remote request, Layer 2 authorization, local signature or password fill, Layer 3 redacted return.
2. Clarify Node.js 22 runtime requirements.
3. Expand grid challenge question sources and strength strategies, rather than only using placeholder data.
4. Maintain stable documentation for SQLite table structures and error codes.
5. Reserve implementation for production environment SecretStore access to system keychain.
