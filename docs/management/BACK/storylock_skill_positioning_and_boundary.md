# StoryLock Skill Positioning And Boundary

## Purpose

This document defines:

1. what Skills exist in the current StoryLock system
2. what each Skill is responsible for
3. where each Skill should run
4. what belongs to the Skill layer, host layer, and storage layer
5. how multiple Skills are assembled into a complete feature

This document is aligned to the current package:

1. `src/engine/`

## Design Principles

Use the following principles when defining StoryLock Skills:

1. define Skills by capability, not by storage location
2. separate Skill contract from security boundary
3. keep sensitive secret access behind a host-controlled interface
4. keep remote-safe text processing separate from local secret handling
5. treat orchestration as a layer above individual Skills

## Boundary Model

StoryLock should be split into three layers:

### 1. Skill Layer

Responsible for:

1. self-description
2. input and output contract
3. execution guidance
4. orchestration-friendly structured results

Not responsible for:

1. defining the trust boundary by itself
2. storing plaintext secrets
3. replacing host-side authorization controls

### 2. Host Layer

Responsible for:

1. `createChallenge(identityId, scope)`
2. `submitChallengeAnswers(identityId, challengeId, answers)`
3. `readSecretObject(identityId, sessionId, secretObjectId)`
4. signer injection and signing-key handling

This is the actual sensitive execution boundary.

### 3. Storage Layer

Responsible for:

1. holding encrypted objects
2. providing location and access control
3. supporting local, cloud, or contract-based storage

Storage location does not change the Skill interface, but it changes the security model.

## System Skill Table

| Skill Name | Main Class | Purpose | Sensitive Data Exposure | Recommended Runtime | Requires Host | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Story Draft | `StoryDraftAssistSkill` | Generate or refine story drafts from structured prompts | low to medium, depends on input | remote-safe for generic prompts; local for private-memory prompts | no | capability-level text generation |
| Story Refine | `StoryRefineAssistSkill` | Refine an existing story draft toward explicit goals | low to medium, depends on draft contents | remote-safe for sanitized text; local for private drafts | no | same capability family as Story Draft |
| Strength Review | `StrengthReviewSkill` | Evaluate StoryLock question-set readiness and return recommendations | medium | local or remote, depending on data sensitivity | no | deterministic analysis and recommendation output |
| Login Authorization | `LoginAuthorizationSkill` | Create challenge, submit answers, and return login authorization package | high | local-only | yes | internal authorization capability |
| Password Fill | `LocalPasswordFillSkill` | Produce site login fields after authorization | high | local-only | yes | scenario-facing wrapper above login authorization |
| Signing Authorization | `SigningAuthorizationSkill` | Create signing authorization package and optional key material access | high | local-only | yes | internal signing capability |
| Challenge Sign | `ChallengeSigningAuthorizationSkill` | Produce challenge-signing result after authorization | high | local-only | yes | scenario-facing wrapper above signing authorization |
| Agent Orchestration Demo | `VideoPublishAgentDemo` | Chain login and signing results into a publish-plan style flow | mixed | hybrid | yes | orchestration example, not a core Skill |

## Skill Classification

The current system can be grouped into three types of Skills.

### A. Remote-safe Skills

These Skills can run remotely when their inputs are already safe:

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. `StrengthReviewSkill`

Condition:

1. input must not contain raw secrets or protected private-memory material unless the runtime is trusted

### B. Local-only Skills

These Skills must remain behind a local or host-controlled trust boundary:

1. `LoginAuthorizationSkill`
2. `LocalPasswordFillSkill`
3. `SigningAuthorizationSkill`
4. `ChallengeSigningAuthorizationSkill`

Reason:

1. they directly depend on challenge flow, session flow, secret-object reads, or key handling

### C. Hybrid Orchestration Skills

These flows mix safe remote planning with local sensitive execution:

1. `VideoPublishAgentDemo`
2. future story flows that combine private-memory access with remote text polishing

## Recommended Agent Split

The system should be described with two cooperating agents or runtimes.

### Remote Agent

Responsible for:

1. routing user intent
2. selecting the correct Skill
3. preparing structured requests
4. packaging local results into user-facing output
5. handling remote-safe text operations

Should not do:

1. hold raw secrets as a trusted long-term store
2. bypass host authorization APIs
3. perform local-only secret reads directly

### Local Agent Or Local Host Runtime

Responsible for:

1. challenge creation
2. answer submission
3. secret-object reads
4. signing-key use
5. local-only execution of password-fill and challenge-sign flows

Should return:

1. structured authorization results
2. structured field packages
3. structured signing results

### Agent Relationship

Recommended relationship:

1. remote agent orchestrates
2. local agent or local host executes sensitive steps
3. remote agent wraps the local result into the final feature output

## Functional Assembly

Individual Skills should be assembled into features as follows.

### Feature 1: Story Draft Workflow

Possible composition:

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. optional local private-memory preprocessing

Assembly logic:

1. prepare prompt context
2. draft story
3. refine story against goals
4. optionally run another local privacy review before export

### Feature 2: StoryLock Readiness Review

Composition:

1. `StrengthReviewSkill`

Assembly logic:

1. collect 24-question set
2. evaluate readiness
3. return issues and recommendations

### Feature 3: Password Fill Workflow

Composition:

1. `LoginAuthorizationSkill`
2. `LocalPasswordFillSkill`

Assembly logic:

1. resolve site bindings
2. create challenge
3. submit answers
4. obtain session
5. read authorized secrets
6. package login fields

### Feature 4: Challenge Sign Workflow

Composition:

1. `SigningAuthorizationSkill`
2. `ChallengeSigningAuthorizationSkill`

Assembly logic:

1. resolve signing secret reference
2. create challenge
3. submit answers
4. obtain session
5. read signing material
6. invoke signer
7. return structured signing result

### Feature 5: Multi-step Agent Workflow

Composition:

1. `LocalPasswordFillSkill`
2. `ChallengeSigningAuthorizationSkill`
3. `VideoPublishAgentDemo`

Assembly logic:

1. remote agent selects workflow
2. local runtime executes sensitive Skills
3. orchestration layer combines outputs into a publish plan

## Practical Boundary Rule

Use this rule in design reviews:

1. if a capability needs challenge answers, session material, secret reads, or signing-key access, it is local-only
2. if a capability only transforms already-safe text or analyzes non-secret structured data, it may be remote-safe
3. if a feature mixes both, split it into a remote orchestration part and a local execution part

## Suggested Next Step

The next design step should be to formalize:

1. which requests are routed to the remote agent first
2. which requests must be handed to the local runtime
3. the structured handoff contract between them
4. `storylock_object_access_policy.md` for thresholds, sessions, and remote retention rules
