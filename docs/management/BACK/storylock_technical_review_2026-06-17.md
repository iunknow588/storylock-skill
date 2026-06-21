# StoryLock 技术评审意见

| 项目 | 内容 |
|------|------|
| 评审日期 | 2026-06-17 |
| 评审对象 | StoryLock 三层 Skill 方案（`skill/src`） |
| 评审依据 | `docs/design/cn` 设计文档 + `docs/usecase/00-参赛说明文档.md` |
| 评审维度 | 技术可行性、安全实现、文档一致性、代码质量、工程风险 |
| 评审结论 | **整体可行，局部存在设计-实现偏差与待收敛项** |

---

## 一、总体评价

StoryLock 的三层 Skill 拆分（本地处理 → 本地授权 → 远程网关）在架构层面是合理的，核心安全机制（九宫格验证、防重放、短时会话、答案摘要存储）已在代码中有可运行的实现。自测脚本覆盖主要链路，WASM 构建脚本存在，说明项目具备可验证性。

**当前状态：架构设计 > 代码实现 > 文档一致性**

代码中仍存在早期"故事读写/challenge sign"口径的历史接口，与参赛说明的收敛方向存在偏差，需要按文档指引完成接口收敛。

---

## 二、技术可行性评审

### 2.1 架构可行性：✅ 通过

| 层级 | 设计定位 | 代码落点 | 评价 |
|------|---------|---------|------|
| 第一层 | 本地故事处理 | `storylock-local-story-processing-skill/index.js` | 规则引擎+模板化实现，不依赖本地 LLM，符合"当前阶段最小落地"策略 |
| 第二层 | 本地受控访问 | `storylock-local-story-access-skill/index.js` + `access-host.js` | 对象强度策略、九宫格验证、本地授权、会话管理均已实现 |
| 第三层 | 远程网关 | `storylock-remote-gateway-skill/index.js` | `requestSignature` / `requestPasswordFill` 主接口已定义，EIP-712 结构已嵌入 |
| 汇总入口 | 示例/自测/WASM | `storylock-skill-engine/` | 存在 demo、selftest、build:wasm 脚本 |

三层边界在代码中有明确目录隔离，调用链方向（第三层 → 第二层 → 第一层）符合设计约束。

### 2.2 核心安全机制：⚠️ 基本实现正确，存在细节偏差

| 安全机制 | 设计文档要求 | 代码实现 | 偏差 |
|---------|------------|---------|------|
| **对象级密码强度** | 根据 `objectType` + `requestedAction` 判断 | `resolveStrength()` 已实现映射 | ✅ 一致 |
| **九宫格验证** | 按强度生成 `3/6/9` cells | `gridPolicyForStrength()` + `buildGridCells()` | ✅ 一致 |
| **防重放** | `requestId` 幂等 + `nonce` 去重 + `expiry` 校验 | `ensureReplaySafe()` 在 SQLite 事务中实现 | ✅ 一致 |
| **答案摘要** | HMAC-SHA256 + `identitySalt` | `deriveIdentityAnswerKey()` + `hmacSha256Hex()` | ✅ 一致 |
| **会话模型** | 短时会话 + 读写预算 | `issueSession()` 支持 `readBudget`/`writeBudget`/`ttlMs` | ✅ 一致 |
| **自动锁定** | 连续失败 3 次锁定 15 分钟 | `failure_window` + `MAX_FAILURES_PER_WINDOW` | ✅ 一致 |
| **状态机** | 8 状态完整闭环 | `challenge_state` 表有 `status` 字段，但代码中状态转换未完整覆盖 `idle`/`answers_submitted`/`verified` | ⚠️ 见 3.1 |
| **GCM nonce 管理** | 每次加密独立 96-bit nonce | `encryptAes256Gcm()` 使用 `randomBytes(12)` | ✅ 一致 |
| **密钥层次** | `masterSalt → rootKey → workKey → objectKey` | `deriveRootKey()` → `deriveWorkKey()` → `deriveIdentityAnswerKey()` | ⚠️ 见 3.2 |
| **单故事终身制** | 用户只维护一个故事，不主动轮换 | 代码中 `masterSalt` 从 SecretStore 读取或创建，但无"单故事"语义绑定 | ⚠️ 见 3.3 |

### 2.3 技术栈可行性：✅ 通过

| 技术选型 | 设计建议 | 代码实现 | 评价 |
|---------|---------|---------|------|
| 运行时 | Node.js >= 18 LTS | `package.json` 未显式声明引擎，但使用 `node:sqlite`（Node 22+） | ⚠️ 需确认最低版本 |
| 本地存储 | SQLite 单文件 | `DatabaseSync` + WAL 模式 | ✅ |
| 敏感存储 | 操作系统密钥链 | `WindowsCredentialSecretStore` / `LinuxSecretServiceStore` / `MemorySecretStore` | ✅ 有统一抽象 |
| 加密算法 | AES-256-GCM / HKDF-SHA256 / HMAC-SHA256 | `node:crypto` 原生实现 | ✅ |
| WASM 构建 | Rust + wasm-pack | `build-wasm.mjs` 调用 `wasm-pack` | ✅ 但需外部依赖 |

---

## 三、设计-实现一致性评审（重点）

### 3.1 Challenge 状态机：⚠️ 部分实现，存在缺口

**设计文档要求**（`Challenge状态机.md`）：
- 8 个状态：`idle` → `challenge_created` → `answers_submitted` → `verified` → `session_active` → `session_expired` / `failed` → `locked` → `idle`
- 状态转换必须串行化，以 `challengeId` 为粒度加锁
- `answers_submitted` 是运行态，应放内存

**代码现状**（`access-host.js`）：
- `challenge_state` 表有 `status` 字段，但实际使用的状态值只有：`challenge_created`、`failed`、`locked`、`verified`、`expired`
- **缺少 `idle` 状态**：创建 challenge 前没有 `idle` 状态记录
- **缺少 `answers_submitted` 状态**：`submitChallengeAnswers()` 中直接 `UPDATE ... SET status = 'answers_submitted'` 后立即校验，没有作为独立运行态保留
- **缺少 `session_active` 状态**：session 状态在 `session_store` 表中管理，challenge 状态不跟踪 `session_active`
- **状态转换校验不完整**：`submitChallengeAnswers()` 中只校验 `status = 'challenge_created'`，未校验前置状态序列

**影响**：状态机不完整，但核心功能（创建 → 提交 → 验证 → 锁定）已运行。建议按文档补齐状态定义，或调整文档以匹配实现。

**建议**：
1. 若参赛时间紧，优先调整文档，将状态机收敛为代码中实际使用的 5 个状态
2. 若时间允许，补齐 `idle` 和 `answers_submitted` 状态，使状态转换链更严谨

### 3.2 密钥层次：⚠️ 实现简化，与文档存在偏差

**设计文档要求**（`安全规范.md`）：
```
用户故事 → masterSalt → rootKey → workKey → objectKey
```

**代码现状**（`access-host.js`）：
```javascript
deriveRootKey(masterSalt) → deriveWorkKey(masterSalt, purpose) → deriveIdentityAnswerKey(masterSalt, identityId)
```

**偏差**：
- 代码中 `deriveWorkKey` 直接以 `masterSalt` 为输入，而非以 `rootKey` 为输入
- 实际层次为：`masterSalt → workKey`（跳过 rootKey）或 `masterSalt → identityKey`
- 缺少 `objectKey` 派生实现

**影响**：当前阶段功能不受影响（答案摘要和身份隔离已工作），但与文档描述的 4 层密钥层次不一致。

**建议**：
1. 短期：修改文档，将密钥层次描述为当前实际实现的 3 层（`masterSalt → workKey → identityKey`）
2. 中期：若需要加密对象存储，补全 `objectKey` 派生

### 3.3 单故事终身制：⚠️ 概念未在代码中体现

**设计文档要求**（`安全规范.md`）：
- 每个用户只维护一个故事根上下文
- `masterSalt` 从故事派生，终身不变
- 题集可扩展，旧题渐进式淘汰

**代码现状**：
- `masterSalt` 由 `randomBytes(32)` 生成，与"用户故事"无派生关系
- 题集存储在 `answer_digest_set` 表中，无版本标记（`active`/`deprecated`/`pending`）
- 无题集扩展或淘汰机制

**影响**："单故事终身制"是安全规范中的核心概念，但代码中完全未体现。当前实现更接近"随机盐值 + 固定答案摘要"模式。

**建议**：
1. 这是设计文档与代码之间的最大偏差
2. 若参赛说明强调"联想记忆线索参与授权"，则需要在代码中体现"故事 → 题集 → 答案摘要"的派生关系
3. 当前代码中的 `enrollAnswers()` 只是静态录入答案，与用户故事无关联

### 3.4 接口收敛：⚠️ 历史接口未清理

**设计文档要求**（`参赛说明文档.md` + `三包接口契约.md`）：
- 第三层主接口只保留：`requestSignature`、`requestPasswordFill`
- 历史接口 `requestChallengeSign` 应收敛为 `requestSignature`
- 故事读写相关接口不作为第三层主线能力

**代码现状**（`storylock-skill-engine/assets/migrated/skills/authorization-skills.js`）：
- 仍存在 `LoginAuthorizationSkill`、`ChallengeSigningAuthorizationSkill`、`SignatureAuthorizationSkill`
- `ChallengeSigningAuthorizationSkill` 的 `skillId()` 为 `"challenge_signing_authorization"`，与收敛方向不符
- `storylock-skill-engine/index.js` 导出 `SignatureAuthorizationSkill`（而非 `SigningAuthorizationSkill`）

**建议**：
1. 按参赛说明完成接口收敛：
   - `storylock-skill-engine` 中导出 `SigningAuthorizationSkill`（或统一为 `LocalPasswordFillSkill` + 新的签名 Skill）
   - 清理 `ChallengeSigningAuthorizationSkill` 的历史命名
2. 在 `docs/management` 中记录接口收敛计划

### 3.5 第三层远程网关：⚠️ 脱敏实现正确，但缺少第二层调用链

**设计文档要求**（`本地Agent网关设计.md`）：
- 第三层只做请求包装、字段校验、过期校验、白名单能力限制
- 第三层不得直接调用第二层内部接口
- 返回结果遵循最小化原则

**代码现状**（`storylock-remote-gateway-skill/index.js`）：
- `redactRemoteValue()` 实现正确，高敏字段（`answers`、`signingKey`、`password`、`privateKey` 等）被替换为 `[redacted]`
- `requestSignature` / `requestPasswordFill` 结构符合 EIP-712 最小定义
- **但**：`StoryLockRemoteGateway` 通过 `transport` 函数或 `executor` 函数执行，没有显式调用第二层 Skill（`ObjectStrengthPolicySkill` / `GridChallengeSkill` / `LocalAuthorizationSkill`）

**影响**：当前第三层是"请求包装器 + 脱敏器"，但缺少与第二层实际 Skill 的集成。demo 和 selftest 中使用的是 mock host，未验证完整三层调用链。

**建议**：
1. 在 `storylock-skill-engine/scripts/demo.mjs` 中补充三层完整调用链（第三层 → 第二层 → 第一层）
2. 或明确说明当前阶段第三层只做"结构化请求包装"，第二层调用由 Host 层内部完成

### 3.6 错误码一致性：⚠️ 部分映射未对齐

**设计文档要求**（`本地Agent网关设计.md` + `三包接口契约.md`）：
- 统一 `SLG` 前缀错误码：`SLG-001` 到 `SLG-012`
- `REQUEST_EXPIRED` 对应 `SLG-011`

**代码现状**（`errors.js`）：
- 错误码映射存在，但 `access-host.js` 中部分错误使用 `err.key`（如 `'CHALLENGE_LOCKED'`、`'CHALLENGE_FAILED'`）而非 `SLG` 前缀
- `ensureReplaySafe()` 中返回 `SLG-013`（文档中未定义）

**建议**：
1. 统一错误码输出格式，全部使用 `SLG-xxx` 前缀
2. 补充 `SLG-013` 到文档，或替换为文档中已有的 `SLG-010`（NONCE_REPLAY_DETECTED）

---

## 四、代码质量评审

### 4.1 安全编码：✅ 基本正确

| 项目 | 评价 |
|------|------|
| 敏感数据清零 | `zeroizeBytes()` 使用 `Uint8Array.fill(0)`，但 `storylock-skill-engine` 中 `signingKeyBytes` 清零在 `finally` 块中执行，正确 |
| 答案不持久化 | 原始答案只用于内存校验，只存储 HMAC 摘要，正确 |
| SQL 注入防护 | 使用参数化查询（`db.prepare().run(...)`），正确 |
| 事务使用 | `BEGIN IMMEDIATE` + `COMMIT/ROLLBACK`，正确 |
| 并发控制 | 以 SQLite 事务为锁，建议补充 `challengeId` 级乐观锁校验 |

### 4.2 潜在问题

| 问题 | 位置 | 风险 | 建议 |
|------|------|------|------|
| `normalizeAnswerValue` 返回空字符串时未过滤 | `access-host.js` | 空答案可能通过校验 | 在 `enrollAnswers` 和 `submitChallengeAnswers` 中过滤空值 |
| `buildGridCells` 中 `seed` 字段未实际使用 | `index.js` | 无实际熵源，cells 只是静态占位 | 若需要真实九宫格，应引入随机选择或基于故事的派生 |
| `session_store` 无 `challenge_id` 外键约束 | `sqlite-schema.sql` | 数据一致性依赖应用层 | 建议添加外键或应用层校验 |
| `request_store` 的 `response_json` 可能无限增长 | `access-host.js` | 长期运行后存储膨胀 | 已实现 `cleanupExpired`，但需确认是否定期触发 |
| `MemorySecretStore` 默认无加密 | `secret-store.js` | 开发模式下敏感数据明文存内存 | 已输出警告，符合预期 |
| Windows 凭据存储依赖 PowerShell 模块 | `secret-store.js` | 环境依赖 `CredentialManager` 模块 | 建议补充环境检查脚本 |

### 4.3 测试覆盖

| 测试 | 覆盖内容 | 评价 |
|------|---------|------|
| `local-story-access selftest` | 对象强度、九宫格、防重放、幂等、锁定、解锁、清理、Schema 迁移、审计、签名 | ✅ 覆盖全面，约 15 个检查点 |
| `remote-gateway selftest` | 请求结构、EIP-712、脱敏、本地执行器、算法校验 | ✅ 覆盖核心功能 |
| `local-story-processing selftest` | 草稿、润色、强度评估 | ✅ 基础覆盖 |
| `skill-engine selftest` | 密码填充、签名、审计元信息 | ✅ 但使用 mock host，非真实链路 |
| `skill-engine demo` | 完整链路演示 | ✅ 但同样使用 mock host |
| `build:wasm` | WASM 构建脚本 | ✅ 存在，但依赖外部 `wasm-pack` 和 Rust 仓库 |

---

## 五、风险评估

### 5.1 高风险

| 风险 | 说明 | 缓解建议 |
|------|------|---------|
| **单故事终身制未实现** | 设计文档的核心安全概念（用户故事 → 密钥派生）在代码中完全缺失，当前 `masterSalt` 是随机生成的 | 短期：调整文档，降低"单故事"语义；中期：实现故事到 `masterSalt` 的派生 |
| **九宫格验证无真实题目** | `buildGridCells` 生成的 cells 只有 `promptRef` 占位，没有实际题目内容，无法用于真实授权 | 短期：在 demo 中补充 mock 题目；中期：实现基于题集的九宫格选择 |

### 5.2 中风险

| 风险 | 说明 | 缓解建议 |
|------|------|---------|
| **第三层与第二层未实际集成** | 远程网关目前只包装请求，未实际调用第二层 Skill | 在 demo 中补充完整调用链 |
| **状态机不完整** | 缺少 `idle`、`answers_submitted`、`session_active` 状态 | 补齐状态或调整文档 |
| **错误码不统一** | 部分使用 `SLG-xxx`，部分使用 `err.key` | 统一为 `SLG` 前缀 |
| **Node.js 版本要求未声明** | 使用 `node:sqlite`（Node 22+），但 `package.json` 无 `engines` 字段 | 补充 `engines` 声明 |

### 5.3 低风险

| 风险 | 说明 | 缓解建议 |
|------|------|---------|
| WSL 环境未测试 | `createPlatformSecretStore` 在 Windows 上返回 `WindowsCredentialSecretStore`，WSL 中可能误判 | 补充 `platform` 检测逻辑 |
| 题集版本管理缺失 | 无 `active`/`deprecated`/`pending` 标记 | 当前阶段可接受，后续扩展 |

---

## 六、行动建议

### 6.1 参赛前必须完成（P0）

1. **统一错误码**：全部收敛为 `SLG-001` ~ `SLG-012` 范围，移除 `SLG-013` 或补充文档
2. **补齐 `package.json` 引擎声明**：声明 `"node": ">=22.0.0"`（因使用 `node:sqlite`）
3. **清理历史接口命名**：`storylock-skill-engine` 中收敛为 `SigningAuthorizationSkill`，移除 `ChallengeSigningAuthorizationSkill` 的对外暴露
4. **补充三层完整调用链 demo**：在 `storylock-skill-engine/scripts/demo.mjs` 中展示"第三层请求 → 第二层授权 → 第一层处理"的完整流程

### 6.2 参赛前建议完成（P1）

1. **补齐 Challenge 状态机**：在 `access-host.js` 中补充 `idle` 和 `answers_submitted` 状态，或调整文档以匹配实现
2. **统一密钥层次描述**：文档与代码对齐为 `masterSalt → workKey → identityKey`
3. **九宫格验证补充真实题目**：在 `buildGridCells` 或 demo 中引入基于题集的 cell 内容
4. **补充 `cleanupExpired` 定期触发机制**：当前只在 `createAccessHost` 时调用一次，建议说明是否需要周期性清理

### 6.3 参赛后扩展（P2）

1. **实现单故事终身制**：将用户故事文本与 `masterSalt` 派生关联，实现题集版本管理
2. **完整链上验证**：替换 EIP-712 的 `1-placeholder` 和零地址为真实链参数
3. **多设备 nonce 同步**：当前随机 nonce 方案仅适用于单设备
4. **硬件密钥模块集成**：将 `signer` 从软件实现扩展到硬件安全模块

---

## 七、结论

StoryLock 是一个**架构设计清晰、安全机制基本到位、具备可运行代码**的 Agent 安全能力方案。三层 Skill 的拆分符合最小权限原则，核心安全功能（九宫格验证、防重放、短时会话、答案摘要）已实现并可自测。

**当前主要偏差在于：设计文档的"单故事终身制"和"完整状态机"在代码中未完全落地，以及第三层与第二层之间的实际集成链路尚未在 demo 中展示。**

建议参赛前优先完成错误码统一、接口收敛、三层调用链 demo 这三项，即可形成一份"设计-文档-代码"三者一致的参赛交付物。

---

*评审完成时间：2026-06-17*
*评审依据：design/cn 全部 18 份设计文档 + usecase/00-参赛说明文档.md + src 全部代码文件*
