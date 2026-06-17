# StoryLock EIP-712 Minimal Request Definition

## Purpose

This document defines the minimum implementable EIP-712 request structure for StoryLock at the current stage.

Applicable scope:

1. Layer 3 remote Skill / proxy authorization Skill
2. Local signature delegation requests
3. Structured signature requests initiated by third-party Skills

## Current Stage Goals

At the current stage, we do not aim to cover all on-chain execution scenarios, but only to solve:

1. Requests can be structurally expressed
2. Request fields are clearly auditable
3. The local signature gateway no longer accepts ambiguous "blind signing" requests

## Minimum Usage Scenarios

Currently the highest priority support is:

1. `request_signature`
2. `delegated_action_sign`

That is:

1. Signing a structured payload submitted remotely
2. Signing a structured action delegated from a third-party Skill

## EIP-712 Structure Suggestions

### Domain

It is recommended to use the following structure at the current stage:

```javascript
const domain = {
  name: "StoryLock",
  version: "1-placeholder",
  chainId: 1,
  verifyingContract: "0x0000000000000000000000000000000000000000"
};
```

## Current Stage Notes

⚠️ At the current stage, `domain.version = "1-placeholder"` and `verifyingContract` is a placeholder address.

This means:

1. The current definition can only serve as a structured signature request format
2. It should not be regarded as a formal domain directly usable for production on-chain verification
3. Before entering production, it must be replaced with real chain environment parameters

1. `chainId`
   - At the current stage, it is recommended to be injected by runtime configuration
   - The default value can be the target chain environment, but it is not recommended to permanently hardcode mainnet values in the code
2. `verifyingContract`
   - At the current stage, if not yet bound to a specific contract, a placeholder address can be used first
   - If subsequently connected to smart accounts or on-chain verification, replace it with a real address
3. `version`
   - At the current stage, it is recommended to use `1-placeholder`
   - If subsequently bound to a formal contract, it can be upgraded to `2` or a more explicit formal version name

## Types

It is recommended to define a minimal request type:

```javascript
const types = {
  StoryLockSignatureRequest: [
    { name: "action", type: "string" },
    { name: "resource", type: "string" },
    { name: "scope", type: "string" },
    { name: "expiry", type: "uint256" },
    { name: "nonce", type: "uint256" },
    { name: "requestedBy", type: "string" },
    { name: "delegationContext", type: "string" }
  ]
};
```

## Value

The suggested minimal value object is as follows:

```javascript
const value = {
  action: "request_signature",
  resource: "storylock:wallet-key",
  scope: "signature_basic",
  expiry: 1760000000,
  nonce: 1001,
  requestedBy: "remote-agent",
  delegationContext: "user:12345/skill:third-party-demo"
};
```

## Field Explanations

| Field | Meaning | Required at Current Stage |
| --- | --- | --- |
| `action` | Request action, e.g., `request_signature` | Yes |
| `resource` | Target resource identifier | Yes |
| `scope` | Request scope | Yes |
| `expiry` | Request expiry time | Yes |
| `nonce` | Anti-replay random number or monotonic value | Yes |
| `requestedBy` | Request initiator identifier | Yes |
| `delegationContext` | Delegation context, recording source chain | Yes |

## Relationship with StoryLock Gateway

The remote gateway should not send raw string messages, but should send:

1. Structured request objects
2. Corresponding EIP-712 domain
3. Corresponding types
4. Corresponding value

That is, the payload of `requestSignature` can subsequently be extended to:

```json
{
  "identityId": "identity-001",
  "keyId": "wallet-key",
  "algorithm": "ed25519",
  "payload": "0x1234...",
  "resourceId": "eth-main",
  "primaryRole": "private_key",
  "eip712": {
    "domain": {},
    "types": {},
    "value": {}
  }
}
```

## Current Stage Boundaries

This minimal definition currently only solves:

1. Request structuring
2. Auditability
3. Attachable delegation context

At the current stage, we do not forcefully solve:

1. Multi-chain unified domain strategy
2. Complete contract account verification flow
3. EIP-1271 actual on-chain return format

These belong to the next stage of expansion.

## Correspondence with Technical Review Comments

This document mainly responds to the following issues in the technical review:

1. The specific construction parameters of `domainSeparator` need to be clarified
2. `chainId` cannot remain only at the conceptual level
3. `verifyingContract` needs a current stage placeholder or injection strategy

## Domain Upgrade Path Suggestions

To avoid breaking semantics when subsequently connecting to real on-chain verification, it is recommended to:

1. Explicitly identify the placeholder version as `placeholder` at the current stage
2. Upgrade `version` after formal on-chain verification is enabled
3. Synchronously record the version switching point when changing `verifyingContract`

This allows clear distinction between:

1. The current stage's "structured signature request"
2. The next stage's "on-chain verifiable signature request"

## Production Environment Checklist

Before entering formal on-chain verification or production deployment, at least check the following items:

1. `version` no longer uses `placeholder`
2. `verifyingContract` no longer uses the zero address
3. `chainId` has been switched to the target formal network
4. `nonce` generation and persistence strategy has been verified
5. `expiry` window conforms to production strategy

## Current Stage Minimal Implementation Suggestions

It is recommended to implement first:

1. `domain`
2. `types`
3. `value`
4. `nonce`
5. `expiry`

Then complete a minimal demo through the local signature delegation chain.

## Conclusion

The most appropriate positioning for EIP-712 in StoryLock at the current stage is not "complete on-chain account abstraction standard", but:

1. A structured signature request format for third-party Skills and remote Agents
2. The minimum auditable input standard for local signature delegation
