# StoryLock Product Brief

## 1. Project Overview

| Item | Details |
|------|---------|
| Product Name | StoryLock |
| Applicable Scenario | Local private-key management and authorization control for Agent scenarios |
| Positioning | A Skill solution for local private-key management and authorization control in Agent scenarios |
| Target Users | Individual developers, small teams, Agent / Skill developers |
| Repository Root | `skill/` |
| Document Version | 2026-06-17 |

---

## 2. Summary

StoryLock is a local private-key and sensitive-object authorization management tool for Agent scenarios. The local Agent manages private keys and file keys, while the remote online Agent does not directly hold user private keys. File contents are encrypted with keys, and only the local Agent with the corresponding file key and permission level can access them. StoryLock introduces associative-memory cues into authorization decisions and packages "permission-gated local sensitive-object access" and "human-understandable, recallable cues for authorization" as reusable Skill capabilities.

The project is organized into three capability layers:

1. Layer 1: Local story processing  
   Handles local processing logic such as story draft generation, story refinement, and question-set strength evaluation.
2. Layer 2: Local controlled authorization  
   Determines the required password strength according to the target object and generates the corresponding grid verification. Whether an object requires low-, medium-, or high-strength verification is part of the object-level access policy defined and executed by this layer.
3. Layer 3: Remote gateway and delegated authorization  
   Wraps external calls into a unified request structure and exposes local capabilities such as signature authorization and password filling to remote Agents in a controlled way. The remote side submits structured requests; the local side completes permission checks, local authorization, signing, or password filling, and then returns minimized results. Through this layer, remote Agents can use local capabilities without directly holding user private keys or file keys.

In one sentence:

> StoryLock is an Agent security capability package in which a local Agent manages private keys and file keys, associative-memory cues participate in authorization decisions, and three coordinated Skill layers allow remote Agents to use local capabilities safely.

---

## 3. Code Structure

### 3.1 Three-Package Structure

| Package | Directory | Role |
|---------|-----------|------|
| `storylock-local-story-processing-skill` | `skill/src/storylock-local-story-processing-skill` | Layer 1, local story processing |
| `storylock-local-story-access-skill` | `skill/src/storylock-local-story-access-skill` | Layer 2, local controlled authorization |
| `storylock-remote-gateway-skill` | `skill/src/storylock-remote-gateway-skill` | Layer 3, remote request wrapping and gateway |

### 3.2 Aggregation Entry

| Module | Directory | Description |
|--------|-----------|-------------|
| `storylock-skill-engine` | `skill/src/storylock-skill-engine` | Unified exports, demo scripts, self-test scripts, and WASM build scripts |
| `shared` | `skill/src/shared` | Shared encryption, SQLite, and SecretStore adapter code |

---

## 4. Core Capabilities

### 4.1 Layer 1: Local Story Processing

Skills:

- `StoryDraftAssistSkill`
- `StoryRefineAssistSkill`
- `StrengthReviewSkill`

Code:

- `skill/src/storylock-local-story-processing-skill/index.js`
- `skill/src/storylock-skill-engine/assets/migrated/skills/story-assist.js`
- `skill/src/storylock-skill-engine/assets/migrated/skills/strength-review.js`

Capabilities:

- Story draft generation
- Story refinement
- Question-set strength evaluation

### 4.2 Layer 2: Local Controlled Authorization

Skills:

- `ObjectStrengthPolicySkill`
- `GridChallengeSkill`
- `LocalAuthorizationSkill`

Code:

- `skill/src/storylock-local-story-access-skill/index.js`
- `skill/src/storylock-local-story-access-skill/access-host.js`

Capabilities:

- Unified request validation
- Object-level strength policy that determines the required password strength according to the target object
- Grid verification generation according to the required password strength
- `requestId` idempotency and `nonce` replay protection
- Local verification creation and answer validation
- Short-lived session issuance
- Authorization result return

### 4.3 Layer 3: Remote Gateway

Interfaces:

- `requestSignature`
- `requestPasswordFill`

Code:

- `skill/src/storylock-remote-gateway-skill/index.js`

Capabilities:

- Unified request envelope structure
- Unified capability naming and scope
- Remote call wrapping
- Signature authorization request wrapping
- Web2 login-form password filling request wrapping
- EIP-712 minimal signing request packaging

Layer 3 is the unified delegated authorization entry for remote Agents to use local capabilities. It transforms remote requests into structured, verifiable, and auditable local calls, and returns minimized execution results.

- Remote Agents initiate authorization requests through Layer 3 without directly touching private keys
- The local Agent performs permission checks, local authorization, signing, or password filling through Layer 3
- Layer 3 separates "whether a request may be made" from "how it is executed locally"
- Returned results follow the minimization principle to avoid leaking sensitive content to the remote side

---

## 5. Security Mechanisms

### 5.1 Security Capabilities

| Capability | Description |
|------------|-------------|
| Object-level password strength and grid verification | Determines the required password strength according to the target object and generates the corresponding grid verification |
| Replay protection | `requestId` idempotency, `nonce` deduplication, and `expiry` validation |
| Session model | Controls the access window through short-lived sessions |
| Auto lock | Enters `locked` after repeated failures |
| Auto unlock | Restores availability after the lock window expires |
| Object encryption | AES-256-GCM |
| Key derivation | HKDF-SHA256 |
| Answer digest | HMAC-SHA256 digest storage |

Additional notes:

- Private keys and file keys are managed by the local Agent
- Remote Agents only request capabilities and do not directly hold private keys
- Authorization uses associative-memory cues instead of relying only on mechanical passphrases

### 5.2 Capability Scope

Primary capabilities include:

1. A unified three-layer Skill structure
2. Object-level password strength policy, grid verification, and short-lived sessions
3. Replay protection, local answer validation, and authorization result return
4. A controlled chain from remote request wrapping to local execution
5. Local controlled execution for signature authorization and Web2 password filling requests

Extension capabilities include:

- Question-set models and progressive grid verification strategies
- Finer-grained object access classification and policy engines
- Cross-platform SecretStore adaptation
- Local signing execution, auditing, and recovery mechanisms
- Standardized HTTP / host integration layers

This document reflects the actual code, interfaces, and scripts in the repository.

---

## 6. Design and Implementation Mapping

Based on the current code and `skill/docs/management/code_doc_consistency_review.md`, this section explains where the current design lands in the codebase and the main directions for future extension.

### 6.1 Current Implementation Mapping

- Three-package directory structure and responsibility layering
- Layer 2 object-strength decision and grid verification generation flow
- Automatic recovery from the `locked` state
- Reference to the EIP-712 standard to define the project-level `StoryLockSignatureRequest` request structure
- Remote gateway focused on signature authorization and password filling request wrapping
- Error-code mapping fix from `REQUEST_EXPIRED` to `SLG-011`
- More robust atomic update strategy for local authorization state updates

### 6.2 Further Extension Directions

- Continue improving the underlying mechanisms for local authorization, permission control, and delegated execution
- Explore typical application scenarios across different application domains to validate generality and portability
- Gradually accumulate reusable integration patterns and engineering practices based on concrete business needs

---

## 7. Run and Verification

### 7.1 Skill Engine Demo

Directory:

`skill/src/storylock-skill-engine`

Command:

```powershell
cd skill/src/storylock-skill-engine
npm run demo
npm run selftest
npm run build:wasm
npm run selftest:wasm
```

Coverage:

- `demo`: example flow for draft generation, password filling, and signature authorization
- `selftest`: verifies core exported capabilities can be invoked
- `build:wasm`: verifies the Rust/WASM artifact build flow
- `selftest:wasm`: verifies that WASM dist artifacts can be loaded

### 7.2 Local Controlled Authorization Self-Test

Directory:

`skill/src/storylock-local-story-access-skill`

Command:

```powershell
cd skill/src/storylock-local-story-access-skill
node scripts/selftest.mjs
```

Coverage:

- Object strength decision
- Grid verification generation
- Replay protection
- Idempotent request handling
- Locking and automatic unlock
- Expired-request error code
- Local authorization result return

### 7.3 Remote Gateway Self-Test

Directory:

`skill/src/storylock-remote-gateway-skill`

Command:

```powershell
cd skill/src/storylock-remote-gateway-skill
node scripts/selftest.mjs
```

Coverage:

- Remote request wrapping
- `requestSignature`
- `requestPasswordFill`
- EIP-712 structure
- Default `policyHints`

---

## 8. Product Characteristics

### 8.1 Characteristic One: Not Only Storing Secrets, but Also Controlling Their Use

Many tools mainly solve where secrets are stored. StoryLock further focuses on the boundary of secret usage:

- who can use them
- when they can be used
- how long they can be used
- what can be returned
- what the remote caller can receive at most

This makes it more than a storage tool. It is a local authorization and key-management framework for Agent scenarios.

### 8.1.1 Associative Memory in Authorization

StoryLock emphasizes associative-memory cues in authorization decisions and does not reduce authorization to mechanical fixed-password input.

This design is closer to everyday usage patterns:

- People more easily recall information tied to their own experiences, stories, and cues
- The local Agent keeps keys and enforces permission control
- The remote Agent initiates requests without directly touching private keys

This design combines local key management with authorization decisions based on understandable cues. It is better suited for long-term use and for Agent participation without privilege overreach.

### 8.2 Characteristic Two: Clear Three-Layer Structure with Explicit Responsibilities

The project is not a single script. It is split into:

- a local processing package
- a local authorization package
- a remote gateway package

The three layers have clear and coordinated responsibilities:

- Layer 1 handles local processing and memory-cue-related capabilities
- Layer 2 handles object-strength decisions, grid verification, and local authorization
- Layer 3 handles remote request wrapping and delegated authorization for signature authorization and password filling

This structure separates processing, authorization, and delegated execution, allowing the system to extend capabilities while keeping clear boundaries and supporting safer application building.

### 8.3 Characteristic Three: Documentation, Code, Self-Tests, and Example Scripts

The repository includes:

- `SKILL.md`
- `references/`
- `assets/schemas/`
- `assets/templates/`
- `demo/selftest/build` scripts

The repository provides documentation, code, self-tests, and example scripts to support understanding, verification, and reuse.

### 8.4 Characteristic Four: Unified Secure Execution Architecture Built with Rust and Pharos

The current architecture uses Rust and Pharos as technical anchors for the lower and upper layers. The repository already includes:

- the unified `storylock-skill-engine` entry
- `dist/wasm` build artifacts
- WASM build and artifact self-test scripts

Within this architecture:

- Rust / WASM is used at the lower layer to improve the security and stability of critical execution paths
- Pharos is used at the upper layer to adapt to different chains, signing flows, and host environments
- StoryLock forms a unified secure execution framework across local authorization, key management, and delegated execution

---

## 9. Business and Application Scenarios

### 9.1 Typical Application Scenarios

| Scenario | StoryLock Role |
|----------|----------------|
| Strategy-exploration automated trading | Explore an application pattern in which the remote Agent initiates strategy requests while the local Agent controls account keys, signing permissions, and operation strength by tier |
| Automated content generation and publishing | Explore an application pattern in which the remote Agent orchestrates content workflows while the local Agent controls platform credentials, material usage, and publishing permissions |
| Multi-account operations | Explore local credential management and high-sensitivity operation authorization across multiple sites, accounts, and publishing channels |
| Multi-object delegated execution | Explore local authorization, signing, and access control when one workflow involves multiple wallets, cloud environments, website accounts, or credential objects |

---

## 10. Repository Contents

| Type | Path | Description |
|------|------|-------------|
| Chinese product brief | `skill/docs/usecase/00-参赛说明文档.md` | Product description |
| English product brief | `skill/docs/usecase/00-submission-brief-en.md` | Product description |
| Consistency review document | `skill/docs/management/code_doc_consistency_review.md` | Code and design document comparison review |
| Layer 1 code | `skill/src/storylock-local-story-processing-skill` | Local story processing |
| Layer 2 code | `skill/src/storylock-local-story-access-skill` | Local controlled authorization |
| Layer 3 code | `skill/src/storylock-remote-gateway-skill` | Remote gateway |
| Aggregation entry | `skill/src/storylock-skill-engine` | Examples, self-tests, and WASM build |
| Shared modules | `skill/src/shared` | Encryption, SQLite, SecretStore |

---

## 11. Extension Directions

### 11.1 Short Term

- Connect the complete processing flow across the gateway layer, authorization layer, and processing layer
- Clarify the chain among remote requests, local authorization, local processing, and result return
- Align demos, self-tests, and documentation around the same primary flow

### 11.2 Mid Term

- Form more stable capability interfaces and request structures
- Improve standardized expression of object access classification, strength policy, and permission control
- Advance standardized adaptation for SecretStore, signing execution, and host integration interfaces

### 11.3 Long Term

- Explore typical application scenarios across different application domains
- Reuse local authorization, key management, and delegated execution capabilities in concrete scenarios
- Gradually accumulate extensible Agent security application patterns

---

## 12. Summary

StoryLock can be summarized as:

1. a three-layer coordinated Skill solution;
2. an Agent security solution in which a local Agent manages private keys and file keys, associative-memory cues participate in authorization decisions, and authorization is executed by permission level;
3. a verifiable project with code, documentation, example scripts, and self-test scripts.

The three Skill layers take on distinct responsibilities:

- Layer 1 handles local processing, story organization, and memory-cue-related capabilities
- Layer 2 determines the required password strength according to the target object, generates the corresponding grid verification under object-level policy, and returns the local authorization result
- Layer 3 handles remote request wrapping, delegated authorization, and minimized result returns

The project operates within the following boundaries:

- The local Agent manages private keys and file keys
- The remote Agent only requests capabilities and does not directly hold private keys
- Authorization uses associative-memory cues rather than relying only on mechanical passphrases
- Sensitive object access follows permission levels, short-lived sessions, and minimized return principles
