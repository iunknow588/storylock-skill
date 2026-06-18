# StoryLock 项目评审意见

## 评审概述

基于 `E:\2026OPC大赛\skill\docs` 设计文档对 `E:\2026OPC大赛\skill\src` 代码进行全面评审。

**评审结论：项目 P0 基线基本完成，具备参赛展示条件；P1 可用性待完善；P2 属于后续探索方向。**

---

## 一、项目结构完整性

### 1.1 代码组织（✅ 通过）

| 层级 | 代码包 | 状态 | 说明 |
|------|--------|------|------|
| 第一层 | `storylock-local-story-processing-skill` | ✅ 完整 | 故事草稿、润色、强度评估 |
| 第二层 | `storylock-local-story-access-skill` | ✅ 完整 | 对象强度、九宫格、授权、审计 |
| 第三层 | `storylock-remote-gateway-skill` | ✅ 完整 | 签名请求、密码填充、脱敏 |
| 兼容演示 | `storylock-skill-engine` | ✅ 完整 | 本地执行示例 |
| 共享层 | `shared/` | ✅ 完整 | crypto、secret-store、sqlite |

### 1.2 文档体系（✅ 通过）

| 文档类别 | 覆盖度 | 状态 |
|----------|--------|------|
| 设计文档（design/cn/） | 主线文档 + 历史归档 | ✅ 完整 |
| 参赛参考（ref/） | 4 份 + 1 份分析 | ✅ 完整 |
| 测试方案（test/） | 1 份详细方案 | ✅ 完整 |
| 管理文档（management/） | 完善度分析 + 实施计划 | ✅ 完整 |
| 中英文 README | 各 1 份 | ✅ 完整 |

---

## 二、功能实现评审

### 2.1 第一层：故事处理（✅ 通过）

**已实现能力：**
- `StoryDraftSkill`：结构化草稿生成
- `StoryRefineSkill`：草稿润色与整理
- `StrengthReviewSkill`：24 题题集强度评估

**边界遵守：**
- ✅ 不签发授权
- ✅ 不读取 secret
- ✅ 不创建 challenge
- ✅ 边界标记正确（`challengeCreated=false`, `sessionIssued=false`）

**自测结果：** ✅ 通过

### 2.2 第二层：本地访问授权（✅ 通过）

**已实现能力：**
- `ObjectStrengthPolicySkill`：对象强度判断（low/medium/high）
- `GridChallengeSkill`：九宫格验证生成与防重放
- `LocalAuthorizationSkill`：答案校验与短时授权

**安全机制：**
- ✅ SQLite 状态存储（7 张表）
- ✅ requestId/nonce 防重放（`SLG-013`）
- ✅ 过期请求拒绝（`SLG-011`）
- ✅ 失败锁定（24h 窗口，3 次失败锁定 15 分钟）
- ✅ 答案摘要存储（HMAC-SHA256，不存明文）
- ✅ 审计日志（`audit_log`）
- ✅ cleanup 过期状态
- ✅ SecretStore 生产约束（持久化库拒绝内存 SecretStore）

**自测结果：** ✅ 通过（14 项检查全部通过）

### 2.3 第三层：远程网关（✅ 通过）

**已实现能力：**
- `requestSignature`：EIP-712 风格签名请求包装
- `requestPasswordFill`：Web2 密码填充请求包装
- `DelegatedSignatureSkill`：委托签名

**安全机制：**
- ✅ 递归脱敏（11 个敏感字段黑名单）
- ✅ 算法白名单（ed25519 / secp256k1）
- ✅ 请求字段校验
- ✅ 过期检查
- ✅ 默认不返回明文密码（`audit_meta_only`）

**自测结果：** ✅ 通过

### 2.4 端到端演示（✅ 通过）

**三层签名链路：**
1. 第三层 `requestSignature` →
2. 第二层对象强度策略 →
3. 第二层九宫格验证 →
4. 第二层本地授权 →
5. 本地签名执行器 →
6. 第三层脱敏返回 →
7. SQLite 审计写入

**自测结果：** ✅ 通过

---

## 三、安全审计

### 3.1 密钥管理（⚠️ 部分通过）

| 项目 | 状态 | 说明 |
|------|------|------|
| 密钥分层（masterSalt → rootKey → workKey → objectKey） | ✅ 实现 | `crypto.js` + `access-host.js` |
| HKDF-SHA256 派生 | ✅ 实现 | 带 salt 和 info |
| 答案摘要（HMAC-SHA256） | ✅ 实现 | `deriveIdentityAnswerKey` |
| masterSalt 存储 | ⚠️ 部分 | 内存模式用于开发，平台适配接口已预留 |
| 长期密钥生产存储 | ⚠️ 待完善 | 仅有 Windows/macOS/Linux 适配指南，未完整实现 |

**风险：** 当前 `MemorySecretStore` 在开发模式下使用，生产环境需要接入平台密钥链。已有 `createPlatformSecretStore()` 工厂，但各平台具体实现可能不完整。

### 3.2 防重放保护（✅ 通过）

| 机制 | 实现 | 验证 |
|------|------|------|
| requestId 幂等返回 | ✅ | 相同请求返回缓存响应 |
| requestId 冲突检测 | ✅ | 不同 payload 返回 `SLG-013` |
| nonce 冲突检测 | ✅ | 返回 `SLG-013` |
| 过期清理 | ✅ | 自动清理 24h 前记录 |
| 批次限制 | ✅ | 默认 1000 条 |

### 3.3 脱敏机制（✅ 通过）

**脱敏字段清单（11 项）：**
```
answers, signingKey, signingKeyBytes, secretBytes, secretValue,
password, privateKey, mnemonic, seed, rawSecret, keyMaterial
```

**验证：** ✅ e2e 自测确认 `signingKeyBytes` 和 `privateKey` 被替换为 `[redacted]`

### 3.4 错误码体系（✅ 通过）

| 错误码 | 类型 | 验证状态 |
|--------|------|----------|
| `SLG-001` | validation_error | ✅ |
| `SLG-003` | challenge_failed | ✅ |
| `SLG-004` | challenge_locked | ✅ |
| `SLG-005` | session_invalid | ✅ |
| `SLG-006` | budget_exhausted | ✅ |
| `SLG-007` | object_not_found | ✅ |
| `SLG-008` | redaction_required | ✅ |
| `SLG-009` | scope_insufficient | ✅ |
| `SLG-010` | secret_unavailable | ✅ |
| `SLG-011` | request_expired | ✅ |
| `SLG-012` | internal_error | ✅ |
| `SLG-013` | replay_detected | ✅ |

---

## 四、文档与代码一致性

### 4.1 一致性检查（✅ 通过）

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 主线接口统一为 `requestSignature`/`requestPasswordFill` | ✅ | 代码与文档一致 |
| 旧接口已移除主线 | ✅ | 不再提及 `requestChallengeSign` 等 |
| 第二层不读写故事 | ✅ | 代码与文档一致 |
| 第三层不持有密钥 | ✅ | 代码与文档一致 |
| 错误码文档与代码一致 | ✅ | `errors.js` 与所有文档对齐 |
| Node.js 22+ 要求 | ✅ | 四个 `package.json` 均声明 |
| Schema 文件完整 | ✅ | 每个包都有输入输出 Schema |

### 4.2 文档质量（✅ 良好）

**优点：**
- 中文设计文档主线覆盖完整，历史材料已归档
- 有明确的术语表和快速决策指南
- 安全规范、脱敏规范、EIP-712 定义清晰
- 测试方案详细（20 页，含 60+ 测试用例）

**待改进：**
- 部分英文设计文档（design/en/）与中文文档存在不同步风险
- `storylock-skill-engine` 的 `assets/migrated/` 目录含旧代码，需明确标注为非主线

---

## 五、测试覆盖度

### 5.1 当前自测（✅ 通过）

| 包 | 测试项数 | 状态 |
|----|----------|------|
| 第一层 | 4 项 | ✅ |
| 第二层 | 14 项 | ✅ |
| 第三层 | 8 项 | ✅ |
| 兼容演示 | 2 项 | ✅ |
| 端到端 | 7 项断言 | ✅ |

### 5.2 测试方案规划（⚠️ 部分完成）

`docs/test/StoryLock测试方案_v1.0.md` 规划了完整的测试体系：

| 测试类型 | 规划 | 当前状态 |
|----------|------|----------|
| 冒烟测试 | ✅ | 已完成（selftest） |
| 单元测试 | ✅ | 部分（selftest 覆盖） |
| 集成测试 | ✅ | 部分（e2e selftest） |
| 安全专项测试 | ✅ | 部分（selftest 覆盖） |
| 契约测试（Schema） | ✅ | 已新增根目录 `npm run test:contract` |
| 文档-代码一致性测试 | ⚠️ | 未实现独立脚本 |
| 覆盖率统计 | ⚠️ | 未实现 |

---

## 六、具体问题与改进建议

### 6.1 高优先级（P0 阻塞项）

**无阻塞项。** 当前代码基线可运行，所有 selftest 通过。

### 6.2 中优先级（P1 完善项）

| 编号 | 问题 | 建议 | 影响 |
|------|------|------|------|
| P1-01 | 九宫格问题来源为占位式 | 接入真实题集或对象策略 | 功能完整性 |
| P1-02 | `storylock-skill-engine` 含旧迁移代码 | 明确标注 `assets/migrated/` 为非主线 | 清晰度 |
| P1-03 | 平台 SecretStore 未完整实现 | 补充 Windows/macOS/Linux 具体实现 | 生产安全 |
| P1-04 | Challenge `revoked` 状态未实现 | 增加主动撤销能力 | 可用性 |
| P1-05 | 独立契约测试脚本缺失 | 已新增轻量 Schema 契约脚本，后续可接入 Ajv 做完整校验 | 质量保障 |
| P1-06 | 覆盖率统计缺失 | 增加 `c8` 或 Node.js 内置覆盖率 | 质量保障 |
| P1-07 | 根目录统一测试入口缺失 | 已新增根 `package.json`，支持 `npm run selftest` 与 `npm run test` | 易用性 |

### 6.3 低优先级（P2 探索项）

| 编号 | 方向 | 说明 |
|------|------|------|
| P2-01 | HTTP 宿主集成 | 当前仅支持内存调用 |
| P2-02 | 多链多钱包场景 | 仅作为应用探索方向 |
| P2-03 | 多平台内容发布 | 仅作为应用探索方向 |
| P2-04 | EIP-1271/ERC-4337 生产集成 | 需要后续扩展 |
| P2-05 | WASM 构建优化 | 当前有 dist/wasm 但未完整集成 |

---

## 七、参赛建议

### 7.1 推荐展示重点

1. **三层端到端签名演示**（`npm run selftest:e2e`）——最能体现项目完整性
2. **第二层安全机制**——防重放、失败锁定、审计日志
3. **第三层脱敏机制**——递归替换敏感字段
4. **文档体系完整性**——主线设计文档 + 测试方案

### 7.2 推荐表述

> StoryLock 是一个本地优先的授权访问 Skill 项目。它通过故事处理、本地访问授权、远程网关三层结构，将九宫格验证、短时授权、签名请求和 Web2 密码填充包装为可调用能力，同时把长期秘密和授权判断保留在本地边界内。

### 7.3 避免表述

- ❌ "已经支持完整多链多钱包生产系统"
- ❌ "远程网关可以直接处理本地明文敏感内容"
- ❌ "兼容演示包就是完整生产安全边界"
- ❌ "九宫格已接入真实题集"（当前为占位式）

---

## 八、总体评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 功能完整性 | 8/10 | P0 完成，P1 部分完成 |
| 安全机制 | 8/10 | 核心安全机制到位，生产 SecretStore 待完善 |
| 代码质量 | 8/10 | 结构清晰，自测覆盖良好 |
| 文档完整性 | 9/10 | 主线设计文档体系完整，历史分析已归档 |
| 测试覆盖度 | 7/10 | selftest 通过，独立测试脚本待补 |
| 可运行性 | 9/10 | 所有 selftest 一键通过 |
| **综合** | **8.2/10** | **具备参赛条件，建议继续完善 P1 项** |

补充说明：已根据本评审补充根目录统一测试入口和轻量 Schema 契约测试脚本，项目可通过 `npm run test` 执行主要自测与契约检查。

---

## 九、结论

**StoryLock 项目当前状态：P0 可验收，P1 持续增强，P2 后续探索。**

项目已形成清晰的三层 Skill 架构，核心安全机制（防重放、失败锁定、脱敏、审计）均已实现并验证。文档体系完整，自测全部通过。建议参赛前重点完善九宫格真实题集接入和平台 SecretStore 实现，以提升项目完整度和生产可用性。

---

*评审日期：2026-06-17*  
*评审依据：`docs/design/cn/`、`docs/ref/`、`docs/test/`、`docs/management/` 及 `src/` 全部代码*
