# StoryLock Story Skill Boundary Analysis

## Core Question

Which parts of StoryLock can be exposed as a Skill package, and which parts must remain local or host-controlled?

## Scope

This note is aligned to the current `storylock-skill-engine` package, not just the older story-generation concept.

Current package-facing capabilities:

1. story draft assistance
2. strength review
3. password-fill authorization packaging
4. challenge-sign authorization packaging

## Capability Split

### Safe to package as Skill

1. **Story draft assistance**
   - template-driven generation
   - refinement of already prepared text
   - structured prompt-to-output mapping

2. **Strength review**
   - question-set validation
   - readiness scoring
   - recommendation generation

3. **Authorization packaging**
   - login field packaging
   - challenge-sign packaging
   - structured outputs for downstream use

### Not safe to expose as remote Skill input

1. **Raw secrets**
   - passwords
   - private keys
   - session credentials
   - challenge answers before authorization

2. **Host-trusted operations**
   - challenge creation
   - challenge answer submission
   - secret-object reads
   - signing-key extraction

## Boundary Principle

The package should be self-describing, but the secrets it operates on must stay behind a separate trust boundary.

That means:

1. the Skill can describe the interface
2. the Skill can validate inputs and outputs
3. the Skill can orchestrate access
4. the Skill must not redefine the trust boundary itself

## Storage vs Skill Boundary

Storage location does not change the Skill interface, but it changes the security model.

| Storage | Role | Note |
| --- | --- | --- |
| local | private secret store | easiest to control |
| cloud | managed secret store | needs access control and audit |
| contract | public or semi-public storage | best for proofs, references, or encrypted blobs, not plaintext secrets |

So the external file location can vary, but the access flow still needs the same two layers:

1. storage access
2. StoryLock authorization

## Current Judgment

The current package is correctly moving toward a self-describing Skill.

What still needs care:

1. do not treat docs-only behavior as equivalent to runtime behavior
2. do not assume cloud or contract storage is the same as local storage
3. do not expose raw sensitive material in agent-facing instructions

## Recommended Boundary Rule

Use this rule:

> storage decides where the encrypted object lives; StoryLock decides whether the object may be used.

## Conclusion

The package should remain self-describing, but the actual security boundary must stay outside the Skill contract.
