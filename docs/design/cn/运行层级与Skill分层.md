# StoryLock 运行层级与 Skill 分层设计

本文说明 StoryLock 当前主线的三层运行结构。文档以现有 `src` 代码为准。

## 结论

当前主线分为三层：

1. 第一层：本地故事处理与强度分析。
2. 第二层：对象强度、九宫格验证与本地授权。
3. 第三层：远程请求包装与代理授权入口。

其中第二层不再定义为故事读取或写回层。

## 第一层：本地故事处理与强度分析

对应包：

`src/storylock-local-story-processing-skill`

当前能力：

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

负责：

1. 生成故事草稿。
2. 润色和整理故事文本。
3. 评估题集或问题集合强度。

边界：

1. 不签发授权。
2. 不读取密码、私钥或长期 secret。
3. 不作为远程网关入口。

## 第二层：对象强度、九宫格验证与本地授权

对应包：

`src/storylock-local-story-access-skill`

当前能力：

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`

负责：

1. 根据目标对象判断需要的验证强度。
2. 生成九宫格验证。
3. 校验答案。
4. 创建短时 session 或 authorization。
5. 维护 requestId、nonce、失败窗口和审计日志。

边界：

1. 不读取故事正文。
2. 不写回故事正文。
3. 不直接执行签名或密码填充。
4. 不返回长期敏感材料。

## 第三层：远程请求包装与代理授权入口

对应包：

`src/storylock-remote-gateway-skill`

当前接口：

1. `requestSignature`
2. `requestPasswordFill`

负责：

1. 接收远程侧签名或密码填充请求。
2. 包装请求并保留审计元信息。
3. 调用可选本地执行器。
4. 对结果做递归脱敏。
5. 返回结构化结果。

边界：

1. 不读取故事内容。
2. 不直接保存私钥或密码。
3. 不暴露故事读写接口。
4. 不使用已废弃旧接口 `requestChallengeSign` 作为主线接口。

## 兼容演示包

`src/storylock-skill-engine` 保留本地执行示例和迁移兼容代码。它可以辅助演示 `LocalPasswordFillSkill`、`SignatureAuthorizationSkill` 等能力，但不作为第四层，也不改变三层主线边界。

## 三层职责表

| 层级 | 名称 | 核心职责 | 当前包 |
| --- | --- | --- | --- |
| 第一层 | 故事处理与强度分析 | 草稿、润色、题集强度评估 | `storylock-local-story-processing-skill` |
| 第二层 | 本地访问授权 | 对象强度、九宫格、session、防重放、审计 | `storylock-local-story-access-skill` |
| 第三层 | 远程网关 | `requestSignature`、`requestPasswordFill`、脱敏返回 | `storylock-remote-gateway-skill` |

## 当前最小落地目录

```text
src/
  shared/
    crypto.js
    secret-store.js
    sqlite.js
    sqlite-schema.sql
  storylock-local-story-processing-skill/
    index.js
    scripts/selftest.mjs
  storylock-local-story-access-skill/
    index.js
    access-host.js
    errors.js
    scripts/selftest.mjs
  storylock-remote-gateway-skill/
    index.js
    scripts/selftest.mjs
  storylock-skill-engine/
    index.js
    scripts/selftest.mjs
```

## 当前开发顺序

1. 先稳定三层接口契约。
2. 再补齐端到端演示。
3. 再完善九宫格问题来源和对象强度策略。
4. 最后扩展具体应用场景。

## 结论

三层划分成立，但要保持克制：

1. 第一层只处理故事和强度分析。
2. 第二层只做本地授权控制。
3. 第三层只做远程请求包装和脱敏返回。

应用场景可以继续探索，但不能把尚未实现的多账户、多平台、多链能力写成当前功能。
