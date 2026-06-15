# StoryLock Skill Invocation

## Overview

This page summarizes the exported invocation surface for the self-describing StoryLock skill package.

Package entry:

1. `index.js`

Primary exported implementation files:

1. `assets/migrated/skills/story-assist.js`
2. `assets/migrated/skills/strength-review.js`
3. `assets/migrated/skills/authorization-skills.js`
4. `assets/migrated/skills/recommended.js`
5. `assets/migrated/agent/video-publish-demo.js`

## Exported Surface

The package exports these skill classes:

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. `StrengthReviewSkill`
4. `LoginAuthorizationSkill`
5. `SigningAuthorizationSkill`
6. `LocalPasswordFillSkill`
7. `ChallengeSigningAuthorizationSkill`

The package also exports:

1. `VideoPublishAgentDemo` as an orchestration example

## Command Template

```js
import {
  StoryDraftAssistSkill,
  StrengthReviewSkill,
  LocalPasswordFillSkill,
  ChallengeSigningAuthorizationSkill,
} from "../index.js";
```

## Constructor Requirements

| Export | Required Dependencies |
| --- | --- |
| `StoryDraftAssistSkill` | `generator` |
| `StoryRefineAssistSkill` | `refiner` |
| `StrengthReviewSkill` | optional `analyzer` |
| `LoginAuthorizationSkill` | `host` |
| `SigningAuthorizationSkill` | `host` |
| `LocalPasswordFillSkill` | `host` |
| `ChallengeSigningAuthorizationSkill` | `host`, `signer` |

## Routing Guidance

Agents should prefer these capability pages for actual task execution:

1. `references/story-assist.md`
2. `references/strength-review.md`
3. `references/password-fill.md`
4. `references/challenge-sign.md`

For product-facing guidance, prioritize:

1. `LocalPasswordFillSkill`
2. `ChallengeSigningAuthorizationSkill`

## Error Handling

| Case | Meaning | Fix |
| --- | --- | --- |
| missing constructor dependency | the selected class was created without its required dependency | provide the dependency listed above |
| wrong capability selection | the exported class does not match the user task | route through the capability index in `SKILL.md` |

## Agent Guidelines

1. Start at `SKILL.md`, not at implementation files.
2. Choose the capability-specific reference file before constructing a skill object.
3. Use `index.js` exports as the stable package entrypoint.
4. Treat `VideoPublishAgentDemo` as an example flow, not as a core capability definition.
