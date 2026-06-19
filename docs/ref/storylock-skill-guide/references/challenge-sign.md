# StoryLock Signature Request

## Status

This page is kept as a compatibility reference. The current mainline third-layer API is:

1. `requestSignature`
2. `requestPasswordFill`

Do not describe `requestChallengeSign` or `ChallengeSigningAuthorizationSkill` as the current main interface.

## Current Mainline Flow

The recommended signature flow is:

1. A remote caller sends `requestSignature`.
2. The gateway wraps the request with a `StoryLockSignatureRequest` EIP-712 style payload.
3. The local access layer resolves object strength.
4. The local access layer creates a grid verification.
5. The user answers locally.
6. The local access layer issues a short-lived authorization.
7. A local signature executor signs.
8. The gateway redacts sensitive fields before returning.

## Invocation Example

```js
import { StoryLockRemoteGateway } from "../../../src/skills/remote-gateway/index.js";

const gateway = new StoryLockRemoteGateway({
  transport(request) {
    return request;
  },
  signatureExecutor(request) {
    return {
      requestId: request.requestId,
      capability: request.capability,
      result: {
        signature: "sig-demo",
        signingKeyBytes: [1, 2, 3],
      },
    };
  },
});

const result = await gateway.requestSignature({
  requestId: "req-demo-sign",
  nonce: "nonce-demo-sign",
  eip712Nonce: "1001",
  expiry: Date.now() + 60_000,
  identityId: "demo-user",
  keyId: "wallet/main/private_key",
  algorithm: "ed25519",
  payload: "sign this payload",
  resourceId: "wallet/main/private_key",
});
```

`signingKeyBytes` will be redacted in the returned result.

## Testable Demo

Run:

```powershell
cd E:\2026OPC大赛\skill\src\storylock-remote-gateway-skill
npm run selftest:e2e
```

Expected output:

```text
StoryLock three-layer e2e selftest passed.
```

