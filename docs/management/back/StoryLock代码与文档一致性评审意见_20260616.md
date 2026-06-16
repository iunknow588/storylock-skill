# StoryLock 代码与文档一致性评审意见

**评审日期**: 2026-06-16  
**评审范围**:  
- 代码目录: `E:\2026OPC大赛\skill\src`
- 文档目录: `E:\2026OPC大赛\skill\docs\design\cn`
- 输出目录: `E:\2026OPC大赛\skill\docs\management`

---

## 一、评审结论总览

| 维度 | 一致性评级 | 说明 |
|------|-----------|------|
| 三层架构划分 | ✅ 一致 | 代码目录与文档定义的三层结构完全对应 |
| 能力边界定义 | ⚠️ 部分一致 | 存在新旧两套能力体系并存问题 |
| 接口契约 | ⚠️ 部分一致 | 契约字段基本对齐，但存在关键差异 |
| 安全规范 | ❌ 不一致 | 代码实现严重偏离文档安全要求 |
| 数据持久化 | ❌ 不一致 | 内存存储替代了文档要求的SQLite |
| 防重放机制 | ⚠️ 部分一致 | 结构存在，但实现不完整 |
| 脱敏规范 | ❌ 不一致 | 代码未实现文档要求的脱敏分级 |
| 错误码体系 | ❌ 不一致 | 代码使用裸Error，未使用SLG错误码 |
| 密钥管理 | ❌ 不一致 | 代码完全缺失平台密钥存储适配 |

**总体评级（原始评审时点）**: ⚠️ **部分一致，存在重大安全实现缺口**

> 注：本评级反映 2026-06-16 20:36 GMT+8 原始审计时点。后续修复记录见第七节，当前代码状态应以第七节和访问层自测结果共同判断。

---

## 二、逐项一致性分析

### 2.1 三层架构划分 ✅ 一致

**文档定义** (`storylock_three_skill_packages_cn.md`):
- 第一包: `storylock-local-story-processing-skill` — 纯本地故事处理
- 第二包: `storylock-local-story-access-skill` — 本地安全访问边界
- 第三包: `storylock-remote-gateway-skill` — 远程代理与委托

**代码实现**:
- `src/storylock-local-story-processing-skill/` — 存在 ✅
- `src/storylock-local-story-access-skill/` — 存在 ✅
- `src/storylock-remote-gateway-skill/` — 存在 ✅
- `src/storylock-skill-engine/` — 旧迁移代码包，文档未定义其角色

**评审意见**: 目录结构完全对应。但 `storylock-skill-engine` 作为旧迁移代码包，文档中未明确其在新三层架构中的定位，存在"第四包"的模糊地带。

---

### 2.2 能力边界定义 ⚠️ 部分一致

**文档定义** (`系统Skill表与能力边界.md`):
- 主Skill: `StoryDraftAssistSkill`, `StoryRefineAssistSkill`, `StrengthReviewSkill`, `LocalPasswordFillSkill`, `ChallengeSigningAuthorizationSkill`
- 内部Skill: `LoginAuthorizationSkill`, `SigningAuthorizationSkill`
- 编排示例: `VideoPublishAgentDemo`

**代码实现**:

| 文档定义 | 代码存在位置 | 状态 |
|---------|------------|------|
| `StoryDraftAssistSkill` | `storylock-skill-engine/assets/migrated/skills/story-assist.js` | ⚠️ 旧包中 |
| `StoryRefineAssistSkill` | `storylock-skill-engine/assets/migrated/skills/story-assist.js` | ⚠️ 旧包中 |
| `StrengthReviewSkill` | `storylock-skill-engine/assets/migrated/skills/strength-review.js` | ⚠️ 旧包中 |
| `LocalPasswordFillSkill` | `storylock-skill-engine/assets/migrated/skills/authorization-skills.js` | ⚠️ 旧包中 |
| `ChallengeSigningAuthorizationSkill` | `storylock-skill-engine/assets/migrated/skills/authorization-skills.js` | ⚠️ 旧包中 |
| `LoginAuthorizationSkill` | `storylock-skill-engine/assets/migrated/skills/authorization-skills.js` | ⚠️ 旧包中 |
| `SigningAuthorizationSkill` | `storylock-skill-engine/assets/migrated/skills/authorization-skills.js` | ⚠️ 旧包中 |
| `StoryDraftSkill` (新包) | `storylock-local-story-processing-skill/index.js` | ✅ 新包中 |
| `StoryRefineSkill` (新包) | `storylock-local-story-processing-skill/index.js` | ✅ 新包中 |
| `StoryReadAccessSkill` | `storylock-local-story-access-skill/index.js` | ✅ 新包中 |
| `StoryWriteAccessSkill` | `storylock-local-story-access-skill/index.js` | ✅ 新包中 |

**问题发现**:
1. **新旧体系并存**: 文档定义的主Skill清单在 `storylock-skill-engine` 旧包中实现，而新三层包中只有部分对应能力
2. **能力名称不一致**: 文档用 `StoryDraftAssistSkill`，新包用 `StoryDraftSkill`（缺少"Assist"后缀）
3. **StrengthReviewSkill 缺失**: 新三层包中完全没有强度评估能力
4. **PasswordFill/ChallengeSign 缺失**: 新包第二层未实现这些核心本地安全能力

**评审意见**: 能力边界在文档层面清晰，但代码实现存在"新旧两套体系"的混乱。建议明确 `storylock-skill-engine` 的定位（是废弃、迁移中、还是保留为参考实现？），并统一命名。

---

### 2.3 接口契约 ⚠️ 部分一致

**文档定义** (`三包接口契约.md`):
- 统一请求字段: `requestId`, `capability`, `scope`, `payload`, `policyHints`, `requestedRetention`, `nonce`, `expiry`
- 统一响应字段: `requestId`, `status`, `capability`, `executionLocation`, `result`, `redactionLevel`, `retentionGranted`, `auditMeta`, `error`
- 统一命名: `camelCase`
- 错误响应必须包含完整外层结构

**代码实现** (第二包 `StoryReadAccessSkill.run` 返回):
```json
{
  "requestId": "...",
  "status": "success",
  "capability": "requestStoryRead",
  "executionLocation": "local",
  "result": { ... },
  "redactionLevel": "none",
  "retentionGranted": "result_only",
  "auditMeta": { ... },
  "error": null
}
```

**差异点**:
| 文档要求 | 代码实现 | 差异说明 |
|---------|---------|---------|
| `capability` 枚举包含 `requestStoryRead`, `requestStoryWrite`, `requestChallengeSign`, `queryStoryMetadata` | 第二包返回 `requestStoryRead` / `requestStoryWrite` | ✅ 一致 |
| 错误响应必须包含 `error.code`, `error.type`, `error.message`, `error.suggestedAction`, `error.retryable` | 代码使用 `throw new Error('CHALLENGE_FAILED')` 抛出裸Error | ❌ 不一致 |
| 响应中 `error` 为 null 时其他字段完整 | 代码在成功时返回完整结构 | ✅ 一致 |
| 第三包请求结构必须包含 `policyHints` | 第三包 `normalizeEnvelope` 包含 `policyHints` | ✅ 一致 |

**评审意见**: 成功路径的响应结构基本对齐文档。但错误处理路径严重偏离：代码使用裸Error抛出，未返回文档要求的结构化错误响应（含 `error.code`, `error.type`, `error.message`, `error.suggestedAction`, `error.retryable`）。

---

### 2.4 安全规范 ❌ 不一致

**文档定义** (`安全规范.md`):
- 对称加密: `AES-256-GCM`
- 摘要/HMAC: `SHA-256` / `HMAC-SHA256`
- 工作密钥派生: `HKDF-SHA256`
- 密钥层次: `masterSalt` → `rootKey` → `workKey` → `objectKey`
- 题集主档加密: 落盘前使用 `AES-256-GCM`
- GCM nonce: 每次加密独立生成 96-bit 随机 nonce，同一密钥下不得复用
- 答案摘要: `HMAC-SHA256(identitySalt, normalizedAnswer)`
- 长期敏感材料: 优先操作系统密钥链，不直接入 SQLite 普通表，不直接入 `.env` 明文文件

**代码实现**:
- 第二包 `storylock-local-story-access-skill/index.js`:
  - 使用 `Map` 内存存储，无加密 ❌
  - 答案验证: `normalizedAnswers.length >= 0`（恒为true，无实际验证） ❌❌❌
  - 无 `AES-256-GCM` 实现 ❌
  - 无 `HKDF-SHA256` 实现 ❌
  - 无 `HMAC-SHA256` 答案摘要 ❌
  - 无 `masterSalt` / `rootKey` 管理 ❌
  - 无操作系统密钥链集成 ❌
  - 挑战对象明文存储于内存Map ❌
  - session 对象明文存储于内存Map ❌
  - story对象明文存储: `content: 'Protected story content'` ❌

- 第一包 `storylock-local-story-processing-skill/index.js`:
  - 纯规则模板处理，无加密需求，不涉及安全规范 ✅

- 第三包 `storylock-remote-gateway-skill/index.js`:
  - 仅做请求包装，无加密实现（符合设计） ✅

- 旧包 `storylock-skill-engine/assets/migrated/skills/authorization-skills.js`:
  - 有 `zeroizeBytes` 实现（密钥清零） ⚠️ 部分实现
  - 有 `cloneSecretBytes` 实现 ⚠️ 部分实现
  - 但无 `AES-256-GCM` ❌
  - 无 `HKDF-SHA256` ❌
  - 无答案摘要规范 ❌

**关键安全漏洞**:
1. **答案验证逻辑失效**: `submitChallengeAnswers` 中 `accepted = normalizedAnswers.length >= 0` 恒为 true，意味着任何答案（包括空数组）都能通过验证。这是**致命安全漏洞**。
2. **无加密存储**: 所有敏感数据（challenge、session、story对象）均以明文存储于内存Map，未使用 `AES-256-GCM`。
3. **无密钥派生**: 未实现 `masterSalt` → `rootKey` → `workKey` → `objectKey` 的密钥层次。
4. **无平台密钥存储**: 未集成 Windows/macOS/Linux 的平台密钥链。

**评审意见**: 安全规范是文档中最严格的部分，但代码实现存在**致命缺口**。特别是答案验证逻辑的失效，使得整个 challenge 机制形同虚设。这是**必须立即修复**的阻塞性问题。

---

### 2.5 数据持久化 ❌ 不一致

**文档定义** (`README.md`, `平台密钥存储适配指南.md`, `Session与防重放策略.md`):
- 本地存储: SQLite 单文件数据库 (`storylock_vault.db`)
- 敏感存储: 操作系统密钥链或平台安全存储
- session/nonce/requestId 去重信息写入 SQLite
- 使用 `BEGIN IMMEDIATE` 事务保证并发安全

**代码实现**:
- 第二包使用 `Map` 内存存储: `challenges = new Map()`, `sessions = new Map()`, `requestStore = new Map()`, `nonceStore = new Set()`
- 无 SQLite 实现 ❌
- 无文件持久化 ❌
- 无操作系统密钥链集成 ❌
- 设备重启后所有数据丢失 ❌

**评审意见**: 代码使用内存Map替代了文档要求的SQLite持久化。这导致：
1. 设备重启后所有 challenge/session 状态丢失
2. 无法实现跨进程并发控制（文档要求的 SQLite 事务锁）
3. 无法持久化审计日志
4. 不符合"本地安全访问边界"的设计目标

---

### 2.6 防重放机制 ⚠️ 部分一致

**文档定义** (`Session与防重放策略.md`):
- `requestId` 幂等: 相同 `requestId` 已成功的请求返回同一结果或明确错误
- `nonce` 去重: 随机 nonce，TTL窗口内记录已使用 nonce 集
- `expiry` 校验: 默认允许 `+/- 30秒` 时钟漂移容差
- 预算扣减原子性: `readBudget`/`writeBudget` 扣减必须与对象读取在同一事务内
- 清理策略: 滑动时间窗口，默认 `2 x maxTTL`，最大不超过24小时

**代码实现**:
- `requestId` 去重: `store.requestStore.has(requestId)` → 抛出 `DUPLICATE_REQUEST` ✅
- `nonce` 去重: `store.nonceStore.has(nonce)` → 抛出 `NONCE_REPLAY_DETECTED` ✅
- `expiry` 校验: `if (expiry <= nowMs()) throw new Error('REQUEST_EXPIRED')` ✅（但无30秒容差）
- 预算扣减: `consumeReadBudget` 先扣减再读取，但未在同一事务中 ❌
- 清理策略: 无实现，nonce/requestStore 无限增长 ❌
- 存储: 内存Set/Map，非SQLite ❌

**评审意见**: 防重放的基础结构存在（requestId/nonce/expiry校验），但：
1. 无30秒时钟漂移容差
2. 预算扣减与对象读取未在同一事务（存在竞态条件）
3. 无清理策略，内存将无限增长
4. 未使用SQLite，无法实现文档要求的原子性

---

### 2.7 脱敏规范 ❌ 不一致

**文档定义** (`脱敏规范.md`):
- 三个级别: `none`（第一层↔第二层）、`partial`（第二层↔第三层）、`full`（第三层↔第三方）
- 默认规则: 第二层→第三层优先 `partial`，高敏对象默认禁止 `none`
- 高敏字段清单: 真实姓名、手机号、邮箱、身份证号、密码、私钥、challenge answers、未脱敏私密故事段落
- 响应结构必须显式返回 `redactionLevel`, `retentionGranted`, `result`
- 审计日志记录脱敏操作，但不记录被删除字段的原始内容

**代码实现**:
- 第二包返回固定 `redactionLevel: 'none'` ❌
- 无 `partial` 或 `full` 脱敏实现 ❌
- 无高敏字段检测 ❌
- 无脱敏处理逻辑（遮盖、摘要替换、结构保留值清空） ❌
- 返回完整 story 对象给第三层: `storyObject: { title, content, version }` ❌
- 无审计日志 ❌

**评审意见**: 代码完全未实现脱敏规范。第二包返回 `redactionLevel: 'none'` 且返回完整 story 对象，这意味着第三层和远程侧可以获得完整的私密故事内容，严重违反"远程只能获得最小结果"的安全原则。

---

### 2.8 错误码体系 ❌ 不一致

**文档定义** (`本地Agent网关设计.md`):
- 统一 `SLG` 前缀错误码: `SLG-001` ~ `SLG-012`
- 错误结构: `errorCode`, `errorType`, `message`, `suggestedAction`, `retryable`
- HTTP状态码映射建议

**代码实现**:
- 使用裸 `Error` 对象: `throw new Error('CHALLENGE_FAILED')` ❌
- 无 `SLG-xxx` 错误码 ❌
- 无 `errorType` 字段 ❌
- 无 `suggestedAction` 字段 ❌
- 无 `retryable` 字段 ❌
- 错误码列表中的 `SLG-009` ~ `SLG-012` 未在代码中体现 ❌

**评审意见**: 代码完全未实现文档定义的错误码体系。裸Error对象无法提供结构化的错误信息，远程Agent无法根据错误类型做出正确的重试或降级决策。

---

### 2.9 状态机实现 ⚠️ 部分一致

**文档定义** (`Challenge状态机.md`):
- 8个状态: `idle`, `challenge_created`, `answers_submitted`, `verified`, `session_active`, `session_expired`, `failed`, `locked`
- 状态转换条件明确
- 失败计数按 `identityId + 时间窗口` 统计，默认24小时窗口，3次/窗口
- 锁定窗口默认15分钟
- 并发控制: 以 `challengeId` 为粒度加本地锁，SQLite事务
- 默认时长: challenge TTL 5分钟，one_shot session 3分钟，short_session 10分钟

**代码实现**:
- 状态存在: `challenge_created`, `verified`, `failed`, `locked`, `session_expired` ✅
- 状态缺失: `idle`, `answers_submitted`, `session_active`（代码中叫 `active`） ⚠️
- 失败计数: 按 `challengeId` 统计，非 `identityId + 时间窗口` ❌
- 锁定窗口: 15分钟 ✅
- 并发控制: 无锁，无SQLite事务 ❌
- challenge TTL: 5分钟 ✅
- session TTL: 5分钟（文档要求 one_shot 3分钟，short_session 10分钟） ⚠️

**评审意见**: 状态机的核心状态存在，但：
1. 失败计数策略错误（按challengeId而非identityId），攻击者可通过创建新challenge绕过限制
2. 无并发控制，存在竞态条件
3. session TTL未区分类型（统一5分钟）

---

### 2.10 答案存储策略 ❌ 不一致

**文档定义** (`挑战答案存储策略.md`):
- 原始答案不做长期持久化，只存在于短时内存
- 持久化只保存 `answerDigestSet`（规范化后计算的摘要）
- 摘要算法: `salt = HMAC(masterSalt, identityId)`，然后 `HMAC-SHA256(identitySalt, normalizedAnswer)`
- 审计日志只记录过程，不记录答案
- 答案规范化: 去除首尾空白、统一全角半角、统一大小写、明确标点策略
- 校验完成后立即清理内存（Buffer.fill(0)）

**代码实现**:
- 原始答案传入后立即用于验证，但未清理 ❌
- 无 `answerDigestSet` 持久化 ❌
- 无 `HMAC-SHA256` 摘要计算 ❌
- 无答案规范化处理 ❌
- 无 `Buffer.fill(0)` 清零逻辑 ❌
- 验证逻辑: `normalizedAnswers.length >= 0`（恒true） ❌❌❌

**评审意见**: 答案存储策略完全未实现。最致命的是验证逻辑失效，使得整个 challenge 安全机制形同虚设。

---

### 2.11 平台密钥存储 ❌ 不一致

**文档定义** (`平台密钥存储适配指南.md`):
- 统一 `SecretStore` 接口: `getSecret`, `setSecret`, `deleteSecret`, `listKeys`
- Windows: 系统保护存储 / 凭据管理
- macOS: Keychain
- Linux: Secret Service / libsecret
- 命名: `storylock/masterSalt`, `storylock/signingRoot/<identityId>`, `storylock/recoveryKey/<identityId>`
- 降级原则: 默认返回明确错误，不静默降级为明文文件
- 开发模式: 必须显式确认，启动时输出告警，审计日志记录

**代码实现**:
- 无 `SecretStore` 接口 ❌
- 无平台密钥存储集成 ❌
- 无 `masterSalt` 管理 ❌
- 降级为内存Map（比明文文件更不安全，因为重启丢失且无加密） ❌

**评审意见**: 平台密钥存储完全未实现。这是安全基础设施的核心缺失。

---

### 2.12 Schema 一致性 ✅ 基本一致

**文档要求**: JSON Schema 定义输入输出结构

**代码实现**:
- `storylock-local-story-access-skill/assets/schemas/story-read-input.schema.json` ✅
- `storylock-local-story-access-skill/assets/schemas/story-write-input.schema.json` ✅
- `storylock-remote-gateway-skill/assets/schemas/remote-gateway-request.schema.json` ✅
- `storylock-remote-gateway-skill/assets/schemas/remote-gateway-response.schema.json` ✅
- `storylock-remote-gateway-skill/assets/schemas/delegated-sign-input.schema.json` ✅
- `storylock-remote-gateway-skill/assets/schemas/delegated-story-read-input.schema.json` ✅
- `storylock-remote-gateway-skill/assets/schemas/delegated-story-write-input.schema.json` ✅
- `storylock-local-story-processing-skill/assets/schemas/story-draft-input.schema.json` ✅
- `storylock-local-story-processing-skill/assets/schemas/story-refine-input.schema.json` ✅
- `storylock-skill-engine/assets/schemas/` 下多个schema ✅

**差异点**:
- `storylock-skill-engine` 的 `story-draft-input.schema.json` 允许 `additionalProperties: true`，而新包的 `story-draft-input.schema.json` 使用 `additionalProperties: false` ⚠️
- 旧包schema与新包schema在字段默认值上有细微差异（如 `audience` 默认值: 旧包无默认，新包 `"self"`）

**评审意见**: Schema 文件基本齐全，结构对齐。但旧包与新包之间存在细微差异，建议统一。

---

### 2.13 代理签名机制 ⚠️ 部分一致

**文档定义** (`代理签名机制协议参考.md`):
- 请求表示层: EIP-712 结构化签名请求格式
- 本地授权层: challenge / session / scope 校验
- 签名验证层: EIP-1271 兼容
- 委托审计层: HDP Protocol 责任链
- 算法选择: EVM用 `secp256k1`，通用用 `ed25519`
- 代码示例: `buildStoryLockEip712Request()` 函数

**代码实现**:
- 第三包 `requestChallengeSign` 方法存在 ✅
- 支持 `algorithm` 字段 ✅
- 但无 EIP-712 结构化请求构建 ❌
- 无 `domain`, `types`, `value` 结构 ❌
- 无 `secp256k1` 支持（模板用 `ed25519`） ⚠️
- 无 EIP-1271 兼容代码 ❌
- 无 HDP Protocol 审计链 ❌

**评审意见**: 代理签名的入口结构存在，但缺少文档要求的 EIP-712 结构化请求格式和审计链实现。当前只是简单的参数透传，未达到"可审计的委托签名"标准。

---

## 三、关键风险清单

### 🔴 阻塞性风险（必须立即修复）

| 编号 | 风险 | 位置 | 影响 |
|------|------|------|------|
| R1 | 答案验证逻辑恒为true | `storylock-local-story-access-skill/index.js:submitChallengeAnswers` | 任何答案都能通过验证，challenge机制完全失效 |
| R2 | 无加密存储 | 所有敏感数据明文存于内存Map | 设备重启后数据丢失，且内存可被dump |
| R3 | 无密钥派生 | 未实现masterSalt→rootKey→workKey→objectKey | 无法建立安全的密钥层次 |
| R4 | 无平台密钥存储 | 未集成操作系统密钥链 | 长期敏感材料无安全存储 |

### 🟠 高风险（应在下一迭代修复）

| 编号 | 风险 | 位置 | 影响 |
|------|------|------|------|
| R5 | 失败计数按challengeId而非identityId | `storylock-local-story-access-skill/index.js` | 攻击者可创建新challenge绕过失败锁定 |
| R6 | 无并发控制 | 第二包所有状态操作 | 存在竞态条件，可能导致重复提交或重复签发session |
| R7 | 无脱敏实现 | 第二包返回完整story对象 | 远程侧可获得完整私密内容 |
| R8 | 裸Error替代结构化错误 | 所有包的错误处理 | 远程Agent无法正确解析错误类型 |
| R9 | 无SQLite持久化 | 第二包使用内存Map | 无法实现事务原子性和跨进程安全 |

### 🟡 中风险（应在近期修复）

| 编号 | 风险 | 位置 | 影响 |
|------|------|------|------|
| R10 | 新旧能力体系并存 | `storylock-skill-engine` vs 新三层包 | 维护成本增加，边界模糊 |
| R11 | 命名不一致 | `StoryDraftSkill` vs `StoryDraftAssistSkill` | 文档与代码对应困难 |
| R12 | 无nonce/requestId清理 | 内存Set/Map无限增长 | 长期运行后内存耗尽 |
| R13 | session TTL未区分类型 | 统一5分钟 | 不符合one_shot/short_session/batch_session的差异化要求 |
| R14 | 无答案规范化 | 第二包直接比较原始答案 | 大小写、全半角差异导致误判 |
| R15 | 无审计日志 | 所有操作无记录 | 无法追溯安全事件 |

---

## 四、修复建议

### 4.1 立即修复（阻塞性问题）

1. **修复答案验证逻辑**:
   ```javascript
   // 当前（错误）:
   const accepted = normalizedAnswers.length >= 0; // 恒为true
   
   // 应改为:
   const accepted = verifyAnswers(identityId, challengeId, normalizedAnswers);
   // 其中 verifyAnswers 应:
   // 1. 规范化答案（去除空白、统一大小写等）
   // 2. 计算 HMAC-SHA256(identitySalt, normalizedAnswer)
   // 3. 与存储的 answerDigestSet 比对
   // 4. 统计匹配数是否达到 threshold
   // 5. 校验完成后清零答案Buffer
   ```

2. **实现SQLite持久化**:
   - 替换内存Map为SQLite数据库
   - 数据库名: `storylock_vault.db`
   - 表: `challenge_state`, `session_store`, `nonce_store`, `request_store`, `protected_story_objects`
   - 使用 `BEGIN IMMEDIATE` 事务

3. **实现密钥层次**:
   - 定义 `SecretStore` 接口
   - 集成操作系统密钥链（Windows凭据管理/macOS Keychain/Linux Secret Service）
   - 实现 `masterSalt` 存储与 `HKDF-SHA256` 派生

### 4.2 短期修复（下一迭代）

4. **实现失败计数策略**: 按 `identityId + 24小时窗口` 统计失败次数
5. **实现并发控制**: 以 `challengeId` 为粒度的SQLite事务锁
6. **实现脱敏分级**: 第二包返回第三层时默认使用 `partial` 脱敏
7. **实现结构化错误**: 统一使用 `SLG-xxx` 错误码和完整错误结构
8. **实现预算原子性**: `readBudget`/`writeBudget` 扣减与对象读取在同一事务

### 4.3 中期修复（近期）

9. **统一能力命名**: 新包中的 `StoryDraftSkill` 改为 `StoryDraftAssistSkill`，与文档一致
10. **明确旧包定位**: 在文档中说明 `storylock-skill-engine` 是迁移参考实现还是废弃
11. **实现nonce/requestId清理**: 滑动窗口清理策略
12. **实现session类型区分**: `one_shot`(3分钟), `short_session`(10分钟), `batch_session`(20分钟)
13. **实现答案规范化**: 去除空白、统一全角半角、统一大小写策略
14. **实现审计日志**: 记录操作但不记录敏感内容

---

## 五、一致性评分

| 维度 | 权重 | 得分(0-10) | 加权得分 |
|------|------|-----------|---------|
| 架构划分 | 15% | 9 | 1.35 |
| 能力边界 | 15% | 5 | 0.75 |
| 接口契约 | 15% | 6 | 0.90 |
| 安全规范 | 25% | 1 | 0.25 |
| 数据持久化 | 10% | 1 | 0.10 |
| 防重放机制 | 10% | 4 | 0.40 |
| 脱敏规范 | 5% | 1 | 0.05 |
| 错误码体系 | 5% | 1 | 0.05 |
| **总分** | **100%** | - | **3.85** |

**评级**: ⚠️ **部分一致（3.85/10）**

---

## 六、结论

StoryLock 的**文档设计是完整且合理的**，三层架构、能力边界、安全规范、接口契约、脱敏策略等设计文档形成了相对完整的安全框架。但**代码实现严重滞后于文档要求**，特别是在安全关键路径上存在致命缺口：

1. **challenge验证失效**（答案恒通过）使得整个访问控制机制形同虚设
2. **无加密持久化**使得所有敏感数据处于明文、易失状态
3. **无平台密钥存储**使得根密钥管理完全缺失
4. **无脱敏实现**使得远程侧可以获得完整私密内容

**建议**:
- **立即暂停功能扩展**，优先修复阻塞性安全漏洞（R1-R4）
- 在修复安全基础后，按"第二包 → 第一包 → 第三包"的顺序补齐实现
- 明确 `storylock-skill-engine` 的定位，避免新旧体系并行维护
- 建立代码与文档的同步检查机制，防止文档更新后代码滞后

---

*评审完成时间: 2026-06-16 20:36 GMT+8*
*评审人: 代码安全审计师*

---

## 七、2026-06-16 后续修复记录

### 7.1 已补齐项

1. 第二包 `storylock-local-story-access-skill` 已接入 SQLite host。
   - schema 文件: `skill/src/shared/sqlite-schema.sql`
   - host 实现: `skill/src/storylock-local-story-access-skill/access-host.js`
   - 覆盖 `challenge_state`、`session_store`、`request_store`、`nonce_store`、`failure_window`、`answer_digest_set`、`protected_story_objects`、`audit_log`

2. 已补齐基础密码学工具。
   - 文件: `skill/src/shared/crypto.js`
   - 覆盖 AES-256-GCM、HKDF-SHA256、HMAC-SHA256

3. 已补齐 `SecretStore` 接口与平台适配入口。
   - 文件: `skill/src/shared/secret-store.js`
   - 已实现 `MemorySecretStore`
   - 已实现 `WindowsCredentialSecretStore`
   - 已实现 `LinuxSecretServiceStore`
   - 已实现 `createPlatformSecretStore`
   - macOS Keychain 本阶段明确不处理

4. 已补充持久化安全约束。
   - 默认构造使用内存 SQLite 与内存 SecretStore，仅适用于开发/测试
   - 显式持久化 `dbPath` 必须同时提供 `usePlatformSecretStore: true` 或自定义持久 `secretStore`
   - 避免 `storylock_vault.db` 与易失 `masterSalt` 组合导致重启后无法解密

5. 已修复 challenge 答案恒通过问题。
   - 答案经 NFKC、trim、空白折叠、小写化后计算 HMAC-SHA256 摘要
   - SQLite 中仅保存 `answer_digest_set`

6. 已实现持久化 replay 防护。
   - `requestId` 写入 `request_store`
   - `nonce` 写入 `nonce_store`
   - 重放返回结构化 `SLG-008`

7. 已实现基础脱敏默认值。
   - 第二包默认 `redactionLevel = partial`
   - 私密故事正文默认只返回 `contentSummary`

8. 已实现失败窗口锁定。
   - 按 `identityId` 统计
   - 3 次失败后返回 `SLG-004`

9. 已补充审计写入。
   - replay 注册/拒绝
   - challenge 成功/失败
   - story read/write 成功

10. 已补充访问层自测与平台 SecretStore 可用性检查。
   - `skill/src/storylock-local-story-access-skill/scripts/selftest.mjs`
   - `npm run selftest`
   - `npm run check:secret-store`
   - 自测覆盖成功读、写、重放拒绝、失败锁定、审计落表、持久库 fail-closed

11. 已补充机器可读验收 schema。
   - `skill/src/storylock-local-story-access-skill/assets/schemas/access-response.schema.json`
   - `skill/src/storylock-local-story-access-skill/assets/schemas/selftest-report.schema.json`
   - 远程响应 schema 已统一 `redactionLevel = none | partial | full`

### 7.1.1 当前修复后状态摘要

| 原风险项 | 当前状态 | 证据 |
|---|---|---|
| R1 答案验证恒为 true | 已修复 | HMAC 摘要比对 + `npm run selftest` |
| R2 无加密存储 | 已缓解 | AES-256-GCM envelope + SQLite `protected_story_objects` |
| R3 无密钥派生 | 已缓解 | HKDF-SHA256 object/identity 派生 |
| R4 无平台密钥存储 | 已部分修复 | Windows/Linux adapter 已实现，真实环境待验证 |
| R5 失败计数按 challengeId | 已修复 | `identityId` failure window + lock selftest |
| R7 无脱敏实现 | 已缓解 | 默认 `partial` redaction + selftest |
| R8 裸 Error | 已缓解 | `SLG-xxx` 结构化错误 payload |
| R9 无 SQLite 持久化 | 已缓解 | SQLite host + schema |

### 7.2 仍需后续验证项

1. Windows Credential Manager 依赖 PowerShell `CredentialManager` 模块，需在目标 Windows 环境做真实读写验证。
2. Linux Secret Service 依赖 `secret-tool` 和用户会话密钥环，需在目标 Linux 桌面/服务环境做真实读写验证。
3. 当前 SQLite 使用 Node.js `node:sqlite`，该接口在当前 Node 版本仍标记为 experimental；生产环境可替换稳定 SQLite adapter。
