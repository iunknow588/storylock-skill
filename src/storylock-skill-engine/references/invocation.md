# StoryLock Skill Invocation

## Exported skill surface

The migrated version preserves these current skill classes from `story-lock`:

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. `StrengthReviewSkill`
4. `LoginAuthorizationSkill`
5. `SigningAuthorizationSkill`
6. `LocalPasswordFillSkill`
7. `ChallengeSigningAuthorizationSkill`

## Product-facing recommendation

For end-user-facing integration guidance, prefer:

1. `LocalPasswordFillSkill`
2. `ChallengeSigningAuthorizationSkill`

## Source files

Read these migrated source copies:

1. `assets/migrated/skills/story-assist.js`
2. `assets/migrated/skills/strength-review.js`
3. `assets/migrated/skills/authorization-skills.js`
4. `assets/migrated/skills/recommended.js`
5. `assets/migrated/agent/video-publish-demo.js`

## Minimal constructor expectations

1. story-assist skills require a `generator` or `refiner`
2. authorization skills require a `host`
3. challenge-signing additionally requires a `signer`
