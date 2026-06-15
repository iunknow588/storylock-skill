# StoryLock Skill Invocation

## 1. Preferred package path

For the current reusable package, prefer:

- `E:\2026OPC大赛\skill\src\storylock-skill-engine`

## 2. Package exports

The migrated package currently exports:

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. `StrengthReviewSkill`
4. `LoginAuthorizationSkill`
5. `SigningAuthorizationSkill`
6. `LocalPasswordFillSkill`
7. `ChallengeSigningAuthorizationSkill`
8. `VideoPublishAgentDemo`

## 3. JS invocation example

```js
import {
  StoryDraftAssistSkill,
  LocalPasswordFillSkill,
  ChallengeSigningAuthorizationSkill,
  LOGIN_BINDING_MODE,
} from "./index.js";
```

## 4. Product-facing recommendation

For end-user-facing integrations, prefer:

1. `LocalPasswordFillSkill`
2. `ChallengeSigningAuthorizationSkill`

## 5. Current testable entry points

Package-local:

1. `npm run demo`
2. `npm run selftest`
3. `npm run build:wasm`
4. `npm run selftest:wasm`

Original repo comparison path:

1. `story-lock/examples/04-skill-layer-demo.mjs`
2. `story-lock/npm run skill-demo`
3. `story-lock/npm run browser-demo`

## 6. Status note

At the moment:

1. JS skill invocation is locally packaged
2. Rust/WASM dist build and export-shape validation are locally packaged
3. full Rust/WASM-backed host invocation is not yet fully migrated into this guide's package target
