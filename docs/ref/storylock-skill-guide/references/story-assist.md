# StoryLock Story Assist

## Overview

这一组能力覆盖两类故事辅助接口：

1. `StoryDraftAssistSkill`：根据目标、受众、语气和约束生成故事草稿
2. `StoryRefineAssistSkill`：基于已有故事草稿给出收束、强化或改写建议

这两类能力属于上层辅助接口，不触发 StoryLock 本地授权流程，也不直接依赖 Rust / WASM 核心。

## Invocation Template

### Draft

```js
import { StoryDraftAssistSkill } from "./index.js";

const skill = new StoryDraftAssistSkill({
  generator: async ({ objective, audience, tone, constraints }) => ({
    objective,
    audience,
    tone,
    constraints,
    draft: `面向 ${audience} 的故事草稿：${objective}`,
  }),
});

const result = await skill.run({
  objective: "说明 StoryLock 为什么适合本地授权场景",
  audience: "Pharos 评委",
  tone: "简洁、可信、可演示",
  constraints: ["突出本地授权", "不要夸大 Rust 集成进度"],
});
```

### Refine

```js
import { StoryRefineAssistSkill } from "./index.js";

const skill = new StoryRefineAssistSkill({
  refiner: async ({ storyDraft, goals, hintStyle }) => ({
    storyDraft,
    goals,
    hintStyle,
    revisions: [
      "把开头先换成用户痛点",
      "把本地授权和登录填充的关系说得更直白",
    ],
  }),
});

const result = await skill.run({
  storyDraft: "当前草稿文本",
  goals: ["更像产品演示", "减少概念堆叠"],
  hintStyle: "short mnemonic cues",
});
```

## Parameters

### `StoryDraftAssistSkill.run(input)`

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `objective` | string | 是 | - | 这次故事输出要达到的目标 |
| `audience` | string | 否 | `individual founder` | 目标受众 |
| `tone` | string | 否 | `memorable and concrete` | 输出风格 |
| `constraints` | string[] | 否 | `[]` | 需要遵守的约束条件 |

### `StoryRefineAssistSkill.run(input)`

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `storyDraft` | string | 是 | - | 已有草稿 |
| `goals` | string[] | 否 | `[]` | 希望优化的方向 |
| `hintStyle` | string | 否 | `short mnemonic cues` | 建议表达风格 |

### Constructor dependencies

| 依赖 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `generator` | function | Draft 必填 | 负责真正生成草稿的函数 |
| `refiner` | function | Refine 必填 | 负责真正执行润色的函数 |

## Output

这两个 Skill 不强制固定业务字段，只保证：

1. 调用时会先做基础输入校验
2. 通过校验后，直接返回 `generator` / `refiner` 的结果

因此输出 shape 由调用方注入函数定义，但建议至少包含：

| 字段 | 说明 |
| --- | --- |
| `objective` / `storyDraft` | 输入上下文回显 |
| `audience` / `goals` | 便于调试和审阅 |
| `draft` / `revisions` | 主要产出内容 |

## Error Handling

常见错误：

| 错误 | 触发条件 | 修复建议 |
| --- | --- | --- |
| `ValidationError: generator must be a function` | Draft 构造函数未传函数 | 注入实际草稿生成函数 |
| `ValidationError: refiner must be a function` | Refine 构造函数未传函数 | 注入实际润色函数 |
| `ValidationError: objective must be a non-empty string` | `objective` 为空 | 提供明确目标 |
| `ValidationError: storyDraft must be a non-empty string` | `storyDraft` 为空 | 传入已有草稿文本 |

## Agent Guidelines

1. 先判断用户要“生成草稿”还是“润色草稿”。
2. 生成草稿时，优先补齐 `objective`，其次再问 `audience`、`tone`、`constraints`。
3. 润色草稿时，确保 `storyDraft` 已存在，再收集 `goals`。
4. 不要把这组能力描述成安全边界或授权能力。
5. 面向比赛文档时，优先把输出组织成可演示、可复述的短结构。
