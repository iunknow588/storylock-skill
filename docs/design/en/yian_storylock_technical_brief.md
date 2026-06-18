# Yian StoryLock Encrypted Authorization System Technical Brief

## 1. Project Overview

| Item | Details |
| --- | --- |
| Product Name | Yian StoryLock Encrypted Authorization System |
| Short Name | Yian System |
| Applicable Scenario | Local private-key management, file-key management, sensitive-object authorization control, and local approval in Agent scenarios |
| Positioning | A local authorization and sensitive-object management solution for remote Agents and cloud services |
| Main Entry | `https://yian.cdao.online` |
| Core Components | Yian Remote Access Interface, Yian App / Private Assistant, StoryLock Local Core |
| Repository Root | `skill/` |
| Document Version | 2026-06-18 |

## 2. Background and Summary

Digital personal agents are changing the traditional model of "user-approved authorization" into a model where Agents can hold keys autonomously. This improves automation efficiency, and it also forces users to place nearly unconditional trust in the servers that run those Agents. For a decentralized direction that emphasizes user control over assets, identity, and permissions, this concentration of trust is a clear technical regression.

The Yian System is created to bring signature authorization, password filling, and sensitive-object access back from centralized servers to the user's trusted local device while preserving the convenience of Agent automation. Remote Agents can submit requests. The Private Assistant can explain those requests, apply tiered management according to the security level of the target object, and determine whether user interaction is required. StoryLock Local Core performs sensitive execution, while the user keeps key approval on a trusted local device.

StoryLock is a local security component for Agent scenarios. It manages authorization for private keys, file keys, and sensitive objects. Local-side components manage private keys, file keys, and story files, while remote Agents submit requests through a controlled entry. File contents are protected by keys, and only local components with the corresponding file key, permission level, and local approval result can access them. StoryLock introduces associative-memory cues into authorization decisions and packages "permission-gated local sensitive-object access" and "human-understandable, recallable authorization cues" as reusable Skill capabilities.

In one sentence:

> The Yian System is a local authorization capability package for Agent scenarios: local devices manage private keys and file keys, associative-memory cues participate in authorization decisions, and three Skill layers allow remote Agents to use local capabilities safely.

## 3. Product Positioning

Yian is the remote access interface and local approval entry in the StoryLock usage flow. It delivers important requests from external Agents, cloud services, or automated workflows to the user side, where the user's own local device completes explanation, approval, and sensitive execution.

From a product-positioning perspective, the Yian System has three responsibilities:

1. Remote access interface
   Provides download, binding, request intake, and request-status views. Remote Agents or cloud services submit controlled requests through this entry, and users view current request status through the same entry.

2. Local approval entry
   The Yian App runs on the user's trusted local device, carries the Private Assistant, and passes requests that require approval to StoryLock Local Core.

3. Local authorization and key management
   StoryLock Local Core manages story files, file keys, private keys, and sensitive-object access policies, then performs signing, password filling, or other sensitive execution according to local approval results.

The Yian App is therefore the local approval entry that connects the Yian Remote Access Interface with StoryLock Local Core. In daily use, users complete download and binding through the Yian Remote Access Interface. When the remote side initiates a password-filling, signing, or other interactive authorization request, users view request status and complete approval on their local device.

The following diagram shows the three runtime relationships in the Yian System: external Agents or cloud services submit controlled requests through the Yian Remote Access Interface; the Yian Remote Access Interface communicates bidirectionally with the Private Assistant; the Private Assistant then collaborates with StoryLock Local Core through controlled local calls.

![Yian System runtime structure diagram](https://raw.githubusercontent.com/iunknow588/storylock-skill/main/src/yian-web/public/assets/yian-network-banner.png)

The key boundary in the diagram is that the Yian Remote Access Interface has no direct channel to StoryLock Local Core. StoryLock Local Core returns only the minimum necessary result to the Private Assistant.

## 4. Runtime Structure

From the software runtime structure, the system is divided into three parts:

1. Yian Remote Access Interface
   Deployed on a cloud service platform. It provides the documentation entry, download entry, binding entry, request entry, and request-status view. External Agents or services submit requests through controlled interfaces, while the remote access interface receives, records, forwards, and displays status.

2. Private Assistant
   Runs on the user's trusted local device and carries the local interaction capability of the Yian App. It keeps bidirectional communication with the Yian Remote Access Interface, receives requests, explains source and risk, displays pending approval content, and passes sensitive execution tasks to StoryLock Local Core.

3. StoryLock Local Core
   Runs on the local-device side and handles story files, key isolation, local approval, and sensitive execution. It collaborates with the Private Assistant through controlled local calls and returns only the minimum necessary result to the Private Assistant.

The overall request path is:

```text
External Agent / Cloud Service / pharos Agent / OpenClaw
  -> Yian Remote Access Interface
  <-> Private Assistant
  <-> StoryLock Local Core
```

The remote access interface coordinates the cloud-side entry and status. The Private Assistant handles user-side explanation, interaction, and relay. StoryLock Local Core handles sensitive material and local approval.

## 5. Three-Layer Skill Capabilities

The project is organized into three layers in code and Skill capabilities. The runtime structure explains how requests move among the cloud side, local assistant, and local core. The capability layers explain how code capabilities are separated, reused, and verified.

### 5.1 Layer 1: Local Story Processing

Layer 1 handles local story drafts, story refinement, and question-set strength evaluation.

| Item | Details |
| --- | --- |
| Code Package | `src/storylock-local-story-processing-skill` |
| Main Capabilities | Story draft generation, story refinement, question-set strength evaluation |
| Typical Skills | `StoryDraftSkill`, `StoryRefineSkill`, `StrengthReviewSkill` |

This layer provides story and memory-cue material that both users and the system can understand, giving later authorization decisions explainable context.

### 5.2 Layer 2: Local Controlled Authorization

Layer 2 determines the authorization strength required by the target object and generates the corresponding local verification flow.

| Item | Details |
| --- | --- |
| Code Package | `src/storylock-local-story-access-skill` |
| Main Capabilities | Object-level strength policy, grid verification, local authorization, short-lived sessions, replay protection |
| Typical Skills | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill` |

This layer focuses on questions such as whether a sensitive object can be accessed, what confirmation strength is required, and how long the authorization window should remain valid.

### 5.3 Layer 3: Remote Gateway and Controlled Delegation

Layer 3 wraps calls from external Agents or cloud services into unified, verifiable, and auditable requests.

| Item | Details |
| --- | --- |
| Code Package | `src/storylock-remote-gateway-skill` |
| Main Interfaces | `requestSignature`, `requestPasswordFill` |
| Main Capabilities | Remote request wrapping, signature approval request wrapping, Web2 password-fill request wrapping, EIP-712 minimal signing request structure, minimized result return |

This layer allows remote Agents to use local capabilities while keeping actual authorization, signing, password filling, and sensitive-material access within the local approval chain.

## 6. Code Structure

### 6.1 Three-Package Structure

| Package | Directory | Responsibility |
| --- | --- | --- |
| `storylock-local-story-processing-skill` | `src/storylock-local-story-processing-skill` | Layer 1, local story processing |
| `storylock-local-story-access-skill` | `src/storylock-local-story-access-skill` | Layer 2, local controlled authorization |
| `storylock-remote-gateway-skill` | `src/storylock-remote-gateway-skill` | Layer 3, remote request wrapping and gateway |

### 6.2 Aggregation and Shared Modules

| Module | Directory | Description |
| --- | --- | --- |
| `storylock-skill-engine` | `src/storylock-skill-engine` | Unified exports, example scripts, self-test scripts, and compatibility demo entry |
| `shared` | `src/shared` | Shared encryption, SQLite, and SecretStore adapter code |
| `yian-web` | `src/yian-web` | Static site and help entry for the Yian Remote Access Interface |
| `android-host` | `android-host` | Android project for the Yian App / Private Assistant / StoryLock Local Core demo loop |

## 7. Core Mechanisms

### 7.1 Associative-Memory Cues

StoryLock uses stories, prompts, and question sets as associative-memory cues to help users complete understandable and recallable authorization approval. It turns authorization approval from simple mechanical input into a local confirmation process where users can understand the request, recall the cue, and judge risk.

Associative-memory cues play three roles in the system:

1. They help users understand the relationship between sensitive objects and authorization requests.
2. They support different local confirmation strengths for different objects.
3. They reduce the long-term risk of forgetting fixed passphrases or mechanical credentials.

The story-processing layer generates and organizes cues. The local controlled-authorization layer selects confirmation strength according to object policy. StoryLock Local Core performs sensitive confirmation locally.

### 7.2 Request Status

Request status is the core runtime view of the system. A request usually moves through these states:

1. Created: an external Agent or service initiates the request.
2. Received: the Yian Remote Access Interface receives the request.
3. Delivered: the Private Assistant receives the request.
4. Pending approval: the user is viewing the request on the local device.
5. Approved or left unapproved: the user completes approval or temporarily leaves the request unapproved.
6. Completed: the result is returned to the Private Assistant and status is synchronized as needed.

Users should focus on whether any request is waiting for processing, whether the request source is trusted, whether the request content matches the current operation, and whether local-device approval is required.

### 7.3 Local Approval

Local approval has two levels:

1. User approval
   The user reviews source, content, and risk, then determines whether the request matches the current operation.

2. Device approval
   The local device confirms user intent through device unlock, biometrics, or device credentials.

When both user approval and device approval meet the required conditions, the request proceeds to StoryLock Local Core for execution.

### 7.4 Minimized Result Return

After StoryLock Local Core completes a sensitive operation, it returns only the minimum necessary result to the Private Assistant. The Private Assistant then synchronizes approval status to the Yian Remote Access Interface as needed.

This minimized return approach reduces the remote system's exposure to sensitive material and keeps key judgment and sensitive execution on the user's local device.

## 8. Security Mechanisms

| Capability | Description |
| --- | --- |
| Local holding | Private keys, file keys, story files, and sensitive material are managed by local-device-side components |
| Object-level strength policy | Determines the required confirmation strength according to the target object |
| Grid verification | Generates local verification challenges according to confirmation strength |
| Replay protection | Uses `requestId`, `nonce`, and expiry constraints to control duplicate submissions and expired requests |
| Short-lived sessions | Controls the authorization window through short-lived sessions |
| Automatic lock and recovery | Enters a lock window after repeated failures and restores availability after the lock window ends |
| Object encryption | Uses encryption mechanisms to protect local object content |
| Key derivation | Uses derivation mechanisms to generate use-specific keys |
| Answer digest | Stores verification answers as digests to avoid plaintext storage |

Together, these mechanisms allow remote requests to be processed while keeping local approval authority and sensitive-material control on the user's trusted device.

## 9. Installation and Verification

Before downloading and installing the Yian App, users should check the version number, file size, and SHA-256 checksum. The checksum confirms that the installation package remains consistent during transmission and distribution.

Installation package information is displayed by the Yian Remote Access Interface, including version number, file size, release type, and SHA-256 checksum. When downloading an installation package, users should use the latest information displayed by the interface as the source of truth. Before installation, users should verify that the downloaded file matches the information shown by the remote access interface.

Recommended verification flow:

1. Open the download entry from the Yian Remote Access Interface.
2. Record the version number, file size, and SHA-256 shown on the page.
3. After download, calculate the SHA-256 of the local file.
4. Compare the local file size and SHA-256 with the values shown on the page.
5. Continue installation after the information matches.

When a formal installation package is released, the version number, file size, release type, and SHA-256 in the remote access interface should be updated at the same time, so users see information that matches the actual distributed file.

## 10. Typical Application Scenarios

| Scenario | StoryLock Role |
| --- | --- |
| Web2 password filling | A remote Agent submits a request, the user confirms the target site and request source on the local device, and StoryLock Local Core completes local authorization |
| Signature approval | External Agents, pharos Agent, or OpenClaw submit signature approval requests through Skills, and the local core returns the minimum necessary result |
| Story-file and sensitive-object access | Object policy determines confirmation strength, generates the local verification flow, and returns an authorization result after approval |
| Multi-account and multi-object collaboration | Different confirmation strengths are configured for different accounts, credential objects, and local files, allowing remote automation workflows to use local capabilities within controlled boundaries |

## 11. Summary

The Yian System can be summarized as a local authorization and sensitive-object management solution for Agent scenarios. The Yian Remote Access Interface handles request intake, binding, download, and status display. The Private Assistant handles user-side explanation, interaction, and relay. StoryLock Local Core handles story files, key isolation, local approval, and sensitive execution.

The core value of this solution includes:

1. Bringing remote requests back to the user's trusted local device for approval.
2. Separating story processing, local controlled authorization, and remote delegated entry through three Skill layers.
3. Using associative-memory cues to help users make understandable and recallable authorization decisions.
4. Controlling sensitive-object access with object-level strength policies, grid verification, replay protection, short-lived sessions, and minimized result return.
5. Allowing remote Agents to use local capabilities while preserving the user's final approval authority on the local device.
