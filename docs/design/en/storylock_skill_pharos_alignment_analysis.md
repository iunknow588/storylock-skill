# StoryLock Skill 开发分析：对照 Pharos PiggyBank 教程

## 核心结论

**StoryLock 当前代码已实现 Skill 的核心逻辑层，但尚未完成 Pharos 标准要求的 Skill 包结构。** 主要差距在于：缺乏标准化的 `SKILL.md` 元数据、Capability Index、以及面向 AI Agent 的参考文件结构。

---

## 一、Pharos Skill 的标准结构（基于 PiggyBank 教程）

根据 Pharos 官方教程，一个完整的 Skill 必须包含以下结构：

```
skill-package/
├── SKILL.md                    ← 入口点：能力索引 + 安全边界 + 触发条件
├── assets/                     ← AI 直接使用的资源文件
│   ├── networks.json           ← 网络配置（RPC、Chain ID、浏览器）
│   ├── tokens.json             ← 已知代币注册表
│   └── <feature>/              ← 功能相关资源（合约、模板、配置）
│       └── <Contract>.sol      ← 智能合约源码
└── references/                 ← AI 指令手册（机器可读）
    ├── query.md                ← 查询操作规范
    ├── transaction.md          ← 交易操作规范
    ├── contract.md             ← 合约部署规范
    └── <feature>.md            ← 功能特定操作规范
```

### 关键组件要求

| 组件 | 用途 | 必须包含 |
|------|------|---------|
| **SKILL.md** | AI 读取的第一个文件 | ✅ 能力索引表、安全边界、触发条件 |
| **assets/** | 合约、配置、模板 | ✅ 网络配置、合约源码、脚本模板 |
| **references/** | 详细操作指令 | ✅ 命令模板、参数表、输出解析、错误处理、Agent 指南 |
| **Capability Index** | 用户意图 → 参考文件映射 | ✅ 意图、能力、参考文件、关键参数 |
| **Agent Guidelines** | AI 执行 checklist | ✅ 有序步骤、预检要求、输出格式 |

---

## 二、StoryLock 当前代码分析

### 2.1 已实现的 Skill 逻辑层（✅ 完成）

从代码审查可见，StoryLock 已实现 7 个 Skill 的完整 JavaScript 类：

| Skill 类 | 文件路径 | 状态 |
|---------|---------|------|
| `StoryDraftAssistSkill` | `skills/story-assist.js` | ✅ 完整实现 |
| `StoryRefineAssistSkill` | `skills/story-assist.js` | ✅ 完整实现 |
| `StrengthReviewSkill` | `skills/strength-review.js` | ✅ 完整实现 |
| `LoginAuthorizationSkill` | `skills/authorization-skills.js` | ✅ 完整实现 |
| `LocalPasswordFillSkill` | `skills/authorization-skills.js` | ✅ 完整实现 |
| `SigningAuthorizationSkill` | `skills/authorization-skills.js` | ✅ 完整实现 |
| `ChallengeSigningAuthorizationSkill` | `skills/authorization-skills.js` | ✅ 完整实现 |

**代码特征：**
- 每个 Skill 都有 `skillId()` 方法返回唯一标识
- 每个 Skill 都有 `run(input)` 异步方法处理输入输出
- 输入验证完整（`ensureNonEmptyString`、`ensureFunction` 等）
- 依赖注入清晰（`host`、`signer` 通过构造函数传入）

### 2.2 已实现的文档层（⚠️ 部分完成）

当前文档结构：

```
docs/ref/
├── 01-参赛概览.md              ← 项目介绍（人类可读）
├── 02-技术映射说明.md          ← 技术映射（人类可读）
├── 03-演示与调用说明.md        ← 演示说明（人类可读）
├── 04-提交材料清单.md          ← 提交清单（人类可读）
├── README.md                   ← 目录说明（人类可读）
└── storylock-skill-guide/
    ├── SKILL.md                ← 简化版 Skill 元数据
    └── references/
        ├── boundary.md         ← 边界说明（人类可读）
        ├── demo.md            ← 演示说明（人类可读）
        └── invocation.md      ← 调用示例（半结构化）
```

---

## 三、与 Pharos 标准的差距分析

### 3.1 关键缺失项

| 缺失项 | Pharos 要求 | StoryLock 现状 | 影响 |
|--------|------------|---------------|------|
| **Capability Index** | `SKILL.md` 中必须有用户意图 → 参考文件的映射表 | 无显式索引 | AI 无法自动定位用户请求对应的 Skill |
| **机器可读参考文件** | 命令模板 + 参数表 + 输出解析 + 错误处理 + Agent 指南 | 仅有调用示例和边界说明 | AI 无法精确执行操作 |
| **assets/ 网络配置** | `networks.json` 包含 RPC、Chain ID、浏览器 URL | 无 | 无法支持多网络部署 |
| **写操作预检** | 每笔交易前必须通过 4 项检查 | 仅有边界说明 | 缺乏标准化安全流程 |
| **Agent Guidelines** | 每个操作的有序 checklist | 无 | AI 执行顺序不确定 |
| **JSON Schema** | 输入输出的结构化定义 | 无 | 无法做自动化验证 |

### 3.2 具体差距示例

**Pharos 标准（PiggyBank）：**

```markdown
## Deploy SimpleVault

### Command Template
```bash
forge script script/DeploySimpleVault.s.sol:DeploySimpleVault \
  --rpc-url <rpc> \
  --private-key $PRIVATE_KEY \
  --broadcast
```

### Parameters
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lockDurationSeconds` | uint256 | Yes | Lock duration in seconds |
| `--rpc-url` | string | Yes | RPC endpoint URL |

### Output Parsing
| Field | Description |
|-------|-------------|
| `Vault address:` | Deployed contract address |

### Error Handling
| Error | Cause | Fix |
|-------|-------|-----|
| `compiler error` | Compilation failed | Check source file path |

> **Agent Guidelines:**
> 1. Complete "Write Operation Pre-checks"
> 2. Ask user for `lockDurationSeconds`
> 3. Copy contract to user's project
> 4. Check deployer balance
> 5. Generate deploy script
> 6. Execute `forge script`
> 7. Extract address, show explorer link
```

**StoryLock 当前（invocation.md）：**

```markdown
## 2.3 登录填充

`LocalPasswordFillSkill` 面向"本地授权后输出登录字段"的产品化模式。

构造依赖：
```js
const host = { ... };
```

调用示例：
```js
const skill = new LocalPasswordFillSkill({ host });
const result = await skill.run({ ... });
```

输出重点：
- `mode`
- `scope`
- `challenge`
- `authorization`
- `fields`
```

**差距：** 缺乏命令模板、参数表、错误处理、Agent Guidelines。

---

## 四、StoryLock 是否"实现了 Skill 开发"？

### 结论：部分实现

| 层面 | 状态 | 说明 |
|------|------|------|
| **Skill 逻辑层** | ✅ 已实现 | 7 个 Skill 类，完整输入输出验证 |
| **Skill 接口层** | ⚠️ 基本实现 | 有 `skillId()` 和 `run()`，但缺乏标准化 |
| **Skill 元数据层** | ❌ 未实现 | 无 Capability Index、无触发条件声明 |
| **AI 参考文件层** | ❌ 未实现 | 无机器可读的操作指令 |
| **资源层** | ❌ 未实现 | 无 `assets/` 目录、无网络配置 |

### 核心问题

StoryLock 当前实现的是**面向开发者的 SDK/库**，而非**面向 AI Agent 的 Skill 包**。

- **SDK 模式**：开发者阅读文档，手动调用 JavaScript 类
- **Skill 模式**：AI 阅读 `SKILL.md`，自动理解能力并执行操作

---

## 五、如何完成 Skill 开发（建议实施路径）

### 阶段一：立即完成（比赛前必须）

#### 1. 重构 `SKILL.md` 为标准格式

```markdown
---
name: storylock-skill-guide
description: |
  StoryLock 本地授权 Skill 包：将基于故事记忆的本地验证
  封装为可复用的登录填充、挑战签名与故事辅助能力。
  
  使用前提：宿主提供 StoryLock Core 接口（createChallenge/submitChallengeAnswers/readSecretObject）
  安全边界：Skill 层不替代 Core 授权判断，不修改底层授权语义
---

# StoryLock Skill

## Capability Index

| 用户意图 | 能力 | 参考文件 | 关键参数 |
|---------|------|---------|---------|
| "帮我生成故事草稿" | 故事草稿生成 | `references/story-assist.md` | `objective`, `audience`, `tone` |
| "润色这个故事" | 故事润色 | `references/story-assist.md` | `storyDraft`, `goals` |
| "评估题集强度" | 强度评估 | `references/strength-review.md` | `questions` |
| "我需要登录字段" | 登录填充 | `references/password-fill.md` | `identityId`, `siteId`, `answers` |
| "签名这个挑战" | 挑战签名 | `references/challenge-sign.md` | `identityId`, `keyId`, `algorithm`, `challengeCode` |

## Asset Index

| 资源 | 路径 | 说明 |
|------|------|------|
| 故事模板 | `assets/templates/story-template.json` | 故事草稿生成模板 |
| 题集评估配置 | `assets/schemas/strength-review-schema.json` | 强度评估输入输出 Schema |

## Write Operation Pre-checks

所有涉及授权的操作（登录填充、挑战签名）必须通过以下检查：

1. **身份验证**：`identityId` 对应的挑战是否已创建？
2. **答案提交**：`answers` 是否已通过验证？
3. **会话有效**：返回的 `sessionId` 是否在有效期内？
4. **授权范围**：`scope` 是否包含目标资源？
```

#### 2. 创建标准化参考文件

将 `references/invocation.md` 拆分为 4 个独立文件：

```
references/
├── story-assist.md          ← 故事辅助类 Skill 规范
├── strength-review.md       ← 强度评估规范
├── password-fill.md         ← 登录填充规范（含预检）
└── challenge-sign.md        ← 挑战签名规范（含预检）
```

每个文件必须包含：
1. **Command Template** — JavaScript 调用模板
2. **Parameters** — 输入参数表（名、类型、必填、默认值、说明）
3. **Output Parsing** — 返回对象结构、字段含义
4. **Error Handling** — 错误码、触发条件、修复建议
5. **Agent Guidelines** — AI 执行 checklist

**示例（password-fill.md）：**

```markdown
# LocalPasswordFillSkill 操作规范

## 概述

本地授权后输出结构化登录字段。必须先完成挑战创建和答案提交，才能获取登录字段。

## 前提条件

- 宿主已实现 `host.createChallenge()`
- 宿主已实现 `host.submitChallengeAnswers()`
- 宿主已实现 `host.readSecretObject()`

## 调用模板

```js
import { LocalPasswordFillSkill } from "story-lock/skills";

const skill = new LocalPasswordFillSkill({ host });
const result = await skill.run({
  identityId: "<identity_id>",
  siteId: "<site_id>",
  bindingMode: "template_only",
  resourceCatalog: <resource_catalog>,
  answers: [],
});
```

## 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `identityId` | string | 是 | - | 身份标识符 |
| `siteId` | string | 是 | - | 站点标识符 |
| `bindingMode` | string | 否 | `"template_only"` | 绑定模式：template_only / bound |
| `resourceCatalog` | object | 是 | - | 资源目录 |
| `answers` | string[] | 否 | `[]` | 挑战答案 |

## 输出解析

| 字段 | 类型 | 说明 |
|------|------|------|
| `mode` | string | 固定值 `"local_password_fill"` |
| `siteId` | string | 站点标识符 |
| `scope` | string | 授权范围：vault_read_basic / vault_read_batch |
| `challenge` | object | 挑战对象（challengeId, questions, expiresAt） |
| `authorization` | object | 授权结果（sessionId, grantedAt, expiresAt） |
| `fields` | array | 登录字段列表，每项包含 fieldName, value, secretObjectId |

## 错误处理

| 错误 | 触发条件 | 修复建议 |
|------|---------|---------|
| `ValidationError: identityId must be a non-empty string` | identityId 为空 | 提供有效的身份标识符 |
| `ValidationError: host.createChallenge must be a function` | 宿主未实现接口 | 检查宿主是否完整实现 Core 接口 |
| `AuthorizationError: session expired` | 授权会话已过期 | 重新提交挑战答案 |

## Agent Guidelines

1. 确认用户提供了 `identityId` 和 `siteId`
2. 检查 `answers` 是否已提供；若未提供，引导用户完成挑战
3. 调用 `skill.run()` 获取登录字段
4. 验证返回的 `fields` 非空
5. 提醒用户 `authorization.expiresAt`，建议及时使用
6. 不要缓存或持久化 `fields` 中的敏感值
```

#### 3. 创建 `assets/` 目录

```
assets/
├── schemas/
│   ├── password-fill-input.schema.json
│   ├── password-fill-output.schema.json
│   ├── challenge-sign-input.schema.json
│   └── challenge-sign-output.schema.json
└── templates/
    └── story-template.json
```

### 阶段二：比赛前优化（提升竞争力）

#### 4. 增加 `agents/openai.yaml`

```yaml
name: storylock-skill-guide
description: StoryLock 本地授权 Skill 包
capabilities:
  - story_draft
  - story_refine
  - strength_review
  - password_fill
  - challenge_sign
triggers:
  - "storylock"
  - "本地授权"
  - "登录填充"
  - "挑战签名"
  - "故事辅助"
  - "题集评估"
```

#### 5. 增加 `llms.txt` 索引

```
# StoryLock Skill 文档索引

## 核心入口
- SKILL.md: Skill 元数据、能力索引、安全边界
- references/capability-index.md: 能力索引表

## 参考文档
- references/story-assist.md: 故事草稿与润色
- references/strength-review.md: 题集强度评估
- references/password-fill.md: 登录字段填充
- references/challenge-sign.md: 挑战签名

## 设计依据
- doc/design/01-系统分析说明书.md: 正式系统边界
```

### 阶段三：比赛后完善

#### 6. 增加自动化测试
- 为每个 Skill 编写单元测试
- 验证输入输出符合 JSON Schema

#### 7. 增加示例项目
- 提供完整的示例项目，展示如何接入 StoryLock Skill

---

## 六、关键判断：StoryLock 是否适合参赛？

### 优势

1. **核心逻辑完整**：7 个 Skill 类已实现，输入输出验证完善
2. **安全边界清晰**：Skill 层与 Core 层分离，不替代授权判断
3. **演示路径明确**：有命令行 Demo 和浏览器 Demo

### 劣势

1. **Skill 包结构不完整**：缺乏 Pharos 标准要求的 Capability Index、Agent Guidelines、机器可读参考文件
2. **AI 友好度不足**：当前文档面向人类开发者，而非 AI Agent
3. **资源层缺失**：无 `assets/` 目录、无网络配置、无 JSON Schema

### 建议

**如果比赛评审标准重视"AI 可直接调用的 Skill 包"**：
- 必须在比赛前完成阶段一（重构 SKILL.md、创建标准化参考文件、建立 assets/ 目录）
- 这是从"SDK"到"Skill"的关键转变

**如果比赛评审标准更重视"创新性和功能完整性"**：
- 当前代码已具备参赛基础
- 但文档优化（已完成）和 Skill 包结构化（建议完成）将显著提升竞争力

---

## 七、实施优先级

| 优先级 | 任务 | 预计时间 | 影响 |
|--------|------|---------|------|
| P0 | 重构 `SKILL.md` 为标准格式（含 Capability Index） | 2 小时 | 核心入口 |
| P0 | 拆分 `invocation.md` 为 4 个标准化参考文件 | 4 小时 | AI 可读性 |
| P1 | 创建 `assets/schemas/` 目录 | 2 小时 | 输入输出验证 |
| P1 | 增加 `agents/openai.yaml` | 30 分钟 | Agent 配置 |
| P2 | 增加 `llms.txt` | 30 分钟 | 文档索引 |
| P2 | 编写示例项目 | 4 小时 | 演示价值 |

**总计：约 13 小时可完成核心改造。**
