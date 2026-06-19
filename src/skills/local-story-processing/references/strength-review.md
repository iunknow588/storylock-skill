# Strength Review

## Command Template

```js
import { StrengthReviewSkill } from "../index.js";

const skill = new StrengthReviewSkill();
const result = await skill.run({
  questions,
});
```

## Boundary

`StrengthReviewSkill` belongs to the first layer. It evaluates question-set strength locally and does not create challenges, issue sessions, read protected objects, or send material to the remote gateway.

## Minimum Shape

`questions` must contain exactly 24 question nodes. Each node needs:

1. `validAnswers`
2. `distractors`
3. exactly 9 unique display candidates across valid answers and distractors

## Output

The result includes:

1. `formalEligibleQuestionCount`
2. `strongestBasicChallengeBits`
3. `questionSetReady`
4. `recommendedActions`
5. boundary flags set to `false`
