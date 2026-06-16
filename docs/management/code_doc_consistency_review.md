# StoryLock 代码与文档一致性评审报告

**评审日期**: 2026-06-16  
**评审范围**: `E:\2026OPC大赛\skill\src` (代码) vs `E:\2026OPC大赛\skill\docs\design\cn` (设计文档)  
**评审方法**: 逐条对比文档要求与代码实现  
**总体一致性评分**: **72%**

---

## 一、总体一致性评分说明

| 维度 | 一致性 | 说明 |
|------|--------|------|
| 三层架构划分 | ✅ 100% | 代码目录结构与文档完全一致 |
| 第二包核心安全机制 | ✅ 85% | 状态机、防重放、脱敏基本实现 |
| 错误码体系 | ⚠️ 60% | 部分映射正确，部分缺失/错位 |
| 第一包功能 | ✅ 80% | 基本实现，但缺少部分边界标记 |
| 第三包网关 | ✅ 70% | 结构正确，但部分能力未完整实现 |
| 安全规范 | ⚠️ 65% | 加密实现正确，但密钥管理有偏差 |
| 文档未覆盖的代码 | ⚠️ - | 存在文档未定义但代码已实现的部分 |

---

## 二、按文档逐条对比

### 2.1 三包拆分策略 (`storylock_three_skill_packages_cn.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 拆分为三个包 | ✅ `storylock-local-story-processing-skill`, `storylock-local-story-access-skill`, `storylock-remote-gateway-skill` | 100% |
| 第一包纯本地 | ✅ 无远程调用逻辑 | 100% |
| 第二包纯本地 | ✅ 无远程调用逻辑 | 100% |
| 第三包远程 | ✅ 通过 `transport` 函数委托 | 100% |
| 调用链：第三→第二→第一 | ⚠️ 第三包只到第二包，第二包未调用第一包 | 50% |
| Pharos 放在第三层 | ⚠️ 代码中未体现 Pharos 集成 | 0% |

**发现**: 第二包 (`storylock-local-story-access-skill`) 未调用第一包 (`storylock-local-story-processing-skill`)，文档要求的"读取后交给本地故事处理"链路未实现。

---

### 2.2 三包接口契约 (`三包接口契约.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 统一请求字段 (requestId, capability, scope, payload, policyHints, requestedRetention, nonce, expiry) | ✅ 第三包 `normalizeEnvelope` 实现 | 100% |
| 统一响应字段 (requestId, status, capability, executionLocation, result, redactionLevel, retentionGranted, auditMeta, error) | ✅ 第二包 `StoryReadAccessSkill.run()` / `StoryWriteAccessSkill.run()` 返回结构匹配 | 100% |
| 错误响应必须返回完整外层结构 | ✅ `toErrorResponse()` 实现 | 100% |
| 契约A：读取后交给本地故事处理 | ❌ 未实现 | 0% |
| 契约B：处理结果写回请求 | ❌ 未实现 | 0% |
| 契约C：远程请求读取故事对象 | ✅ `requestStoryRead` 实现 | 100% |
| 契约D：远程请求写回故事对象 | ✅ `requestStoryWrite` 实现 | 100% |
| 契约E：远程请求本地签名 | ⚠️ `requestChallengeSign` 结构正确，但无实际签名执行 | 50% |
| 统一 camelCase 字段命名 | ✅ 代码使用 camelCase | 100% |
| 禁止越权规则 | ⚠️ 代码层面无显式禁止第三包调用第一包的校验 | 30% |

**发现**: 契约A和契约B（第一包与第二包之间的接口）完全未实现。第二包读取的故事对象未交给第一包处理，第一包的处理结果也未通过第二包写回。

---

### 2.3 Skill定位与边界 (`Skill定位与边界.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Skill层：自说明、输入输出契约 | ✅ 所有 Skill 类有 `skillId()` 方法 | 100% |
| Skill层：不定义信任边界 | ⚠️ 部分 Skill 直接访问 host 方法 | 70% |
| Host层：createChallenge, submitChallengeAnswers, readSecretObject | ✅ `access-host.js` 中 `SqliteStore` 实现 | 100% |
| Host层：真正的敏感执行边界 | ✅ `createAccessHost()` 封装 | 100% |
| Storage层：保存加密对象 | ✅ SQLite 存储加密对象 | 100% |
| 远程安全 Skill：StoryDraftAssistSkill, StoryRefineAssistSkill, StrengthReviewSkill | ✅ 第一包和 engine 中实现 | 100% |
| 本地专属 Skill：LoginAuthorizationSkill, LocalPasswordFillSkill, SigningAuthorizationSkill, ChallengeSigningAuthorizationSkill | ✅ engine 中实现 | 100% |
| 混合编排：VideoPublishAgentDemo | ✅ engine 中实现 | 100% |
| 远程 Agent 不持有原始秘密 | ✅ 第三包无 secret 访问 | 100% |
| 本地 Agent 负责敏感执行 | ✅ 第二包纯本地 | 100% |

---

### 2.4 对象访问策略 (`对象访问策略.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 对象分类：story_public, story_private, auth_login, auth_signing, review_data | ⚠️ 代码中无显式对象分类字段 | 20% |
| 访问强度 L1-L5 | ⚠️ 代码中无 L1-L5 分级实现 | 10% |
| 渐进式挑战策略：L2=6-of-24, L3=12-of-24, L4=22-of-24 | ❌ 代码中 challenge 只验证单个答案，无 24 题题集 | 0% |
| 会话模型：one_shot, short_session, batch_session, privileged_session | ✅ `sessionType` 字段存在，但代码中只使用 `one_shot` | 40% |
| 远程能力保留规则 | ⚠️ `retentionGranted` 字段存在，但无策略引擎 | 30% |
| 本地执行强制规则 | ✅ 第二包纯本地执行 | 100% |
| 访问决策表 | ❌ 无策略决策表实现 | 0% |
| 挑战配置示例 | ❌ 无配置系统 | 0% |
| 第三层可见信息边界 | ⚠️ 脱敏实现部分正确 | 60% |

**严重偏离**: 文档要求 24 题题集渐进式挑战，但代码中 `enrollAnswers` 只存储单个答案摘要，`createChallenge` 只返回一个期望摘要集合，无 24 题题集概念。

---

### 2.5 Challenge状态机 (`Challenge状态机.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 状态列表：idle, challenge_created, answers_submitted, verified, session_active, session_expired, failed, locked | ✅ 数据库字段和代码逻辑覆盖 | 90% |
| 状态转换：idle -> challenge_created | ✅ `createChallenge()` 实现 | 100% |
| 状态转换：challenge_created -> answers_submitted | ✅ `submitChallengeAnswers()` 中 UPDATE status | 100% |
| 状态转换：answers_submitted -> verified | ✅ 验证通过后 UPDATE status='verified' | 100% |
| 状态转换：answers_submitted -> failed | ✅ 验证失败后 UPDATE status='failed' 或 'locked' | 100% |
| 状态转换：verified -> session_active | ✅ `issueSession()` 创建 session | 100% |
| 状态转换：session_active -> session_expired | ✅ TTL 到期和预算耗尽处理 | 100% |
| 状态转换：failed -> locked | ✅ 失败次数达到阈值后 locked | 100% |
| 状态转换：locked -> idle | ❌ 代码中无自动解锁回 idle 逻辑 | 0% |
| maxRetryCount 默认值 3 | ✅ `MAX_FAILURES_PER_WINDOW = 3` | 100% |
| 失败计数按 identityId + 时间窗口 | ✅ `failure_window` 表实现 | 100% |
| 时间窗口 24 小时 | ✅ `FAILURE_WINDOW_MS = 24 * 60 * 60 * 1000` | 100% |
| 锁定窗口 15 分钟 | ✅ `FAILURE_LOCK_MS = 15 * 60 * 1000` | 100% |
| 成功验证后重置失败计数 | ✅ `submitChallengeAnswers()` 中重置 | 100% |
| locked 状态返回 retryAfter | ✅ `retryAfter` 字段返回 | 100% |
| 并发控制：以 challengeId 为粒度串行化 | ⚠️ SQLite 事务提供部分保护，但无显式 challengeId 级锁 | 50% |
| 默认时长：challenge TTL 5分钟 | ✅ `expiresAt = nowMs() + 5 * 60 * 1000` | 100% |
| 默认时长：one_shot session TTL 3分钟 | ✅ `ttlMs = 3 * 60 * 1000` | 100% |
| 默认时长：locked 窗口 15分钟 | ✅ `FAILURE_LOCK_MS` | 100% |
| 状态持久化：运行态 vs 持久态 | ⚠️ 全部持久化到 SQLite，无内存运行态区分 | 60% |
| 设备重启后恢复策略 | ❌ 未实现恢复逻辑 | 0% |

**发现**: `locked` 状态无自动解锁机制。代码中 `locked` 状态通过 `lock_until` 字段记录，但 `createChallenge()` 只检查 `window.lockedUntil > nowMs()`，不会自动将状态改回 `idle`。

---

### 2.6 Session与防重放策略 (`Session与防重放策略.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Session 绑定字段：challengeId, identityId, scope, resourceScope, expiresAt, readBudget/writeBudget | ✅ `session_store` 表字段完整 | 100% |
| Session 类型：one_shot, short_session, batch_session, privileged_session | ✅ `session_type` 字段支持，但只使用 one_shot | 40% |
| 防重放：requestId 幂等 | ✅ `request_store` 表 + `ensureReplaySafe()` 实现 | 100% |
| 防重放：nonce 去重 | ✅ `nonce_store` 表 + `ensureReplaySafe()` 实现 | 100% |
| 防重放：expiry 校验 | ✅ `normalizeRequestEnvelope()` 中校验 | 100% |
| 时钟漂移容差 +/- 30秒 | ✅ `REPLAY_DRIFT_MS = 30_000` | 100% |
| 预算扣减原子性 | ⚠️ 使用 SQLite 事务，但非单语句原子扣减 | 70% |
| 统一使用 SQLite 单文件数据库 | ✅ `storylock_vault.db` 概念实现 | 100% |
| Nonce 存储格式：uint256 转 hex | ⚠️ 代码中 nonce 为任意字符串，无 uint256 限制 | 30% |
| Nonce 清理策略：滑动窗口、批次大小、索引 | ⚠️ 有 `created_at` 和 `expiry` 索引，但无后台清理 | 50% |
| RequestId 清理策略 | ⚠️ 同上，无后台清理 | 50% |
| 版本兼容窗口与 Session 关系 | ❌ 未实现 | 0% |
| EIP-712 中 nonce/expiry 放入 value 字段 | ✅ 第三包 `buildEip712Request()` 实现 | 100% |
| 防重放判定由本地网关执行 | ✅ 第二包 `ensureReplaySafe()` 执行 | 100% |
| 当前阶段最小实现：requestId 幂等、nonce 去重、expiry 校验、session TTL、预算校验 | ✅ 全部实现 | 100% |
| 暂不要求多设备同步或分布式一致性 | ✅ 未实现 | 100% |

**发现**: 预算扣减在事务中完成，但分两步（先 SELECT 校验，再 UPDATE），存在竞态窗口。建议合并为单条 `UPDATE ... WHERE read_budget > 0` 语句。

---

### 2.7 EIP-712最小请求定义 (`EIP-712最小请求定义.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Domain：name="StoryLock", version="1-placeholder", chainId=1, verifyingContract=0x000...000 | ✅ 第三包 `buildEip712Request()` 实现 | 100% |
| Types：StoryLockSignatureRequest | ⚠️ 第三包使用 `ChallengeSignRequest` 类型，字段不同 | 40% |
| Value 字段：action, resource, scope, expiry, nonce, requestedBy, delegationContext | ⚠️ 实际字段为 identityId, keyId, algorithm, payload, resourceId, primaryRole | 30% |
| 远程网关发送结构化请求对象而非原始字符串 | ✅ 发送结构化对象 | 100% |
| 当前阶段不解决多链统一domain、合约验证、EIP-1271返回格式 | ✅ 未实现 | 100% |
| 生产前检查清单 | ❌ 无检查逻辑 | 0% |

**严重偏离**: 文档要求 EIP-712 类型为 `StoryLockSignatureRequest`，字段为 action/resource/scope/expiry/nonce/requestedBy/delegationContext。但代码中实际类型为 `ChallengeSignRequest`，字段为 identityId/keyId/algorithm/payload/resourceId/primaryRole。两者结构完全不同。

---

### 2.8 本地Agent网关设计 (`本地Agent网关设计.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 当前阶段允许对外暴露：requestStoryRead, requestStoryWrite, requestChallengeSign, requestStrengthReview, requestCapabilityStatus, queryStoryMetadata | ⚠️ 第三包实现了 requestStoryRead, requestStoryWrite, requestChallengeSign, requestCapabilityStatus, requestPasswordFill, requestLocalStoryAssist, queryStoryMetadata，但缺少 requestStrengthReview | 70% |
| 当前阶段禁止对外暴露：createChallenge, submitChallengeAnswers, activateSession, readSecretObject, deriveKey, consumeReadBudget, expireSession | ✅ 第三包未暴露这些接口 | 100% |
| 请求结构统一字段 | ✅ `normalizeEnvelope()` 实现 | 100% |
| 响应结构统一字段 | ✅ 第二包返回结构匹配 | 100% |
| 错误码：SLG-001 到 SLG-012 | ⚠️ 部分正确，部分映射错位（见 2.11） | 60% |
| HTTP 状态码映射建议 | ❌ 未实现 HTTP 层 | 0% |
| 性能基准：requestCapabilityStatus < 50ms, challenge 创建 < 100ms 等 | ❌ 无性能测试 | 0% |
| 本地签名委托定位 | ⚠️ 第三包有 `requestChallengeSign`，但无实际签名执行 | 50% |
| 远程可保留内容规则 | ⚠️ `retentionGranted` 字段存在，但无策略引擎 | 30% |
| 分层校验责任：第三层校验 capability/结构/格式，第二层校验 challenge/session/scope | ⚠️ 第三层有基础校验，第二层有安全校验，但无明确分层校验清单 | 60% |

---

### 2.9 代理签名机制协议参考 (`代理签名机制协议参考.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 第三方 Skill 不应直接持有私钥 | ✅ 第三包无密钥访问 | 100% |
| 第三方 Skill 提交结构化签名请求 | ✅ `requestChallengeSign` 结构正确 | 100% |
| 本地 Skill / Host 在授权后完成签名 | ⚠️ 第二包无实际签名执行能力 | 30% |
| 返回结构化签名结果而非私钥 | ⚠️ 无实际签名，无法验证 | 0% |
| 参考 EIP-712 用于请求表示 | ✅ 第三包实现 | 100% |
| 参考 EIP-1271 用于签名验证兼容性 | ❌ 未实现 | 0% |
| 参考 HDP Protocol 用于委托审计链 | ❌ 未实现 | 0% |
| 参考 ERC-4337 / EIP-7702 用于执行层扩展 | ❌ 未实现 | 0% |
| 本地密钥存储：操作系统密钥链优先 | ✅ `createPlatformSecretStore()` 实现 Windows/Linux 适配 | 100% |
| 多身份密钥管理：派生式管理 | ✅ `deriveIdentityAnswerKey()`, `deriveObjectKey()` 实现 | 100% |
| 签名审计：记录 requestId, scope, resource, signatureHash, timestamp | ⚠️ `audit_log` 表有 event_type/identity_id/request_id/timestamp，但无 scope/resource/signatureHash | 50% |
| 签名算法选择：secp256k1 优先 EVM，ed25519 通用 | ✅ 第三包 `ensureAllowedAlgorithm()` 限制 | 100% |
| 算法选择决策树 | ⚠️ 代码只校验算法名，无决策逻辑 | 30% |
| EIP-712 适配代码示例 | ⚠️ 代码实现与文档示例字段不一致 | 40% |

---

### 2.10 挑战答案存储策略 (`挑战答案存储策略.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 原始答案不做长期持久化 | ✅ 答案只存摘要 (`answer_digest_set` 表) | 100% |
| 原始答案只存在于短时内存 | ⚠️ 代码中答案以字符串传入，无 Buffer 转换 | 40% |
| 持久化只保存派生结果 | ✅ 只存 HMAC 摘要 | 100% |
| 远程/第三方不得接触答案原文 | ✅ 答案不离开第二包 | 100% |
| 审计日志不记录答案 | ✅ `audit_log` 无答案字段 | 100% |
| 答案规范化：去除首尾空白、统一全角半角、统一大小写 | ✅ `normalizeAnswerValue()` 实现 NFKC + trim + 空格归一 + lowerCase | 100% |
| 规范化版本兼容策略 | ❌ 无 `normalizationVersion` 字段 | 0% |
| 题集主档与 challenge 实例档分开 | ❌ 无题集主档概念 | 0% |
| 题集主档静态加密 | ❌ 无题集主档 | 0% |
| 内存清理：Buffer 清零 | ❌ 代码使用字符串，未使用 Buffer | 0% |
| 内存清理：数组元素覆写 | ❌ 未实现 | 0% |

**发现**: 文档建议使用 Buffer 并清零，但代码中答案全程以字符串处理。JavaScript 字符串不可变，无法安全清除。

---

### 2.11 脱敏规范 (`脱敏规范.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 三个级别：none, partial, full | ✅ `redactionLevel` 支持三个级别 | 100% |
| 默认规则：第一层-第二层可用 none | ⚠️ 代码中默认 `partial` | 60% |
| 默认规则：第二层-第三层优先 partial | ✅ 默认 `partial` | 100% |
| 默认规则：第三层-第三方优先 partial/full | ✅ 高敏时强制 full | 100% |
| 高敏字段清单 | ✅ `HIGH_SENSITIVITY_FIELD_PATTERN` 覆盖 password/secret/token/privateKey/mnemonic/seed/phone/email/idCard/credential | 100% |
| 高敏值模式：邮箱、手机号、身份证号、hex64 | ✅ `HIGH_SENSITIVITY_VALUE_PATTERNS` 覆盖 | 100% |
| 最小脱敏检查清单 | ✅ `collectSensitiveSignals()` 实现 | 100% |
| 局部遮盖：电话/邮箱/用户名 | ❌ 未实现局部遮盖，只实现全量 redaction | 0% |
| 摘要替换：私密故事正文 | ⚠️ `contentSummary` 显示 `[redacted:N chars]` | 50% |
| 结构保留、值清空 | ✅ `redactStoryObject()` / `redactWriteResult()` 实现 | 100% |
| 各对象类别默认脱敏级别 | ❌ 无对象类别概念 | 0% |
| 审计要求：记录原始对象类别、脱敏级别、高敏字段存在性 | ⚠️ `auditMeta` 记录 `hasHighSensitivityFields` 和 `redactionLevel`，但无对象类别 | 60% |
| 审计不得记录被删除字段原始内容 | ✅ 审计只记录元信息 | 100% |

---

### 2.12 运行层级与Skill分层 (`运行层级与Skill分层.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 第一层：纯本地故事处理 | ✅ `storylock-local-story-processing-skill` | 100% |
| 第二层：本地故事访问 | ✅ `storylock-local-story-access-skill` | 100% |
| 第三层：远程 Skill / 代理授权 | ✅ `storylock-remote-gateway-skill` | 100% |
| 第一层能力：StoryDraftAssistSkill, StoryRefineAssistSkill | ✅ 实现 | 100% |
| 第二层能力：challenge 验证、session 建立、对象读写 | ✅ 实现 | 100% |
| 第三层能力：接收第三方请求、包装、编排、委托 | ⚠️ 有包装和委托，但无编排能力 | 60% |
| 第一层不直接对第三层开放 | ⚠️ 代码层面无显式禁止 | 30% |
| 最小落地目录结构 | ✅ 与代码目录基本一致 | 100% |
| 开发顺序建议 | ❌ 无法验证 | - |

---

### 2.13 系统Skill表与能力边界 (`系统Skill表与能力边界.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 主 Skill：StoryDraftAssistSkill, StoryRefineAssistSkill, StrengthReviewSkill, LocalPasswordFillSkill, ChallengeSigningAuthorizationSkill | ✅ 全部实现 | 100% |
| 内部 Skill：LoginAuthorizationSkill, SigningAuthorizationSkill | ✅ 实现 | 100% |
| 编排示例：VideoPublishAgentDemo | ✅ 实现 | 100% |
| 能力边界判定规则 | ⚠️ 代码中无显式规则校验 | 30% |
| 5个主 Skill 固定 | ✅ 代码中对应类存在 | 100% |
| 网关优先围绕主 Skill 开放接口 | ⚠️ 第三包开放接口与主 Skill 部分对应 | 60% |
| 代理签名是 ChallengeSigningAuthorizationSkill 的受控调用 | ⚠️ 第三包有 `requestChallengeSign`，但无实际签名 | 50% |

---

### 2.14 安全规范 (`安全规范.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 对称加密：AES-256-GCM | ✅ `encryptAes256Gcm()` / `decryptAes256Gcm()` 实现 | 100% |
| 摘要/HMAC：SHA-256 / HMAC-SHA256 | ✅ `hmacSha256Hex()` 实现 | 100% |
| 工作密钥派生：HKDF-SHA256 | ✅ `deriveHkdfSha256()` 实现 | 100% |
| 本地随机数：至少 32 字节 | ✅ `randomBytes(32)` 多处使用 | 100% |
| 密钥层次：masterSalt -> rootKey -> workKey -> objectKey | ✅ `deriveRootKey()`, `deriveWorkKey()`, `deriveObjectKey()` 实现 | 100% |
| HKDF salt/info 显式包含用途 | ✅ `info` 和 `salt` 包含 `storylock:v1:...` 前缀 | 100% |
| 题集主档加密：AES-256-GCM | ❌ 无题集主档 | 0% |
| GCM nonce：每次独立生成 96-bit | ✅ `randomBytes(12)` | 100% |
| GCM nonce 不复用 | ✅ 每次新生成 | 100% |
| GCM nonce 与密文一起存储 | ✅ envelope 包含 nonce | 100% |
| GCM nonce 存储格式：方案A二进制拼接 / 方案B结构化JSON | ✅ 使用方案B（JSON） | 100% |
| masterSalt 长度至少 32 字节 | ✅ `randomBytes(32)` | 100% |
| masterSalt 长期保存于操作系统密钥链 | ⚠️ `MemorySecretStore` 默认使用内存，未强制使用平台密钥链 | 40% |
| masterSalt 不允许明文写入配置文件 | ✅ 无配置文件写入 | 100% |
| identitySalt = HMAC-SHA256(masterSalt, identityId) | ✅ `deriveIdentityAnswerKey()` 使用 HKDF 派生 | 100% |
| 答案摘要：identitySalt + normalizedAnswer + HMAC-SHA256 | ✅ `hmacSha256Hex(identityKey, normalizeAnswerValue(answer))` | 100% |
| 长期密钥存储优先操作系统密钥链 | ⚠️ 默认 `MemorySecretStore`，平台存储需显式配置 | 40% |
| 单故事终身制 | ⚠️ 代码中 `masterSalt` 全局唯一，无多故事支持 | 60% |
| 根级重建边界 | ❌ 未实现 | 0% |
| 题集版本标记 active/deprecated/pending | ❌ 无题集概念 | 0% |
| 对象封装版本标记 | ❌ 无版本标记 | 0% |
| 兼容窗口配置 | ❌ 未实现 | 0% |

**发现**: `masterSalt` 默认使用 `MemorySecretStore`（内存存储），仅在持久化模式时要求 `secretStore` 或 `usePlatformSecretStore=true`。但 `createAccessHost()` 中 `persistent=true` 时检查条件正确，开发模式 (`:memory:`) 允许无密钥链。

---

### 2.15 平台密钥存储适配指南 (`平台密钥存储适配指南.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 保护对象：masterSalt, 签名根密钥, 恢复材料包装密钥, 凭据别名 | ⚠️ 代码中只保护 masterSalt | 50% |
| Windows：系统保护存储 / 凭据管理 | ✅ `WindowsCredentialSecretStore` 实现 | 100% |
| macOS：Keychain | ❌ 未实现 macOS 适配 | 0% |
| Linux：Secret Service / libsecret | ✅ `LinuxSecretServiceStore` 实现 | 100% |
| 统一适配接口：SecretStore | ✅ `MemorySecretStore`, `WindowsCredentialSecretStore`, `LinuxSecretServiceStore` 统一接口 | 100% |
| 命名建议：storylock/masterSalt, storylock/signingRoot/<identityId>, storylock/recoveryKey/<identityId> | ⚠️ 代码中只使用 `storylock/masterSalt` | 40% |
| 不应放入平台存储的对象：requestId, nonce, session, 题集密文主体 | ✅ 这些对象入 SQLite | 100% |
| 降级原则：默认返回明确错误 | ⚠️ `allowMemoryFallback` 可降级为内存 | 50% |
| 开发模式安全边界 | ⚠️ 有 `developmentMode` 标记，但无显式告警输出 | 40% |

---

### 2.16 快速决策指南 / 术语表 / 设计文档优化建议

这些文档属于辅助性文档，无直接代码映射要求。代码中已体现部分术语（Skill, Host, Gateway, Challenge, Session 等）。

---

## 三、不一致项汇总（按严重程度）

### 🔴 高严重程度（安全或架构风险）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 1 | **24 题题集未实现**：文档要求渐进式挑战（6/12/22-of-24），代码只支持单答案验证 | 无法按文档实现 L2-L5 访问强度分级 | 实现题集主档和 24 题选择逻辑 |
| 2 | **EIP-712 类型字段与文档不一致**：文档要求 `StoryLockSignatureRequest` 含 action/resource/scope 等字段，代码使用 `ChallengeSignRequest` 含 identityId/keyId/algorithm 等字段 | 第三方无法按文档构造 EIP-712 请求 | 统一 EIP-712 类型定义，或更新文档 |
| 3 | **契约A/B 未实现**：第一包与第二包之间无接口 | 故事处理链路断裂 | 实现 `StoryAccessSkill` 作为第二包正式能力，连接第一包 |
| 4 | **locked 状态无自动解锁**：`locked` 状态不会自动回到 `idle` | 锁定窗口结束后状态仍显示 locked，影响用户体验 | 在 `getFailureWindow()` 或 `createChallenge()` 中添加状态自动迁移 |
| 5 | **答案以字符串处理，无 Buffer 清零**：文档要求 Buffer 安全清除，代码使用不可变字符串 | 答案可能留在内存中无法清除 | 将答案处理改为 Buffer 流程，校验后显式清零 |
| 6 | **masterSalt 默认内存存储**：`MemorySecretStore` 默认使用，非持久化场景不强制平台密钥链 | 开发模式下 masterSalt 明文存内存 | 默认启用平台密钥链，内存存储需显式确认 |
| 7 | **macOS 密钥存储未实现**：文档要求 macOS Keychain，代码只有 Windows/Linux | macOS 平台无法安全存储 | 实现 `MacOSKeychainSecretStore` |
| 8 | **无对象分类和访问决策表**：文档要求对象分类（story_public/private 等）和 L1-L5 决策，代码无此概念 | 无法按策略自动选择访问强度 | 实现对象分类字段和策略决策引擎 |

### 🟡 中严重程度（功能缺失或偏差）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 9 | **错误码映射错位**：`CHALLENGE_LOCKED` 文档为 SLG-004，代码中 `SLG-003` 同时用于 challenge_failed 和 challenge_locked；`REQUEST_EXPIRED` 文档为 SLG-011，代码中为 SLG-002 | 错误码语义混乱，调用方处理困难 | 修正错误码映射，与文档对齐 |
| 10 | **session 类型只使用 one_shot**：文档定义 4 种类型，代码只使用 one_shot | 无法支持短时连续读取或批量处理 | 扩展 `issueSession()` 支持多种类型 |
| 11 | **无后台清理任务**：nonce/requestId/session 过期记录无自动清理 | 数据库无限增长 | 实现定时清理任务 |
| 12 | **预算扣减非单语句原子**：先 SELECT 再 UPDATE，存在竞态 | 高并发下可能超预算 | 合并为单条 `UPDATE ... WHERE read_budget > 0` |
| 13 | **第三包无 requestStrengthReview**：文档要求暴露，第三包未实现 | 无法远程查询题集强度 | 添加 `requestStrengthReview()` 方法 |
| 14 | **无性能基准测试**：文档建议 <50ms/<100ms 等目标，代码无测试 | 无法验证性能达标 | 添加性能基准测试 |
| 15 | **无 HTTP 状态码映射**：文档建议 HTTP 状态码映射，代码无 HTTP 层 | 若后续接入 HTTP 需重新设计 | 在网关层实现 HTTP 映射 |
| 16 | **签名审计字段不完整**：文档要求记录 scope/resource/signatureHash，代码只记录 event_type/identity_id/request_id | 审计信息不足 | 扩展 audit_log 表字段 |
| 17 | **无题集版本标记**：文档要求 active/deprecated/pending 标记 | 无法平滑升级题集 | 添加题集版本字段 |
| 18 | **无规范化版本兼容策略**：文档要求双版本校验窗口 | 升级规范化规则后旧答案失效 | 实现 `normalizationVersion` 字段和兼容校验 |
| 19 | **无设备重启恢复策略**：文档要求重启后恢复 locked/session 状态 | 重启后状态丢失 | 实现启动时状态恢复逻辑 |
| 20 | **第三包无 Pharos 集成**：文档建议第三包基于 Pharos Skill Engine | 未利用 Pharos 能力 | 评估 Pharos 集成可行性 |

### 🟢 低严重程度（优化建议）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 21 | **局部遮盖未实现**：文档要求电话/邮箱局部遮盖（如 138****0000），代码只实现全量 redaction | 用户体验差 | 实现局部遮盖函数 |
| 22 | **nonce 无 uint256 限制**：文档建议 nonce 为 uint256，代码接受任意字符串 | 与 EIP-712 类型定义不一致 | 添加 nonce 格式校验 |
| 23 | **开发模式无显式告警**：文档要求启动时输出清晰告警 | 开发者可能忽略安全风险 | 在 `MemorySecretStore` 初始化时输出告警 |
| 24 | **无对象封装版本标记**：文档要求对象密文有版本标记 | 无法追踪加密上下文版本 | 添加 `encryptionVersion` 字段 |
| 25 | **代码中 session 字段名与文档不一致**：文档用 `maxReads`，代码用 `readBudget`；文档用 `issuedAt`，代码用 `issuedAt` | 字段名不一致，但语义相同 | 统一字段命名或更新文档 |
| 26 | **第二包无显式禁止第三包调用第一包的校验**：文档要求禁止越权 | 依赖开发者自觉遵守 | 添加运行时校验或文档明确 |
| 27 | **无 GCM 方案A（二进制拼接）**：文档建议方案A优先，代码只用方案B（JSON） | 存储稍大 | 支持方案A作为可选项 |
| 28 | **第三包 `requestPasswordFill` 存在但文档未列入当前阶段允许暴露列表**：文档允许列表无此项，但代码实现 | 文档与代码不一致 | 更新文档或移除接口 |

---

## 四、代码实现超出文档的部分（正向偏差）

| # | 代码实现 | 文档覆盖 | 评价 |
|---|----------|----------|------|
| 1 | **敏感内容分析**：`analyzeSensitiveContent()` 自动检测高敏字段和值模式 | 文档只列清单，无自动检测 | ⭐ 代码优于文档 |
| 2 | **Schema 文件**：各包包含 JSON Schema 定义输入/输出 | 文档无 Schema 要求 | ⭐ 增强接口契约 |
| 3 | **自测试脚本**：`selftest.mjs` 覆盖幂等、重放、锁定、脱敏等场景 | 文档无测试要求 | ⭐ 质量保证 |
| 4 | **平台密钥存储自动检测**：`createPlatformSecretStore()` 自动检测 Windows/Linux | 文档只列建议 | ⭐ 实用增强 |
| 5 | **答案规范化**：`normalizeAnswerValue()` 实现 NFKC + 空格归一 + 大小写统一 | 文档有要求，代码实现完整 | ⭐ 符合规范 |
| 6 | **WASM 加密模块**：`storylock-skill-engine/dist/wasm/` 包含 Rust/WASM 实现 | 文档未提及 | ⭐ 性能增强 |
| 7 | **Skill 边界文档**：各包 `references/boundary.md` 说明边界 | 文档无此要求 | ⭐ 自说明增强 |
| 8 | **统一错误构建**：`buildErrorPayload()` 标准化错误结构 | 文档有要求，代码实现完整 | ⭐ 符合规范 |
| 9 | **稳定字符串序列化**：`stableStringify()` 用于请求哈希 | 文档无此要求 | ⭐ 防哈希冲突 |
| 10 | **多身份支持**：`identityId` 贯穿所有操作 | 文档有要求，代码实现完整 | ⭐ 符合规范 |

---

## 五、建议修复清单（优先级排序）

### P0（阻塞性，必须修复）

1. [ ] **实现 24 题题集**：添加 `question_set` 表，实现 6/12/22-of-24 渐进式挑战
2. [ ] **统一 EIP-712 类型定义**：将 `ChallengeSignRequest` 改为 `StoryLockSignatureRequest`，字段与文档对齐
3. [ ] **修复错误码映射**：`CHALLENGE_LOCKED` -> `SLG-004`，`REQUEST_EXPIRED` -> `SLG-011`
4. [ ] **实现 locked 自动解锁**：在 `getFailureWindow()` 或 `createChallenge()` 中检查 `lockUntil` 并自动清除
5. [ ] **将答案处理改为 Buffer**：`normalizeAnswerValue()` 返回 Buffer，校验后 `zeroizeBytes()`

### P1（重要，建议尽快修复）

6. [ ] **实现契约A/B**：第二包读取后调用第一包处理，处理结果通过第二包写回
7. [ ] **默认启用平台密钥链**：`createAccessHost()` 默认 `usePlatformSecretStore=true`
8. [ ] **实现 macOS Keychain**：添加 `MacOSKeychainSecretStore`
9. [ ] **实现对象分类和策略决策**：添加 `object_category` 字段和 `AccessPolicyEngine`
10. [ ] **合并预算扣减为单语句**：`UPDATE session_store SET read_budget = read_budget - 1 WHERE session_id = ? AND read_budget > 0`
11. [ ] **添加 `requestStrengthReview`**：第三包暴露强度评估接口
12. [ ] **实现后台清理**：定时清理过期 nonce/requestId/session
13. [ ] **添加题集版本标记**：`question_set` 表添加 `status` 字段 (active/deprecated/pending)

### P2（优化，可后续迭代）

14. [ ] **实现局部遮盖**：电话/邮箱局部遮盖函数
15. [ ] **添加 nonce uint256 校验**：限制 nonce 格式
16. [ ] **开发模式告警**：`MemorySecretStore` 初始化时输出 `console.warn()`
17. [ ] **添加性能基准测试**：`benchmark.mjs`
18. [ ] **扩展审计字段**：`audit_log` 添加 `scope`, `resource`, `signature_hash`
19. [ ] **实现设备重启恢复**：启动时扫描并恢复 `locked`/`session_active` 状态
20. [ ] **评估 Pharos 集成**：第三包适配 Pharos Skill Engine 结构

---

## 六、结论

### 总体评价

StoryLock 代码与文档在**架构层面**高度一致（三层划分、目录结构、核心安全机制），但在**细节实现**上存在显著偏差：

1. **架构骨架**：✅ 优秀。三包拆分、状态机、防重放、脱敏、加密等核心机制基本落地。
2. **功能完整性**：⚠️ 中等。24 题题集、契约A/B、策略决策表等关键功能缺失。
3. **安全细节**：⚠️ 中等。答案字符串处理、错误码错位、locked 状态管理有漏洞。
4. **文档一致性**：⚠️ 中等。EIP-712 类型定义、错误码映射、字段命名有偏差。

### 核心风险

1. **24 题题集缺失**：导致无法按文档实现渐进式访问控制，所有访问强度相同。
2. **EIP-712 类型不一致**：第三方按文档构造的请求无法被代码正确解析。
3. **答案字符串处理**：高敏感材料可能长期滞留内存。
4. **错误码错位**：调用方按文档处理错误会失败。

### 建议

1. **短期（1-2 周）**：修复 P0 项（题集、EIP-712、错误码、locked 解锁、Buffer 处理）。
2. **中期（3-4 周）**：修复 P1 项（契约A/B、平台密钥链、macOS、策略决策、后台清理）。
3. **长期**：完善 P2 优化项，实现完整文档定义的能力。

---

*评审完成时间：2026-06-16 23:55 GMT+8*  
*评审人：代码安全审计师*
