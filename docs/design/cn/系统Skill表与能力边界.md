# StoryLock 系统 Skill 表与能力边界

## 文档版本

| 版本 | 日期 | 变更说明 |
| --- | --- | --- |
| v1.0 | 2026-06-16 | 初始版本，定义主 Skill、内部 Skill、编排示例 |
| v1.1 | 2026-06-16 | 明确三层运行结构与三个 Skill 包的映射关系 |

## 目的

本文档优先解决两个问题：

1. 当前系统到底有哪些 Skill
2. 每个 Skill 的边界到底在哪里

后续所有设计都应以这份 Skill 表为基础，包括：

1. 本地 / 远程运行划分
2. 本地 Agent 网关设计
3. 代理签名机制
4. 对象访问策略

## 先给结论

当前 `storylock-skill-engine` 中，建议把导出的能力分成三层：

1. **主 Skill**
   - 面向外部使用
   - 直接对应用户任务或第三方 Skill 任务
2. **内部 Skill**
   - 提供更底层的授权或签名能力
   - 一般不作为第三方的首选入口
3. **编排示例**
   - 用于说明如何把多个 Skill 串成一个流程
   - 不作为核心 Skill 定义本身

## 一、主 Skill 表

这些 Skill 应视为系统的主要能力入口。

| Skill 名称 | 主类 | 输入类型 | 输出类型 | 是否接触敏感对象 | 推荐运行位置 | 边界说明 |
| --- | --- | --- | --- | --- | --- | --- |
| 故事草稿生成 | `StoryDraftAssistSkill` | 结构化提示 | 结构化草稿结果 | 视输入而定 | 脱敏输入可远程；私密输入本地 | 只负责生成/整理文本，不负责秘密访问 |
| 故事润色 | `StoryRefineAssistSkill` | 草稿 + 目标 | 结构化润色结果 | 视输入而定 | 脱敏输入可远程；私密输入本地 | 只负责改写与优化，不负责秘密访问 |
| 强度评估 | `StrengthReviewSkill` | 24题题集 | 就绪度评估结果 | 中 | 视数据可本地或远程 | 只负责分析与建议，不负责授权 |
| 登录字段填充 | `LocalPasswordFillSkill` | 身份、站点、绑定、答案 | 字段填充结果 | 高 | 仅本地 | 负责把授权后的秘密打包成登录字段 |
| 挑战签名 | `ChallengeSigningAuthorizationSkill` | 身份、签名请求、对象引用、答案 | 签名结果 | 很高 | 仅本地 | 负责在本地完成授权与签名 |

## 二、内部 Skill 表

这些 Skill 是实现层能力，应视为内部构件或底层能力。

| Skill 名称 | 主类 | 作用 | 是否应直接暴露给第三方 | 推荐运行位置 | 边界说明 |
| --- | --- | --- | --- | --- | --- |
| 登录授权 | `LoginAuthorizationSkill` | 创建 challenge、提交答案、建立登录访问上下文 | 不建议作为首选入口 | 仅本地 | 它是 password-fill 的底层能力 |
| 签名授权 | `SigningAuthorizationSkill` | 创建签名授权上下文，并读取签名材料 | 不建议作为首选入口 | 仅本地 | 它是 challenge-sign 的底层能力 |

## 三、编排示例表

这些对象不是系统核心 Skill，而是“如何组装 Skill”的示例。

| 名称 | 主类 | 类型 | 作用 | 是否属于核心 Skill |
| --- | --- | --- | --- | --- |
| 发布流程示例 | `VideoPublishAgentDemo` | 编排示例 | 串联登录与签名结果，形成多步骤工作流 | 否 |

## 四、能力边界判定规则

判断一个能力是不是独立 Skill、是不是本地专属，建议按下面规则执行。

### 规则 1：是否直接对应用户任务

如果一个能力直接对应明确任务，可视为主 Skill，例如：

1. 生成故事草稿
2. 评估题集强度
3. 填充登录字段
4. 对挑战进行签名

### 规则 2：是否只是底层授权动作

如果一个能力只是给另一个能力打底，不直接对应用户任务，则优先视为内部 Skill，例如：

1. 登录授权
2. 签名授权

### 规则 3：是否接触 challenge / session / secret read / signing key

如果一个能力涉及以下任一项：

1. challenge answers
2. session material
3. secret-object reads
4. signing-key access

那么它必须是本地专属能力。

### 规则 4：是否只是纯文本处理或非秘密结构化分析

如果一个能力只做：

1. 文本生成
2. 文本改写
3. 非秘密结构化数据分析

则它可以是远程安全能力，但前提是输入本身已脱敏或运行环境可信。

## 五、系统最终 Skill 清单

从系统设计角度，建议把 StoryLock 最终对外能力先固定为以下 5 个：

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. `StrengthReviewSkill`
4. `LocalPasswordFillSkill`
5. `ChallengeSigningAuthorizationSkill`

这 5 个就是后续：

1. 写网关接口
2. 定义对象访问策略
3. 设计代理签名能力
4. 设计远程 / 本地交接契约

时要围绕的主能力集合。

## 六、后续设计应如何依赖这张表

### 对于本地 / 远程划分

先看这张表，再决定：

1. 哪些主 Skill 可以远程运行
2. 哪些主 Skill 只能本地执行

### 对于本地 Agent 网关

网关应优先围绕主 Skill 提供接口，而不是先围绕内部 Skill 提供接口。

建议优先开放：

1. `requestLocalStoryAssist`
2. `requestStrengthReview`
3. `requestPasswordFill`
4. `requestChallengeSign`

### 对于代理签名机制

代理签名不应被视为单独替代所有签名逻辑的黑盒，而应被视为：

1. 对 `ChallengeSigningAuthorizationSkill` 的受控调用方式

## 七、结论

在继续完善系统之前，建议先把能力边界固定如下：

1. **主 Skill：**
   - `StoryDraftAssistSkill`
   - `StoryRefineAssistSkill`
   - `StrengthReviewSkill`
   - `LocalPasswordFillSkill`
   - `ChallengeSigningAuthorizationSkill`
2. **内部 Skill：**
   - `LoginAuthorizationSkill`
   - `SigningAuthorizationSkill`
3. **编排示例：**
   - `VideoPublishAgentDemo`

后续所有网关、策略、协议参考、委托签名设计，都应基于这张 Skill 表展开，而不是再反复变动能力边界。

## 八、与三层运行结构的对应关系

从运行结构角度，这张 Skill 表还可以映射到三层：

1. **第一层：纯本地故事处理 Skill**
   - `StoryDraftAssistSkill`
   - `StoryRefineAssistSkill`
2. **第二层：故事访问 Skill**
   - 当前代码中尚未独立成正式 Story Access Skill
   - 但它的职责边界应与授权读取、受保护对象读写对应
3. **第三层：远程 Skill / 代理授权 Skill**
   - `ChallengeSigningAuthorizationSkill` 的远程委托入口
   - 后续本地 Agent 网关能力

因此，当前 Skill 表是能力边界表；后续还需要把“故事访问 Skill”从概念边界继续细化成正式能力定义。
