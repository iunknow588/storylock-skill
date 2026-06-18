# StoryLock 项目说明

版本：2026-06-18  
适用目录：`skill/`

## 1. 项目定位

StoryLock 是一个本地优先的授权访问 Skill 项目。它将故事记忆线索、对象强度策略、九宫格验证、短时本地授权和远程请求包装拆成三层能力，使远程 Agent 可以请求本地完成签名或 Web2 密码填充，但不直接持有长期秘密。

当前项目的重点是证明一条可运行、可审计、可脱敏的本地授权链路，而不是宣称已经实现多链钱包、多平台账号或完整业务编排。

一句话概括：

> StoryLock 通过三层 Skill 结构，将本地故事记忆验证与远程代理请求连接起来，让签名和密码填充等敏感操作保留在本地授权边界内完成。

## 2. 当前三层结构

| 层级 | 代码包 | 当前能力 |
| --- | --- | --- |
| 第一层：故事处理与强度分析 | `src/storylock-local-story-processing-skill` | `StoryDraftSkill`、`StoryRefineSkill`、`StrengthReviewSkill` |
| 第二层：本地访问授权 | `src/storylock-local-story-access-skill` | `ObjectStrengthPolicySkill`、`GridChallengeSkill`、`LocalAuthorizationSkill` |
| 第三层：远程网关 | `src/storylock-remote-gateway-skill` | `requestSignature`、`requestPasswordFill` |
| 兼容演示包 | `src/storylock-skill-engine` | 本地密码填充与签名授权示例 |

当前主线对外聚焦两个远程入口：`requestSignature` 与 `requestPasswordFill`。本地授权流程由第二层完成，远程网关只负责请求包装、委托执行和脱敏返回。

## 3. 核心能力

### 3.1 第一层

第一层负责处理故事内容和题集质量：

1. 生成故事草稿。
2. 润色和整理故事文本。
3. 评估题集或问题集合强度。

第一层不签发授权、不读取长期秘密、不作为远程网关入口。

### 3.2 第二层

第二层负责本地授权控制：

1. 根据目标对象判断验证强度：`low`、`medium`、`high`。
2. 生成对应九宫格验证。
3. 校验本地答案。
4. 签发短时 session 或 authorization。
5. 维护 requestId、nonce、失败窗口和审计日志。

SQLite 当前保存：

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `audit_log`

答案只保存摘要，不保存明文答案。

### 3.3 第三层

第三层是远程 Agent 请求本地能力的统一入口：

1. `requestSignature`：包装签名请求，使用 `StoryLockSignatureRequest` 风格的 EIP-712 结构。
2. `requestPasswordFill`：包装 Web2 密码填充请求，默认只返回审计元信息。
3. 调用可选本地执行器：`signatureExecutor`、`passwordFillExecutor`。
4. 对返回结果执行递归脱敏。

第三层不保存私钥、密码、九宫格答案或明文故事内容。

## 4. 安全机制

| 机制 | 当前状态 |
| --- | --- |
| Node.js 运行时 | 要求 Node.js 22 或以上 |
| SQLite 状态存储 | 已实现 |
| requestId/nonce 防重放 | 已实现，冲突使用 `SLG-013` |
| 请求过期检查 | 已实现，过期使用 `SLG-011` |
| 九宫格失败锁定 | 已实现 |
| 过期状态清理 | 已实现 `npm run cleanup` |
| 答案摘要 | HMAC-SHA256 摘要存储 |
| SecretStore | 支持内存开发模式与平台适配工厂 |
| 远程返回脱敏 | 已实现递归脱敏 |

持久化 SQLite 不允许默认使用普通 `MemorySecretStore`。开发测试可以显式使用 `developmentMode=true`，生产环境应使用平台 SecretStore 或等价安全存储。

生产持久化主机默认不启用 legacy 答案回退。九宫格验证必须来自足量 active 题集；题集不足时返回 `SLG-010` / `question_set_unavailable`。仅开发或演示兼容场景可以显式传入 `allowLegacyFallback: true`。

## 5. 运行与验证

在 `skill/` 目录下执行以下命令。

### 5.0 一键验证

```powershell
npm run test
```

### 5.1 第一层自测

```powershell
Push-Location src/storylock-local-story-processing-skill
npm run selftest
Pop-Location
```

### 5.2 第二层自测

```powershell
Push-Location src/storylock-local-story-access-skill
npm run selftest
Pop-Location
```

覆盖对象强度、九宫格、防重放、本地授权、失败锁定、cleanup、SQLite 审计和 SecretStore 约束。

### 5.3 第三层自测

```powershell
Push-Location src/storylock-remote-gateway-skill
npm run selftest
Pop-Location
```

覆盖 `requestSignature`、`requestPasswordFill`、EIP-712 包装和递归脱敏。

### 5.4 三层端到端签名演示

```powershell
Push-Location src/storylock-remote-gateway-skill
npm run selftest:e2e
Pop-Location
```

该脚本串联：

1. 第三层 `requestSignature`。
2. 第二层对象强度策略。
3. 第二层九宫格验证。
4. 第二层本地授权。
5. 本地签名执行器。
6. 第三层脱敏返回。
7. SQLite 审计写入。

### 5.5 兼容演示包自测

```powershell
Push-Location src/storylock-skill-engine
npm run selftest
Pop-Location
```

### 5.6 SQLite 清理命令

```powershell
Push-Location src/storylock-local-story-access-skill
npm run cleanup -- 2 --development-memory-secret-store
Pop-Location
```

## 6. 文档入口

| 文档 | 路径 |
| --- | --- |
| 工作区根目录入口 | `README.md` |
| 中文设计入口 | `docs/design/cn/README.md` |
| 参赛参考材料 | `docs/ref/README.md` |
| 测试方案 | `docs/test/StoryLock测试方案_v1.0.md` |
| 开发落地进展 | `docs/design/cn/开发落地路线与当前进展.md` |
| 评审讲解与演示说明 | `docs/ref/06-评审讲解与演示说明.md` |
| 参赛概览 | `docs/ref/01-参赛概览.md` |

## 7. 当前完成度

已完成：

1. 三层代码包与能力边界。
2. 四个包的 selftest。
3. 三层端到端签名演示。
4. SQLite 审计、防重放、失败锁定、过期清理。
5. SecretStore 生产约束。
6. 设计、测试、参赛参考文档的当前口径同步。

仍需完善：

1. 生产题集已有导入 dry-run 和 schema；仍需补充更完整的运营发布、迁移、回滚说明。
2. HTTP/宿主集成层仍是后续扩展。
3. 多链、多平台、多账号等属于应用场景探索，不是当前已实现能力。

## 8. 推荐对外表述

建议使用：

> StoryLock 是一个本地优先的授权访问 Skill 项目。它通过故事处理、本地访问授权、远程网关三层结构，将九宫格验证、短时授权、签名请求和 Web2 密码填充包装为可调用能力，同时把长期秘密和授权判断保留在本地边界内。

避免使用：

1. “已经支持完整多链多钱包生产系统”。
2. “远程网关可以直接处理本地明文敏感内容”。
3. “兼容演示包就是完整生产安全边界”。

## 9. 总结

StoryLock 当前已经具备可验证的三层 Skill 基线：

1. 第一层负责故事处理与题集强度。
2. 第二层负责本地验证、授权和审计。
3. 第三层负责远程请求包装、委托执行和脱敏返回。

它适合用于展示 Agent 场景下“远程可请求、本地可授权、结果可审计、秘密不外泄”的安全能力模型。
