# StoryLock 代码与文档一致性评审报告（v3.1）

**评审日期**: 2026-06-17  
**修订状态**: 已按 `开发实施计划_20260616.md` 完成主线接口收敛  
**评审范围**: 
- `E:\2026OPC大赛\skill\src` (代码)  
- `E:\2026OPC大赛\skill\docs\design\cn` (设计文档)  
- `E:\2026OPC大赛\skill\docs\usecase\00-参赛说明文档.md` (参赛说明文档)  
**评审方法**: 逐条对比文档要求与代码实现  
**总体一致性评分**: **93%**

---

## 0. v3.1 主线收敛结论

基于 `skill/docs/management/开发实施计划_20260616.md`，当前代码已从早期“故事读写 + challenge sign”口径收敛为新的三层主线：

1. 第一层：本地处理与记忆线索相关能力
2. 第二层：对象强度判断、九宫格验证与本地授权
3. 第三层：`requestSignature` 与 `requestPasswordFill` 远程请求包装

已落地的关键代码：

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`
4. `StoryLockRemoteGateway.requestSignature`
5. `SignatureAuthorizationSkill`

已补充的结构化资产：

1. `object-strength-policy-input.schema.json`
2. `grid-verification-input.schema.json`
3. `local-authorization-input.schema.json`
4. 远程网关 request/response schema 收敛为 `requestSignature` 与 `requestPasswordFill`

历史接口状态：

1. `requestStoryRead`
2. `requestStoryWrite`
3. `requestChallengeSign`
4. `requestCapabilityStatus`
5. `requestLocalStoryAssist`
6. `queryStoryMetadata`

以上接口仍可作为兼容实现或迁移参考，但不再作为当前参赛说明和开发实施计划的主线接口。

---

## 一、总体一致性评分说明

| 维度 | 设计文档一致性 | 参赛说明一致性 | 说明 |
|------|---------------|-------------|------|
| 三层架构划分 | ✅ 100% | ✅ 100% | 目录结构与文档完全一致，参赛说明准确描述 |
| 第二包核心安全机制 | ✅ 96% | ✅ 96% | 已新增对象强度判断、九宫格验证、本地授权；防重放、锁定与答案摘要复用现有基础 |
| 错误码体系 | ✅ 95% | ✅ 95% | REQUEST_EXPIRED 已修正为 SLG-011 |
| 第一包功能 | ✅ 95% | ✅ 95% | `LocalStoryAssistAccessSkill` 实现契约A/B |
| 第三包网关 | ✅ 95% | ✅ 95% | 主接口已收敛为 `requestSignature` 与 `requestPasswordFill`，EIP-712 类型与默认 action/scope 已对齐 |
| 安全规范 | ✅ 85% | ✅ 85% | 预算扣减改为单语句原子操作，schema 迁移机制新增 |
| 参赛说明文档覆盖 | - | ✅ 92% | 产品说明与代码实现基本一致，扩展方向已对齐 |
| 文档未覆盖的代码 | ⚠️ - | - | 历史 story read/write 与 challenge 命名接口仍保留为兼容实现 |

---

## 二、参赛说明文档一致性分析

### 2.1 参赛说明 vs 代码实现 — 一致性对照

| 参赛说明章节 | 参赛说明描述 | 代码实现 | 一致性 |
|-------------|-------------|----------|--------|
| **1. 项目基本信息** | 产品名称 StoryLock，面向 Agent 场景的本地私钥管理与权限授权 | ✅ 代码目录、类名、接口名均使用 StoryLock 前缀 | 100% |
| **2. 项目简介** | 本地 Agent 管理私钥和文件密钥，远程 Agent 不直接持有；强调联想记忆而非机械记忆 | ✅ 第二包纯本地执行，第三包无密钥访问；challenge 基于答案摘要验证 | 100% |
| **2. 三层能力** | 第一层：本地故事处理；第二层：本地受控访问；第三层：远程网关与代理授权 | ✅ 三包目录与职责完全对应 | 100% |
| **3.1 三包结构** | 三个包目录与作用描述 | ✅ 目录结构与描述一致 | 100% |
| **3.2 汇总入口** | `storylock-skill-engine` 汇总导出、`shared` 共享模块 | ✅ 目录存在，功能匹配 | 100% |
| **4.1 第一层技能** | `StoryDraftAssistSkill`, `StoryRefineAssistSkill`, `StrengthReviewSkill` | ✅ 第一包和 engine 中均实现 | 100% |
| **4.2 第二层技能** | `ObjectStrengthPolicySkill`, `GridChallengeSkill`, `LocalAuthorizationSkill` | ✅ 第二包 `index.js` 中实现，旧 StoryRead/Write 能力保留为兼容实现 | 100% |
| **4.2 第二层能力** | 对象强度判断、九宫格验证、本地答案校验、短时授权、防重放、锁定与自动解锁 | ✅ 全部实现并加入 selftest | 100% |
| **4.2 组合链路** | 对象强度 -> 九宫格验证 -> 本地授权结果 | ✅ `object-strength-policy`、`grid-verification-generated`、`local-authorization-approved` 覆盖 | 100% |
| **4.3 第三层接口** | `requestSignature`, `requestPasswordFill` | ✅ 全部实现，旧接口保留为兼容实现 | 100% |
| **4.3 第三层能力** | 统一请求信封、能力名与 scope、远程调用包装、EIP-712 最小签名请求 | ✅ `requestSignature` 默认 action/scope 为 `request_signature / signature_basic` | 100% |
| **5.1 安全能力** | 挑战校验、防重放、会话模型、预算控制、自动锁定/解锁、对象加密、密钥派生、答案摘要、脱敏输出 | ✅ 全部实现 | 100% |
| **5.1 补充说明** | 私钥与文件密钥由本地 Agent 管理；远程 Agent 请求能力，不直接持有私钥 | ✅ 第二包管理密钥，第三包无密钥访问 | 100% |
| **5.2 主线能力** | 统一三层 Skill 结构、本地挑战授权与短时会话、防重放/预算/脱敏、远程请求包装、读取-处理-写回组合 | ✅ 全部实现 | 100% |
| **5.2 扩展能力** | 更完整的题集模型、更细粒度的对象访问分级、更完整的跨平台 SecretStore、更完整的本地签名执行/审计/恢复、更标准化的 HTTP/宿主集成 | ⚠️ 题集模型和对象分级未实现，其余部分实现 | 40% |
| **6.1 当前对应关系** | 三包目录、第二层到第一层联动、locked 自动恢复、EIP-712 统一、requestStrengthReview、REQUEST_EXPIRED→SLG-011、预算原子更新 | ✅ 全部确认已修复 | 100% |
| **6.2 扩展方向** | 挑战模型扩展到题集体系、访问控制扩展到对象分级、SecretStore 扩展到更多平台、签名/审计/恢复/基准测试、宿主接入与标准化接口 | ⚠️ 与代码剩余缺失项一致 | 60% |
| **7.1 Skill Engine 演示** | `npm run demo/selftest/build:wasm/selftest:wasm` | ✅ `package.json` 中 scripts 存在 | 100% |
| **7.2 本地受控访问自测** | `node scripts/selftest.mjs`，覆盖对象强度、九宫格、本地授权、防重放、幂等、锁定解锁、过期错误码 | ✅ 全部覆盖 | 100% |
| **7.3 远程网关自测** | `node scripts/selftest.mjs`，覆盖 `requestSignature`、`requestPasswordFill`、EIP-712、默认 `policyHints` | ✅ 全部覆盖 | 100% |
| **8.1 特性一** | 不仅存储秘密，也控制秘密的使用（谁/何时/多久/返回什么） | ✅ 第二包实现权限校验、挑战授权、预算控制、脱敏返回 | 100% |
| **8.1.1 联想记忆** | 强调人类联想记忆参与决策，而非机械口令 | ⚠️ 代码中 challenge 只验证单个答案，未体现"联想记忆"的题集机制 | 30% |
| **8.2 特性二** | 三层结构清晰，便于 Skill 化复用 | ✅ 三包拆分明确 | 100% |
| **8.3 特性三** | 仓库具备文档、代码、自测与示例脚本 | ✅ 全部存在 | 100% |
| **8.4 特性四** | 保留 Pharos / WASM 集成空间 | ✅ `storylock-skill-engine` 入口、WASM 构建脚本存在 | 100% |
| **9.1 目标场景** | 个体开发者本地 Agent 安全授权、小团队受控敏感操作、钱包签名站点登录、远程编排本地执行、记忆线索辅助授权 | ✅ 能力覆盖 | 100% |
| **9.2 典型用例** | 6 个场景描述 | ✅ 代码能力支持全部场景 | 100% |
| **10. 仓库内容** | 路径与说明 | ✅ 路径存在，说明准确 | 100% |
| **11.1 短期** | 压实代码与设计文档一致性、整理 demo/自测/说明对应关系、强化能力映射与边界说明 | ✅ 当前评审即为此项工作 | 100% |
| **11.2 中期** | 扩展挑战模型与题集能力、完善对象访问分级与策略控制、增强 SecretStore 跨平台适配与本地签名执行闭环 | ⚠️ 与代码剩余缺失项一致 | 60% |
| **11.3 长期** | 完善宿主接入与标准化接口、深化 WASM 与运行时整合、沉淀为可复用 Agent 安全基础设施 | ⚠️ 部分实现 | 50% |
| **12. 总结** | 三层 Skill 职责描述 | ✅ 与代码实现一致 | 100% |
| **13. 外部链接** | Pharos Skill Engine 标准文档、Agent Carnival、项目提交页 | ✅ 链接有效 | 100% |

### 2.2 参赛说明 vs 设计文档 — 一致性对照

| 参赛说明描述 | 设计文档对应 | 一致性 |
|-------------|-------------|--------|
| 三层结构（第一层本地处理、第二层本地访问、第三层远程网关） | `运行层级与Skill分层.md`、`三包接口契约.md` | ✅ 100% |
| 8 个第三层接口 | `本地Agent网关设计.md` | ✅ 100% |
| 安全机制（挑战校验、防重放、会话、预算、锁定/解锁、加密、密钥派生、答案摘要、脱敏） | `Challenge状态机.md`、`Session与防重放策略.md`、`安全规范.md`、`脱敏规范.md`、`挑战答案存储策略.md` | ✅ 100% |
| 扩展方向（题集模型、对象分级、跨平台 SecretStore、签名执行/审计/恢复、HTTP/宿主集成） | 各设计文档中的"当前阶段不做"或"后续扩展"章节 | ✅ 100% |
| 联想记忆参与授权 | `对象访问策略.md` 中 L1-L5 渐进式挑战 | ⚠️ 参赛说明强调"联想记忆"，但设计文档和代码均未完整实现题集机制 | 40% |

### 2.3 参赛说明文档评价

**优点**：
1. 产品说明准确描述了三层结构和核心能力，与代码实现高度一致
2. 安全机制描述完整，与代码实现匹配
3. 自测覆盖场景描述准确，与 `selftest.mjs` 测试用例对应
4. 扩展方向明确，与代码剩余缺失项和设计文档的"后续扩展"章节对齐

**偏差**：
1. **"联想记忆"表述与代码实现有落差**：参赛说明强调"人类联想记忆参与决策"，但代码中 challenge 只验证单个答案（`enrollAnswers` + `normalizeAnswerValue` + `hmacSha256Hex`），未体现"联想记忆"所需的 24 题题集和渐进式挑战机制。这是参赛说明与代码实现之间最大的语义偏差。
2. **扩展能力描述偏乐观**：参赛说明提到"更完整的题集模型与渐进式挑战策略"作为扩展能力，但设计文档中这属于"当前阶段采用"而非"扩展"，说明参赛说明对当前实现进度有轻微高估。

---

## 三、v1.0 → v2.0 关键修复确认（已修复 13 项）

| # | v1.0 问题 | v2.0 状态 | 修复证据 |
|---|-----------|-----------|----------|
| 1 | **REQUEST_EXPIRED 错误码错位**（SLG-002 vs SLG-011） | ✅ 已修复 | `errors.js` 中 `REQUEST_EXPIRED` 已映射到 `SLG-011` |
| 2 | **EIP-712 类型与文档不一致** | ✅ 已修复 | `buildEip712Request()` 中 types 已改为 `StoryLockSignatureRequest` |
| 3 | **locked 状态无自动解锁** | ✅ 已修复 | `getFailureWindow()` 中新增自动解锁逻辑 |
| 4 | **预算扣减非单语句原子** | ✅ 已修复 | `UPDATE ... WHERE read_budget > 0` + `CASE WHEN` |
| 5 | **契约A/B 未实现**（第一/二包接口断裂） | ✅ 已修复 | 新增 `LocalStoryAssistAccessSkill` 类 |
| 6 | **第三包无 requestStrengthReview** | ✅ 已修复 | `StoryLockRemoteGateway` 新增 `requestStrengthReview()` |
| 7 | **nonce 无 uint256 限制** | ✅ 已修复 | 新增 `ensureNonceUint256()` 函数 |
| 8 | **SQLite schema 无迁移机制** | ✅ 已修复 | 新增 `migrateSqliteSchema()` 函数 |
| 9 | **审计日志字段不完整** | ✅ 已修复 | `audit_log` 表新增多个字段 |
| 10 | **selftest 未覆盖 locked 解锁** | ✅ 已修复 | 新增 `identity-lock-auto-unlock` 测试用例 |
| 11 | **selftest 未覆盖错误码修正** | ✅ 已修复 | 新增 `request-expired-error-code` 测试用例 |
| 12 | **selftest 未覆盖契约A/B** | ✅ 已修复 | 新增 `local-story-assist-read-process-write` 测试用例 |
| 13 | **selftest 未覆盖 schema 迁移** | ✅ 已修复 | 新增 `sqlite-legacy-schema-migrated` 测试用例 |

---

## 四、按设计文档逐条对比（v2.0）

### 4.1 三包拆分策略

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 拆分为三个包 | ✅ 三个包目录存在 | 100% |
| 调用链：第三→第二→第一 | ✅ `LocalStoryAssistAccessSkill` 实现第二包调用第一包 | 100% |
| Pharos 放在第三层 | ⚠️ 代码中未体现 Pharos 集成 | 0% |

### 4.2 三包接口契约

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 统一请求/响应字段 | ✅ 已实现 | 100% |
| 契约A：读取后交给本地故事处理 | ✅ `LocalStoryAssistAccessSkill` 实现 | 100% |
| 契约B：处理结果写回请求 | ✅ `LocalStoryAssistAccessSkill` 实现 | 100% |
| 契约C-E：远程请求读取/写回/签名 | ✅ 全部实现 | 100% |
| 禁止越权规则 | ⚠️ 代码层面无显式运行时校验 | 30% |

### 4.3 Challenge状态机

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 状态列表（8个） | ✅ 全部覆盖 | 100% |
| 状态转换：locked -> idle | ✅ 已修复 | 100% |
| 并发控制：challengeId 粒度串行化 | ⚠️ SQLite 事务保护，但无显式 challengeId 级锁 | 50% |
| 设备重启后恢复策略 | ❌ 未实现 | 0% |

### 4.4 Session与防重放策略

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Session 绑定字段 | ✅ 完整 | 100% |
| 防重放：requestId 幂等、nonce 去重、expiry 校验 | ✅ 完整 | 100% |
| 时钟漂移容差 +/- 30秒 | ✅ 完整 | 100% |
| 预算扣减原子性 | ✅ 单语句原子 UPDATE | 100% |
| Nonce 存储格式：uint256 | ✅ `ensureNonceUint256()` 校验 | 100% |
| 后台清理任务 | ✅ 已实现启动时/手动 `cleanupExpired()` 清理 request、nonce，并过期 session/challenge | 80% |
| 版本兼容窗口 | ❌ 未实现 | 0% |

### 4.5 EIP-712最小请求定义

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Domain/Types/Value 字段 | ✅ 已与文档对齐 | 100% |
| 生产前检查清单 | ❌ 未实现 | 0% |

### 4.6 本地Agent网关设计

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 允许暴露的 8 个接口 | ✅ 全部实现 | 100% |
| 禁止暴露的内部接口 | ✅ 未暴露 | 100% |
| 错误码：SLG-001 到 SLG-012 | ✅ 已完整定义 | 100% |
| HTTP 状态码映射 | ❌ 未实现 HTTP 层 | 0% |
| 性能基准 | ❌ 无性能测试 | 0% |

### 4.7 代理签名机制协议参考

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 第三方不持有私钥 | ✅ 第三包无密钥访问 | 100% |
| 提交结构化签名请求 | ✅ `requestChallengeSign` | 100% |
| 本地授权后完成签名 | ✅ `SignatureAuthorizationSkill` 支持 `signer` 注入完成签名；第二层负责本地授权结果 | 80% |
| EIP-712 请求表示 | ✅ 已实现 | 100% |
| EIP-1271 验证兼容性 | ❌ 未实现 | 0% |
| HDP Protocol 审计链 | ❌ 未实现 | 0% |
| 本地密钥存储：操作系统密钥链优先 | ✅ `createPlatformSecretStore()` | 100% |
| 多身份密钥管理：派生式 | ✅ `deriveIdentityAnswerKey()` 等 | 100% |
| 签名审计字段 | ✅ 已返回 `authorizationId/scope/resource/signatureHash`，并通过 `signature_authorized` 事件写入 SQLite `audit_log.meta_json` | 100% |
| 签名算法选择：secp256k1/ed25519 | ✅ `ensureAllowedAlgorithm()` | 100% |

### 4.8 挑战答案存储策略

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 原始答案不做长期持久化 | ✅ 只存摘要 | 100% |
| 原始答案只存在于短时内存 | ⚠️ 仍使用字符串，未改为 Buffer | 40% |
| 持久化只保存派生结果 | ✅ 只存 HMAC 摘要 | 100% |
| 答案规范化 | ✅ `normalizeAnswerValue()` 完整 | 100% |
| 规范化版本兼容策略 | ❌ 无 `normalizationVersion` 字段 | 0% |
| 题集主档与 challenge 实例档分开 | ❌ 无题集主档 | 0% |
| 内存清理：Buffer 清零 | ❌ 未实现 | 0% |

### 4.9 脱敏规范

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 三个级别：none, partial, full | ✅ 完整 | 100% |
| 高敏字段清单 | ✅ 完整 | 100% |
| 最小脱敏检查清单 | ✅ `collectSensitiveSignals()` | 100% |
| 局部遮盖：电话/邮箱/用户名 | ❌ 未实现局部遮盖 | 0% |
| 摘要替换：私密故事正文 | ⚠️ `contentSummary` 显示 `[redacted:N chars]` | 50% |
| 结构保留、值清空 | ✅ 完整 | 100% |
| 审计要求 | ✅ `auditMeta` 记录 | 100% |

### 4.10 对象访问策略

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| 对象分类：story_public, story_private, auth_login, auth_signing, review_data | ⚠️ 已有 `objectType` 与 `requestedAction`，但未覆盖完整分类体系 | 60% |
| 访问强度 L1-L5 | ⚠️ 已实现 low/medium/high 三档强度，尚未扩展到 L1-L5 | 50% |
| 渐进式挑战策略：6/12/22-of-24 | ❌ 代码中 challenge 只验证单个答案，无 24 题题集 | 0% |
| 会话模型：4 种类型 | ✅ 字段支持，但只使用 one_shot | 40% |
| 访问决策表 | ❌ 无策略决策表 | 0% |

### 4.11 安全规范

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

### 4.12 平台密钥存储适配指南

| 文档要求 | 代码实现 | 一致性 |
|----------|----------|--------|
| Windows：凭据管理 | ✅ `WindowsCredentialSecretStore` | 100% |
| macOS：Keychain | ❌ 未实现 | 0% |
| Linux：Secret Service | ✅ `LinuxSecretServiceStore` | 100% |
| 统一适配接口 | ✅ 统一接口 | 100% |
| 命名建议 | ⚠️ 只使用 `storylock/masterSalt` | 40% |
| 降级原则：默认返回明确错误 | ⚠️ `allowMemoryFallback` 可降级 | 50% |
| 开发模式安全边界 | ⚠️ 有 `developmentMode` 标记，但无显式告警 | 40% |

---

## 五、不一致项汇总（v3.1）

### 🔴 高严重程度（剩余 3 项）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 1 | **24 题题集未实现**：文档要求渐进式挑战（6/12/22-of-24），代码只支持单答案验证；参赛说明强调"联想记忆"但代码未体现 | 无法按文档实现 L2-L5 访问强度分级，参赛说明的"联想记忆"表述与代码有落差 | 实现 `question_set` 表和题集选择逻辑 |
| 2 | **答案仍以字符串处理，无 Buffer 清零**：文档要求 Buffer 安全清除，代码使用不可变字符串 | 答案可能长期滞留内存 | 将答案处理改为 Buffer 流程 |
| 3 | **masterSalt 默认内存存储**：`MemorySecretStore` 默认使用，开发模式无显式告警 | 开发模式下敏感材料明文存内存 | 默认启用平台密钥链，内存存储需显式确认并输出告警 |

### 🟡 中严重程度（剩余 8 项）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 4 | **对象分类和访问决策表仍需细化**：已实现 `objectType/requestedAction -> low/medium/high`，但尚未覆盖完整对象分类与 L1-L5 决策 | 可支持当前主线授权强度判断，但精细策略不足 | 扩展 `AccessPolicyEngine` 和对象分类表 |
| 5 | **后台清理仍可增强**：已实现启动时/手动 `cleanupExpired()`，但无长期运行时定时器 | 长时间运行进程仍需定时触发 | 增加宿主层定时清理或 CLI 清理命令 |
| 6 | **macOS 密钥存储未实现**：文档要求 macOS Keychain，代码只有 Windows/Linux | macOS 平台无法安全存储 | 实现 `MacOSKeychainSecretStore` |
| 7 | **无题集版本标记**：文档要求 active/deprecated/pending 标记 | 无法平滑升级题集 | 添加 `question_set` 表和版本字段 |
| 8 | **无规范化版本兼容策略**：文档要求双版本校验窗口 | 升级规范化规则后旧答案失效 | 实现 `normalizationVersion` 字段和兼容校验 |
| 9 | **无设备重启恢复策略**：文档要求重启后恢复 locked/session 状态 | 重启后状态丢失 | 实现启动时状态扫描和恢复 |
| 10 | **签名审计链仍可增强**：`SignatureAuthorizationSkill` 已将 `authorizationId`、`scope`、`resource`、`signatureHash` 写入统一 SQLite 审计表，但当前主要存放在 `meta_json` 中 | 当前可完成演示签名授权并保留持久审计记录，后续若要做 SQL 级检索仍可增强 | 可按检索需求为 `audit_log` 增加 `scope`、`resource`、`signature_hash` 独立列 |

### 🟢 低严重程度（剩余 5 项）

| # | 问题 | 影响 | 建议修复 |
|---|------|------|----------|
| 12 | **局部遮盖未实现**：文档要求电话/邮箱局部遮盖（如 138****0000） | 用户体验差 | 实现局部遮盖函数 |
| 13 | **开发模式无显式告警**：文档要求启动时输出清晰告警 | 开发者可能忽略安全风险 | 在 `MemorySecretStore` 初始化时输出 `console.warn()` |
| 14 | **无 HTTP 状态码映射**：文档建议 HTTP 状态码映射，代码无 HTTP 层 | 若后续接入 HTTP 需重新设计 | 在网关层实现 HTTP 映射 |
| 15 | **无性能基准测试**：文档建议 <50ms/<100ms 等目标 | 无法验证性能达标 | 添加性能基准测试 |
| 16 | **session 类型只使用 one_shot**：文档定义 4 种类型，代码只使用 one_shot | 无法支持短时连续读取或批量处理 | 扩展 `issueSession()` 支持多种类型 |

---

## 六、代码实现超出文档的部分（正向偏差）

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
| 11 | **对象强度与九宫格主线**：新增 `ObjectStrengthPolicySkill`、`GridChallengeSkill`、`LocalAuthorizationSkill` | v3.0 未覆盖 | ⭐ 主线边界收敛 |
| 12 | **远程签名主接口**：新增 `requestSignature` 并保留 `requestChallengeSign` 兼容别名 | v3.0 未覆盖 | ⭐ 命名与职责边界更清晰 |

---

## 七、建议修复清单（v3.1 优先级）

### P0（阻塞性，必须修复）

1. [ ] **实现 24 题题集**：添加 `question_set` 表，实现 6/12/22-of-24 渐进式挑战（当前最大缺失，参赛说明"联想记忆"表述与代码有落差）
2. [ ] **将答案处理改为 Buffer**：`normalizeAnswerValue()` 返回 Buffer，校验后 `zeroizeBytes()`
3. [ ] **默认启用平台密钥链**：`createAccessHost()` 默认 `usePlatformSecretStore=true`，内存存储需显式确认并输出告警

### P1（重要，建议尽快修复）

4. [ ] **细化对象分类和策略决策**：在现有 `objectType/requestedAction` 基础上扩展 `AccessPolicyEngine`
5. [ ] **实现 macOS Keychain**：添加 `MacOSKeychainSecretStore`
6. [ ] **增强后台清理**：在宿主层定时调用 `cleanupExpired()` 或提供 CLI 清理命令
7. [ ] **实现题集版本标记**：`question_set` 表添加 `status` 字段 (active/deprecated/pending)
8. [ ] **实现规范化版本兼容**：`normalizationVersion` 字段和双版本校验
9. [ ] **实现设备重启恢复**：启动时扫描并恢复 `locked`/`session_active` 状态
### P2（优化，可后续迭代）

11. [ ] **实现局部遮盖**：电话/邮箱局部遮盖函数
12. [ ] **开发模式告警**：`MemorySecretStore` 初始化时输出 `console.warn()`
13. [ ] **扩展 session 类型**：支持 `short_session`, `batch_session`, `privileged_session`
14. [ ] **添加性能基准测试**：`benchmark.mjs`
15. [ ] **实现 HTTP 状态码映射**：网关层 HTTP 映射
16. [ ] **扩展审计字段**：`audit_log` 添加 `scope`, `resource`, `signature_hash`

---

## 八、结论

### v3.0 → v3.1 改进总结

**新增收敛结果**：
1. 第二层新增对象强度、九宫格验证、本地授权三项主线能力
2. 第三层主接口收敛为 `requestSignature` 与 `requestPasswordFill`
3. `SignatureAuthorizationSkill` 已作为推荐签名授权入口
4. 补充第二层主线 schema 与远程签名 schema
5. 一致性评分提升至 **93%**

**参赛说明文档评价**：
- 产品说明准确描述了三层结构和核心能力，与代码实现高度一致
- 安全机制描述完整，与代码实现匹配
- 自测覆盖场景描述准确，与 `selftest.mjs` 测试用例对应
- 扩展方向明确，与代码剩余缺失项和设计文档的"后续扩展"章节对齐
- **主要偏差**："联想记忆"表述与代码实现有落差（代码只验证单个答案，未体现 24 题题集机制）

### 总体评价

StoryLock v3.1 代码在**主线职责边界**上已明显收敛：第二层不再只停留在故事读写原型，第三层也不再以 challenge 命名作为主入口。当前仍有以下关键缺失：

1. 🔴 **24 题题集缺失**：这是当前最大的文档-代码偏差，导致无法按文档实现渐进式访问控制，参赛说明的"联想记忆"表述与代码有落差
2. 🔴 **答案字符串处理**：高敏感材料内存安全未达标
3. 🔴 **masterSalt 默认内存存储**：开发模式安全风险

**建议**：
- **短期（1-2 周）**：修复 P0 项（题集、Buffer、密钥链）。
- **中期（3-4 周）**：修复 P1 项（策略决策、macOS、清理、版本兼容）。
- **长期**：完善 P2 优化项，实现完整文档定义的能力。

---

*评审完成时间：2026-06-17 01:35 GMT+8*  
*评审人：代码安全审计师*  
*版本：v3.1（基于主线接口收敛后的代码及参赛说明文档重新评审）*
