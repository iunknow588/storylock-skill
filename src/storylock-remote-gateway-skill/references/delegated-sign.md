# Delegated Signature

## Command Template

```js
import { StoryLockRemoteGateway } from "../index.js";

const gateway = new StoryLockRemoteGateway({ transport });

const response = await gateway.requestSignature({
  requestId: "req-sign-001",
  identityId: "identity-001",
  keyId: "wallet-key",
  algorithm: "ed25519",
  payload: "sign this payload",
  resourceId: "eth-main",
  primaryRole: "private_key",
  nonce: "nonce-003",
  expiry: Date.now() + 60_000,
  eip712Domain: {
    name: "StoryLock",
    version: "1-placeholder",
    chainId: 1,
    verifyingContract: "0x0000000000000000000000000000000000000000",
    environment: "demo",
  },
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `requestId` | string | yes | none | Remote request id. |
| `identityId` | string | yes | none | Requesting identity. |
| `keyId` | string | yes | none | Local signing key id. |
| `algorithm` | string | yes | none | Requested signing algorithm. Allowed values: `ed25519`, `secp256k1`. |
| `payload` | any | yes | none | Payload to sign. |
| `resourceId` | string | no | `null` | Optional target resource id. |
| `primaryRole` | string | no | `null` | Optional role hint. |
| `nonce` | string | yes | none | Replay-protection nonce. |
| `expiry` | number | yes | none | Request expiry timestamp. |
| `requestedRetention` | string | no | `result_only` | Retention requested from the gateway. |
| `policyHints` | object | no | `{}` | Route and redaction hints. |
| `eip712Domain` | object | no | gateway default or demo placeholder domain | Optional request-level EIP-712 domain override, including `name`, `version`, `chainId`, `verifyingContract`, and `environment`. |

## Output Parsing

The remote gateway returns a structured local signing request rather than raw key material.
The request payload includes a minimal EIP-712 structure under `payload.eip712`.
When `eip712Domain` is not supplied, the gateway uses the constructor-level `eip712Domain` default if present, otherwise a demo-oriented placeholder domain.
Production callers should inject an explicit domain with a real `version`, `chainId`, and `verifyingContract`.
If `environment` is `production`, the gateway rejects placeholder versions, `chainId: 0`, and the zero verifying contract address.

For callers that want an explicit production preset, use `createProductionEip712Domain({ version, chainId, verifyingContract })`.
For demo flows, use `createDemoEip712Domain()` or pass a request-level placeholder domain explicitly.

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `AUTHORIZATION_REQUIRED` | Local signing requires authorization | Complete local grid verification. |
| `SCOPE_INSUFFICIENT` | Signing level too low | Retry with the correct access level. |
| `NONCE_REPLAY_DETECTED` | Nonce replayed | Issue a fresh request. |
| `algorithm must be ed25519 or secp256k1` | Unsupported algorithm | Use the approved algorithm list. |

## Agent Guidelines

1. This path delegates signing, not key ownership.
2. Remote layers must never receive raw signing key bytes.
3. Algorithm approval must still happen locally even if the gateway accepted the request shape.
4. Do not omit replay fields when calling the remote gateway.
