# Strength Review

## Overview

Use this capability to analyze whether a 24-question StoryLock question set is strong enough for the documented basic-read threshold.

Primary implementation source:

1. `assets/migrated/skills/strength-review.js`

## Command Template

```js
import { StrengthReviewSkill } from "../assets/migrated/skills/strength-review.js";

const skill = new StrengthReviewSkill();

const result = await skill.run({
  questions: my24Questions,
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `questions` | `object[]` | yes | none | Must contain exactly 24 question nodes for full analysis. |

Each question object is expected to provide the fields required by the analyzer, including valid answers, distractors, and optional verify-policy fields.

## Output Parsing

The result includes:

1. `productionReady`: whether the current set passes the implemented checks
2. `rootKdfMeetsProductionMinimum`
3. `answerKdfMeetsProductionMinimum`
4. `formalEligibleQuestionCount`
5. `requiredBasicQuestionCount`
6. `basicMinChallengeBits`
7. `strongestBasicChallengeBits`
8. `strongestBasicChallengeMeetsMinimum`
9. `questionSetReady`
10. `issues`
11. `recommendedActions`

Agents should treat `questionSetReady` and `recommendedActions` as the primary user-facing summary.

## Error Handling

| Error Code | Trigger | Fix |
| --- | --- | --- |
| `VALIDATION_ERROR` | `questions` is missing or not an array | Supply an array of question objects. |
| `VALIDATION_ERROR` | `questions.length !== 24` | Provide exactly 24 nodes. |
| `VALIDATION_ERROR` | one or more questions have invalid structure | Repair `validAnswers`, `distractors`, or policy fields. |

## Agent Guidelines

1. Use this capability only for question-set evaluation, not story drafting.
2. Check that the caller actually supplied 24 questions before invoking the skill.
3. Surface `questionSetReady`, `issues`, and `recommendedActions` first.
4. If the set is not ready, tell the user which missing condition blocked readiness.
5. Do not claim production readiness when `productionReady` is false.
