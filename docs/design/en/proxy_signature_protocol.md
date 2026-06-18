# StoryLock Proxy Signature Mechanism Reference Design

## Purpose

This document answers the following questions:

1. Should StoryLock provide a "proxy signature" capability?
2. Can third-party Skills delegate signing actions to local Skills?
3. Which protocols in existing protocols are suitable for direct reference?
4. Which protocols are suitable as execution layer references, and which are only suitable as audit or governance layer references?

## Core Judgment

The conclusion is:

1. StoryLock should provide "local signature delegation capability"
2. Third-party Skills should not directly hold private keys
3. Third-party Skills should submit structured signature requests
4. Local Skills / local Hosts complete signatures after authorization
5. What is returned to third-party Skills should be structured signature results, not private keys or long-term signing capabilities

Therefore, the role of the remote Agent is not "private key holder", but rather:

1. Request router
2. Capability wrapper
3. Result organizer

The real signature executor should always be local.

## Suggested Proxy Signature Mechanism Layers

It is recommended to split the proxy signature mechanism into four layers:

### 1. Request Representation Layer

Responsible for:

1. Describing "what to sign"
2. Describing "what scope is allowed for signing"
3. Describing "what is the signing purpose"

This layer is most worth referencing:

1. `EIP-712`

### 2. Local Authorization Layer

Responsible for:

1. Challenge creation
2. Answer submission
3. Scope verification
4. Local object access policy judgment

This layer is mainly StoryLock's own responsibility and does not directly copy external protocols.

### 3. Signature Verification / Account Capability Layer

Responsible for:

1. How on-chain acknowledges a signature is valid
2. How contract accounts replace EOAs
3. Whether smart accounts, abstract accounts are allowed to participate

This layer is most worth referencing:

1. `EIP-1271`
2. `ERC-4337`
3. `EIP-7702`

### 4. Delegation Audit and Responsibility Chain Layer

Responsible for:

1. Recording who authorized what
2. Recording whether the remote Agent was only forwarding
3. Recording the source of responsibility for local execution results

This layer is most worth referencing:

1. `HDP Protocol`

## Protocol Reference Summary Table

| Name | Type | Reference Value for StoryLock Proxy Signature | Suggested Usage |
| --- | --- | --- | --- |
| `EIP-712` | Off-chain signature standard | Very high | As the structured message format for third-party Skills to submit signature requests |
| `EIP-1271` | On-chain standard | Very high | As the verification interface when "local signer is not EOA, but contract account" |
| `EIP-2612` | On-chain standard | Medium | As a reference for permit-type signature sub-scenarios, not as a general proxy signature framework |
| `ERC-4337` | On-chain standard | High | As a reference for smart account execution layer, applicable to abstract account wallets or policy wallets |
| `EIP-7702` | On-chain standard | Medium to high | As a reference for EOA temporary delegation of execution logic capability |
| `Metaplex Agent Registry` | Ecosystem framework | Medium | Reference the "on-chain identity + off-chain execution" model, does not directly solve local signature delegation |
| `Keyless Collective SDK` | Project framework | High | Reference the "Agent does not hold private keys, executed by external coordination layer" overall architecture |
| `Turnkey Agentic Wallet` | Project framework | High | Reference the "keys in controlled infrastructure, Agent only calls limited capabilities" model |
| `HDP Protocol` | Standardization protocol / IETF draft | Very high | As the audit chain model for "who authorized the signature request, through which Agents it was transferred" |

## Most Worth Directly Referencing Protocols

### 1. EIP-712

Most worth referencing points:

1. Structured signature data
2. Domain isolation
3. Strong readability
4. Suitable for offline signature request expression

Suggestions for StoryLock:

1. Signature requests sent by remote Agents to local Agents can adopt EIP-712 style structured objects
2. Requests should at least contain:
   - `action`
   - `resource`
   - `scope`
   - `expiry`
   - `nonce`
   - `requestedBy`
   - `delegationContext`

This avoids "blind signing" requests entering the local signature gateway.

### 2. EIP-1271

Most worth referencing points:

1. Signature validity is no longer bound to a single EOA
2. Contract accounts can define their own signature verification logic

Suggestions for StoryLock:

1. If the local signer is subsequently upgraded to a smart account, local security module controlled contract wallet, or policy wallet, then `EIP-1271` should be prioritized for compatibility
2. This prevents StoryLock's local signature delegation from being bound to "EOA private key direct signing"

### 3. HDP Protocol

Most worth referencing points:

1. Recording human authorization sources
2. Recording multi-hop Agent delegation chains
3. Recording scope and provenance

Suggestions for StoryLock:

1. Attach delegation provenance to each proxy signature in the local Agent gateway
2. When the remote Agent is only forwarding, it should also be recorded in the delegation chain
3. This can prove:
   - Who initiated the request
   - Which Agent transferred the request
   - Under what scope the local Agent executed the signature

## Worth Referencing in Execution Layer

### 4. ERC-4337

Most worth referencing points:

1. `UserOperation` as a unified operation object
2. Account abstraction
3. Supporting policy wallets, gas sponsorship, batch execution

Suggestions for StoryLock:

1. If StoryLock's local signature delegation ultimately faces smart account execution transactions, the `UserOperation` request modeling can be referenced
2. But it is not a replacement for the local authorization layer itself
3. It is more suitable for the "execution layer after signing"

### 5. EIP-7702

Most worth referencing points:

1. EOA can temporarily delegate execution logic
2. Closer to "granting capability per single transaction"

Suggestions for StoryLock:

1. Can be used as a reference for the idea of "after local signing, EOA temporarily borrows contract logic for execution"
2. Suitable as a "limited scope, short-term authorization" execution layer supplement
3. Not suitable as the sole solution for StoryLock's local authorization layer

## Worth Referencing as Specific Business Sub-Scenarios

### 6. EIP-2612

Most worth referencing points:

1. Permit mode
2. Offline authorization + third-party submission flow

Suggestions for StoryLock:

1. Suitable for referencing the "offline authorization, then submitted by third party" user experience
2. Very suitable for token allowance and other narrow scenarios
3. But it is not broad enough to directly assume the "general proxy signature mechanism"

## Worth Referencing as Product Architecture

### 7. Turnkey Agentic Wallet

Most worth referencing points:

1. Keys are not exposed to Agents
2. Controlled infrastructure executes business signatures
3. Uses policies to restrict Agent behavior

Suggestions for StoryLock:

1. This is a product reference very close to StoryLock's target architecture
2. StoryLock can reference its ideas, but replace "controlled infrastructure" with "local Agent / local Host"

### 8. Keyless Collective SDK

Most worth referencing points:

1. Agents do not manage private keys themselves
2. External coordination layer completes payment or execution
3. Restricts permission scope through policies

Suggestions for StoryLock:

1. Very suitable for referencing its "capability proxy" idea
2. StoryLock can treat the local gateway as its own coordination layer
3. But specific cryptography and network topology do not necessarily need to be copied

### 9. Metaplex Agent Registry

Most worth referencing points:

1. On-chain identity
2. Verifiable proxy identity
3. Off-chain execution + on-chain registration combination mode

Suggestions for StoryLock:

1. Suitable for referencing identity registration and discovery mechanisms
2. Does not directly solve the "private keys cannot be handed to third-party Skills" problem
3. More suitable as a proxy identity layer, not a signature delegation layer

## Most Suitable Combination for StoryLock

It is recommended to adopt the following combination:

### Mandatory References

1. `EIP-712`
   - Used to define signature request format
2. `HDP Protocol`
   - Used to define delegation chains, audit chains, and responsibility sources
3. `EIP-1271`
   - Used to be compatible with future smart accounts or contract verification signatures

### Conditional References

1. `ERC-4337`
   - Introduced when preparing to connect to abstract account wallets
2. `EIP-7702`
   - Introduced when needing EOA single-time temporary delegation logic
3. `EIP-2612`
   - Introduced when doing permit / token allowance class specialized functions

### Architecture References

1. `Turnkey Agentic Wallet`
2. `Keyless Collective SDK`
3. `Metaplex Agent Registry`

## Local Key Storage Suggestions

It is recommended to fix the key storage strategy separately at the current stage.

Recommended priority:

1. Operating system keychain
2. Platform secure storage
3. Optional hardware security module or hardware key

Platform suggestions:

1. Windows: Prefer connecting to system protected storage capabilities
2. macOS: Prefer using Keychain
3. Linux: Prefer using Secret Service or equivalent secure storage

Not recommended at the current stage:

1. Directly writing long-term private keys in plaintext to ordinary configuration files
2. Directly placing long-term private keys in SQLite tables readable by Layer 3

## Multi-Identity Key Management Suggestions

If multi-identity is supported later, it is recommended to adopt derived management rather than scattered management.

Recommended direction:

1. Retain a controlled root key
2. Derive working keys or signing contexts by `identityId`
3. Derivation logic is controlled by the local security layer, invisible to Layer 3

This can:

1. Reduce the number of independent key files for multiple identities
2. Reduce key directory management complexity
3. Make auditing easier to converge to the identity level

## Signature Audit Suggestions

After each signature, it is recommended to at least record:

1. `requestId`
2. `scope`
3. `resource`
4. `signatureHash`
5. `timestamp`

Auditing should not record:

1. Private key plaintext
2. Seed plaintext
3. Raw challenge material that can lead to replay

## Signature Algorithm Selection Suggestions

At the current stage, it is not recommended to open all algorithms at once, but to converge by scenario.

Recommended decision order:

1. If the target ecosystem is Ethereum / EVM:
   - Prefer `secp256k1`
2. If the target is general offline signatures, cross-chain proxy identifiers, or non-EVM systems:
   - Optional `ed25519`
3. If there are explicit compliance or national cryptography requirements:
   - Reserve `SM2`, but do not default implement at the current stage

Current stage recommendations:

1. Ethereum-related requests default to `secp256k1`
2. StoryLock internal non-chain demonstrations or general message signatures can use `ed25519`
3. Do not mix multiple algorithms on the same identity unless the Host has explicit policy

## Algorithm Selection Minimal Decision Tree

```text
Is the target an EVM on-chain signature?
|- Yes -> secp256k1
|- No -> Is it a general message / proxy identifier signature?
   |- Yes -> ed25519
   |- No -> Are there compliance requirements?
      |- Yes -> SM2 (subsequent extension)
      |- No -> Determined by Host whitelist
```

## EIP-712 Adaptation Code Example

At the current stage, it is recommended to align Layer 3 packaging requests and Layer 2 local signature entry into a unified structure.

Example:

```ts
export function buildStoryLockEip712Request({
  action,
  resource,
  scope,
  expiry,
  nonce,
  requestedBy,
  delegationContext,
  chainId,
  verifyingContract,
}: {
  action: string;
  resource: string;
  scope: string;
  expiry: bigint;
  nonce: bigint;
  requestedBy: string;
  delegationContext: string;
  chainId: number;
  verifyingContract: `0x${string}`;
}) {
  const domain = {
    name: "StoryLock",
    version: "1-placeholder",
    chainId,
    verifyingContract,
  };

  const types = {
    StoryLockSignatureRequest: [
      { name: "action", type: "string" },
      { name: "resource", type: "string" },
      { name: "scope", type: "string" },
      { name: "expiry", type: "uint256" },
      { name: "nonce", type: "uint256" },
      { name: "requestedBy", type: "string" },
      { name: "delegationContext", type: "string" },
    ],
  };

  const value = {
    action,
    resource,
    scope,
    expiry,
    nonce,
    requestedBy,
    delegationContext,
  };

  return { domain, types, value };
}
```

After Layer 2 local gateway receives the request, it should at least verify:

1. `action`
2. `scope`
3. `expiry`
4. `nonce`
5. `requestedBy`
6. `delegationContext`

Only after passing challenge / session / scope / policy checks is it allowed to enter the actual signing step.

## Suggested StoryLock Proxy Signature Positioning

StoryLock should not define "proxy signature" as a simple signature API.

A more reasonable positioning is:

1. Third-party Skills submit structured signature intents
2. Remote Agents package requests
3. Local Agents execute challenges, scope checks, object access control, and signatures
4. Return structured signature results and minimal audit metadata

So it is more like:

1. `Local Signing Delegate`
2. `Local Capability Gateway`
3. `Policy-controlled Signing Broker`

Rather than:

1. "Give private keys to remote Skills"
2. "Let remote Skills directly sign"

## Conclusion

For StoryLock, the most correct positioning is not "third-party Skills learn to sign", but rather:

1. Third-party Skills learn to initiate signature delegation
2. Remote Agents learn to package and transfer requests
3. Local Skills learn to complete signatures after authorization

In terms of protocol references, the most worth prioritizing are:

1. `EIP-712` for request representation
2. `EIP-1271` for signature verification compatibility
3. `HDP Protocol` for delegation and audit chains
4. `ERC-4337` / `EIP-7702` for subsequent execution layer extensions
