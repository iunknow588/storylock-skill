# FunctionGraph 部署模式分析：单用户 vs 多用户共享

## 一、核心结论

**推荐方案：所有用户共享一个 FunctionGraph 部署**

理由：
1. **Serverless 天然多租户**：函数实例按请求隔离，同一函数可同时服务多个用户
2. **成本优化**：OPC 早期阶段，单函数部署降低运维复杂度与费用
3. **用户隔离通过逻辑实现**：通过 `identityId` + 设备绑定实现用户级隔离，而非物理隔离

---

## 二、两种部署模式对比

### 2.1 单用户单部署（不推荐）

```
用户A -> FunctionGraph-instance-A -> 本地宿主A
用户B -> FunctionGraph-instance-B -> 本地宿主B
用户C -> FunctionGraph-instance-C -> 本地宿主C
```

| 维度 | 评估 |
|------|------|
| **隔离性** | 物理隔离，安全性最高 |
| **成本** | 每个用户独立函数，冷启动频繁，费用高 |
| **运维复杂度** | 需为每个用户创建/更新/删除函数，自动化成本高 |
| **扩展性** | 用户增长时，函数数量线性增长，难以管理 |
| **适用场景** | 企业级 SaaS 多租户、合规要求极高的金融场景 |

**问题**：
- OPC 早期阶段，用户量小，单用户单函数成本过高
- 函数数量受华为云配额限制（默认单账户 200 个函数）
- 更新迭代时需批量更新所有函数，运维噩梦

### 2.2 多用户共享部署（推荐）

```
用户A -> │                    │ -> 本地宿主A
用户B -> │  FunctionGraph   │ -> 本地宿主B
用户C -> │  (共享函数)      │ -> 本地宿主C
         │                    │
         └────────────────────┘
              APIG 统一入口
```

| 维度 | 评估 |
|------|------|
| **隔离性** | 逻辑隔离（identityId + 设备绑定），安全性足够 |
| **成本** | 单函数多实例，按需计费，成本最优 |
| **运维复杂度** | 单一函数，更新迭代一次生效 |
| **扩展性** | 用户增长仅增加并发实例，函数数量不变 |
| **适用场景** | OPC 早期、C 端产品、成本敏感场景 |

**优势**：
- 华为云 FunctionGraph 自动扩缩容，实例间隔离
- 通过代码层面实现用户隔离，安全可控
- 符合 StoryLock "信任技术原理" 的设计哲学

---

## 三、多用户共享部署的安全隔离机制

### 3.1 请求级隔离

```javascript
// FunctionGraph 入口函数
exports.handler = async (event, context) => {
  const { identityId, deviceId, requestType, payload } = parseRequest(event);
  
  // 1. 身份验证：校验 EIP-712 签名
  const isValid = await verifyEIP712Signature(payload, identityId);
  if (!isValid) return { error: 'SLG-001', message: '身份验证失败' };
  
  // 2. 设备绑定：校验设备指纹
  const deviceBound = await verifyDeviceBinding(identityId, deviceId);
  if (!deviceBound) return { error: 'SLG-002', message: '设备未绑定' };
  
  // 3. 请求路由：根据 identityId 路由至对应本地宿主
  const localHost = await getLocalHostEndpoint(identityId, deviceId);
  
  // 4. 脱敏处理：转发前脱敏敏感字段
  const redactedPayload = redact(payload, 'partial');
  
  // 5. 转发请求：HTTPS 至本地宿主
  const result = await forwardToLocalHost(localHost, redactedPayload);
  
  // 6. 结果脱敏：返回前再次脱敏
  return redact(result, 'full');
};
```

### 3.2 数据隔离策略

| 层级 | 隔离机制 | 说明 |
|------|---------|------|
| **函数实例** | 华为云自动隔离 | 每个请求独立实例，内存不共享 |
| **身份认证** | EIP-712 签名 + identityId | 确保请求来自合法用户 |
| **设备绑定** | 设备指纹 + 白名单 | 限制用户可授权的本地设备 |
| **会话隔离** | 短时 session 令牌 | 3-30 分钟有效期，过期失效 |
| **审计隔离** | identityId 分区存储 | 审计日志按用户分区，防止越权查询 |

### 3.3 防重放与限流

```javascript
// 双层防护：APIG 限流 + 函数内防重放
const rateLimitConfig = {
  identityLevel: '100/min',  // 单用户每分钟 100 请求
  globalLevel: '10000/min',   // 全函数每分钟 10000 请求
  burstAllowance: 10          // 突发允许 10 请求
};

const replayProtection = {
  requestIdWindow: '24h',     // requestId 24 小时去重
  nonceWindow: '5min',        // nonce 5 分钟有效期
  expiryDefault: '5min'       // 请求 5 分钟过期
};
```

---

## 四、华为云 FunctionGraph 多租户特性

### 4.1 实例隔离机制

| 特性 | 说明 |
|------|------|
| **容器隔离** | 每个函数实例运行在独立容器，内存/CPU 隔离 |
| **临时存储** | `/tmp` 目录临时存储，实例销毁后清空 |
| **环境变量** | 函数级别配置，实例间不共享 |
| **并发控制** | 单实例仅处理单个请求，避免并发安全问题 |

### 4.2 自动扩缩容

```
请求量低：1-2 个实例运行
请求量高：自动扩展至 N 个实例
请求结束：空闲实例自动销毁（保留 1-2 个预热实例）
```

**优势**：
- 无需预置资源，按实际调用计费
- 突发流量自动承载，无需人工干预
- 实例销毁后内存清零，无数据残留风险

---

## 五、部署架构建议

### 5.1 推荐架构

```
┌─────────────────────────────────────────┐
│           APIG（API 网关）               │
│  - 统一入口：api.yian.cdao.online        │
│  - 限流：100/min/identity               │
│  - HTTPS 终止                            │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│      FunctionGraph（共享函数）           │
│  - 运行时：Node.js 22                    │
│  - 内存：512MB                           │
│  - 超时：30s                             │
│  - 并发：100（自动扩展）                 │
│  - 环境变量：KMS 加密存储                │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│         本地宿主（多用户）                │
│  - Windows / Android / Linux             │
│  - 每个用户独立本地设备                   │
│  - 设备绑定认证                          │
└─────────────────────────────────────────┘
```

### 5.2 函数配置

```yaml
# functiongraph.yaml
function:
  name: storylock-gateway
  runtime: nodejs22
  handler: index.handler
  memory: 512
  timeout: 30
  concurrency: 100
  
environment:
  KMS_KEY_ID: ${KMS_KEY_ID}
  EIP712_DOMAIN: ${EIP712_DOMAIN}
  OBS_BUCKET: ${OBS_BUCKET}
  
triggers:
  - type: api
    path: /api/storylock-gateway
    method: POST
    auth: eip712
```

### 5.3 用户隔离实现

```javascript
// 用户注册时：生成唯一 identityId，绑定设备指纹
async function registerUser(walletAddress, deviceFingerprint) {
  const identityId = deriveIdentityId(walletAddress);
  const deviceId = hashDeviceFingerprint(deviceFingerprint);
  
  // 存储至 CloudTable（用户-设备映射表）
  await cloudTable.put({
    partitionKey: identityId,
    sortKey: deviceId,
    walletAddress,
    registeredAt: Date.now(),
    status: 'active'
  });
  
  return { identityId, deviceId };
}

// 请求处理时：校验用户身份与设备绑定
async function verifyRequest(identityId, deviceId, signature) {
  // 1. 校验 EIP-712 签名
  const isValid = verifyEIP712(signature, identityId);
  if (!isValid) throw new Error('SLG-001');
  
  // 2. 校验设备绑定
  const binding = await cloudTable.get({ partitionKey: identityId, sortKey: deviceId });
  if (!binding) throw new Error('SLG-002');
  
  // 3. 校验设备状态
  if (binding.status !== 'active') throw new Error('SLG-003');
  
  return true;
}
```

---

## 六、成本估算（OPC 早期阶段）

### 6.1 单用户单函数 vs 共享函数

| 场景 | 单用户单函数 | 共享函数 |
|------|-------------|---------|
| **月活跃用户** | 100 | 100 |
| **函数数量** | 100 | 1 |
| **月调用次数** | 10,000 | 10,000 |
| **月费用估算** | ¥200-500 | ¥20-50 |
| **运维人力** | 2-4 小时/周 | 0.5 小时/周 |

*注：费用估算基于华为云 FunctionGraph 标准计费，实际以官网为准*

### 6.2 共享函数成本构成

| 项目 | 单价 | 月用量 | 费用 |
|------|------|--------|------|
| 调用次数 | ¥0.02/万次 | 10,000 次 | ¥0.02 |
| 执行时间 | ¥0.0001/GB-s | 500 GB-s | ¥0.05 |
| 出网流量 | ¥0.8/GB | 10 GB | ¥8.00 |
| **合计** | - | - | **¥8.07** |

---

## 七、结论

| 维度 | 推荐方案 |
|------|---------|
| **部署模式** | 所有用户共享一个 FunctionGraph 函数 |
| **隔离机制** | 逻辑隔离（identityId + 设备绑定 + EIP-712 签名） |
| **安全策略** | APIG 限流 + 函数内防重放 + 审计日志分区 |
| **成本优化** | 单函数多实例，按需计费 |
| **扩展路径** | 用户增长至 10,000+ 时，考虑函数分片（按地域/用户组） |

**一句话总结**：FunctionGraph 的 Serverless 特性天然支持多租户共享部署，通过密码学验证（EIP-712）和设备绑定实现用户级隔离，既保证安全又降低成本，符合 StoryLock "信任技术原理"的设计哲学。

---

*FunctionGraph 部署模式分析完成。如需进一步展开（如函数代码示例、KMS 配置、CloudTable 表结构设计），告诉我。*
