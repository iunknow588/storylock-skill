# StoryLock 测试方案

**版本**: v1.0  
**日期**: 2026-06-16  
**适用范围**: `E:\2026OPC大赛\skill\src` 全部 Skill 包  
**输出目录**: `E:\2026OPC大赛\skill\docs\test`

---

## 一、测试目标

1. 验证三层 Skill 包（第一包/第二包/第三包）功能正确性
2. 验证接口契约与文档定义的一致性
3. 验证安全机制（challenge/session/防重放/脱敏）有效性
4. 验证新旧能力体系（`storylock-skill-engine` 迁移代码）兼容性
5. 建立可复用的自动化测试基线，支撑持续迭代

---

## 二、测试范围

| 测试对象 | 路径 | 优先级 |
|---------|------|--------|
| 第二包 — 本地故事访问 | `src/storylock-local-story-access-skill/` | P0 |
| 第一包 — 本地故事处理 | `src/storylock-local-story-processing-skill/` | P0 |
| 第三包 — 远程网关 | `src/storylock-remote-gateway-skill/` | P0 |
| 旧迁移包 — Skill Engine | `src/storylock-skill-engine/` | P1 |
| 跨包集成链路 | 第一包↔第二包↔第三包 | P0 |
| Schema 校验 | `assets/schemas/*.json` | P1 |

---

## 三、测试策略分层

### 3.1 单元测试（Unit Test）

**目标**: 单个 Skill/函数/模块的独立验证  
**工具**: Node.js Test Runner (`node --test`) 或 Vitest  
**位置**: 各包 `tests/` 目录

### 3.2 集成测试（Integration Test）

**目标**: 跨模块/跨包调用链路的端到端验证  
**工具**: Node.js Test Runner + 内存/临时 SQLite 存储  
**位置**: 根目录 `tests/integration/`

### 3.3 安全专项测试（Security Test）

**目标**: 验证安全机制不被绕过  
**工具**: 自定义攻击用例 + 模糊测试  
**位置**: 根目录 `tests/security/`

### 3.4 契约一致性测试（Contract Test）

**目标**: 验证代码实现与 JSON Schema、设计文档的字段/类型/约束一致  
**工具**: Ajv（JSON Schema 校验）+ 自定义字段映射检查  
**位置**: 根目录 `tests/contract/`

---

## 四、测试环境

### 4.1 最小环境

```
Node.js >= 18 LTS
npm >= 9
SQLite3（若测试持久化层）
```

### 4.2 测试数据隔离

| 环境类型 | 存储方式 | 用途 |
|---------|---------|------|
| 单元测试 | 内存 Mock Host | 快速、无状态、并行 |
| 集成测试 | 临时 SQLite 文件（`test-vault-{uuid}.db`） | 验证持久化与事务 |
| 安全测试 | 内存 + 临时 SQLite | 验证攻击场景 |

### 4.3 测试数据工厂

```javascript
// tests/fixtures/factory.js
export function makeTestIdentity(overrides = {}) {
  return {
    identityId: 'test-identity-001',
    masterSalt: crypto.randomBytes(32), // 测试用随机盐
    ...overrides,
  };
}

export function makeTestChallengeAnswers(overrides = {}) {
  return {
    answers: [
      { questionId: 'q-001', answer: 'test answer one' },
      { questionId: 'q-002', answer: 'test answer two' },
      // ... 至少6个用于 L2, 12个用于 L3, 22个用于 L4
    ],
    ...overrides,
  };
}

export function makeTestStoryObject(overrides = {}) {
  return {
    storyObjectId: 'story-test-001',
    title: 'Test Story',
    content: 'This is a test story content for validation.',
    sensitivity: 'private',
    version: 1,
    ...overrides,
  };
}

export function makeTestRequestEnvelope(overrides = {}) {
  return {
    requestId: `req-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    nonce: `nonce-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    expiry: Date.now() + 300_000, // 5分钟后过期
    ...overrides,
  };
}
```

---

## 五、测试用例详细设计

### 5.1 第二包 — 本地故事访问 Skill（P0）

#### 5.1.1 StoryReadAccessSkill

| 用例编号 | 用例名称 | 前置条件 | 输入 | 预期结果 | 验证点 |
|---------|---------|---------|------|---------|--------|
| R001 | 正常读取成功 | identity已注册，storyObject存在，answers正确 | identityId, storyObjectId, 正确answers, 有效requestId/nonce/expiry | status=success, result.storyObject存在, redactionLevel=partial | 挑战通过、session签发、对象读取、预算扣减 |
| R002 | 缺失identityId | - | 无identityId | 返回错误，code=SLG-012 | 参数校验 |
| R003 | 缺失storyObjectId | - | 无storyObjectId | 返回错误，code=SLG-012 | 参数校验 |
| R004 | 空answers数组 | - | answers=[] | 返回错误，code=SLG-003 | 挑战失败 |
| R005 | 错误answers | - | answers=[错误答案] | 返回错误，code=SLG-003, retryable=true | 挑战验证逻辑有效 |
| R006 | 过期expiry | - | expiry=过去时间 | 返回错误，code=SLG-011 | 时间校验 |
| R007 | 重复requestId | 同一requestId已成功 | 相同requestId | 返回错误，code=SLG-009 | 幂等校验 |
| R008 | 重复nonce | 同一nonce已使用 | 相同nonce | 返回错误，code=SLG-010 | 重放校验 |
| R009 | 时钟漂移容差 | expiry=nowMs()-20秒 | 请求 | 应成功（30秒容差内） | 漂移容差 |
| R010 | 超出时钟漂移 | expiry=nowMs()-40秒 | 请求 | 返回错误，code=SLG-011 | 漂移边界 |
| R011 | 对象不存在 | storyObjectId未注册 | 不存在的storyObjectId | 返回错误，code=SLG-007 | 对象存在性 |
| R012 | 脱敏partial | redactionLevel=partial | 正确请求 | result.storyObject.content为摘要，非原文 | 脱敏实现 |
| R013 | 脱敏full | redactionLevel=full | 正确请求 | result最小化，仅metadata | 脱敏实现 |
| R014 | 脱敏none | redactionLevel=none | 正确请求 | result.storyObject为完整对象 | 脱敏透传 |
| R015 | 预算耗尽后重读 | 同一session已读1次 | 再次读取 | 返回错误，code=SLG-006 | 预算限制 |
| R016 | session过期后读取 | session TTL已过 | 使用过期session | 返回错误，code=SLG-005 | 时间失效 |
| R017 | 失败3次后锁定 | 连续3次错误答案 | 第4次提交 | 返回错误，code=SLG-004, retryAfter存在 | 锁定策略 |
| R018 | 锁定期间提交 | 处于locked状态 | 提交答案 | 返回错误，code=SLG-004 | 锁定拒绝 |
| R019 | 锁定窗口结束后 | lockUntil已过 | 提交答案 | 应允许新challenge | 锁定恢复 |
| R020 | 并发重复提交 | 同一challengeId并行提交 | 两个并行请求 | 一个成功，一个返回并发错误 | 并发控制 |

#### 5.1.2 StoryWriteAccessSkill

| 用例编号 | 用例名称 | 前置条件 | 输入 | 预期结果 | 验证点 |
|---------|---------|---------|------|---------|--------|
| W001 | 正常写回成功 | identity已注册，storyObject存在，answers正确 | identityId, storyObjectId, content, 正确answers | status=success, writeResult.version递增 | 挑战通过、写权限、版本递增 |
| W002 | 缺失content | - | 无content | 返回错误，code=SLG-012 | 参数校验 |
| W003 | content非对象 | - | content="string" | 返回错误，code=SLG-012 | 类型校验 |
| W004 | 写预算耗尽 | 同一session已写1次 | 再次写入 | 返回错误，code=SLG-006 | 写预算限制 |
| W005 | 写回后读取验证 | 刚完成写回 | 用新session读取同一对象 | 读取内容应与写入一致 | 读写一致性 |
| W006 | 脱敏写回结果 | redactionLevel=partial | 正确请求 | writeResult不含完整content | 写回脱敏 |

#### 5.1.3 状态机验证

| 用例编号 | 用例名称 | 操作序列 | 预期状态序列 |
|---------|---------|---------|------------|
| SM001 | 基本流程 | createChallenge → submitAnswers(正确) → issueSession | idle → challenge_created → verified → session_active |
| SM002 | 失败重试 | createChallenge → submitAnswers(错误) → submitAnswers(正确) | idle → challenge_created → failed → challenge_created → verified |
| SM003 | 失败锁定 | createChallenge → submitAnswers(错误)x3 | idle → challenge_created → failed → locked |
| SM004 | 锁定恢复 | locked状态等待15分钟 | locked → idle |
| SM005 | session过期 | session_active等待TTL过期 | session_active → session_expired |
| SM006 | 主动撤销 | session_active → revokeSession | session_active → session_revoked |

#### 5.1.4 失败计数策略

| 用例编号 | 用例名称 | 操作 | 预期结果 |
|---------|---------|------|---------|
| FC001 | 同一identity不同challenge | identityA创建chl-1失败，再创建chl-2失败 | 失败计数=2（按identityId累计） |
| FC002 | 不同identity | identityA失败，identityB失败 | 各自独立计数，不互相影响 |
| FC003 | 24小时窗口重置 | 23小时前失败2次，现在再失败1次 | 计数=3（仍在同一窗口） |
| FC004 | 窗口过期后 | 25小时前失败3次，现在再失败 | 计数=1（新窗口） |
| FC005 | 成功后重置 | 失败2次 → 成功1次 → 再失败 | 计数=1（成功重置连续失败计数） |

---

### 5.2 第一包 — 本地故事处理 Skill（P0）

#### 5.2.1 StoryDraftSkill

| 用例编号 | 用例名称 | 输入 | 预期结果 | 验证点 |
|---------|---------|------|---------|--------|
| D001 | 正常生成 | objective="test", audience="self", tone="neutral" | draft.content包含objective, draft.title存在 | 基本生成逻辑 |
| D002 | 缺失objective | 无objective | 抛出错误 | 必填校验 |
| D003 | 空objective | objective="" | 抛出错误 | 非空校验 |
| D004 | 自定义generator | 传入自定义generator函数 | 应调用自定义generator | 依赖注入 |
| D005 | 约束条件传递 | constraints=["c1", "c2"] | draft.cues包含约束 | 约束传递 |
| D006 | source校验 | source="invalid" | 抛出错误 | 枚举校验 |
| D007 | 默认source | 不传入source | source="approved_local_input" | 默认值 |
| D008 | boundary标记 | 任意有效输入 | boundary.challengeCreated=false, sessionIssued=false | 边界标记 |
| D009 | notes传递 | notes=["n1", "n2"] | 返回notes数组 | 附加字段 |
| D010 | 异步generator | 传入异步generator | 应await并返回结果 | 异步支持 |

#### 5.2.2 StoryRefineSkill

| 用例编号 | 用例名称 | 输入 | 预期结果 | 验证点 |
|---------|---------|------|---------|--------|
| RF001 | 正常润色 | storyDraft={title, content}, goals=["g1"] | refinedDraft.content包含原内容和goals | 润色逻辑 |
| RF002 | 缺失storyDraft | 无storyDraft | 抛出错误 | 必填校验 |
| RF003 | 无效storyDraft | storyDraft="string" | 抛出错误 | 类型校验 |
| RF004 | 自定义refiner | 传入自定义refiner | 应调用自定义refiner | 依赖注入 |
| RF005 | source限制 | source="template_only" | 抛出错误（Refine不允许template_only） | 枚举限制 |
| RF006 | boundary标记 | 任意有效输入 | boundary.protectedObjectRead=false | 边界标记 |

#### 5.2.3 别名导出验证

| 用例编号 | 用例名称 | 验证点 |
|---------|---------|--------|
| AL001 | StoryDraftAssistSkill === StoryDraftSkill | 别名一致性 |
| AL002 | StoryRefineAssistSkill === StoryRefineSkill | 别名一致性 |

---

### 5.3 第三包 — 远程网关 Skill（P0）

#### 5.3.1 StoryLockRemoteGateway

| 用例编号 | 用例名称 | 前置条件 | 输入 | 预期结果 | 验证点 |
|---------|---------|---------|------|---------|--------|
| G001 | requestStoryRead | transport已注入 | 有效payload | 调用transport，参数结构正确 | 请求包装 |
| G002 | requestStoryWrite | transport已注入 | 有效payload | 调用transport，参数结构正确 | 请求包装 |
| G003 | requestChallengeSign | transport已注入 | 有效payload | 调用transport，参数结构正确 | 请求包装 |
| G004 | queryStoryMetadata | transport已注入 | 有效payload | 调用transport，参数结构正确 | 请求包装 |
| G005 | 缺失transport | - | 构造时无transport | 抛出错误 | 依赖校验 |
| G006 | transport非函数 | - | transport="string" | 抛出错误 | 类型校验 |
| G007 | 缺失requestId | - | payload无requestId | 抛出错误 | 必填校验 |
| G008 | 缺失nonce | - | payload无nonce | 抛出错误 | 必填校验 |
| G009 | 过期expiry | - | expiry=过去时间 | 抛出错误，REQUEST_EXPIRED | 时间校验 |
| G010 | 无效expiry | - | expiry="string" | 抛出错误 | 类型校验 |
| G011 | requestStoryRead默认policyHints | - | 最小payload | policyHints包含redactionPreferred=true | 默认值 |
| G012 | requestStoryWrite默认policyHints | - | 最小payload | policyHints包含writeReason | 默认值 |
| G013 | requestChallengeSign默认policyHints | - | 最小payload | policyHints包含minAccessLevel=L4 | 默认值 |
| G014 | algorithm白名单 | - | algorithm="invalid" | 抛出错误 | 算法白名单 |
| G015 | algorithm支持ed25519 | - | algorithm="ed25519" | 成功 | 算法支持 |
| G016 | algorithm支持secp256k1 | - | algorithm="secp256k1" | 成功 | 算法支持 |
| G017 | invoke透传 | transport返回自定义结果 | 任意请求 | 返回transport的结果 | 透传验证 |

#### 5.3.2 DelegatedChallengeSignSkill

| 用例编号 | 用例名称 | 输入 | 预期结果 | 验证点 |
|---------|---------|------|---------|--------|
| DS001 | 正常委托签名 | 有效参数 | 调用gateway.requestChallengeSign | 委托链路 |
| DS002 | 无效algorithm | algorithm="rsa" | 抛出错误 | 算法校验透传 |
| DS003 | skillId | - | 返回'delegated_challenge_sign' | 标识一致性 |

---

### 5.4 跨包集成测试（P0）

#### 5.4.1 第三包→第二包读取链路

| 用例编号 | 用例名称 | 操作序列 | 预期结果 |
|---------|---------|---------|---------|
| I001 | 完整读取链路 | Gateway.requestStoryRead → transport → StoryReadAccessSkill.run | 最终返回结构化响应，status=success |
| I002 | 完整写回链路 | Gateway.requestStoryWrite → transport → StoryWriteAccessSkill.run | 最终返回结构化响应，status=success |
| I003 | 完整签名链路 | Gateway.requestChallengeSign → transport → 本地签名执行 | 最终返回签名结果 |
| I004 | 链路错误传播 | Gateway请求 → transport → 第二包抛出错误 | 错误结构完整传播到Gateway调用方 |
| I005 | 越权调用阻断 | 第三包尝试直接调用第一包 | 应失败或不存在此路径 | 边界隔离 |

#### 5.4.2 第二包→第一包处理链路

| 用例编号 | 用例名称 | 操作序列 | 预期结果 |
|---------|---------|---------|---------|
| I006 | 读取后处理 | StoryReadAccessSkill读取 → 将结果交给StoryDraftSkill处理 | 第一包能正常处理第二包交付的内容 |
| I007 | 处理后写回 | StoryRefineSkill处理 → 将结果交给StoryWriteAccessSkill写回 | 写回成功，版本递增 |

---

### 5.5 旧迁移包 — Skill Engine（P1）

#### 5.5.1 authorization-skills.js

| 用例编号 | 用例名称 | 验证点 |
|---------|---------|--------|
| M001 | LoginAuthorizationSkill.run | challenge创建、答案提交、session解析、字段填充 |
| M002 | LocalPasswordFillSkill.run | 委托LoginAuthorizationSkill、返回mode=local_password_fill |
| M003 | SigningAuthorizationSkill.run | 签名授权上下文、密钥读取 |
| M004 | ChallengeSigningAuthorizationSkill.run | challenge+签名完整流程、zeroize清零 |
| M005 | zeroizeBytes有效性 | 清零后Uint8Array全为0 |
| M006 | cloneSecretBytes | 返回独立副本，修改不影响原值 |

#### 5.5.2 story-assist.js

| 用例编号 | 用例名称 | 验证点 |
|---------|---------|--------|
| M007 | StoryDraftAssistSkill.run | 调用注入的generator |
| M008 | StoryRefineAssistSkill.run | 调用注入的refiner |

#### 5.5.3 strength-review.js

| 用例编号 | 用例名称 | 验证点 |
|---------|---------|--------|
| M009 | StrengthReviewSkill.run | 返回profile、questionSetReady、recommendedActions |
| M010 | 空questions | 抛出ValidationError |

---

### 5.6 Schema 契约测试（P1）

#### 5.6.1 JSON Schema 校验

| 用例编号 | 用例名称 | Schema文件 | 验证点 |
|---------|---------|-----------|--------|
| SC001 | story-read-input有效 | story-read-input.schema.json | 有效输入通过校验 |
| SC002 | story-read-input缺失必填 | story-read-input.schema.json | 缺失identityId应失败 |
| SC003 | story-write-input有效 | story-write-input.schema.json | 有效输入通过校验 |
| SC004 | story-write-input缺失content | story-write-input.schema.json | 缺失content应失败 |
| SC005 | remote-gateway-request有效 | remote-gateway-request.schema.json | 有效请求通过校验 |
| SC006 | remote-gateway-request无效capability | remote-gateway-request.schema.json | capability不在枚举中应失败 |
| SC007 | remote-gateway-response有效 | remote-gateway-response.schema.json | 有效响应通过校验 |
| SC008 | story-draft-input有效 | story-draft-input.schema.json | 有效输入通过校验 |
| SC009 | story-refine-input有效 | story-refine-input.schema.json | 有效输入通过校验 |
| SC010 | challenge-sign-input有效 | challenge-sign-input.schema.json | 有效输入通过校验 |
| SC011 | password-fill-input有效 | password-fill-input.schema.json | 有效输入通过校验 |

#### 5.6.2 字段映射一致性

| 用例编号 | 用例名称 | 验证点 |
|---------|---------|--------|
| FM001 | 代码字段与Schema字段一致 | 代码中使用的字段名与Schema定义一致 |
| FM002 | 代码字段与文档契约一致 | 代码响应字段与三包接口契约.md一致 |
| FM003 | 错误码与文档一致 | 代码使用的SLG错误码与本地Agent网关设计.md一致 |

---

### 5.7 安全专项测试（P0）

#### 5.7.1 挑战绕过攻击

| 用例编号 | 攻击场景 | 输入 | 预期防御结果 |
|---------|---------|------|------------|
| SEC001 | 空答案绕过 | answers=[] | 拒绝，code=SLG-003 |
| SEC002 | 随机答案猜测 | answers=[随机字符串] | 拒绝，累计失败计数 |
| SEC003 | 超长答案溢出 | answer=10MB字符串 | 拒绝或截断，不崩溃 |
| SEC004 | 特殊字符注入 | answer="'; DROP TABLE--" | 拒绝，无SQL注入风险 |
| SEC005 | 并发暴力破解 | 1000并发错误答案 | 锁定触发，后续请求拒绝 |
| SEC006 | 跨identity攻击 | 用identityB的答案尝试identityA | 拒绝，identity隔离 |

#### 5.7.2 会话劫持攻击

| 用例编号 | 攻击场景 | 输入 | 预期防御结果 |
|---------|---------|------|------------|
| SEC007 | 伪造sessionId | 随机sessionId | 拒绝，SESSION_INVALID |
| SEC008 | 过期session复用 | 已过期的sessionId | 拒绝，SESSION_EXPIRED |
| SEC009 | 跨identity session | identityA的session用于identityB | 拒绝，身份隔离 |
| SEC010 | 预算耗尽后复用 | 已耗尽预算的session | 拒绝，BUDGET_EXHAUSTED |

#### 5.7.3 重放攻击

| 用例编号 | 攻击场景 | 输入 | 预期防御结果 |
|---------|---------|------|------------|
| SEC011 | 相同requestId重放 | 完全相同的请求再发一次 | 拒绝，DUPLICATE_REQUEST |
| SEC012 | 相同nonce重放 | 相同nonce不同requestId | 拒绝，NONCE_REPLAY_DETECTED |
| SEC013 | 过期请求重放 | expiry已过的请求 | 拒绝，REQUEST_EXPIRED |
| SEC014 | 未来时间请求 | expiry=未来10年 | 应拒绝或限制最大TTL |

#### 5.7.4 越权访问攻击

| 用例编号 | 攻击场景 | 输入 | 预期防御结果 |
|---------|---------|------|------------|
| SEC015 | 第三包直接访问第一包 | 尝试直接调用StoryDraftSkill | 无此路径或拒绝 |
| SEC016 | 第二包暴露内部接口 | 尝试调用createChallenge | 不在白名单中 |
| SEC017 | 读取非授权对象 | session绑定story-001，尝试读story-002 | 拒绝，SCOPE_INSUFFICIENT |
| SEC018 | 写操作伪装成读 | 用read session尝试write | 拒绝，scope不匹配 |

#### 5.7.5 信息泄露攻击

| 用例编号 | 攻击场景 | 输入 | 预期防御结果 |
|---------|---------|------|------------|
| SEC019 | 错误信息泄露 | 触发各种错误 | error.message不包含challenge细节或秘密 |
| SEC020 | 脱敏绕过 | 请求redactionLevel=none | 仅在策略允许时生效，否则强制partial |
| SEC021 | 审计Meta泄露 | 成功请求 | auditMeta不包含敏感实现细节 |
| SEC022 | 堆栈跟踪泄露 | 触发异常 | 生产环境不暴露内部堆栈 |

---

## 六、测试实现规范

### 6.1 目录结构

```
E:\2026OPC大赛\skill\src\
├── storylock-local-story-access-skill/
│   ├── tests/
│   │   ├── unit/
│   │   │   ├── story-read-access.test.js
│   │   │   ├── story-write-access.test.js
│   │   │   ├── state-machine.test.js
│   │   │   └── failure-count.test.js
│   │   └── fixtures/
│   │       └── factory.js
├── storylock-local-story-processing-skill/
│   ├── tests/
│   │   ├── unit/
│   │   │   ├── story-draft.test.js
│   │   │   └── story-refine.test.js
│   │   └── fixtures/
│   │       └── factory.js
├── storylock-remote-gateway-skill/
│   │   ├── tests/
│   │   │   ├── unit/
│   │   │   │   ├── gateway.test.js
│   │   │   │   └── delegated-sign.test.js
│   │   │   └── fixtures/
│   │   │       └── factory.js
├── storylock-skill-engine/
│   ├── tests/
│   │   ├── unit/
│   │   │   ├── authorization-skills.test.js
│   │   │   ├── story-assist.test.js
│   │   │   └── strength-review.test.js
│   │   └── fixtures/
│   │       └── factory.js
└── tests/
    ├── integration/
    │   ├── cross-package-read.test.js
    │   ├── cross-package-write.test.js
    │   └── cross-package-sign.test.js
    ├── security/
    │   ├── challenge-bypass.test.js
    │   ├── session-hijack.test.js
    │   ├── replay-attack.test.js
    │   └── privilege-escalation.test.js
    ├── contract/
    │   ├── schema-validation.test.js
    │   └── field-mapping.test.js
    └── fixtures/
        └── shared-factory.js
```

### 6.2 测试文件模板

```javascript
// tests/unit/story-read-access.test.js
import { describe, it, beforeEach } from 'node:test';
import assert from 'node:assert';
import { StoryReadAccessSkill } from '../../index.js';
import { makeTestIdentity, makeTestStoryObject, makeTestRequestEnvelope } from '../fixtures/factory.js';

describe('StoryReadAccessSkill', () => {
  let skill;
  let mockHost;

  beforeEach(() => {
    mockHost = createMockHost(); // 内存Mock
    skill = new StoryReadAccessSkill({ host: mockHost });
  });

  it('R001: 正常读取成功', async () => {
    const input = {
      ...makeTestIdentity(),
      ...makeTestStoryObject(),
      ...makeTestRequestEnvelope(),
      answers: makeTestChallengeAnswers().answers,
    };
    const result = await skill.run(input);
    assert.strictEqual(result.status, 'success');
    assert.ok(result.result.storyObject);
    assert.strictEqual(result.redactionLevel, 'partial');
  });

  it('R005: 错误答案应失败', async () => {
    const input = {
      ...makeTestIdentity(),
      ...makeTestStoryObject(),
      ...makeTestRequestEnvelope(),
      answers: [{ questionId: 'q-001', answer: 'wrong answer' }],
    };
    const result = await skill.run(input);
    assert.strictEqual(result.status, 'error');
    assert.strictEqual(result.error.code, 'SLG-003');
    assert.strictEqual(result.error.retryable, true);
  });

  // ... 更多用例
});
```

### 6.3 Mock Host 规范

```javascript
// tests/fixtures/mock-host.js
export function createMockHost(overrides = {}) {
  const challenges = new Map();
  const sessions = new Map();
  const objects = new Map();
  const requestStore = new Set();
  const nonceStore = new Set();
  const failureCounts = new Map(); // identityId -> { count, windowStart }

  return {
    createChallenge(identityId, scope) {
      const challengeId = `chl-${Date.now()}`;
      challenges.set(challengeId, {
        challengeId, identityId, scope,
        status: 'challenge_created',
        failureCount: 0,
        maxRetryCount: 3,
        lockUntil: 0,
        createdAt: Date.now(),
        expiresAt: Date.now() + 5 * 60 * 1000,
      });
      return challenges.get(challengeId);
    },

    submitChallengeAnswers(identityId, challengeId, answers) {
      const challenge = challenges.get(challengeId);
      if (!challenge) throw new Error('CHALLENGE_NOT_FOUND');
      if (challenge.identityId !== identityId) throw new Error('IDENTITY_MISMATCH');
      if (challenge.lockUntil > Date.now()) {
        return { approved: false, retryAfter: Math.ceil((challenge.lockUntil - Date.now()) / 1000) };
      }

      // 模拟验证：检查答案是否匹配测试预期
      const expectedAnswers = overrides.expectedAnswers ?? [];
      const matched = answers.filter(a => 
        expectedAnswers.some(e => e.questionId === a.questionId && e.answer === a.answer)
      ).length;
      const threshold = overrides.threshold ?? 6;

      if (matched >= threshold) {
        challenge.status = 'verified';
        challenge.failureCount = 0;
        return { approved: true, challenge };
      }

      challenge.failureCount += 1;
      if (challenge.failureCount >= challenge.maxRetryCount) {
        challenge.status = 'locked';
        challenge.lockUntil = Date.now() + 15 * 60 * 1000;
      } else {
        challenge.status = 'failed';
      }
      return { approved: false, challenge };
    },

    issueSession(identityId, challenge, scope, resourceScope, budgets) {
      const sessionId = `ses-${Date.now()}`;
      sessions.set(sessionId, {
        sessionId, challengeId: challenge.challengeId, identityId,
        scope, resourceScope,
        readBudget: budgets.readBudget ?? 1,
        writeBudget: budgets.writeBudget ?? 0,
        issuedAt: Date.now(),
        expiresAt: Date.now() + (budgets.ttlMs ?? 3 * 60 * 1000),
        status: 'active',
        sessionType: budgets.sessionType ?? 'one_shot',
      });
      return sessions.get(sessionId);
    },

    readStoryObjectWithBudget(identityId, sessionId, storyObjectId) {
      const session = sessions.get(sessionId);
      if (!session || session.identityId !== identityId) throw new Error('SESSION_INVALID');
      if (session.status !== 'active') throw new Error('SESSION_INVALID');
      if (session.expiresAt <= Date.now()) throw new Error('SESSION_EXPIRED');
      if (session.readBudget <= 0) throw new Error('BUDGET_EXHAUSTED');
      if (!session.resourceScope.includes(storyObjectId)) throw new Error('SCOPE_INSUFFICIENT');

      session.readBudget -= 1;
      if (session.readBudget <= 0) session.status = 'session_expired';

      const obj = objects.get(storyObjectId);
      if (!obj) throw new Error('OBJECT_NOT_FOUND');
      return { session, storyObject: obj };
    },

    writeStoryObject(identityId, sessionId, storyObjectId, content) {
      const session = sessions.get(sessionId);
      if (!session || session.identityId !== identityId) throw new Error('SESSION_INVALID');
      if (session.writeBudget <= 0) throw new Error('BUDGET_EXHAUSTED');

      const existing = objects.get(storyObjectId);
      const next = {
        storyObjectId,
        ...content,
        version: (existing?.version ?? 0) + 1,
        sensitivity: existing?.sensitivity ?? 'private',
      };
      objects.set(storyObjectId, next);
      session.writeBudget -= 1;
      return next;
    },

    ensureReplaySafe(requestId, nonce, expiry) {
      if (requestStore.has(requestId)) throw Object.assign(new Error('DUPLICATE_REQUEST'), { code: 'SLG-009' });
      if (nonceStore.has(nonce)) throw Object.assign(new Error('NONCE_REPLAY_DETECTED'), { code: 'SLG-010' });
      requestStore.add(requestId);
      nonceStore.add(nonce);
    },

    // 测试辅助方法
    seedObject(obj) { objects.set(obj.storyObjectId, obj); },
    getChallenge(id) { return challenges.get(id); },
    getSession(id) { return sessions.get(id); },
    ...overrides,
  };
}
```

---

## 七、测试执行计划

### 7.1 执行顺序

| 阶段 | 测试集 | 预计耗时 | 通过标准 |
|------|--------|---------|---------|
| 1 | 第二包单元测试 | 5分钟 | 全部通过 |
| 2 | 第一包单元测试 | 3分钟 | 全部通过 |
| 3 | 第三包单元测试 | 3分钟 | 全部通过 |
| 4 | 集成测试 | 5分钟 | 全部通过 |
| 5 | 安全专项测试 | 5分钟 | 全部通过（攻击均被防御） |
| 6 | Schema契约测试 | 3分钟 | 全部通过 |
| 7 | 旧迁移包测试 | 5分钟 | 全部通过 |

### 7.2 CI/CD 集成

```yaml
# .github/workflows/test.yml（建议）
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [18, 20, 22]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
      - run: npm ci
      - run: npm run test:unit
      - run: npm run test:integration
      - run: npm run test:security
      - run: npm run test:contract
```

### 7.3 npm scripts 建议

```json
{
  "scripts": {
    "test": "npm run test:unit && npm run test:integration && npm run test:security && npm run test:contract",
    "test:unit": "node --test src/*/tests/unit/*.test.js",
    "test:integration": "node --test tests/integration/*.test.js",
    "test:security": "node --test tests/security/*.test.js",
    "test:contract": "node --test tests/contract/*.test.js",
    "test:coverage": "c8 node --test src/*/tests/unit/*.test.js tests/**/*.test.js"
  }
}
```

---

## 八、测试通过标准

### 8.1 功能测试通过标准

1. 所有 P0 用例 100% 通过
2. 所有 P1 用例 >= 90% 通过（允许旧迁移包存在已知限制）
3. 无未处理的 Promise 拒绝
4. 无内存泄漏（通过 `--detect-leaks` 或手动检查）

### 8.2 安全测试通过标准

1. 所有攻击场景（SEC001-SEC022）均被正确防御
2. 错误响应不泄露敏感信息（challenge细节、secret内容、堆栈跟踪）
3. 失败计数正确累计，锁定策略有效触发
4. 预算扣减原子性验证通过

### 8.3 契约一致性通过标准

1. 所有 JSON Schema 文件可被 Ajv 正确加载
2. 有效输入 100% 通过 Schema 校验
3. 无效输入 100% 被 Schema 校验拒绝
4. 代码字段名与 Schema 字段名 100% 一致
5. 错误码与文档定义 100% 一致

---

## 九、风险与应对

| 风险 | 影响 | 应对策略 |
|------|------|---------|
| 测试数据工厂复杂度高 | 测试编写效率低 | 先实现核心factory，逐步扩展 |
| Mock Host 与真实Host行为偏差 | 测试通过但生产失败 | 集成测试使用真实SQLite Host |
| 安全测试覆盖不足 | 遗漏攻击向量 | 参考OWASP Top 10定期补充用例 |
| 旧迁移包测试价值低 | 投入产出比差 | P1优先级，仅做基础冒烟测试 |
| 并发测试不稳定 | 偶发失败 | 使用SQLite事务或单线程串行测试 |

---

## 十、附录

### 10.1 测试用例与评审意见映射

| 评审风险 | 对应测试用例 | 验证方式 |
|---------|------------|---------|
| R1 答案验证恒为true | R004, R005, SEC001-SEC006 | 错误答案必须被拒绝 |
| R2 无加密存储 | 集成测试使用真实SQLite | 验证持久化存在 |
| R5 失败计数按challengeId | FC001-FC005 | 验证按identityId累计 |
| R6 无并发控制 | R020 | 并行提交验证 |
| R7 无脱敏实现 | R012-R014, SEC020 | 验证脱敏级别生效 |
| R8 裸Error | 所有错误用例 | 验证返回SLG结构化错误 |
| R12 无清理策略 | 长期运行测试 | 验证内存/数据库不无限增长 |

### 10.2 依赖安装

```bash
# 基础测试（Node.js 18+ 内置）
# 无需额外安装

# Schema 校验
npm install --save-dev ajv

# 覆盖率（可选）
npm install --save-dev c8

# 性能测试（可选）
npm install --save-dev autocannon
```

---

*测试方案制定时间: 2026-06-16 21:31 GMT+8*
*制定人: 代码安全审计师*
