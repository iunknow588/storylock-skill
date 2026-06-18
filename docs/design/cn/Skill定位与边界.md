# StoryLock Skill 定位与边界

本文用于说明 StoryLock 当前代码中的 Skill 如何分层、各层承担什么责任，以及哪些能力不应越界。

## 当前实现包

| 包 | 层级 | 当前已实现能力 |
| --- | --- | --- |
| `storylock-local-story-processing-skill` | 第一层 | `StoryDraftSkill`、`StoryRefineSkill`、`StrengthReviewSkill` |
| `storylock-local-story-access-skill` | 第二层 | `ObjectStrengthPolicySkill`、`GridChallengeSkill`、`LocalAuthorizationSkill` |
| `storylock-remote-gateway-skill` | 第三层 | `requestSignature`、`requestPasswordFill` |
| `storylock-skill-engine` | 兼容演示包 | `LocalPasswordFillSkill`、`SignatureAuthorizationSkill` |

`storylock-skill-engine` 用于兼容和演示本地执行路径，不作为第四个安全层。

## 三层边界

### 第一层：本地故事处理与强度分析

负责：

1. 生成故事草稿。
2. 对故事内容做润色和整理。
3. 评估题集或问题集合的强度。

不负责：

1. 读取受保护对象。
2. 使用签名密钥。
3. 填充 Web2 密码。
4. 签发本地授权。

### 第二层：对象强度、九宫格验证与本地授权

负责：

1. 根据目标对象判断需要的访问强度。
2. 根据强度生成九宫格验证。
3. 校验答案并生成短时授权结果。
4. 记录 session、防重放、失败窗口、答案摘要与审计日志。

不负责：

1. 故事对象读取或写回。
2. 远程请求编排。
3. 直接返回长期密钥或长期凭据。

### 第三层：远程请求包装与代理授权入口

负责：

1. 接收远程侧的签名或密码填充请求。
2. 将请求包装成可审计、可脱敏、可转交本地执行的结构。
3. 调用可选的本地执行器：`signatureExecutor` 或 `passwordFillExecutor`。
4. 对返回结果执行递归脱敏。

当前对外主接口：

1. `requestSignature`
2. `requestPasswordFill`

不负责：

1. 读取故事内容。
2. 生成故事内容。
3. 直接保存或使用长期私钥。
4. 绕过第二层授权。

## Skill 清单

| 能力 | 当前名称 | 所在包 | 说明 |
| --- | --- | --- | --- |
| 故事草稿 | `StoryDraftSkill` | 第一层 | 根据结构化输入生成草稿 |
| 故事润色 | `StoryRefineSkill` | 第一层 | 对已有草稿做改写和整理 |
| 强度评估 | `StrengthReviewSkill` | 第一层 | 评估题集强度并给出建议 |
| 对象强度策略 | `ObjectStrengthPolicySkill` | 第二层 | 判断对象需要的验证强度 |
| 九宫格验证 | `GridChallengeSkill` | 第二层 | 生成、提交和验证九宫格答案 |
| 本地授权 | `LocalAuthorizationSkill` | 第二层 | 返回短时授权，不返回长期秘密 |
| 远程签名请求 | `requestSignature` | 第三层 | 包装 EIP-712 风格签名请求 |
| 远程密码填充请求 | `requestPasswordFill` | 第三层 | 包装 Web2 密码填充请求 |
| 本地密码填充 | `LocalPasswordFillSkill` | 兼容演示包 | 本地执行示例 |
| 本地签名授权 | `SignatureAuthorizationSkill` | 兼容演示包 | 本地签名执行示例 |

## 边界判断规则

1. 只处理已脱敏文本或非敏感结构化数据的能力，可以放在第一层或远程侧。
2. 需要本地答案验证、session、nonce、短时授权或审计写入的能力，属于第二层。
3. 需要面向第三方或远程 Agent 暴露入口的能力，属于第三层。
4. 需要真实使用密码、私钥或签名器的能力，必须留在本地执行边界内。
5. 第三层可以请求签名或密码填充，但不能自己保存私钥、密码或九宫格答案。

## 典型组合

### 故事处理

1. `StoryDraftSkill` 生成草稿。
2. `StoryRefineSkill` 做润色。
3. `StrengthReviewSkill` 评估题集质量。

### 签名授权

1. 第三层调用 `requestSignature` 包装请求。
2. 第二层判断目标对象强度并完成九宫格验证。
3. 第二层签发短时授权。
4. 本地签名执行器完成签名。
5. 第三层返回脱敏后的结构化结果。

### Web2 密码填充

1. 第三层调用 `requestPasswordFill` 包装请求。
2. 第二层判断凭据对象强度并完成九宫格验证。
3. 第二层签发短时授权。
4. 本地密码填充执行器完成填充。
5. 第三层返回最小化审计结果。

## 当前已废弃的主线口径

以下名称可能仍出现在历史文档或迁移代码中，但不再作为当前主线接口：

1. 已废弃旧接口 `requestStoryRead`
2. 已废弃旧接口 `requestStoryWrite`
3. 已废弃旧接口 `requestChallengeSign`
4. 已废弃旧接口 `requestCapabilityStatus`
5. 已废弃旧 Skill `StoryReadAccessSkill`
6. 已废弃旧 Skill `StoryWriteAccessSkill`
7. 已废弃旧 Skill `ChallengeSigningAuthorizationSkill`
