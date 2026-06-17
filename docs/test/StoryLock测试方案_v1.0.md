# StoryLock 测试方案 v1.0

版本：v1.0  
日期：2026-06-17  
适用范围：`E:\2026OPC大赛\skill\src` 当前代码基线  
测试方案位置：`E:\2026OPC大赛\skill\docs\test\StoryLock测试方案_v1.0.md`

## 一、测试目标

本测试方案用于验证 StoryLock 当前三层 Skill 架构是否按设计工作：

1. 第一层能完成故事草稿、故事润色和题集强度评估。
2. 第二层能完成对象强度判断、九宫格验证、短时本地授权、防重放、失败锁定和 SQLite 审计。
3. 第三层能完成远程签名请求、Web2 密码填充请求、EIP-712 结构包装、本地执行器委托和递归脱敏。
4. 兼容演示包能保持基础可运行，不作为新的主线安全层。
5. 文档、Schema、错误码和当前代码保持一致。

本方案不再测试旧接口 `requestStoryRead`、`requestStoryWrite`、`requestChallengeSign`、`StoryReadAccessSkill`、`StoryWriteAccessSkill`。这些接口不属于当前主线。

## 二、测试范围

| 测试对象 | 路径 | 优先级 | 说明 |
| --- | --- | --- | --- |
| 第一层故事处理包 | `src/storylock-local-story-processing-skill` | P0 | 草稿、润色、强度评估 |
| 第二层本地访问授权包 | `src/storylock-local-story-access-skill` | P0 | 对象强度、九宫格、本地授权、SQLite 状态 |
| 第三层远程网关包 | `src/storylock-remote-gateway-skill` | P0 | `requestSignature`、`requestPasswordFill`、脱敏 |
| 兼容演示包 | `src/storylock-skill-engine` | P1 | 本地密码填充和签名授权示例 |
| 跨包集成链路 | 第三层 -> 第二层 -> 本地执行器 | P0 | 端到端授权执行 |
| Schema 与文档契约 | `assets/schemas`、`docs/design/cn` | P1 | 字段、错误码、能力名一致 |

## 三、测试环境

| 项目 | 要求 |
| --- | --- |
| 操作系统 | Windows 10/11，兼容 Linux/macOS |
| Node.js | 22.0.0 或以上 |
| npm | 随 Node.js 22 安装版本即可 |
| SQLite | 使用 Node.js `node:sqlite` |
| 测试数据库 | 默认使用临时 SQLite 文件或 `:memory:` |
| SecretStore | 单元和自测使用 `MemorySecretStore`，持久化测试必须显式注入 secretStore |

环境检查命令：

```powershell
node -v
npm -v
```

当前最小验证命令：

```powershell
Push-Location E:\2026OPC大赛\skill\src\storylock-local-story-processing-skill; npm run selftest; Pop-Location
Push-Location E:\2026OPC大赛\skill\src\storylock-local-story-access-skill; npm run selftest; Pop-Location
Push-Location E:\2026OPC大赛\skill\src\storylock-remote-gateway-skill; npm run selftest; Pop-Location
Push-Location E:\2026OPC大赛\skill\src\storylock-skill-engine; npm run selftest; Pop-Location
```

## 四、测试分层策略

| 测试类型 | 目标 | 建议工具 | 优先级 |
| --- | --- | --- | --- |
| 冒烟测试 | 确认四个包当前自测可运行 | 现有 `npm run selftest` | P0 |
| 单元测试 | 验证单个 Skill 或函数行为 | Node.js Test Runner | P0 |
| 集成测试 | 验证跨包链路 | Node.js Test Runner + 临时 SQLite | P0 |
| 安全测试 | 验证防重放、锁定、脱敏、敏感字段保护 | 自定义用例 | P0 |
| 契约测试 | 验证 Schema、错误码、文档口径一致 | Ajv + 静态扫描 | P1 |
| 兼容测试 | 验证 `storylock-skill-engine` 可运行 | 现有 selftest | P1 |

## 五、当前自测基线

当前代码已有四个自测入口：

| 包 | 命令 | 当前覆盖重点 |
| --- | --- | --- |
| `storylock-local-story-processing-skill` | `npm run selftest` | 草稿、润色、强度评估、边界标记 |
| `storylock-local-story-access-skill` | `npm run selftest` | 对象强度、九宫格、防重放、本地授权、失败锁定、清理、SQLite 审计 |
| `storylock-remote-gateway-skill` | `npm run selftest` | `requestSignature`、`requestPasswordFill`、EIP-712、脱敏、本地执行器 |
| `storylock-skill-engine` | `npm run selftest` | 本地密码填充和签名授权兼容示例 |

P0 通过标准：四个 selftest 必须全部通过。

## 六、第一层测试用例

测试对象：

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

### 6.1 StoryDraftSkill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| P-DRAFT-001 | 正常生成草稿 | `objective`、`audience`、`tone`、`constraints` | 返回 `mode=story_draft`，包含 `draft` | P0 |
| P-DRAFT-002 | 缺失 objective | 不传 `objective` | 抛出校验错误 | P0 |
| P-DRAFT-003 | 空字符串 objective | `objective=""` | 抛出校验错误 | P0 |
| P-DRAFT-004 | 默认 audience/tone | 只传 `objective` | 使用默认 `self`、`neutral` | P1 |
| P-DRAFT-005 | 自定义 generator | 注入 generator | 返回自定义草稿，且结构被校验 | P1 |
| P-DRAFT-006 | 非法 source | `source=remote_raw_secret` | 抛出 source 枚举错误 | P0 |
| P-DRAFT-007 | 边界标记 | 有效输入 | `challengeCreated=false`、`sessionIssued=false`、`protectedObjectRead=false` | P0 |

### 6.2 StoryRefineSkill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| P-REFINE-001 | 正常润色 | `storyDraft`、`goals`、`hintStyle` | 返回 `mode=story_refine` 与 `refinedDraft` | P0 |
| P-REFINE-002 | 缺失 storyDraft | 不传 `storyDraft` | 抛出校验错误 | P0 |
| P-REFINE-003 | storyDraft.content 为空 | `content=""` | 抛出校验错误 | P0 |
| P-REFINE-004 | 非法 source | `source=template_only` | 抛出 source 枚举错误 | P0 |
| P-REFINE-005 | 自定义 refiner | 注入 refiner | 返回自定义润色结果 | P1 |
| P-REFINE-006 | 边界标记 | 有效输入 | 不创建 challenge，不签发 session | P0 |

### 6.3 StrengthReviewSkill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| P-STRENGTH-001 | 24 题强题集 | 24 个问题，每题 9 个候选项 | `questionSetReady=true` | P0 |
| P-STRENGTH-002 | 题数不足 | 少于 24 题 | 抛出校验错误 | P0 |
| P-STRENGTH-003 | 候选项不是 9 个 | 单题候选项不足或重复 | 抛出校验错误 | P0 |
| P-STRENGTH-004 | 缺少有效答案 | `validAnswers=[]` | 抛出校验错误 | P0 |
| P-STRENGTH-005 | 弱题集建议 | 合规题不足 | 返回 `issues` 和 `recommendedActions` | P1 |
| P-STRENGTH-006 | 边界标记 | 任意有效输入 | 不签发 session，不读保护对象 | P0 |

## 七、第二层测试用例

测试对象：

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`
4. `access-host.js`
5. `sqlite-schema.sql`
6. `errors.js`

### 7.1 ObjectStrengthPolicySkill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| A-POLICY-001 | 签名对象默认高强度 | `objectType=signature_key` 或 `requestedAction=signature` | `requiredStrength=high`，`requiredCells=9` | P0 |
| A-POLICY-002 | 凭据对象默认中强度 | `objectType=credential` 或 `requestedAction=password_fill` | `requiredStrength=medium`，`requiredCells=6` | P0 |
| A-POLICY-003 | 普通对象默认低强度 | `objectType=generic_secret` | `requiredStrength=low`，`requiredCells=3` | P1 |
| A-POLICY-004 | policyHints 覆盖强度 | `policyHints.requiredStrength=high` | 使用指定强度 | P0 |
| A-POLICY-005 | 非法 objectType | `objectType=invalid` | 返回 `SLG-001` | P0 |
| A-POLICY-006 | 非法 requestedAction | `requestedAction=invalid` | 返回 `SLG-001` | P0 |
| A-POLICY-007 | 缺失 identityId | 不传 `identityId` | 返回 `SLG-001` | P0 |
| A-POLICY-008 | 缺失 objectRef | 不传 `objectRef/credentialRef/keyId` | 返回 `SLG-001` | P0 |

### 7.2 GridChallengeSkill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| A-GRID-001 | 低强度九宫格 | `requiredStrength=low` | 9 格展示，`requiredCells=3` | P0 |
| A-GRID-002 | 中强度九宫格 | `requiredStrength=medium` | 9 格展示，`requiredCells=6` | P0 |
| A-GRID-003 | 高强度九宫格 | `requiredStrength=high` | 9 格展示，`requiredCells=9` | P0 |
| A-GRID-004 | 网格不返回答案 | 已 enroll 答案 | `grid.cells` 中不包含 `answer` | P0 |
| A-GRID-005 | requestId 幂等重放 | 完全相同请求重复提交 | 第二次返回第一次缓存响应 | P0 |
| A-GRID-006 | requestId 冲突 | 相同 requestId，不同 nonce 或 payload | 返回 `SLG-013` | P0 |
| A-GRID-007 | nonce 冲突 | 不同 requestId，相同 nonce | 返回 `SLG-013` | P0 |
| A-GRID-008 | 过期请求 | `expiry` 已过 | 返回 `SLG-011` | P0 |
| A-GRID-009 | 非法 requiredStrength | `requiredStrength=extreme` | 返回 `SLG-001` | P0 |
| A-GRID-010 | 未注册答案摘要 | 未 `enrollAnswers` | 返回 `SLG-010` | P0 |
| A-GRID-011 | 输入长度限制 | 超长 requestId/nonce/answer | 返回 `SLG-001` | P1 |

### 7.3 LocalAuthorizationSkill

| 编号 | 用例 | 前置条件 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- | --- |
| A-AUTH-001 | 正确答案授权签名 | 已创建 challenge，已登记答案 | `allowedAction=signature`，正确答案 | `approved=true`，签发 `authorizationId` | P0 |
| A-AUTH-002 | 正确答案授权密码填充 | 已创建 challenge，已登记答案 | `allowedAction=password_fill` | `readBudget=1`，`writeBudget=0` | P0 |
| A-AUTH-003 | 错误答案拒绝 | 已创建 challenge | 错误答案 | 返回 `SLG-003` | P0 |
| A-AUTH-004 | 连续失败锁定 | 同 identity 连续错 3 次 | 第 4 次创建或提交 | 返回 `SLG-004`，含 `retryAfter` | P0 |
| A-AUTH-005 | 锁定窗口恢复 | 锁定时间已过 | 再次创建 challenge | 允许继续 | P1 |
| A-AUTH-006 | identity 不匹配 | identityB 提交 identityA 的 challenge | 返回 `SLG-003` | P0 |
| A-AUTH-007 | challenge 重复提交 | 同一 challenge 成功后再提交 | 返回 `SLG-003` | P0 |
| A-AUTH-008 | challenge 过期 | 修改 expires_at 为过去 | 提交答案 | 返回 `SLG-003` | P1 |
| A-AUTH-009 | 非法 answers | answers 非数组或超过数量 | 返回 `SLG-001` | P0 |

### 7.4 SQLite 与审计

| 编号 | 用例 | 验证点 | 优先级 |
| --- | --- | --- | --- |
| A-SQL-001 | schema 初始化 | 创建所有当前表：`challenge_state`、`session_store`、`request_store`、`nonce_store`、`failure_window`、`answer_digest_set`、`audit_log` | P0 |
| A-SQL-002 | 不创建旧表 | 不存在 `protected_story_objects` | P0 |
| A-SQL-003 | 答案摘要写入 | `answer_digest_set` 只保存 HMAC 摘要，不保存明文答案 | P0 |
| A-SQL-004 | replay 注册审计 | `replay_registered` 写入 `audit_log` | P1 |
| A-SQL-005 | challenge 失败审计 | `challenge_failed` 写入 `audit_log` | P1 |
| A-SQL-006 | 签名授权审计 | 签名成功写入 `signature_authorized` 审计 | P0 |
| A-SQL-007 | cleanupExpired 清理 | 过期 request/nonce 删除，过期 session/challenge 标记 | P0 |
| A-SQL-008 | cleanup batch 限制 | batchSize 大于 1000 时仍限制到 1000 | P1 |
| A-SQL-009 | 持久化数据库 SecretStore 要求 | 持久化 dbPath 未注入 SecretStore 时拒绝 | P0 |
| A-SQL-010 | 旧 schema 迁移 | 旧表结构自动补齐新列，且不恢复旧故事对象表 | P1 |

## 八、第三层测试用例

测试对象：

1. `StoryLockRemoteGateway`
2. `DelegatedSignatureSkill`
3. 远程请求 Schema

### 8.1 requestSignature

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| G-SIGN-001 | 正常签名请求包装 | `identityId`、`keyId`、`algorithm`、`requestId`、`nonce`、`expiry` | `capability=requestSignature` | P0 |
| G-SIGN-002 | EIP-712 类型结构 | 有效签名请求 | 包含 `StoryLockSignatureRequest` | P0 |
| G-SIGN-003 | nonce 必须为 uint256 字符串 | `eip712Nonce=abc` | 抛出校验错误 | P0 |
| G-SIGN-004 | 支持 ed25519 | `algorithm=ed25519` | 请求通过 | P0 |
| G-SIGN-005 | 支持 secp256k1 | `algorithm=secp256k1` | 请求通过 | P0 |
| G-SIGN-006 | 拒绝非法算法 | `algorithm=rsa` | 抛出校验错误 | P0 |
| G-SIGN-007 | 过期请求拒绝 | `expiry` 小于当前时间 | 抛出 `REQUEST_EXPIRED` | P0 |
| G-SIGN-008 | 本地执行器路径 | 注入 `signatureExecutor` | 调用执行器并返回脱敏结果 | P0 |
| G-SIGN-009 | transport 路径 | 未注入执行器 | 调用 `transport` | P0 |
| G-SIGN-010 | 递归脱敏 | 返回中含 `privateKey/password/answers/secretBytes` | 全部替换为 `[redacted]` | P0 |

### 8.2 requestPasswordFill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| G-PASS-001 | 正常密码填充请求 | `identityId`、`credentialRef`、`targetOrigin` | `capability=requestPasswordFill` | P0 |
| G-PASS-002 | 默认保留策略 | 最小有效输入 | `requestedRetention=audit_meta_only` | P0 |
| G-PASS-003 | 默认策略提示 | 最小有效输入 | `noRemoteSecretReturn=true` | P0 |
| G-PASS-004 | 缺失 credentialRef | 不传凭据引用 | 抛出校验错误 | P0 |
| G-PASS-005 | 缺失 targetOrigin | 不传目标站点 | 抛出校验错误 | P0 |
| G-PASS-006 | 本地执行器路径 | 注入 `passwordFillExecutor` | 调用执行器并脱敏返回 | P0 |
| G-PASS-007 | 明文密码脱敏 | 执行器错误返回 `password` | 返回中为 `[redacted]` | P0 |

### 8.3 DelegatedSignatureSkill

| 编号 | 用例 | 输入 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| G-DELEGATE-001 | 委托签名成功 | 有效参数 | 调用 gateway 的 `requestSignature` | P1 |
| G-DELEGATE-002 | skillId | 无 | 返回 `delegated_signature` | P1 |
| G-DELEGATE-003 | 非法 algorithm | `algorithm=rsa` | 抛出校验错误 | P1 |

## 九、跨包集成测试

当前建议新增端到端测试脚本，例如：

`tests/integration/signature-flow.test.mjs`

### 9.1 签名授权端到端

| 编号 | 用例 | 流程 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| I-SIGN-001 | 远程签名到本地授权 | `requestSignature` -> 对象强度策略 -> 九宫格 -> 本地授权 -> signatureExecutor | 返回签名结果，敏感字段脱敏 | P0 |
| I-SIGN-002 | 授权失败传播 | 错误答案 | 第三层收到结构化错误，不泄露答案细节 | P0 |
| I-SIGN-003 | 审计落库 | 签名成功 | `audit_log` 有签名授权记录 | P0 |
| I-SIGN-004 | replay 防护 | 重复 requestId/nonce | 返回 `SLG-013` | P0 |

### 9.2 密码填充端到端

| 编号 | 用例 | 流程 | 预期结果 | 优先级 |
| --- | --- | --- | --- | --- |
| I-PASS-001 | 远程密码填充到本地授权 | `requestPasswordFill` -> 对象强度策略 -> 九宫格 -> 本地授权 -> passwordFillExecutor | 返回填充成功元信息 | P0 |
| I-PASS-002 | 不返回明文密码 | 执行器返回 password 字段 | 第三层脱敏为 `[redacted]` | P0 |
| I-PASS-003 | 错误答案拒绝 | 错误答案 | 返回授权失败 | P0 |

## 十、安全专项测试

| 编号 | 攻击场景 | 输入 | 预期防御结果 | 优先级 |
| --- | --- | --- | --- | --- |
| SEC-001 | 空答案绕过 | `answers=[]` | 授权失败 | P0 |
| SEC-002 | 随机答案猜测 | 随机字符串 | 失败计数增加 | P0 |
| SEC-003 | 超长答案 | 单答案超过 512 字符 | 返回 `SLG-001` | P0 |
| SEC-004 | 超多答案 | 超过 10 条答案 | 返回 `SLG-001` | P0 |
| SEC-005 | SQL 注入字符串 | `'; DROP TABLE audit_log; --` | 不破坏 SQLite 表 | P0 |
| SEC-006 | requestId 重放 | 相同 requestId，不同 payload | 返回 `SLG-013` | P0 |
| SEC-007 | nonce 重放 | 不同 requestId，相同 nonce | 返回 `SLG-013` | P0 |
| SEC-008 | 过期请求 | expiry 过期 | 返回 `SLG-011` 或 `REQUEST_EXPIRED` | P0 |
| SEC-009 | 跨 identity 提交 | identityB 用 identityA challenge | 授权失败 | P0 |
| SEC-010 | 连续失败暴力尝试 | 同 identity 错 3 次以上 | 锁定并返回 retryAfter | P0 |
| SEC-011 | 远程返回敏感字段 | `privateKey/password/answers/mnemonic` | 全部脱敏 | P0 |
| SEC-012 | 生产持久库无 SecretStore | dbPath 指向文件且不注入 SecretStore | 拒绝创建 Host | P0 |

## 十一、Schema 与契约测试

当前 Schema 清单：

| 包 | Schema |
| --- | --- |
| 本地访问授权包 | `access-response.schema.json`、`grid-verification-input.schema.json`、`local-authorization-input.schema.json`、`object-strength-policy-input.schema.json`、`selftest-report.schema.json` |
| 本地故事处理包 | `story-draft-input.schema.json`、`story-refine-input.schema.json` |
| 远程网关包 | `delegated-sign-input.schema.json`、`remote-gateway-request.schema.json`、`remote-gateway-response.schema.json` |
| 兼容演示包 | `password-fill-input.schema.json`、`password-fill-output.schema.json`、`story-draft-input.schema.json`、`strength-review-output.schema.json` |

契约测试项：

| 编号 | 用例 | 预期结果 | 优先级 |
| --- | --- | --- | --- |
| C-SCHEMA-001 | 所有 JSON Schema 可被 Ajv 加载 | 无语法错误 | P1 |
| C-SCHEMA-002 | 有效 object strength 输入 | 通过校验 | P1 |
| C-SCHEMA-003 | 有效 grid verification 输入 | 通过校验 | P1 |
| C-SCHEMA-004 | 有效 local authorization 输入 | 通过校验 | P1 |
| C-SCHEMA-005 | 有效 remote gateway request | 仅允许 `requestSignature`、`requestPasswordFill` | P0 |
| C-SCHEMA-006 | access response 错误码格式 | `SLG-xxx` 格式通过 | P1 |
| C-DOC-001 | 错误码文档一致 | 与 `errors.js` 中 `ERROR_DEFS` 一致 | P1 |
| C-DOC-002 | 不出现旧主线接口 | 文档不得把旧接口写成当前主线 | P0 |
| C-DOC-003 | Node 版本一致 | 文档和 package.json 均要求 Node.js 22+ | P0 |

## 十二、兼容演示包测试

测试对象：

`src/storylock-skill-engine`

| 编号 | 用例 | 预期结果 | 优先级 |
| --- | --- | --- | --- |
| E-COMPAT-001 | `npm run selftest` | 通过 | P1 |
| E-COMPAT-002 | `LocalPasswordFillSkill` | 返回 `mode=local_password_fill` | P1 |
| E-COMPAT-003 | `SignatureAuthorizationSkill` | 返回 `mode=signature_authorization` | P1 |
| E-COMPAT-004 | 顶层导出检查 | 不把旧 challenge sign 作为主线导出 | P1 |
| E-COMPAT-005 | WASM 自测 | 如存在构建产物，运行 `npm run selftest:wasm` | P2 |

## 十三、测试数据设计

### 13.1 基础身份

```js
export const testIdentity = {
  identityId: 'id-test-001',
};
```

### 13.2 答案登记

第二层测试必须先登记答案摘要：

```js
host.enrollAnswers('id-test-001', [
  'correct grid answer',
  'backup answer',
]);
```

测试中提交答案：

```js
[
  { cellId: 'cell-1', answer: 'correct grid answer' }
]
```

### 13.3 请求 envelope

```js
export function makeEnvelope(prefix = 'req') {
  const suffix = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return {
    requestId: `${prefix}-${suffix}`,
    nonce: `nonce-${suffix}`,
    expiry: Date.now() + 60_000,
  };
}
```

### 13.4 临时 SQLite

```js
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';

export function tempDbPath() {
  return join(tmpdir(), `storylock_test_${randomUUID().replaceAll('-', '')}.db`);
}
```

## 十四、建议测试目录

当前可先保留各包 `scripts/selftest.mjs`。后续若扩展正式测试目录，建议如下：

```text
skill/
  tests/
    integration/
      signature-flow.test.mjs
      password-fill-flow.test.mjs
    security/
      replay.test.mjs
      redaction.test.mjs
      lockout.test.mjs
    contract/
      schemas.test.mjs
      docs-code-consistency.test.mjs
  src/
    storylock-local-story-processing-skill/
      scripts/selftest.mjs
    storylock-local-story-access-skill/
      scripts/selftest.mjs
    storylock-remote-gateway-skill/
      scripts/selftest.mjs
    storylock-skill-engine/
      scripts/selftest.mjs
```

## 十五、执行计划

| 阶段 | 内容 | 产物 | 优先级 |
| --- | --- | --- | --- |
| T0 | 跑通现有四个 selftest | 当前冒烟基线 | P0 |
| T1 | 补端到端签名链路测试 | `signature-flow.test.mjs` | P0 |
| T2 | 补端到端密码填充链路测试 | `password-fill-flow.test.mjs` | P0 |
| T3 | 补安全专项测试 | replay、lockout、redaction 测试 | P0 |
| T4 | 补 Schema 契约测试 | Ajv 校验脚本 | P1 |
| T5 | 补文档-代码一致性测试 | 静态扫描脚本 | P1 |
| T6 | 补覆盖率统计 | 覆盖率报告 | P2 |

## 十六、通过标准

### 16.1 P0 通过标准

1. 四个包 `npm run selftest` 全部通过。
2. 第二层错误答案不能授权。
3. requestId 或 nonce 冲突返回 `SLG-013`。
4. 过期请求返回 `SLG-011` 或抛出 `REQUEST_EXPIRED`。
5. 远程返回中敏感字段被递归脱敏。
6. SQLite 中不保存明文答案。
7. 签名授权审计写入 `audit_log`。
8. 文档和 Schema 不把旧接口作为当前主线。

### 16.2 P1 通过标准

1. JSON Schema 全部可加载。
2. 主请求和响应样例能通过 Schema 校验。
3. 错误码表与 `errors.js` 一致。
4. 兼容演示包基础能力可运行。

### 16.3 P2 通过标准

1. 形成覆盖率报告。
2. 加入 CI 自动执行。
3. 增加压力测试或长时间运行测试。

## 十七、CI 建议

当前仓库没有统一根 `package.json` 测试入口时，可先使用 PowerShell 脚本：

```powershell
$ErrorActionPreference = "Stop"

Push-Location src/storylock-local-story-processing-skill
npm run selftest
Pop-Location

Push-Location src/storylock-local-story-access-skill
npm run selftest
Pop-Location

Push-Location src/storylock-remote-gateway-skill
npm run selftest
Pop-Location

Push-Location src/storylock-skill-engine
npm run selftest
Pop-Location
```

后续可增加根目录脚本：

```json
{
  "scripts": {
    "test:self": "powershell -ExecutionPolicy Bypass -File scripts/runtime/test_self.ps1",
    "test:integration": "node --test tests/integration/*.test.mjs",
    "test:security": "node --test tests/security/*.test.mjs",
    "test:contract": "node --test tests/contract/*.test.mjs"
  }
}
```

## 十八、风险与应对

| 风险 | 影响 | 应对 |
| --- | --- | --- |
| 测试方案沿用旧接口 | 测试失真 | 本方案已移除故事读写和 challenge sign 主线测试 |
| `node:sqlite` 运行时差异 | 自测无法运行 | 固定 Node.js 22+ |
| Mock 与真实 SQLite 行为不一致 | 集成缺陷漏测 | P0 集成测试必须使用真实临时 SQLite |
| 端到端链路尚未完整自动化 | 参赛演示风险 | 优先补 `signature-flow` 和 `password-fill-flow` |
| SecretStore 生产适配未完成 | 生产安全风险 | 持久化测试必须覆盖无 SecretStore 拒绝逻辑 |

## 十九、附录：当前错误码测试口径

| 错误码 | 类型 | 测试重点 |
| --- | --- | --- |
| `SLG-001` | `validation_error` | 字段缺失、类型错误、长度超限 |
| `SLG-002` | `replay_rejected` | 保留历史兼容，不作为当前 replay 冲突主码 |
| `SLG-003` | `challenge_failed` | 答案错误、challenge 无效 |
| `SLG-004` | `challenge_locked` | 连续失败锁定 |
| `SLG-005` | `session_invalid` | session 不存在、过期或状态无效 |
| `SLG-006` | `budget_exhausted` | 预算耗尽 |
| `SLG-007` | `object_not_found` | 对象不存在 |
| `SLG-008` | `redaction_required` | 结果必须脱敏 |
| `SLG-009` | `scope_insufficient` | scope 不足 |
| `SLG-010` | `secret_unavailable` | SecretStore 或答案摘要不可用 |
| `SLG-011` | `request_expired` | 请求过期 |
| `SLG-012` | `internal_error` | 未分类内部错误 |
| `SLG-013` | `replay_detected` | requestId 或 nonce 冲突 |

## 二十、附录：不测试为当前能力的内容

以下内容不纳入当前 v1.0 主线验收：

1. 故事对象读取。
2. 故事对象写回。
3. `requestChallengeSign`。
4. 完整链上验证。
5. EIP-1271/ERC-4337 生产级集成。
6. 多链、多钱包、多账号应用编排。

这些内容只能作为后续应用探索或扩展测试项。

