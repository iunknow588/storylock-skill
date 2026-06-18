# Android Host Contract

## Purpose

Use this reference when Layer 3 runs as a cloud gateway and Layers 1-2 run inside an Android-local host.

## Current Contract

The current repo defines a minimal Android-host boundary through the mock server in:

`src/storylock-local-story-access-skill/android-host-server.js`

This contract is intentionally small:

1. `GET /health`
2. `POST /execute`

## `GET /health`

`GET /health` returns a small readiness summary for the Android-local host.

Schema:

`assets/schemas/android-host-health.schema.json`

Current fields:

1. `status`
2. `layer1.mode`
3. `layer1.questionSetReady`
4. `layer1.strongestBasicChallengeBits`
5. `layer2.identityId`
6. `layer2.questionSetVersion`
7. `layer2.activeQuestionCount`
8. `stats.requestCount`

## `POST /execute`

`POST /execute` is the host-side execution endpoint that receives third-layer requests.

Input schema:

`assets/schemas/remote-gateway-request.schema.json`

Output schema:

`assets/schemas/remote-gateway-response.schema.json`

Current supported capabilities:

1. `requestSignature`
2. `requestPasswordFill`

The Android host must not expose direct Layer 2 internal methods such as:

1. `createChallenge`
2. `submitChallengeAnswers`
3. `issueSession`
4. raw secret-store reads

## Shared Secret

The current mock host accepts an optional shared header:

`x-storylock-shared-secret`

When present, the cloud gateway must forward the same shared secret to the Android host.

## Runtime Roles

### Cloud / Vercel-side Layer 3

Responsibilities:

1. Receive remote requests.
2. Enforce gateway method allowlist.
3. Forward only standard gateway envelopes to Android host.
4. Redact sensitive fields before returning results.

### Android-side Layers 1-2

Responsibilities:

1. Run `StrengthReviewSkill` or equivalent local readiness checks.
2. Hold active question-set state.
3. Complete object-strength policy, grid verification, and local authorization.
4. Execute local signing or password-fill operations.

## Current Truthful Status

The repo currently proves this split with:

1. `api/storylock-gateway.mjs`
2. `src/storylock-remote-gateway-skill/vercel-handler.js`
3. `src/storylock-remote-gateway-skill/scripts/selftest-vercel-android.mjs`

It does not yet include a full Android application project, Android Keystore integration, or a production mobile networking wrapper.
