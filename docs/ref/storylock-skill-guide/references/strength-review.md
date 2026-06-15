# StoryLock Strength Review

## Overview

`StrengthReviewSkill` 用于评估题集配置是否达到 StoryLock 的基本要求，并输出可执行的改进建议。

它调用的是迁移包中的分析逻辑，不直接暴露授权结果，也不替代 StoryLock Core 的正式安全判断。

## Invocation Template

```js
import { StrengthReviewSkill } from "./index.js";

const skill = new StrengthReviewSkill();

const result = await skill.run({
  questions: [
    {
      id: "q1",
      prompt: "你小学班主任的姓？",
      formalEligible: true,
      estimatedChallengeBits: 14,
    },
  ],
});
```

如果需要对接自定义分析器，也可以注入：

```js
const skill = new StrengthReviewSkill({
  analyzer: async ({ questions }) => ({
    formalEligibleQuestionCount: questions.length,
    requiredBasicQuestionCount: 6,
    strongestBasicChallengeBits: 18,
    basicMinChallengeBits: 12,
    strongestBasicChallengeMeetsMinimum: true,
    rootKdfMeetsProductionMinimum: true,
    answerKdfMeetsProductionMinimum: true,
  }),
});
```

## Parameters

### `StrengthReviewSkill.run({ questions })`

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `questions` | array | 是 | - | 题集配置数组，不能为空 |

### Constructor dependencies

| 依赖 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `analyzer` | function | 否 | 内置 `defaultAnalyzeStrength` | 自定义强度分析器 |

## Output

返回对象会在分析结果基础上补充两个关键字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `questionSetReady` | boolean | 是否达到基本可用状态 |
| `recommendedActions` | string[] | 后续改进建议列表 |

常见原始字段还包括：

| 字段 | 说明 |
| --- | --- |
| `formalEligibleQuestionCount` | 当前正式可用题数 |
| `requiredBasicQuestionCount` | 基本要求的题数 |
| `strongestBasicChallengeBits` | 当前最强题目的挑战强度 |
| `basicMinChallengeBits` | 最低要求的挑战强度 |
| `rootKdfMeetsProductionMinimum` | 根密钥 KDF 是否达标 |
| `answerKdfMeetsProductionMinimum` | 答案 KDF 是否达标 |

## Error Handling

| 错误 | 触发条件 | 修复建议 |
| --- | --- | --- |
| `ValidationError: analyzer must be a function` | 构造函数注入了错误类型 | 传入函数 |
| `ValidationError: questions must be a non-empty array` | 题集为空或非数组 | 传入至少一个问题 |

## Agent Guidelines

1. 用户问“题集是否够强”“是否达标”“还差什么”时，优先使用这项能力。
2. 始终把 `recommendedActions` 翻译成可执行结论，不只复述布尔值。
3. 不要把 `questionSetReady` 说成正式的生产授权结论，它只是题集层面的准备度。
4. 若用户要比赛表述，优先强调“可评估、可解释、可给出整改建议”。
