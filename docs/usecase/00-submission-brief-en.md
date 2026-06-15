# StoryLock Submission Brief (DoraHacks Pharos Phase 1)

## 1. Project Overview

| Item | Details |
|------|---------|
| **Project Name** | StoryLock |
| **Track** | Skill Hackathon (Phase 1) |
| **Positioning** | Local Authorization Kernel Skill Package |
| **Target Users** | Individual founders, lean teams, Agent / Skill developers |
| **Core Functions** | Story assistance, question-set strength review, local login filling, challenge signing |
| **Current Form** | JS skill-layer + migrated package resources + Rust/WASM build and validation |
| **Submission Date** | 2026-06-15 |

---

## 2. Summary

StoryLock is a local authorization kernel Skill package for individual founders and Agent scenarios. It stores high-value credentials such as login passwords, API keys, and signing private keys in a local encrypted vault. When external tools, Agents, or host applications need access, the user completes local verification and receives a short-lived, scope-limited read result.

**Key Differentiation:**

- Not just another password manager
- Not handing long-term master credentials directly to Agents
- Instead, packaging "local verification + short-lived session + scope-limited read" into a reusable Skill package

Why it fits the Skill track:

- It is not a single demo, but a reusable interface package with a clear capability boundary
- It already has a standardized migration structure: `SKILL.md + references + assets + runnable scripts`
- It can be explained, invoked, demonstrated, and verified

---

## 3. Architecture

### 3.1 Current Implementation Architecture

| Layer | Responsibility | Current Implementation |
|-------|--------------|------------------------|
| **Skill Package** | Unified entry, capability index, reference docs | `skill/src/storylock-skill-engine/SKILL.md` |
| **JS Skill-Layer** | Story assistance, login filling, challenge signing, strength review | `skill/src/storylock-skill-engine/index.js` + `assets/migrated/` |
| **Package Resources** | Migrated code, reference docs, minimal schema/template | `assets/migrated/`, `references/`, `assets/schemas/`, `assets/templates/` |
| **Rust/WASM Build Layer** | Build and dist validation, not the current JS main runtime path | `scripts/build-wasm.mjs`, `scripts/selftest-wasm-dist.mjs`, `dist/wasm/` |

### 3.2 Current Boundary

| Item | Status |
|------|--------|
| JS skill-layer | Independently runnable |
| Demo / self-test | Independently runnable |
| Rust/WASM build | Build and artifact validation wired in |
| Full WASM host integration | Not finished yet |
| Cloud backup module | Not a main delivered line in this round |

### 3.3 Core Security Mechanisms

| Mechanism | Description |
|-----------|-------------|
| **Tiered Authorization Model** | Normal read `6-of-24` / High privilege `12-of-24` / Recovery `22-of-24` |
| **3x3 Grid Multi-Select** | 9 candidates per question, 2–4 correct answers |
| **Short-Lived Session** | Time-bound session established after authorization |
| **Per-Object Read** | Each secret object is authorized independently |

---

## 4. Implemented Skill Capabilities

### 4.1 Seven Completed Skills

| Skill Name | Skill ID | Function |
|-----------|----------|----------|
| `StoryDraftAssistSkill` | `story_draft_assist` | Story draft generation assistance |
| `StoryRefineAssistSkill` | `story_refine_assist` | Story refinement and prompt organization |
| `StrengthReviewSkill` | `strength_review` | Question set strength evaluation |
| `LoginAuthorizationSkill` | `login_authorization` | Login authorization encapsulation |
| `LocalPasswordFillSkill` | `local_password_fill` | Output login fields after local authorization |
| `SigningAuthorizationSkill` | `signing_authorization` | Compatible signing authorization encapsulation |
| `ChallengeSigningAuthorizationSkill` | `challenge_signing_authorization` | Output signing result directly after local authorization |

### 4.2 Recommended Core Skills for Demo

| Skill | Reason for Recommendation |
|-------|---------------------------|
| `LocalPasswordFillSkill` | Most intuitive input/output; easy to explain login filling |
| `ChallengeSigningAuthorizationSkill` | Clear boundary; easy to explain Agent signing scenarios |

---

## 5. Demo & Invocation

### 5.1 Package-Local Demo

```powershell
cd E:\2026OPC大赛\skill\src\storylock-skill-engine
npm run demo
npm run selftest
npm run build:wasm
npm run selftest:wasm
```

### 5.2 Code Invocation Example

```js
import {
  LocalPasswordFillSkill,
  ChallengeSigningAuthorizationSkill,
} from "./index.js";
```

### 5.3 Current Verifiability

| Verification Item | Command | Meaning |
|------|------|---------|
| JS skill-layer demo | `npm run demo` | Verifies the migrated package can run independently |
| JS skill-layer self-test | `npm run selftest` | Verifies the core skills can be called independently |
| WASM build | `npm run build:wasm` | Verifies Rust/WASM artifacts can be generated |
| WASM dist self-test | `npm run selftest:wasm` | Verifies generated artifacts can load and key exports exist |

---

## 6. Innovation

### 6.1 Core Innovations

| Innovation | Description |
|------------|-------------|
| **Secret Storage & Access Separation** | Solves not only "where to store" but "who can actually read locally" |
| **Story-Structured Memory Entry** | Replaces random strings with story structures |
| **Short-Lived, Scope-Limited Authorization** | Agent does not hold master credentials by default |
| **3x3 Grid Replaces Fixed Password** | Shifts authorization from fixed password input to a selection set |

---

## 7. Business Value

### 7.1 Target Markets

1. Individual founders and one-person companies
2. Small SaaS and tool teams
3. Agent / automation tool developers

### 7.2 Typical Scenarios

| Scenario | Model |
|------|------|
| Local authorization SDK / component licensing | Embedded authorization capability for browser / desktop tools |
| Professional template packs | Login site templates, signing action templates |
| Agent security plugin marketplace | Local authorization plugin for third-party Agents |

---

## 8. Roadmap

### 8.1 Current Completed Items

- [x] Project summary and positioning
- [x] Technical architecture and functional value
- [x] Innovation and business value analysis
- [x] Project planning and delivery roadmap
- [x] Seven Skill classes fully implemented
- [x] Migrated package demo and self-test
- [x] Rust/WASM build and artifact validation

### 8.2 Next Phases

| Phase | Goal |
|------|------|
| **Phase 1** | Documentation consistency, stable demo scenarios, stable self-test |
| **Phase 2** | Complete WASM host integration and fuller runtime integration |
| **Phase 3** | Expand more skill templates and host integration docs |

---

## 9. Application Showcase

### 9.1 Ready-to-Present Artifacts

| Artifact | Path | Proof Point |
|----------|------|-------------|
| Skill package entry | `skill/src/storylock-skill-engine/SKILL.md` | Unified capability index and boundary description |
| Local demo | `skill/src/storylock-skill-engine/scripts/demo.mjs` | JS skill-layer can run |
| Local self-test | `skill/src/storylock-skill-engine/scripts/selftest.mjs` | Core skills can be verified |
| WASM build validation | `skill/src/storylock-skill-engine/scripts/build-wasm.mjs` | Rust/WASM artifacts can be built |
| WASM dist self-test | `skill/src/storylock-skill-engine/scripts/selftest-wasm-dist.mjs` | Artifacts can be loaded and exports exist |

---

## 10. Competition Positioning

Core alignment with the track:

1. For individual founders and Agent scenarios
2. Turns local authorization, short-lived sessions, and scope-limited reads into a reusable Skill package
3. Already has a runnable, testable, and presentable JS skill-layer migration package
4. Rust/WASM already has build and validation entry points, but full host integration is not finished yet
5. The current version is already verifiable while leaving room for deeper Rust integration later

**One-sentence positioning:**

> StoryLock is a local authorization kernel Skill package for individual founders and Agent scenarios. It replaces long-term manual credential hunting with story-based local verification and controls sensitive object access through short-lived, scope-limited sessions.

---

## 11. Submission Checklist

| Material | Status | Path |
|----------|--------|------|
| Project summary | Completed | `skill/docs/usecase/00-参赛说明文档.md` |
| Technical architecture | Completed | `skill/docs/ref/storylock-skill-guide/` |
| Skill code | Completed | `skill/src/storylock-skill-engine/` |
| Demo commands | Completed | `npm run demo` / `npm run selftest` / `npm run build:wasm` / `npm run selftest:wasm` |

---

## 12. External Links

| Link | Description |
|------|-------------|
| https://docs.pharos.xyz/tooling-and-infrastructure/pharos-skill-engine-guide | Pharos Skill Engine standard docs |
| https://www.pharos.xyz/agent-carnival | Pharos Agent Carnival official site |
| https://bit.ly/4v6QsWm | Competition submission entry |
