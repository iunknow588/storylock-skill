# StoryLock 代码与文档一致性评审意见

## 评审概述

**评审日期：** 2026-06-16  
**评审对象：**  
- 代码：`E:\2026OPC大赛\skill\src\`  
- 设计文档：`E:\2026OPC大赛\skill\docs\design\cn\`  

**评审维度：**  
1. 三层架构一致性  
2. 安全规范合规性  
3. 接口契约一致性  
4. 状态机实现完整性  
5. 防重放策略实现  
6. 密钥管理合规性  
7. 脱敏规范执行  
8. 代码安全漏洞  

---

## 一、总体评价

StoryLock 项目实现了文档定义的三层 Skill 架构：
- **第一包** `storylock-local-story-processing-skill`：本地故事处理
- **第二包** `storylock-local-story-access-skill`：本地故事访问控制
- **第三包** `storylock-remote-gateway-skill`：远程网关代理

代码与文档在**宏观架构层面基本一致**，但在**安全细节实现、状态机完整性、错误码规范、密钥管理**等方面存在多处不一致和潜在风险。

---

## 二、一致性分析

### 2.1 三层架构一致性 ✅ 基本符合

| 文档定义 | 代码实现 | 状态 |
|---------|---------|------|
| 第一包：本地故事处理 | `storylock-local-story-processing-skill` | ✅ 符合 |
| 第二包：本地故事访问 | `storylock-local-story-access-skill` | ✅ 符合 |
| 第三包：远程网关 | `storylock-remote-gateway-skill` | ✅ 符合 |
| 调用链：第三→第二→第一 | `index.js` 中通过 `transport` 委托 | ✅ 符合 |
| 第一包不直接对第三包开放 | 第一包无远程调用能力 | ✅ 符合 |

**结论：** 三层架构划分与文档一致。

### 2.2 安全规范一致性 ⚠️ 部分偏离

#### 2.2.1 算法基线 ✅ 符合

| 文档要求 | 代码实现 | 状态 |
|---------|---------|------|
| 对称加密：`AES-256-GCM` | `shared/crypto.js` 使用 `aes-256-gcm` | ✅ 符合 |
| 摘要/HMAC：`SHA-256`/`HMAC-SHA256` | `hmacSha256Hex` 实现 | ✅ 符合 |
| 密钥派生：`HKDF-SHA256` | `deriveHkdfSha256` 实现 | ✅ 符合 |
| 随机数：≥32字节 | `randomBytes(32)` | ✅ 符合 |

#### 2.2.2 GCM nonce 管理 ⚠️ 存在隐患

**文档要求：**
> "每次加密独立生成 96-bit 随机 nonce，同一密钥下不得复用 nonce"

**代码实现：**
```javascript
const nonce = randomBytes(12); // 96-bit
```

**问题：**
1. **nonce 随机生成无碰撞检测**：虽然 96-bit 随机 nonce 碰撞概率极低，但文档建议的"独立生成"在密码学上应理解为"确保唯一性"，而非仅依赖随机性。
2. **nonce 存储格式**：文档建议两种格式（二进制拼接/结构化JSON），代码使用结构化JSON（base64url），与文档一致，但缺少对 nonce 唯一性的持久化校验机制。

**风险等级：** 中  
**建议：** 增加 nonce 使用记录或采用计数器方案确保绝对唯一性。

#### 2.2.3 密钥层次 ⚠️ 实现不完整

**文档要求四层密钥层次：**
```
masterSalt → rootKey → workKey → objectKey
```

**代码实现：**
```javascript
// access-host.js 中直接使用 masterSalt 派生 objectKey
const key = deriveHkdfSha256(this.masterSalt, {
  salt: Buffer.from(`storylock:object:${storyObjectId}`),
  info: Buffer.from(`storylock:object:${storyObjectId}`)
});
```

**问题：**
1. **缺少 rootKey 和 workKey 中间层**：代码直接从 `masterSalt` 派生 `objectKey`，缺少文档要求的 `rootKey` 和 `workKey` 中间层。
2. **salt 和 info 使用相同值**：`salt` 和 `info` 都使用 `storylock:object:${storyObjectId}`，不符合 HKDF 最佳实践（salt 应随机，info 应描述用途）。

**风险等级：** 中  
**建议：** 按文档要求实现完整四层密钥层次，区分 salt 和 info 的用途。

#### 2.2.4 masterSalt 存储 ⚠️ 存在隐患

**文档要求：**
> "长期保存于操作系统密钥链，不允许明文写入普通配置文件"

**代码实现：**
```javascript
// secret-store.js 中 MemorySecretStore 作为默认回退
const DEFAULT_SECRET_STORE = new MemorySecretStore();

// createAccessHost 中允许不使用平台密钥存储
const resolvedSecretStore = secretStore ?? (usePlatformSecretStore ? createPlatformSecretStore() : DEFAULT_SECRET_STORE);
```

**问题：**
1. **默认使用内存存储**：当 `usePlatformSecretStore=false` 且未注入 `secretStore` 时，默认使用 `MemorySecretStore`，masterSalt 仅存于内存，进程重启即丢失。
2. **非持久化场景无警告**：虽然 `persistent=true` 时强制要求密钥存储，但非持久化场景（`:memory:`）下默认回退到内存存储，可能导致用户误以为数据安全。

**风险等级：** 中  
**建议：** 非持久化场景也应强制要求安全存储，或至少输出明确警告。

### 2.3 接口契约一致性 ⚠️ 部分偏离

#### 2.3.1 统一字段命名 ✅ 符合

文档要求统一使用 `camelCase`，代码实现一致。

#### 2.3.2 请求/响应结构 ⚠️ 字段缺失

**文档要求响应必须包含：**
```json
{
  "requestId", "status", "capability", "executionLocation",
  "result", "redactionLevel", "retentionGranted", "auditMeta", "error"
}
```

**代码实现（第二包）：**
```javascript
return {
  requestId,
  status: 'success',
  capability: 'requestStoryRead',
  executionLocation: 'local',
  result: { ... },
  redactionLevel,
  retentionGranted: 'result_only',
  auditMeta: { challengeId, sessionId },
  error: null,
};
```

**问题：**
1. **auditMeta 内容不完整**：文档要求 `auditMeta` 包含最小审计信息，但代码中仅包含 `challengeId` 和 `sessionId`，缺少 `timestamp`、`errorCode` 等字段。
2. **错误响应中 auditMeta 为空**：`toErrorResponse` 中 `auditMeta: {}`，不符合文档要求的最小故障审计信息。

**风险等级：** 低  
**建议：** 完善 auditMeta 字段，确保错误响应也包含最小审计信息。

#### 2.3.3 错误码规范 ⚠️ 不一致

**文档定义错误码：**
| 错误码 | 类型 | 含义 |
|-------|------|------|
| SLG-001 | CAPABILITY_NOT_AVAILABLE | 能力不可用 |
| SLG-002 | CHALLENGE_REQUIRED | 需要挑战 |
| SLG-003 | CHALLENGE_FAILED | 挑战失败 |
| ... | ... | ... |

**代码实现：**
```javascript
// access-host.js 中使用自定义 key 而非 SLG 错误码
err.key = 'CHALLENGE_LOCKED';  // 非 SLG 格式
err.code = 'SLG-008';          // 与文档定义的 SLG-008 含义不一致
```

**问题：**
1. **错误码与文档定义不一致**：代码中 `SLG-008` 被用于 "REDACTION_REQUIRED"（脱敏要求），但文档中 `SLG-008` 定义为 "REDACTION_REQUIRED"，而代码中实际用于 "replay_detected"。
2. **内部错误码与外部错误码混用**：`access-host.js` 使用 `err.key`（如 `CHALLENGE_LOCKED`、`SESSION_INVALID`），而 `index.js` 使用 `err.code`（如 `SLG-003`），缺乏统一映射。

**风险等级：** 中  
**建议：** 统一错误码体系，建立内部错误到 SLG 错误码的映射表。

### 2.4 状态机实现 ⚠️ 部分缺失

#### 2.4.1 状态定义 ✅ 基本符合

**文档要求状态：**
```
idle → challenge_created → answers_submitted → verified → session_active → session_expired
                    ↓
                  failed → locked
```

**代码实现：**
- `challenge_state` 表包含 `status` 字段，支持：
  - `challenge_created`
  - `answers_submitted`
  - `verified`
  - `failed`
  - `locked`

**问题：**
1. **缺少 `idle` 状态**：文档定义 `idle` 为初始状态，但代码中 challenge 直接创建为 `challenge_created`，没有 `idle` 状态。
2. **缺少 `session_active` 和 `session_expired` 状态**：session 状态存储在 `session_store` 表的 `status` 字段中，但 challenge 状态机未包含 session 状态转换。

**风险等级：** 低  
**建议：** 明确状态机边界，challenge 状态与 session 状态可分离管理。

#### 2.4.2 状态转换条件 ⚠️ 不完整

**文档要求：**
> "以 challengeId 为粒度加本地锁，每次状态迁移都校验当前状态是否符合预期"

**代码实现：**
```javascript
// submitChallengeAnswers 中直接更新状态，无状态校验
this.db.prepare('UPDATE challenge_state SET status = ? WHERE challenge_id = ?').run('answers_submitted', challengeId);
```

**问题：**
1. **缺少状态转换校验**：代码直接更新状态，未校验当前状态是否允许转换（如从 `challenge_created` 才能到 `answers_submitted`）。
2. **缺少并发控制**：虽然使用了 `BEGIN IMMEDIATE`，但未在 UPDATE 中加入 `AND status = ?` 条件校验。

**风险等级：** 中  
**建议：** 增加状态转换校验，确保状态迁移的合法性。

#### 2.4.3 锁定策略 ⚠️ 实现不完整

**文档要求：**
> "maxRetryCount 默认值为 3，达到阈值后进入 locked，锁定窗口默认 15 分钟"

**代码实现：**
```javascript
const MAX_FAILURES_PER_WINDOW = 3;
const FAILURE_LOCK_MS = 15 * 60 * 1000;
```

**问题：**
1. **锁定后无自动解锁机制**：文档要求 `locked -> idle`（锁定窗口结束后），但代码中 `locked` 状态仅通过时间判断，未自动清理或状态转换。
2. **失败计数按 identityId 而非 challengeId**：文档要求 "按 identityId + 时间窗口统计"，代码实现符合，但未在 challenge 创建时重置失败计数。

**风险等级：** 低  
**建议：** 增加定时清理或查询时自动解锁逻辑。

### 2.5 防重放策略 ⚠️ 部分偏离

#### 2.5.1 requestId 幂等 ✅ 基本实现

**代码实现：**
```javascript
ensureReplaySafe(requestId, nonce, expiry) {
  const existingRequest = this.db.prepare('SELECT request_id FROM request_store WHERE request_id = ?').get(requestId);
  if (existingRequest) {
    throw err; // SLG-009 DUPLICATE_REQUEST
  }
}
```

**问题：**
1. **缺少相同 requestId 返回同一结果**：文档要求 "相同 requestId 的请求若已成功执行，返回同一结果摘要"，但代码中直接抛出错误。
2. **缺少 requestId 与负载绑定校验**：文档要求 "相同 requestId 且负载不同，直接拒绝"，代码未实现。

**风险等级：** 低  
**建议：** 实现 requestId 到响应结果的缓存，支持幂等返回。

#### 2.5.2 nonce 去重 ✅ 基本实现

**代码实现：**
```javascript
const existingNonce = this.db.prepare('SELECT nonce FROM nonce_store WHERE nonce = ?').get(nonce);
if (existingNonce) {
  throw err; // SLG-010 NONCE_REPLAY_DETECTED
}
```

**问题：**
1. **nonce 清理策略不完整**：文档建议 "滑动时间窗口，窗口大小默认 2 x maxTTL"，代码中清理逻辑为 `cutoff = nowMs() - REPLAY_WINDOW_MS`（24小时），但未考虑 `maxTTL` 动态调整。
2. **nonce 索引未优化**：虽然创建了 `idx_nonce_store_expiry`，但 nonce 查询使用主键索引，符合要求。

**风险等级：** 低  
**建议：** 根据实际 maxTTL 动态调整清理窗口。

#### 2.5.3 expiry 校验 ⚠️ 容差处理不一致

**文档要求：**
> "expiry 校验默认允许 +/- 30 秒 容差"

**代码实现：**
```javascript
const REPLAY_DRIFT_MS = 30_000;
// 多处使用：challenge.expires_at + REPLAY_DRIFT_MS <= nowMs()
// 以及：expiry + REPLAY_DRIFT_MS <= nowMs()
```

**问题：**
1. **容差方向不一致**：文档要求 "±30秒"，但代码中仅对过期时间增加容差（`+REPLAY_DRIFT_MS`），对未到期时间未减容差（`-REPLAY_DRIFT_MS`）。
2. **request_store 清理使用负容差**：`DELETE FROM request_store WHERE expiry <= nowMs() - REPLAY_DRIFT_MS`，逻辑正确，但与文档表述的 "容差" 概念不完全一致。

**风险等级：** 低  
**建议：** 统一容差处理逻辑，明确文档与代码的映射关系。

### 2.6 挑战答案存储策略 ⚠️ 部分偏离

#### 2.6.1 原始答案内存处理 ✅ 符合

**文档要求：**
> "原始 challenge answers 不做长期持久化，只允许存在于本地短时内存"

**代码实现：**
```javascript
const normalizedAnswers = answers.map(normalizeAnswerValue).filter(Boolean);
// 计算摘要后立即丢弃原始答案
const answerDigests = normalizedAnswers.map((answer) => hmacSha256Hex(salt, answer));
```

**问题：**
1. **未显式清理 normalizedAnswers**：虽然 `normalizedAnswers` 在函数作用域内会被 GC，但文档建议显式清零 Buffer。
2. **答案以字符串数组传入**：`normalizeAnswerValue` 返回字符串，未使用 Buffer，不符合文档建议的 "优先使用 Buffer"。

**风险等级：** 低  
**建议：** 将答案处理改为 Buffer 流程，校验后显式清零。

#### 2.6.2 答案摘要持久化 ✅ 符合

**代码实现：**
```javascript
// answer_digest_set 表存储 HMAC 摘要
INSERT INTO answer_digest_set (identity_id, answer_digest, created_at)
```

**问题：**
1. **缺少 normalizationVersion 记录**：文档建议记录规范化版本，代码未实现。
2. **缺少 digestAlgorithm 和 saltStrategyVersion**：文档建议记录摘要算法和盐值策略版本，代码未实现。

**风险等级：** 低  
**建议：** 增加版本字段，支持未来算法升级。

### 2.7 脱敏规范 ⚠️ 执行不完整

#### 2.7.1 脱敏级别 ✅ 基本实现

**代码实现：**
```javascript
redactionLevel === 'none' ? storyObject : {
  storyObjectId: storyObject.storyObjectId,
  title: storyObject.title,
  version: storyObject.version,
  sensitivity: storyObject.sensitivity,
  contentSummary: `[redacted:${storyObject.content.length} chars]`,
};
```

**问题：**
1. **脱敏逻辑过于简单**：仅对 `content` 做长度摘要，未检查高敏字段（如真实姓名、手机号、邮箱等）。
2. **缺少正则匹配**：文档建议 "先做结构化字段检查，再做正则匹配"，代码未实现。
3. **partial 和 full 级别区分不明显**：代码中 `partial` 和 `full` 的处理逻辑相同。

**风险等级：** 中  
**建议：** 实现完整的脱敏检查清单，区分 partial 和 full 级别。

#### 2.7.2 审计日志 ⚠️ 不完整

**代码实现：**
```javascript
recordAudit(eventType, { identityId, storyObjectId, requestId, result }) {
  INSERT INTO audit_log (event_type, identity_id, story_object_id, request_id, result, created_at)
}
```

**问题：**
1. **缺少脱敏级别记录**：文档要求审计日志记录 "应用的脱敏级别"，代码未记录。
2. **缺少高敏字段检测结果**：文档要求记录 "是否存在高敏字段"，代码未记录。

**风险等级：** 低  
**建议：** 完善审计日志字段，记录脱敏操作详情。

### 2.8 本地 Agent 网关设计 ⚠️ 部分偏离

#### 2.8.1 白名单接口 ✅ 符合

**文档要求暴露：**
- `requestStoryRead`
- `requestStoryWrite`
- `requestChallengeSign`
- `queryStoryMetadata`

**代码实现（第三包）：**
- `requestStoryRead` ✅
- `requestStoryWrite` ✅
- `requestChallengeSign` ✅
- `queryStoryMetadata` ✅

**问题：**
1. **缺少 `requestCapabilityStatus`**：文档要求暴露，代码未实现。
2. **缺少 `requestPasswordFill` 和 `requestLocalStoryAssist`**：文档要求暴露，代码未实现。

**风险等级：** 低  
**建议：** 补充缺失的网关接口。

#### 2.8.2 禁止暴露的内部接口 ✅ 符合

**代码实现：**
- 第三包未暴露 `createChallenge`、`submitChallengeAnswers`、`readSecretObject` 等内部接口。

**结论：** 符合文档要求。

### 2.9 EIP-712 最小请求定义 ⚠️ 未实现

**文档要求：**
> "当前阶段最小可实现的 EIP-712 请求结构"

**代码实现：**
- 第三包 `requestChallengeSign` 中未包含 EIP-712 结构。
- 仅传递 `algorithm`、`payload` 等字段，无 `domain`、`types`、`value`。

**风险等级：** 低（文档标注为"待实现"）  
**建议：** 按文档要求实现 EIP-712 结构化请求格式。

---

## 三、代码安全漏洞发现

### 3.1 高危漏洞

#### 3.1.1 硬编码默认答案（高危）

**位置：** `access-host.js:enrollDefaultAnswers()`

```javascript
enrollDefaultAnswers() {
  for (const identityId of ['identity-001', 'id-1', 'id-2']) {
    const existing = this.db.prepare('SELECT answer_digest FROM answer_digest_set WHERE identity_id = ? LIMIT 1').get(identityId);
    if (!existing) {
      this.enrollAnswers(identityId, ['normalized answer', 'correct answer']);
    }
  }
}
```

**风险：**
1. **硬编码默认答案**：所有未注册 identity 都使用相同的默认答案 `['normalized answer', 'correct answer']`。
2. **可预测性攻击**：攻击者知道默认答案即可通过 challenge。
3. **生产环境风险**：若用户未自定义答案，系统使用默认答案，形同虚设。

**修复建议：**
1. 移除硬编码默认答案。
2. 强制要求用户在首次使用时设置自定义答案。
3. 或生成随机默认答案并安全存储。

---

### 3.2 中危漏洞

#### 3.2.1 错误信息泄露（中危）

**位置：** `access-host.js` 多处

```javascript
if (challenge.identity_id !== identityId) {
  const err = new Error('challenge identity mismatch');
  err.key = 'SCOPE_INSUFFICIENT';
  throw err;
}
```

**风险：**
1. **错误信息暴露内部状态**："challenge identity mismatch" 暴露了 challenge 存在但身份不匹配的信息。
2. **可被用于枚举攻击**：攻击者可通过不同错误信息区分 challenge 是否存在、身份是否匹配。

**修复建议：**
1. 统一错误信息为模糊表述，如 "challenge verification failed"。
2. 不区分 "challenge not found" 和 "identity mismatch"。

#### 3.2.2 缺少输入长度限制（中危）

**位置：** `index.js` 多处

```javascript
function normalizeString(value, fieldName) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return value.trim();
}
```

**风险：**
1. **无长度上限**：`identityId`、`storyObjectId` 等字段无长度限制，可能导致：
   - 数据库性能问题（超长字符串索引）
   - 内存耗尽攻击（超大 payload）
   - 日志注入攻击

**修复建议：**
1. 增加字段长度限制（如 `identityId` ≤ 128 字符）。
2. 对 `content` 等对象增加大小限制。

#### 3.2.3 SQL 注入风险（中危）

**位置：** `access-host.js` 多处

**风险：**
1. **参数化查询使用正确**：代码使用 `db.prepare().run()` 参数化查询，基本安全。
2. **但 `serializeJson()` 返回的字符串直接插入**：虽然参数化，但若 `serializeJson` 被篡改，可能引入风险。

**修复建议：**
1. 确保 `serializeJson` 仅返回合法 JSON 字符串。
2. 对动态 SQL 构建场景增加审计。

#### 3.2.4 敏感数据内存残留（中危）

**位置：** `shared/secret-store.js`

```javascript
class MemorySecretStore {
  getSecret(key) {
    const value = this.secrets.get(key);
    return value ? Buffer.from(value) : null;
  }
}
```

**风险：**
1. **Buffer 未清零**：从 Map 取出的 Buffer 使用后未显式清零。
2. **Map 删除不等于内存清除**：`deleteSecret` 仅从 Map 删除引用，底层内存可能仍保留。

**修复建议：**
1. 实现 `wipeBuffer` 工具函数。
2. 在 `deleteSecret` 中先清零再删除。

#### 3.2.5 平台密钥存储回退风险（中危）

**位置：** `secret-store.js:createPlatformSecretStore()`

```javascript
export function createPlatformSecretStore({ platform = process.platform, allowMemoryFallback = false } = {}) {
  if (platform === 'win32') {
    return new WindowsCredentialSecretStore();
  }
  if (platform === 'linux') {
    return new LinuxSecretServiceStore();
  }
  if (allowMemoryFallback) {
    return new MemorySecretStore({ developmentMode: true });
  }
  throw new Error(`No platform SecretStore adapter configured for ${platform}`);
}
```

**风险：**
1. **macOS 无支持**：文档标注 "macOS Keychain is intentionally out of scope"，但 macOS 用户将被迫使用 `allowMemoryFallback` 或报错。
2. **Windows 依赖外部模块**：`CredentialManager` PowerShell 模块非系统自带，可能未安装。

**修复建议：**
1. 增加 macOS Keychain 支持。
2. 在 Windows 上增加模块检测和安装提示。

---

### 3.3 低危漏洞

#### 3.3.1 时间函数可预测（低危）

**位置：** `access-host.js`

```javascript
function nowMs() {
  return Date.now();
}
```

**风险：**
1. **时间可被系统时钟影响**：若系统时钟被篡改，challenge TTL、session 过期等安全机制失效。

**修复建议：**
1. 使用单调时钟（如 `process.hrtime.bigint()`）辅助校验。
2. 或增加 NTP 同步检测。

#### 3.3.2 随机数生成器依赖（低危）

**位置：** `access-host.js`

```javascript
function makeId(prefix) {
  return `${prefix}-${randomBytes(4).toString('hex')}-${Date.now().toString(16)}`;
}
```

**风险：**
1. **ID 可预测**：`randomBytes(4)` 仅 32-bit 随机性，加上时间戳，整体可预测性较高。
2. **不适用于高安全场景**：challengeId、sessionId 等安全敏感标识应使用更高熵值。

**修复建议：**
1. 增加随机字节长度至 16 字节（128-bit）。
2. 或采用 CSPRNG 生成完全随机标识。

#### 3.3.3 日志记录敏感信息（低危）

**位置：** `access-host.js:recordAudit()`

```javascript
recordAudit(eventType, { identityId, storyObjectId, requestId, result }) {
  // 未记录 result 内容是否包含敏感信息
}
```

**风险：**
1. **result 字段可能包含敏感信息**：调用方可能传入包含敏感数据的 result。

**修复建议：**
1. 对 audit_log 的 result 字段做脱敏处理。
2. 或限制 result 仅允许预定义的安全值。

---

## 四、评审结论

### 4.1 一致性评分

| 评审维度 | 评分 | 说明 |
|---------|------|------|
| 三层架构一致性 | 90% | 架构划分符合文档，部分接口未完全实现 |
| 安全规范一致性 | 75% | 算法基线符合，密钥层次和存储有偏差 |
| 接口契约一致性 | 80% | 字段命名符合，错误码和 auditMeta 有偏差 |
| 状态机完整性 | 70% | 基本状态实现，缺少转换校验和自动解锁 |
| 防重放策略 | 85% | 基本实现，幂等返回和动态清理待完善 |
| 密钥管理合规性 | 70% | 算法正确，层次不完整，存储回退有风险 |
| 脱敏规范执行 | 65% | 基本实现，缺少高敏字段检测和分级处理 |
| 代码安全漏洞 | - | 发现 1 个高危、5 个中危、3 个低危 |

**综合一致性评分：76%**

### 4.2 关键问题清单

| 优先级 | 问题 | 影响 | 建议修复时间 |
|-------|------|------|------------|
| P0 | 硬编码默认答案 | 安全绕过 | 立即 |
| P1 | 密钥层次不完整 | 密钥管理风险 | 1-2 天 |
| P1 | 错误信息泄露 | 信息枚举 | 1-2 天 |
| P1 | 错误码不一致 | 调试困难 | 2-3 天 |
| P2 | 状态转换无校验 | 并发竞态 | 3-5 天 |
| P2 | 脱敏逻辑不完整 | 数据泄露 | 3-5 天 |
| P2 | 输入无长度限制 | DoS 攻击 | 3-5 天 |
| P3 | EIP-712 未实现 | 功能缺失 | 后续迭代 |
| P3 | macOS 密钥存储缺失 | 平台支持 | 后续迭代 |

### 4.3 总体建议

1. **立即修复高危漏洞**：移除硬编码默认答案，强制用户自定义。
2. **完善密钥管理**：实现完整四层密钥层次，区分 salt 和 info。
3. **统一错误码体系**：建立内部错误到 SLG 错误码的映射。
4. **加强状态机校验**：增加状态转换条件校验和并发控制。
5. **完善脱敏逻辑**：实现高敏字段检测清单，区分 partial/full 级别。
6. **增加输入校验**：对所有输入字段增加长度和格式限制。
7. **补充缺失接口**：实现 `requestCapabilityStatus`、`requestPasswordFill` 等网关接口。
8. **完善审计日志**：记录脱敏级别、高敏字段检测结果等。

---

*评审完成。本评审意见基于 2026-06-16 的代码和文档状态，后续迭代应持续更新一致性检查。*
