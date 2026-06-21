# StoryLock 代码与文档一致性评审报告（v3.0）

**评审日期**: 2026-06-17  
**评审范围**: 
- `E:\2026OPC大赛\skill\src` (代码)  
- `E:\2026OPC大赛\skill\docs\design\cn` (设计文档)  
- `E:\2026OPC大赛\skill\docs\usecase\00-参赛说明文档.md` (参赛说明文档)  
**评审方法**: 逐条对比文档要求与代码实现  
**总体一致性评分**: **88%**

---

## 一、总体一致性评分说明

| 维度 | 一致性 | 变化 | 说明 |
|------|--------|------|------|
| 三层架构划分 | ✅ 100% | → | 目录结构与文档完全一致 |
| 第二包核心安全机制 | ✅ 95% | ↑+10 | 状态机、防重放、脱敏、预算原子扣减均已完善 |
| 错误码体系 | ✅ 95% | ↑+35 | REQUEST_EXPIRED 已修正为 SLG-011，CHALLENGE_LOCKED 保持 SLG-004 |
| 第一包功能 | ✅ 95% | ↑+15 | `LocalStoryAssistAccessSkill` 实现契约A/B，连接第一/二包 |
| 第三包网关 | ✅ 90% | ↑+20 | `requestStrengthReview` 已添加，EIP-712 类型已与文档对齐 |
| 安全规范 | ✅ 85% | ↑+20 | 预算扣减改为单语句原子操作，schema 迁移机制新增 |
| 参赛说明文档一致性 | ✅ 92% | 新增 | 产品说明与代码实现基本一致，扩展方向已对齐 |
| 文档未覆盖的代码 | ⚠️ - | - | 存在文档未定义但代码已实现的部分 |

---

## 二、v1.0 → v2.0 关键修复确认

### ✅ 已修复（P0/P1 项）

| # | v1.0 问题 | v2.0 状态 | 修复证据 |
|---|-----------|-----------|----------|
| 1 | **REQUEST_EXPIRED 错误码错位**（SLG-002 vs SLG-011） | ✅ 已修复 | `errors.js` 中 `REQUEST_EXPIRED` 已映射到 `SLG-011`，`normalizeErrorCode()` 中 `message === 'REQUEST_EXPIRED'` 返回 `'SLG-011'` |
| 2 | **EIP-712 类型与文档不一致** | ✅ 已修复 | `buildEip712Request()` 中 types 已改为 `StoryLockSignatureRequest`，字段为 action/resource/scope/expiry/nonce/requestedBy/delegationContext，与文档完全一致 |
| 3 | **locked 状态无自动解锁** | ✅ 已修复 | `getFailureWindow()` 中新增：窗口过期时自动将 `locked` 状态 UPDATE 为 `idle`；`locked_until <= now` 时同样自动解锁 |
| 4 | **预算扣减非单语句原子** | ✅ 已修复 | `readStoryObjectWithBudget()` 和 `writeStoryObject()` 中 budget 扣减合并为单条 `UPDATE ... WHERE read_budget > 0`，使用 `CASE WHEN` 同步更新 status |
| 5 | **契约A/B 未实现**（第一/二包接口断裂） | ✅ 已修复 | 新增 `LocalStoryAssistAccessSkill` 类，实现读取→处理→写回完整链路 |
| 6 | **第三包无 requestStrengthReview** | ✅ 已修复 | `StoryLockRemoteGateway` 新增 `requestStrengthReview()` 方法 |
| 7 | **nonce 无 uint256 限制** | ✅ 已修复 | 新增 `ensureNonceUint256()` 函数，校验 nonce 为纯数字字符串 |
| 8 | **SQLite schema 无迁移机制** | ✅ 已修复 | 新增 `migrateSqliteSchema()` 函数，自动添加缺失列（request_hash/response_json/redaction_level 等） |
| 9 | **审计日志字段不完整** | ✅ 已修复 | `audit_log` 表新增 `redaction_level`, `has_high_sensitivity_fields`, `error_code`, `meta_json` 字段；`recordAudit()` 方法同步更新 |
| 10 | **selftest 未覆盖 locked 解锁** | ✅ 已修复 | `selftest.mjs` 新增 `identity-lock-auto-unlock` 测试用例，验证 locked 过期后自动解锁 |
| 11 | **selftest 未覆盖错误码修正** | ✅ 已修复 | `selftest.mjs` 新增 `request-expired-error-code` 测试用例，验证 `expiry` 过期返回 `SLG-011` |
| 12 | **selftest 未覆盖契约A/B** | ✅ 已修复 | `selftest.mjs` 新增 `local-story-assist-read-process-write` 测试用例，验证 `LocalStoryAssistAccessSkill` 完整链路 |
| 13 | **selftest 未覆盖 schema 迁移** | ✅ 已修复 | `selftest.mjs` 新增 `sqlite-legacy-schema-migrated` 测试用例，验证旧数据库自动迁移 |

---

## 三、按文档逐条对比（v2.0）

### 3.1 三包拆分策略 (`storylock_three_skill_packages_cn.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 拆分为三个包 | ✅ 三个包目录存在 | 100% |
| 调用链：第三→第二→第一 | ✅ `LocalStoryAssistAccessSkill` 实现第二包调用第一包 | 100% |
| Pharos 放在第三层 | ⚠️ 代码中未体现 Pharos 集成 | 0% |

**状态**: 调用链已完整实现。Pharos 集成属后续扩展，当前阶段不要求。

---

### 3.2 三包接口契约 (`三包接口契约.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 统一请求/响应字段 | ✅ 已实现 | 100% |
| 契约A：读取后交给本地故事处理 | ✅ `LocalStoryAssistAccessSkill` 中 `storyReadSkill.run()` → `draftSkill.run()` / `refineSkill.run()` | 100% |
| 契约B：处理结果写回请求 | ✅ `LocalStoryAssistAccessSkill` 中 `storyWriteSkill.run()` 写回 | 100% |
| 契约C：远程请求读取故事对象 | ✅ `requestStoryRead` | 100% |
| 契约D：远程请求写回故事对象 | ✅ `requestStoryWrite` | 100% |
| 契约E：远程请求本地签名 | ⚠️ 结构正确，但无实际签名执行（仍为委托模式） | 50% |
| 禁止越权规则 | ⚠️ 代码层面无显式运行时校验 | 30% |

**状态**: 契约A/B 已完整实现。禁止越权规则依赖开发者自觉遵守，建议添加运行时校验。

---

### 3.3 Challenge状态机 (`Challenge状态机.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 状态列表（8个） | ✅ 全部覆盖 | 100% |
| 状态转换：locked -> idle | ✅ `getFailureWindow()` 中自动解锁 | 100% |
| 并发控制：challengeId 粒度串行化 | ⚠️ SQLite 事务保护，但无显式 challengeId 级锁 | 50% |
| 状态持久化：运行态 vs 持久态 | ⚠️ 全部持久化到 SQLite，无内存运行态区分 | 60% |
| 设备重启后恢复策略 | ❌ 未实现 | 0% |

**状态**: locked→idle 已修复。设备重启恢复仍为缺失项。

---

### 3.4 Session与防重放策略 (`Session与防重放策略.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Session 绑定字段 | ✅ 完整 | 100% |
| 防重放：requestId 幂等 | ✅ 完整 | 100% |
| 防重放：nonce 去重 | ✅ 完整 | 100% |
| 防重放：expiry 校验 | ✅ 完整 | 100% |
| 时钟漂移容差 +/- 30秒 | ✅ 完整 | 100% |
| 预算扣减原子性 | ✅ 单语句原子 UPDATE | 100% |
| Nonce 存储格式：uint256 | ✅ `ensureNonceUint256()` 校验 | 100% |
| Nonce 清理策略 | ⚠️ 有索引，但无后台清理任务 | 50% |
| RequestId 清理策略 | ⚠️ 同上 | 50% |
| 版本兼容窗口与 Session 关系 | ❌ 未实现 | 0% |

**状态**: 核心防重放机制已完善。后台清理和版本兼容窗口仍为缺失项。

---

### 3.5 EIP-712最小请求定义 (`EIP-712最小请求定义.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Domain：name/version/chainId/verifyingContract | ✅ 完整 | 100% |
| Types：`StoryLockSignatureRequest` | ✅ 已统一 | 100% |
| Value 字段：action/resource/scope/expiry/nonce/requestedBy/delegationContext | ✅ 已统一 | 100% |
| 生产前检查清单 | ❌ 未实现 | 0% |

**状态**: EIP-712 类型定义已完全与文档对齐。生产检查清单为后续扩展。

---

### 3.6 本地Agent网关设计 (`本地Agent网关设计.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 允许暴露：requestStoryRead, requestStoryWrite, requestChallengeSign, requestStrengthReview, requestCapabilityStatus, queryStoryMetadata | ✅ 全部实现（新增 requestStrengthReview） | 100% |
| 禁止暴露：createChallenge, submitChallengeAnswers 等 | ✅ 未暴露 | 100% |
| 错误码：SLG-001 到 SLG-012 | ✅ 已完整定义 | 100% |
| HTTP 状态码映射 | ❌ 未实现 HTTP 层 | 0% |
| 性能基准 | ❌ 无性能测试 | 0% |
| 分层校验责任 | ⚠️ 有分层校验，但无显式清单 | 60% |

**状态**: 网关接口已完整。HTTP 映射和性能测试为后续扩展。

---

### 3.7 代理签名机制协议参考 (`代理签名机制协议参考.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 第三方不持有私钥 | ✅ 第三包无密钥访问 | 100% |
| 提交结构化签名请求 | ✅ `requestChallengeSign` | 100% |
| 本地授权后完成签名 | ⚠️ 第二包无实际签名执行能力 | 30% |
| 参考 EIP-712 用于请求表示 | ✅ 已实现 | 100% |
| 参考 EIP-1271 用于验证兼容性 | ❌ 未实现 | 0% |
| 参考 HDP Protocol 用于审计链 | ❌ 未实现 | 0% |
| 本地密钥存储：操作系统密钥链优先 | ✅ `createPlatformSecretStore()` | 100% |
| 多身份密钥管理：派生式 | ✅ `deriveIdentityAnswerKey()` 等 | 100% |
| 签名审计：requestId, scope, resource, signatureHash, timestamp | ⚠️ `audit_log` 有 requestId/timestamp，但无 scope/resource/signatureHash | 50% |
| 签名算法选择：secp256k1/ed25519 | ✅ `ensureAllowedAlgorithm()` | 100% |

**状态**: 请求表示层已完善。实际签名执行、EIP-1271、HDP 审计链仍为缺失项。

---

### 3.8 挑战答案存储策略 (`挑战答案存储策略.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 原始答案不做长期持久化 | ✅ 只存摘要 | 100% |
| 原始答案只存在于短时内存 | ⚠️ 仍使用字符串，未改为 Buffer | 40% |
| 持久化只保存派生结果 | ✅ 只存 HMAC 摘要 | 100% |
| 答案规范化 | ✅ `normalizeAnswerValue()` 完整 | 100% |
| 规范化版本兼容策略 | ❌ 无 `normalizationVersion` 字段 | 0% |
| 题集主档与 challenge 实例档分开 | ❌ 无题集主档 | 0% |
| 内存清理：Buffer 清零 | ❌ 未实现 | 0% |

**状态**: 答案摘要策略正确。Buffer 安全处理和题集主档仍为缺失项。

---

### 3.9 脱敏规范 (`脱敏规范.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 三个级别：none, partial, full | ✅ 完整 | 100% |
| 高敏字段清单 | ✅ 完整 | 100% |
| 最小脱敏检查清单 | ✅ `collectSensitiveSignals()` | 100% |
| 局部遮盖：电话/邮箱/用户名 | ❌ 未实现局部遮盖 | 0% |
| 摘要替换：私密故事正文 | ⚠️ `contentSummary` 显示 `[redacted:N chars]` | 50% |
| 结构保留、值清空 | ✅ 完整 | 100% |
| 审计要求 | ✅ `auditMeta` 记录 | 100% |

**状态**: 脱敏核心机制完善。局部遮盖（如 138****0000）仍为缺失项。

---

### 3.10 对象访问策略 (`对象访问策略.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 对象分类：story_public, story_private, auth_login, auth_signing, review_data | ⚠️ 代码中 `sensitivity` 字段存在，但无显式分类系统 | 30% |
| 访问强度 L1-L5 | ⚠️ 代码中无 L1-L5 分级实现 | 10% |
| 渐进式挑战策略：6/12/22-of-24 | ❌ 代码中 challenge 只验证单个答案，无 24 题题集 | 0% |
| 会话模型：4 种类型 | ✅ 字段支持，但只使用 one_shot | 40% |
| 访问决策表 | ❌ 无策略决策表 | 0% |

**状态**: 24 题题集和访问强度分级仍为重大缺失项。这是当前最大的文档-代码偏差。

---

### 3.11 安全规范 (`安全规范.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 算法基线：AES-256-GCM, SHA-256, HKDF-SHA256 | ✅ 完整 | 100% |
| 密钥层次：masterSalt -> rootKey -> workKey -> objectKey | ✅ 完整 | 100% |
| GCM nonce 管理 | ✅ 完整 | 100% |
| masterSalt 长度/存储 | ⚠️ 默认 `MemorySecretStore`，未强制平台密钥链 | 40% |
| 单故事终身制 | ⚠️ `masterSalt` 全局唯一，无多故事支持 | 60% |
| 题集版本标记 active/deprecated/pending | ❌ 无题集概念 | 0% |
| 对象封装版本标记 | ❌ 无版本标记 | 0% |
| 兼容窗口配置 | ❌ 未实现 | 0% |

**状态**: 加密实现正确。题集版本和对象封装版本仍为缺失项。

---

### 3.12 平台密钥存储适配指南 (`平台密钥存储适配指南.md`)

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Windows：凭据管理 | ✅ `WindowsCredentialSecretStore` | 100% |
| macOS：Keychain | ❌ 未实现 | 0% |
| Linux：Secret Service | ✅ `LinuxSecretServiceStore` | 100% |
| 统一适配接口 | ✅ 统一接口 | 100% |
| 命名建议 | ⚠️ 只使用 `storylock/masterSalt` | 40% |
| 降级原则：默认返回明确错误 | ⚠️ `allowMemoryFallback` 可降级 | 50% |
| 开发模式安全边界 | ⚠️ 有 `developmentMode` 标记，但无显式告警 | 40% |

**状态**: macOS 适配仍为缺失项。开发模式告警建议添加。

---

## 四、不一致项汇总（v2.0）

### 🔴 高严重程度（剩余 3 项，较 v1.0 减少 5 项）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 1 | **24 题题集未实现**：文档要求渐进式挑战（6/12/22-of-24），代码只支持单答案验证 | 无法按文档实现 L2-L5 访问强度分级 | 实现 `question_set` 表和题集选择逻辑 |
| 2 | **答案仍以字符串处理，无 Buffer 清零**：文档要求 Buffer 安全清除，代码使用不可变字符串 | 答案可能长期滞留内存 | 将答案处理改为 Buffer 流程 |
| 3 | **masterSalt 默认内存存储**：`MemorySecretStore` 默认使用，开发模式无显式告警 | 开发模式下敏感材料明文存内存 | 默认启用平台密钥链，内存存储需显式确认并输出告警 |

### 🟡 中严重程度（剩余 8 项，较 v1.0 减少 4 项）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 4 | **无对象分类和访问决策表**：文档要求对象分类（story_public/private 等）和 L1-L5 决策 | 无法自动选择访问强度 | 添加 `object_category` 字段和 `AccessPolicyEngine` |
| 5 | **无后台清理任务**：nonce/requestId/session 过期记录无自动清理 | 数据库无限增长 | 实现定时清理任务或启动时清理 |
| 6 | **macOS 密钥存储未实现**：文档要求 macOS Keychain，代码只有 Windows/Linux | macOS 平台无法安全存储 | 实现 `MacOSKeychainSecretStore` |
| 7 | **无题集版本标记**：文档要求 active/deprecated/pending 标记 | 无法平滑升级题集 | 添加 `question_set` 表和版本字段 |
| 8 | **无规范化版本兼容策略**：文档要求双版本校验窗口 | 升级规范化规则后旧答案失效 | 实现 `normalizationVersion` 字段和兼容校验 |
| 9 | **无设备重启恢复策略**：文档要求重启后恢复 locked/session 状态 | 重启后状态丢失 | 实现启动时状态扫描和恢复 |
| 10 | **签名审计字段不完整**：文档要求记录 scope/resource/signatureHash | 审计信息不足 | 扩展 `audit_log` 表字段 |
| 11 | **无实际签名执行能力**：第二包有签名授权结构但无实际签名 | 代理签名无法完成 | 实现 `signer` 注入和实际签名执行 |

### 🟢 低严重程度（剩余 5 项，较 v1.0 减少 2 项）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 12 | **局部遮盖未实现**：文档要求电话/邮箱局部遮盖（如 138****0000） | 用户体验差 | 实现局部遮盖函数 |
| 13 | **开发模式无显式告警**：文档要求启动时输出清晰告警 | 开发者可能忽略安全风险 | 在 `MemorySecretStore` 初始化时输出 `console.warn()` |
| 14 | **无 HTTP 状态码映射**：文档建议 HTTP 状态码映射，代码无 HTTP 层 | 若后续接入 HTTP 需重新设计 | 在网关层实现 HTTP 映射 |
| 15 | **无性能基准测试**：文档建议 <50ms/<100ms 等目标 | 无法验证性能达标 | 添加性能基准测试 |
| 16 | **session 类型只使用 one_shot**：文档定义 4 种类型，代码只使用 one_shot | 无法支持短时连续读取或批量处理 | 扩展 `issueSession()` 支持多种类型 |

---

## 五、代码实现超出文档的部分（正向偏差）

| # | 代码实现 | 文档覆盖 | 评价 |
|---|----------|----------|------|
| 1 | **SQLite Schema 迁移机制**：`migrateSqliteSchema()` 自动添加缺失列 | 文档未提及 | ⭐ 优秀工程实践 |
| 2 | **Selftest 覆盖全面**：新增 locked 解锁、错误码、契约A/B、schema 迁移等测试 | 文档无测试要求 | ⭐ 质量保证 |
| 3 | **审计日志增强**：新增 redaction_level, has_high_sensitivity_fields, error_code, meta_json | 文档有要求，代码超额实现 | ⭐ 审计完善 |
| 4 | **预算扣减单语句原子**：使用 `UPDATE ... WHERE read_budget > 0` + `CASE WHEN` | 文档有要求，代码正确实现 | ⭐ 并发安全 |
| 5 | **EIP-712 nonce uint256 校验**：`ensureNonceUint256()` 强制校验 | 文档有要求，代码正确实现 | ⭐ 类型安全 |
| 6 | **LocalStoryAssistAccessSkill**：完整实现读取→处理→写回链路 | 文档有要求，代码正确实现 | ⭐ 契约完整 |
| 7 | **敏感内容自动检测**：`analyzeSensitiveContent()` 自动检测高敏字段和值模式 | 文档只列清单，无自动检测 | ⭐ 代码优于文档 |
| 8 | **JSON Schema 接口契约**：各包包含 JSON Schema 定义 | 文档无 Schema 要求 | ⭐ 增强接口契约 |
| 9 | **WASM 加密模块**：`storylock-skill-engine/dist/wasm/` 包含 Rust/WASM 实现 | 文档未提及 | ⭐ 性能增强 |
| 10 | **稳定字符串序列化**：`stableStringify()` 用于请求哈希 | 文档无此要求 | ⭐ 防哈希冲突 |

---

## 六、建议修复清单（v2.0 优先级）

### P0（阻塞性，必须修复）

1. [ ] **实现 24 题题集**：添加 `question_set` 表，实现 6/12/22-of-24 渐进式挑战（当前最大缺失）
2. [ ] **将答案处理改为 Buffer**：`normalizeAnswerValue()` 返回 Buffer，校验后 `zeroizeBytes()`
3. [ ] **默认启用平台密钥链**：`createAccessHost()` 默认 `usePlatformSecretStore=true`，内存存储需显式确认并输出告警

### P1（重要，建议尽快修复）

4. [ ] **实现对象分类和策略决策**：添加 `object_category` 字段和 `AccessPolicyEngine`
5. [ ] **实现 macOS Keychain**：添加 `MacOSKeychainSecretStore`
6. [ ] **实现后台清理**：定时清理过期 nonce/requestId/session
7. [ ] **实现题集版本标记**：`question_set` 表添加 `status` 字段 (active/deprecated/pending)
8. [ ] **实现规范化版本兼容**：`normalizationVersion` 字段和双版本校验
9. [ ] **实现设备重启恢复**：启动时扫描并恢复 `locked`/`session_active` 状态
10. [ ] **实现实际签名执行**：`ChallengeSigningAuthorizationSkill` 注入 `signer` 完成实际签名

### P2（优化，可后续迭代）

11. [ ] **实现局部遮盖**：电话/邮箱局部遮盖函数
12. [ ] **开发模式告警**：`MemorySecretStore` 初始化时输出 `console.warn()`
13. [ ] **扩展 session 类型**：支持 `short_session`, `batch_session`, `privileged_session`
14. [ ] **添加性能基准测试**：`benchmark.mjs`
15. [ ] **实现 HTTP 状态码映射**：网关层 HTTP 映射
16. [ ] **扩展审计字段**：`audit_log` 添加 `scope`, `resource`, `signature_hash`

---

## 七、结论

### v1.0 → v2.0 改进总结

**已修复的重大问题**：
1. ✅ 错误码映射修正（REQUEST_EXPIRED → SLG-011）
2. ✅ EIP-712 类型定义与文档对齐
3. ✅ locked 状态自动解锁机制
4. ✅ 预算扣减单语句原子操作
5. ✅ 契约A/B 实现（LocalStoryAssistAccessSkill）
6. ✅ requestStrengthReview 接口添加
7. ✅ nonce uint256 校验
8. ✅ SQLite schema 迁移机制
9. ✅ 审计日志字段扩展
10. ✅ selftest 覆盖全面增强

**剩余核心风险**：
1. 🔴 **24 题题集缺失**：这是当前最大的文档-代码偏差，导致无法按文档实现渐进式访问控制
2. 🔴 **答案字符串处理**：高敏感材料内存安全未达标
3. 🔴 **masterSalt 默认内存存储**：开发模式安全风险

### 总体评价

StoryLock v2.0 代码在**安全机制细节**上已大幅完善（错误码、状态机、防重放、预算原子性、契约链路），但在**核心功能完整性**上仍有关键缺失（24 题题集、对象分类、访问强度分级）。

**建议**：
- **短期（1-2 周）**：修复 P0 项（题集、Buffer、密钥链）。
- **中期（3-4 周）**：修复 P1 项（策略决策、macOS、清理、版本兼容）。
- **长期**：完善 P2 优化项，实现完整文档定义的能力。

---

*评审完成时间：2026-06-17 01:10 GMT+8*  
*评审人：代码安全审计师*  
*版本：v2.0（基于完善后的代码重新评审）*
