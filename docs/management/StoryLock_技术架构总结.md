# StoryLock 技术架构总结

## 一、总体架构

StoryLock 采用三层技术架构，将敏感操作从云端拉回用户本地设备，同时保留 Agent 协作的便利性。

`
┌─────────────────────────────────────────────────────────────┐
│  第三层：远程网关（Remote Gateway）                           │
│  职责：统一对外入口，请求包装，结果脱敏                        │
│  接口：requestSignature、requestPasswordFill                 │
│  部署：Vercel / 云平台                                       │
└──────────────────┬──────────────────────────────────────────┘
                   │  HTTPS / 安全通道
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  第二层：本地授权（Local Authorization）                      │
│  职责：对象强度判断、九宫格验证、短时授权、防重放、审计        │
│  组件：ObjectStrengthPolicySkill、GridChallengeSkill、       │
│        LocalAuthorizationSkill、LocalRevocationSkill          │
│  存储：SQLite（challenge_state、session_store、audit_log）   │
│  运行：本地设备（Android / Windows / Linux）                   │
└──────────────────┬──────────────────────────────────────────┘
                   │  本地进程调用
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  第一层：故事处理（Story Processing）                        │
│  职责：故事草稿、润色、题集强度评估、对象加密                  │
│  组件：StoryDraftSkill、StoryRefineSkill、StrengthReviewSkill │
│  存储：本地文件系统（加密故事文件、题集主档）                  │
│  运行：本地设备                                               │
└─────────────────────────────────────────────────────────────┘
`

---

## 二、各层详细设计

### 2.1 第一层：故事处理（Story Processing）

| 项目 | 说明 |
|------|------|
| **职责** | 处理故事草稿生成、润色、题集强度评估；管理加密故事文件 |
| **核心组件** | StoryDraftSkill、StoryRefineSkill、StrengthReviewSkill |
| **边界约束** | 不读取受保护对象；不签发授权；不使用签名密钥或密码凭据 |
| **输入** | 用户提供的原始故事材料、润色目标、强度评估请求 |
| **输出** | 故事草稿、润色后文本、题集强度评分、24题题集候选 |
| **存储** | 题集主档（question-set-master.json）、故事模板（story-template.json） |
| **加密** | 故事文件使用 AES-256-GCM 加密，密钥由 masterSalt 派生 |

**关键流程**：
1. 用户提供故事素材（时间、地点、人物、主题、冲突、情节、结论）
2. StoryDraftSkill 生成 24 节点故事框架
3. StoryRefineSkill 润色和整理故事文本
4. StrengthReviewSkill 评估题集强度，确保满足最低挑战位数要求
5. 题集存入本地加密文件，等待第二层调用

---

### 2.2 第二层：本地授权（Local Authorization）

| 项目 | 说明 |
|------|------|
| **职责** | 对象强度判断、九宫格验证、短时授权、防重放、失败锁定、审计日志 |
| **核心组件** | ObjectStrengthPolicySkill、GridChallengeSkill、LocalAuthorizationSkill、LocalRevocationSkill |
| **边界约束** | 不提供故事对象读写接口；不返回长期秘密；不直接执行远程请求 |
| **输入** | 对象类型、动作类型、请求上下文、用户答案 |
| **输出** | 挑战结果、短时 session、审计元数据 |
| **存储** | SQLite 8 张表：challenge_state、session_store、request_store、nonce_store、failure_window、answer_digest_set、question_set_item、audit_log |

**对象强度策略（L1-L5）**：

| 级别 | 名称 | 阈值 | 适用场景 |
|------|------|------|---------|
| L1 | basic | 6/24 | 公开故事、低敏感对象 |
| L2 | low | 6/24 | 一般对象读取 |
| L3 | medium | 12/24 | 密码填充、API 调用 |
| L4 | high | 22/24 | 签名授权、高敏感操作 |
| L5 | critical | Host 决定 | 根密钥重建、灾难恢复 |

**Challenge 状态机**：
`
created -> verified -> active -> expired
   ↓         ↓
failed -> locked -> created (窗口结束后)
`

**Session 模型**：
| 类型 | TTL | 读写预算 | 适用场景 |
|------|-----|---------|---------|
| one_shot | 3 分钟 | 1/0 | 单次签名/密码填充 |
| short_session | 10 分钟 | 5/2 | 批量操作 |
| batch_session | 30 分钟 | 20/5 | 工作流编排 |
| privileged_session | 5 分钟 | 无限制 | 根级重建（需 L5） |

**防重放机制**：
- requestId：唯一请求标识，去重窗口 24 小时
- nonce：随机数，必须递增且未使用过
- expiry：请求过期时间，默认 5 分钟
- 重放检测：SLG-013（REPLAY_DETECTED）

**失败锁定策略**：
- 失败计数按 identityId + 24 小时窗口统计
- 最大失败次数：3 次
- 锁定窗口：15 分钟
- 成功验证后重置连续失败计数
- 锁定期间返回 retryAfter，不泄露哪一题错误

---

### 2.3 第三层：远程网关（Remote Gateway）

| 项目 | 说明 |
|------|------|
| **职责** | 接收远程 Agent 请求，包装请求，调用本地执行器，返回脱敏结果 |
| **核心接口** | requestSignature、requestPasswordFill |
| **边界约束** | 不持有私钥/密码/答案；不读取故事内容；不透传敏感字段 |
| **输入** | EIP-712 结构化请求、远程 Agent 身份、请求上下文 |
| **输出** | 脱敏后的签名结果/填充结果、审计元数据 |
| **部署** | Vercel Serverless / 自托管 Node.js |

**EIP-712 请求结构**：
`json
{
  "capability": "requestSignature",
  "identityId": "storylock-demo-user",
  "keyId": "wallet/main/private_key",
  "algorithm": "ed25519",
  "payload": "message to sign",
  "eip712": {
    "domain": {
      "name": "StoryLock",
      "version": "1",
      "chainId": 1,
      "verifyingContract": "0x..."
    },
    "message": {
      "action": "sign",
      "resource": "wallet/ethereum/main",
      "scope": "transaction",
      "expiry": 1234567890,
      "nonce": "1234567890",
      "requestedBy": "agent-xxx",
      "delegationContext": "DeFi swap approval"
    }
  }
}
`

**脱敏规范**：
| 级别 | 规则 | 适用场景 |
|------|------|---------|
| none | 不脱敏 | 第一层->第二层内部 |
| partial | 遮盖高敏字段，保留结构 | 第二层->第三层 |
| full | 仅返回状态/摘要，清空值 | 第三层->外部 Agent |

**高敏字段清单**：
- 私钥、密码、mnemonic、keystore_password
- 九宫格答案原文、challenge 正确答案
- session 密钥、长期 secret token
- 故事原文、对象明文内容

---

## 三、密钥层次与加密体系

### 3.1 密钥派生链

`
masterSalt（随机 256-bit，存平台 SecretStore）
    ↓ HKDF-SHA256
rootKey（故事根密钥，不直接暴露）
    ↓ HKDF-SHA256 + identitySalt
workKey（工作密钥，按对象类型派生）
    ↓ HKDF-SHA256 + objectNonce
objectKey（对象密钥，用于 AES-256-GCM 加密）
`

### 3.2 加密规范

| 项目 | 规范 |
|------|------|
| 对称加密 | AES-256-GCM |
| 密钥派生 | HKDF-SHA256 |
| 摘要 | HMAC-SHA256 |
| GCM nonce | 96-bit 随机数，每次独立生成，同一密钥下不得复用 |
| 身份盐值 | identitySalt = HMAC-SHA256(rootKey, identityId) |
| 答案摘要 | answerDigest = HMAC-SHA256(identitySalt, normalizedAnswer) |

### 3.3 平台密钥存储适配

| 平台 | 推荐方案 | 存储对象 |
|------|---------|---------|
| Windows | DPAPI / Credential Manager | masterSalt、签名根密钥句柄 |
| macOS | Keychain | masterSalt、签名根密钥别名 |
| Linux | Secret Service / libsecret | masterSalt、签名根密钥 |
| Android | Android Keystore | masterSalt、签名私钥 |
| 开发模式 | MemorySecretStore（显式启用） | 仅用于测试，生产环境拒绝 |

---

## 四、数据持久化设计

### 4.1 SQLite 表结构

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| challenge_state | 单次 challenge 状态 | challengeId、status、manifest_json、required_threshold、expiry |
| session_store | 短时授权会话 | sessionId、identityId、scope、budget、ttl、status |
| request_store | 请求去重 | requestId、nonce、expiry、capability |
| nonce_store | nonce 去重 | nonce、usedAt、identityId |
| failure_window | 失败计数窗口 | identityId、windowStart、count、lockedUntil |
| answer_digest_set | 答案派生摘要 | questionId、answerDigest、identitySalt、version |
| question_set_item | 题集主档 | questionId、text、difficulty、status、version |
| audit_log | 审计日志 | timestamp、identityId、capability、objectType、redactionLevel、errorCode |

### 4.2 存储分层

| 层级 | 存储介质 | 内容 |
|------|---------|------|
| 长期敏感材料 | 平台 SecretStore | masterSalt、签名根密钥句柄 |
| 运行态数据 | SQLite | challenge、session、nonce、审计日志 |
| 普通文件 | 本地文件系统 | 加密故事文件、题集密文、配置模板 |
| 内存 | 运行时堆 | 答案原文、session 密钥、解密后的对象明文（使用后立即清零） |

---

## 五、宿主与部署架构

### 5.1 本地宿主（Host）

| 平台 | 实现 | 状态 |
|------|------|------|
| Android | Kotlin + Android Keystore | 骨架代码，待真机验证 |
| Windows | Rust + DPAPI + tiny_http | 原型完成，已实现注册/relay/execute 闭环 |
| Linux | Rust + Secret Service（规划中） | 未启动 |
| Web/WASM | Rust -> WASM | 已完成，用于浏览器端加密演示 |

**宿主最小契约**：
- GET /health：健康检查
- POST /execute：执行签名/密码填充/其他敏感操作
- POST /relay：接收来自第三层的请求，转发至第二层

### 5.2 远程部署

| 组件 | 部署方式 | 说明 |
|------|---------|------|
| 易安网站（yian-web） | Vercel 静态站点 | 下载入口、绑定引导、状态展示 |
| 第三层网关（web-api） | Vercel Serverless | requestSignature、requestPasswordFill 接口 |
| 文档站点 | Vercel / 自托管 | 设计文档、API 参考、使用指南 |

---

## 六、安全机制总结

### 6.1 核心安全原则

1. **本地优先**：敏感操作始终在本地设备执行，远端不接触密钥
2. **最小权限**：短时 session 按预算授权，过期自动失效
3. **可审计**：所有操作记录审计日志，支持事后追溯
4. **可脱敏**：递归脱敏确保高敏字段不出本地边界
5. **防重放**：requestId + nonce + expiry 三重机制
6. **防暴力**：失败窗口 + 锁定策略，不泄露哪一题错误

### 6.2 风险触发与响应

| 风险场景 | 触发条件 | 响应措施 |
|---------|---------|---------|
| 密钥泄露 | 检测到异常签名/密码使用模式 | 立即撤销所有 session，强制重新 challenge |
| 设备丢失 | 用户报告或长期无活动 | 远程撤销设备绑定，新设备需重新初始化 |
| 题集泄露 | 答案组合被枚举 | 渐进式淘汰旧题，扩展新题，不更换故事 |
| 根密钥重建 | 用户主动要求或泄露确认 | 重建 masterSalt、rootKey、identitySalt、题集主档、对象封装层 |

---

## 七、技术栈清单

| 层级 | 技术 | 用途 |
|------|------|------|
| 运行时 | Node.js 22 | Skill 脚本、网关服务 |
| 运行时 | Rust | Windows 宿主、WASM 模块 |
| 运行时 | Kotlin | Android 宿主 |
| 加密 | AES-256-GCM | 对象加密 |
| 加密 | HKDF-SHA256 | 密钥派生 |
| 加密 | HMAC-SHA256 | 摘要、答案验证 |
| 存储 | SQLite | 本地状态持久化 |
| 存储 | 平台 SecretStore | 长期敏感材料 |
| 部署 | Vercel | 网站、网关、文档 |
| 协议 | EIP-712 | 结构化签名请求 |
| 协议 | HTTPS | 层间通信 |

---

## 八、与外部系统关系

| 系统 | 关系 | 边界 |
|------|------|------|
| OpenClaw / Pharos | 生态共建方 | 第三层接口对接，不暴露本地密钥 |
| Vercel | 基础设施供应商 | 仅部署非敏感层，不接触密钥逻辑 |
| Ethereum / Solana | 区块链网络 | 仅接收签名结果，私钥本地持有 |
| 第三方 Agent | 请求发起方 | 经 EIP-712 请求，接收脱敏结果 |

---

*技术架构总结完成。如需针对某一具体模块（如 WASM 模块、Android 宿主、EIP-712 实现）展开详细设计，可进一步分析。*

