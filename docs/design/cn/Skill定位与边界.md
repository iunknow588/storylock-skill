# StoryLock Skill 定位与边界设计

## 目的

本文档用于回答以下问题：

1. 当前 StoryLock 系统有哪些 Skill
2. 每个 Skill 分别负责什么
3. 每个 Skill 应该在什么位置运行
4. 哪些责任属于 Skill 层、Host 层、Storage 层
5. 多个 Skill 如何组装成完整功能

本文档对应当前代码包：

1. `src/storylock-skill-engine/`

## 设计原则

在定义 StoryLock Skill 时，采用以下原则：

1. 按能力定义 Skill，不按存储位置定义 Skill
2. Skill 契约与安全边界分离
3. 敏感秘密访问必须放在 Host 控制接口之后
4. 远程安全文本处理与本地敏感处理分离
5. 编排层在单个 Skill 之上，不等于某个具体 Skill

## 三层边界模型

StoryLock 建议拆成三层：

### 1. Skill 层

负责：

1. 自说明
2. 输入输出契约
3. 调用说明
4. 面向编排的结构化结果

不负责：

1. 自己定义最终信任边界
2. 存储明文秘密
3. 取代 Host 侧授权控制

### 2. Host 层

负责：

1. `createChallenge(identityId, scope)`
2. `submitChallengeAnswers(identityId, challengeId, answers)`
3. `readSecretObject(identityId, sessionId, secretObjectId)`
4. signer 注入与签名密钥使用

这一层才是真正的敏感执行边界。

### 3. Storage 层

负责：

1. 保存加密对象
2. 提供对象定位和访问控制
3. 支持本地、云端、合约等不同存储形态

存储位置不会改变 Skill 接口，但会改变安全模型。

## 系统 Skill 总表

| Skill 名称 | 主类 | 功能 | 敏感数据暴露程度 | 推荐运行位置 | 是否依赖 Host | 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| 故事草稿生成 | `StoryDraftAssistSkill` | 根据结构化提示生成或整理故事草稿 | 低到中，取决于输入 | 通用提示可远程；私密记忆输入应本地 | 否 | 文本生成能力 |
| 故事润色 | `StoryRefineAssistSkill` | 根据目标对现有故事草稿进行改写与优化 | 低到中，取决于草稿内容 | 脱敏文本可远程；私密草稿应本地 | 否 | 与故事草稿属于同一能力族 |
| 强度评估 | `StrengthReviewSkill` | 评估题集是否满足 StoryLock 就绪条件并给出建议 | 中 | 视数据内容可本地或远程 | 否 | 偏确定性分析 |
| 登录授权 | `LoginAuthorizationSkill` | 创建挑战、提交答案并返回登录授权包 | 高 | 仅本地 | 是 | 内部授权能力 |
| 登录字段填充 | `LocalPasswordFillSkill` | 在授权后输出站点登录字段结果 | 高 | 仅本地 | 是 | 面向场景的封装能力 |
| 签名授权 | `SigningAuthorizationSkill` | 生成签名授权包，并可选访问签名材料 | 高 | 仅本地 | 是 | 内部签名能力 |
| 挑战签名 | `ChallengeSigningAuthorizationSkill` | 在授权后输出挑战签名结果 | 高 | 仅本地 | 是 | 面向场景的封装能力 |
| Agent 编排示例 | `VideoPublishAgentDemo` | 把登录和签名结果串成发布计划风格流程 | 混合 | 混合 | 是 | 编排示例，不是核心 Skill |

## Skill 分类

当前系统里的 Skill 建议分成三类。

### A. 远程安全 Skill

在输入已经安全的前提下，这些 Skill 可以远程运行：

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. `StrengthReviewSkill`

条件：

1. 输入中不能包含原始秘密
2. 输入中不能包含受保护的私密记忆，除非远程运行环境本身可信

### B. 本地专属 Skill

这些 Skill 必须放在本地或 Host 控制边界之后：

1. `LoginAuthorizationSkill`
2. `LocalPasswordFillSkill`
3. `SigningAuthorizationSkill`
4. `ChallengeSigningAuthorizationSkill`

原因：

1. 它们直接依赖 challenge 流程
2. 它们依赖 session 流程
3. 它们依赖 secret object 读取
4. 它们依赖签名密钥或等价的高敏感材料

### C. 混合编排 Skill

这类流程把远程安全规划和本地敏感执行混合在一起：

1. `VideoPublishAgentDemo`
2. 未来可能出现的“私密记忆本地处理 + 远程脱敏润色”故事流

## 推荐的 Agent 划分

系统建议拆成两个协作的 Agent 或运行时。

### 远程 Agent

负责：

1. 识别用户意图
2. 选择正确的 Skill
3. 生成结构化请求
4. 包装本地结果为用户可读输出
5. 处理远程安全文本能力

不应负责：

1. 长期持有原始秘密
2. 绕过 Host 授权 API
3. 直接执行本地专属的 secret read

### 本地 Agent / 本地 Host 运行时

负责：

1. 创建 challenge
2. 提交答案
3. 读取 secret object
4. 使用签名密钥
5. 执行本地专属的 password-fill 与 challenge-sign 流程

应返回：

1. 结构化授权结果
2. 结构化字段包
3. 结构化签名结果

### Agent 关系

推荐关系如下：

1. 远程 Agent 负责路由与编排
2. 本地 Agent 或本地 Host 负责敏感执行
3. 远程 Agent 负责包装本地结果，形成最终功能输出

## 功能组装方式

多个 Skill 应按功能进行组装。

### 功能一：故事草稿工作流

可能由以下能力组成：

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. 可选的本地私密记忆预处理

组装逻辑：

1. 准备提示上下文
2. 生成故事草稿
3. 根据目标进行润色
4. 输出前可选再做一次本地隐私审查

### 功能二：StoryLock 就绪度评估

组成：

1. `StrengthReviewSkill`

组装逻辑：

1. 收集 24 题题集
2. 评估是否满足要求
3. 返回问题与建议

### 功能三：登录字段填充流程

组成：

1. `LoginAuthorizationSkill`
2. `LocalPasswordFillSkill`

组装逻辑：

1. 解析站点绑定关系
2. 创建 challenge
3. 提交答案
4. 获取 session
5. 读取授权后的秘密对象
6. 打包成登录字段结果

### 功能四：挑战签名流程

组成：

1. `SigningAuthorizationSkill`
2. `ChallengeSigningAuthorizationSkill`

组装逻辑：

1. 解析签名秘密引用
2. 创建 challenge
3. 提交答案
4. 获取 session
5. 读取签名材料
6. 调用 signer
7. 返回结构化签名结果

### 功能五：多步骤 Agent 工作流

组成：

1. `LocalPasswordFillSkill`
2. `ChallengeSigningAuthorizationSkill`
3. `VideoPublishAgentDemo`

组装逻辑：

1. 远程 Agent 选择流程
2. 本地运行时执行敏感 Skill
3. 编排层把多个结果组合成发布计划

## 实用边界规则

在设计评审时，可以使用如下规则：

1. 如果一个能力需要 challenge answers、session material、secret reads 或 signing-key access，那么它必须是本地专属
2. 如果一个能力只处理已安全的文本，或只分析非秘密结构化数据，那么它可以是远程安全
3. 如果一个功能同时包含两类步骤，那么应拆成“远程编排部分 + 本地执行部分”

## 下一步建议

下一步应继续形式化以下内容：

1. 哪些请求先进入远程 Agent
2. 哪些请求必须交给本地运行时
3. 远程 Agent 与本地 Agent 之间的结构化交接契约
4. 访问门槛、会话与远程保留规则，见 `storylock_object_access_policy_cn.md`

## 最小代码映射示例

### Skill 层示例

```ts
export class StoryDraftAssistSkill {
  async invoke(input) {
    return {
      capability: "storyDraftAssist",
      result: {
        draft: "...",
      },
    };
  }
}
```

### Host 层示例

```ts
export interface StoryLockHost {
  createChallenge(identityId: string, scope: string): Promise<object>;
  submitChallengeAnswers(identityId: string, challengeId: string, answers: object[]): Promise<object>;
  readSecretObject(identityId: string, sessionId: string, secretObjectId: string): Promise<object>;
}
```

### Storage 层示例

```ts
export interface SecretObjectStore {
  read(secretObjectId: string): Promise<Uint8Array | null>;
  write(secretObjectId: string, payload: Uint8Array): Promise<void>;
}
```

这三个层次的关系应始终保持为：

1. Skill 层定义能力与输入输出
2. Host 层定义敏感执行边界
3. Storage 层定义持久化与对象定位
