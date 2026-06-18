# StoryLock 系统 Skill 表与能力边界

## 当前代码基线

本文档按当前 `skill/src` 代码整理，不再沿用旧评审中的历史接口口径。

## 第一层：本地故事处理

包：`storylock-local-story-processing-skill`

| 能力 | 类 | 当前状态 | 边界 |
| --- | --- | --- | --- |
| 故事草稿 | `StoryDraftSkill` | 已实现并自测 | 只处理本地输入，不创建授权 |
| 故事润色 | `StoryRefineSkill` | 已实现并自测 | 只处理本地输入，不读取 secret |
| 题集强度评估 | `StrengthReviewSkill` | 已实现并自测 | 评估 24 题题集，不签发 session |

## 第二层：本地授权

包：`storylock-local-story-access-skill`

| 能力 | 类 | 当前状态 | 边界 |
| --- | --- | --- | --- |
| 对象强度策略 | `ObjectStrengthPolicySkill` | 已实现并自测 | 判断 low/medium/high 与九宫格策略 |
| 九宫格验证 | `GridChallengeSkill` | 已实现并自测 | 生成验证、记录 requestId/nonce、防重放 |
| 本地授权 | `LocalAuthorizationSkill` | 已实现并自测 | 校验答案并返回短时授权结果 |

第二层不再提供故事读取、故事写回或受保护内容持久化接口。

## 第三层：远程网关

包：`storylock-remote-gateway-skill`

| 能力 | 接口/类 | 当前状态 | 边界 |
| --- | --- | --- | --- |
| 远程签名请求 | `requestSignature` | 已实现并自测 | 包装 EIP-712 请求，可选调用 `signatureExecutor` |
| Web2 密码填充请求 | `requestPasswordFill` | 已实现并自测 | 包装本地密码填充请求，默认不返回明文密码 |
| 委托签名 Skill | `DelegatedSignatureSkill` | 已实现 | 通过网关请求签名 |

第三层会统一脱敏返回值，不暴露私钥、密码、答案或 signing key bytes。

## 兼容包

包：`storylock-skill-engine`

当前作为兼容与演示包使用，不是新的安全层。顶层导出仅保留：

1. `LocalPasswordFillSkill`
2. `LOGIN_BINDING_MODE`
3. `SignatureAuthorizationSkill`

`assets/migrated/` 下仍可能保留旧实现文件或旧类名，但不作为当前主入口。

## 当前对外主接口

1. `requestSignature`
2. `requestPasswordFill`

## 当前内部主能力

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`
4. `ObjectStrengthPolicySkill`
5. `GridChallengeSkill`
6. `LocalAuthorizationSkill`

## 不再作为当前主线的能力

1. 已废弃旧接口 `requestStoryRead`
2. 已废弃旧接口 `requestStoryWrite`
3. 已废弃旧接口 `requestChallengeSign`
4. 已废弃旧接口 `requestCapabilityStatus`
5. 已废弃旧接口 `requestLocalStoryAssist`
6. 已废弃旧接口 `queryStoryMetadata`
7. 已废弃第二层故事对象读写

## 后续设计原则

1. 新文档和新代码不得再把故事读写作为第二层职责。
2. 新网关能力必须进入白名单后才能对外暴露。
3. 签名能力统一使用 `requestSignature` 与 `SignatureAuthorizationSkill` 口径。
4. 题集强度评估归第一层。
5. 单故事终身制、题集版本管理和真实九宫格题目选择作为后续增强，不应伪装成已完成能力。
