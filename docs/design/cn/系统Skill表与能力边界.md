# StoryLock 系统 Skill 表与能力边界

## 当前代码基线

本文档按当前 `skill/src` 代码整理，不再沿用旧评审材料中的历史接口口径。

## 第一层：本地故事处理

目录：`src/skills/local-story-processing`

| 能力 | 类 | 当前状态 | 边界 |
| --- | --- | --- | --- |
| 故事草稿 | `StoryDraftSkill` | 已实现并自测 | 只处理本地输入，不创建授权 |
| 故事润色 | `StoryRefineSkill` | 已实现并自测 | 只处理本地文本，不读取 secret |
| 题集强度评估 | `StrengthReviewSkill` | 已实现并自测 | 评估题集质量，不签发 session |

## 第二层：本地授权

目录：`src/skills/local-story-access`

| 能力 | 类 | 当前状态 | 边界 |
| --- | --- | --- | --- |
| 对象强度策略 | `ObjectStrengthPolicySkill` | 已实现并自测 | 判断 `low` / `medium` / `high` 与 challenge 策略 |
| 九宫格验证 | `GridChallengeSkill` | 已实现并自测 | 创建验证、记录 `requestId` / `nonce`、防重放 |
| 本地授权 | `LocalAuthorizationSkill` | 已实现并自测 | 校验答案并返回短时授权结果 |
| 本地撤销 | `LocalRevocationSkill` | 已实现 | 负责本地授权撤销与失效控制 |

第二层不再承担故事读写职责，也不直接暴露内部 challenge 方法给远程侧。

## 第三层：远程网关

目录：`src/skills/remote-gateway`

| 能力 | 接口 / 类 | 当前状态 | 边界 |
| --- | --- | --- | --- |
| 远程签名请求 | `requestSignature` | 已实现并自测 | 包装 EIP-712 请求，可选调用 `signatureExecutor` |
| Web2 密码填充请求 | `requestPasswordFill` | 已实现并自测 | 包装本地密码填充请求，不返回明文密码 |
| 委托签名 Skill | `DelegatedSignatureSkill` | 已实现 | 通过网关发起签名，不直接持有私钥 |

第三层统一脱敏返回值，不暴露私钥、密码、答案或 `signingKeyBytes`。

## 兼容与演示包

目录：`src/engine`

当前定位：

1. 兼容与演示包
2. 统一导出和示例入口
3. 不构成新的安全层级

它不是新的第四层，也不改变三层主线边界。

## 当前对外主接口

当前主线只保留：

1. `requestSignature`
2. `requestPasswordFill`

## 当前内部主能力

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`
4. `ObjectStrengthPolicySkill`
5. `GridChallengeSkill`
6. `LocalAuthorizationSkill`
7. `LocalRevocationSkill`

## 不再作为当前主线的旧能力口径

以下历史接口或历史表述不再作为当前正式口径：

1. `requestStoryRead`
2. `requestStoryWrite`
3. 历史 challenge 签名接口
4. `requestCapabilityStatus`
5. `requestLocalStoryAssist`
6. `queryStoryMetadata`
7. 把第二层描述成内容存取主层的旧说法

## 后续设计原则

1. 新文档和新代码不得再把故事读写写成第二层职责
2. 新网关能力必须进入白名单后才能对外暴露
3. 签名能力统一使用 `requestSignature` 和 `DelegatedSignatureSkill` 口径
4. 题集强度评估归第一层
5. Android 宿主、Windows 宿主属于平台实现方向，不改变 Skill 分层
