# StoryLock 中 KMS 具体存储信息清单

## 一、核心结论

**KMS 在 StoryLock 中存储三类信息：验证材料、配置加密密钥、审计日志加密密钥。**

| 类别 | 具体信息 | 敏感度 | 是否可公开 |
|------|---------|--------|-----------|
| **验证材料** | EIP-712 Domain 公钥、X.509 证书、合约地址 | 低 | 可公开 |
| **配置加密密钥** | 数据库密码加密密钥、API 密钥加密密钥、环境变量主密钥 | 中 | 不可公开 |
| **审计日志加密密钥** | 审计日志加密密钥、访问记录加密密钥 | 中 | 不可公开 |

**一句话：KMS 存储"用来验证别人的材料"和"用来保护配置的钥匙"，不存储"用来保护自己的核心秘密"。**

---

## 二、KMS 存储信息详细清单

### 2.1 验证材料类（可公开，低敏感）

#### 1. EIP-712 Domain 公钥

存储位置：KMS 非对称密钥对（RSA/ECC）
用途：验证 Agent 提交的 EIP-712 结构化请求签名

格式示例：
{
  "keyId": "alias/storylock-eip712",
  "keySpec": "ECC_NIST_P256",
  "keyUsage": "SIGN_VERIFY",
  "publicKey": "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----"
}

为什么可以公开：
- 公钥天然可公开，用于验证而非保密
- 即使泄露，攻击者也无法伪造签名（无对应私钥）
- 符合"不信任任何第三方"原则：任何人都可以验证，但无法伪造

使用场景：
// FunctionGraph 验证 Agent 请求签名
const publicKey = await kms.getPublicKey({ keyId: "alias/storylock-eip712" });
const isValid = verifyEIP712Signature(request.body, publicKey);
if (!isValid) return { error: "SLG-001", message: "身份验证失败" };

#### 2. X.509 证书

存储位置：KMS 证书管理模块
用途：HTTPS 通信、代码签名

格式示例：
{
  "certificateId": "storylock-https-cert",
  "subject": "CN=yian.cdao.online",
  "issuer": "CN=Huawei Cloud CA",
  "validFrom": "2026-06-01T00:00:00Z",
  "validTo": "2027-06-01T00:00:00Z",
  "pem": "-----BEGIN CERTIFICATE-----\nMIIDXTCCAkWgAwIBAgIJAJC1HiIAZAiU...\n-----END CERTIFICATE-----"
}

为什么可以公开：
- 证书本身就是公开分发的
- 浏览器/客户端需要证书建立 TLS 连接

#### 3. 链上合约地址（可选）

存储位置：KMS 自定义标签/元数据
用途：验证链上锚点交易

格式示例：
{
  "keyId": "alias/storylock-contract",
  "tags": {
    "contractAddress": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "chainId": "1",
    "network": "ethereum"
  }
}

为什么可以公开：
- 合约地址在链上公开可见
- 仅用于验证交易是否存在，不涉及密钥

---

### 2.2 配置加密密钥类（不可公开，中敏感）

#### 4. 数据库密码加密密钥（DEK）

存储位置：KMS 对称密钥（AES-256）
用途：加密 FunctionGraph 环境变量中的数据库密码

格式示例：
{
  "keyId": "alias/storylock-db-dek",
  "keySpec": "AES_256",
  "keyUsage": "ENCRYPT_DECRYPT",
  "description": "用于加密数据库连接密码"
}

实际使用：
# 环境变量文件（加密后）
DATABASE_URL: "AQICAHg6S9A4lP8LQ3Q8K1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7=="
# 明文："postgresql://user:MyP@ssw0rd123@host:5432/storylock"

KMS_KEY_ID: "alias/storylock-db-dek"

为什么不可公开：
- 泄露 DEK 意味着可以解密数据库密码
- 但 DEK 本身不直接存储在代码中，而是通过 IAM 权限控制访问

#### 5. API 密钥加密密钥（DEK）

存储位置：KMS 对称密钥（AES-256）
用途：加密第三方 API 密钥（如邮件服务、短信服务）

格式示例：
{
  "keyId": "alias/storylock-api-dek",
  "keySpec": "AES_256",
  "keyUsage": "ENCRYPT_DECRYPT",
  "description": "用于加密第三方 API 密钥"
}

实际使用：
# 环境变量文件（加密后）
EMAIL_API_KEY: "AQICAHg6S9A4lP8LQ3Q8K1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7=="
# 明文："sk_live_abc123def456ghi789"

SMS_API_KEY: "AQICAHg6S9A4lP8LQ3Q8K1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7=="
# 明文："sms_key_xyz789uvw456"

#### 6. 环境变量主密钥（Envelope Encryption）

存储位置：KMS 对称密钥（AES-256）
用途：加密所有环境变量的"主密钥"

格式示例：
{
  "keyId": "alias/storylock-env-master",
  "keySpec": "AES_256",
  "keyUsage": "ENCRYPT_DECRYPT",
  "description": "环境变量信封加密主密钥"
}

信封加密机制：
1. 生成随机数据密钥（DEK）
2. 用 DEK 加密环境变量明文
3. 用 KMS 主密钥加密 DEK
4. 存储：加密后的 DEK + 加密后的环境变量

解密时：
1. 用 KMS 主密钥解密 DEK
2. 用 DEK 解密环境变量

优势：
- 主密钥不直接加密大量数据，性能更好
- 可以轮换 DEK 而不影响主密钥
- 符合信封加密最佳实践

---

### 2.3 审计日志加密密钥类（不可公开，中敏感）

#### 7. 审计日志加密密钥

存储位置：KMS 对称密钥（AES-256）
用途：加密审计日志中的敏感字段（identityId、操作类型、时间戳）

格式示例：
{
  "keyId": "alias/storylock-audit-dek",
  "keySpec": "AES_256",
  "keyUsage": "ENCRYPT_DECRYPT",
  "description": "用于加密审计日志"
}

实际使用：
// 敏感审计日志写入前加密
const auditLog = {
  identityId: "user_abc123",
  operation: "requestSignature",
  objectId: "story_private_xyz",
  timestamp: "2026-06-28T09:23:00Z",
  result: "success"
};

// 加密敏感字段
const encryptedLog = await kms.encrypt({
  keyId: "alias/storylock-audit-dek",
  plaintext: JSON.stringify(auditLog)
});

// 写入 CloudTable
await cloudTable.put({
  partitionKey: "audit",
  sortKey: "2026-06-28T09:23:00Z",
  logData: encryptedLog.ciphertext
});

为什么需要加密：
- 审计日志包含用户行为轨迹，属于敏感数据
- 即使数据库泄露，攻击者也无法读取日志内容
- 需要审计时，通过 KMS 解密，记录解密操作

#### 8. 访问记录加密密钥

存储位置：KMS 对称密钥（AES-256）
用途：加密访问记录（IP、User-Agent、请求路径）

格式示例：
{
  "keyId": "alias/storylock-access-dek",
  "keySpec": "AES_256",
  "keyUsage": "ENCRYPT_DECRYPT",
  "description": "用于加密访问记录"
}

---

## 三、KMS 不存储的信息（明确边界）

### 3.1 第二层核心密钥（绝对不上云）

| 密钥 | 存储位置 | 原因 |
|------|---------|------|
| masterSalt | 本地 SecretStore（Windows DPAPI / Android Keystore） | 故事根盐值，泄露意味着所有密钥可重新派生 |
| rootKey | 本地 SecretStore | 故事根密钥，泄露意味着故事内容可解密 |
| workKey | 本地内存（派生后使用，不持久化） | 工作密钥，按对象类型派生 |
| objectKey | 本地内存（派生后使用，不持久化） | 对象密钥，直接用于 AES-256-GCM 加密 |
| 私钥（Ed25519/ECDSA） | 本地 SecretStore | 身份签名私钥，泄露意味着身份可被伪造 |

### 3.2 第二层会话密钥（不持久化）

| 密钥 | 存储位置 | 生命周期 |
|------|---------|---------|
| sessionKey | 本地内存 | 3-30 分钟，过期自动销毁 |
| challengeToken | 本地内存 | 九宫格挑战期间有效，挑战完成即销毁 |

### 3.3 用户故事内容（绝对不上云）

| 数据 | 存储位置 | 原因 |
|------|---------|------|
| 故事原文 | 本地加密文件（AES-256-GCM） | 用户隐私，绝对不上云 |
| 题集答案 | 本地加密数据库（SQLite + AES-256-GCM） | 验证凭据，绝对不上云 |
| 九宫格题目 | 本地内存（动态抽取） | 挑战内容，不持久化 |

---

## 四、KMS 存储信息汇总表

### 4.1 按敏感度分类

| 敏感度 | 信息类型 | 具体示例 | 数量 |
|--------|---------|---------|------|
| 低（可公开） | 验证材料 | EIP-712 公钥、X.509 证书、合约地址 | 3-5 个 |
| 中（不可公开） | 配置加密密钥 | 数据库 DEK、API DEK、环境变量主密钥 | 3-5 个 |
| 中（不可公开） | 审计加密密钥 | 审计日志 DEK、访问记录 DEK | 2-3 个 |
| 高（绝对不上云） | 第二层核心密钥 | masterSalt、rootKey、workKey、私钥 | 0 个（明确不上云） |

### 4.2 按密钥类型分类

| 密钥类型 | 数量 | 用途 | 轮换频率 |
|---------|------|------|---------|
| 非对称密钥对（ECC/RSA） | 1-2 个 | EIP-712 签名验证、HTTPS 证书 | 每年 |
| 对称密钥（AES-256） | 5-8 个 | 配置加密、审计日志加密 | 每季度 |
| 自定义标签/元数据 | 2-3 个 | 合约地址、配置标识 | 按需 |

---

## 五、KMS 存储信息的实际示例

### 5.1 完整 KMS 配置示例

{
  "keys": [
    {
      "keyId": "alias/storylock-eip712",
      "keySpec": "ECC_NIST_P256",
      "keyUsage": "SIGN_VERIFY",
      "description": "EIP-712 请求签名验证公钥",
      "tags": {
        "purpose": "agent-request-verification",
        "layer": "third",
        "sensitivity": "low"
      }
    },
    {
      "keyId": "alias/storylock-https-cert",
      "keySpec": "RSA_2048",
      "keyUsage": "SIGN_VERIFY",
      "description": "HTTPS 通信证书",
      "tags": {
        "purpose": "tls-termination",
        "layer": "third",
        "sensitivity": "low"
      }
    },
    {
      "keyId": "alias/storylock-db-dek",
      "keySpec": "AES_256",
      "keyUsage": "ENCRYPT_DECRYPT",
      "description": "数据库密码加密密钥",
      "tags": {
        "purpose": "config-encryption",
        "layer": "third",
        "sensitivity": "medium"
      }
    },
    {
      "keyId": "alias/storylock-api-dek",
      "keySpec": "AES_256",
      "keyUsage": "ENCRYPT_DECRYPT",
      "description": "API 密钥加密密钥",
      "tags": {
        "purpose": "config-encryption",
        "layer": "third",
        "sensitivity": "medium"
      }
    },
    {
      "keyId": "alias/storylock-env-master",
      "keySpec": "AES_256",
      "keyUsage": "ENCRYPT_DECRYPT",
      "description": "环境变量信封加密主密钥",
      "tags": {
        "purpose": "envelope-encryption",
        "layer": "third",
        "sensitivity": "medium"
      }
    },
    {
      "keyId": "alias/storylock-audit-dek",
      "keySpec": "AES_256",
      "keyUsage": "ENCRYPT_DECRYPT",
      "description": "审计日志加密密钥",
      "tags": {
        "purpose": "audit-encryption",
        "layer": "third",
        "sensitivity": "medium"
      }
    }
  ]
}

### 5.2 实际使用流程示例

// FunctionGraph 启动时：从 KMS 解密环境变量
async function initializeConfig() {
  // 1. 获取数据库密码
  const encryptedDbPassword = process.env.DATABASE_PASSWORD; // "AQICAHg..."
  const decryptedDbPassword = await kms.decrypt({
    keyId: "alias/storylock-db-dek",
    ciphertext: encryptedDbPassword
  });
  
  // 2. 获取 API 密钥
  const encryptedApiKey = process.env.API_KEY; // "AQICAHg..."
  const decryptedApiKey = await kms.decrypt({
    keyId: "alias/storylock-api-dek",
    ciphertext: encryptedApiKey
  });
  
  // 3. 获取 EIP-712 公钥（用于验证 Agent 请求）
  const publicKey = await kms.getPublicKey({
    keyId: "alias/storylock-eip712"
  });
  
  return {
    dbPassword: decryptedDbPassword,
    apiKey: decryptedApiKey,
    eip712PublicKey: publicKey
  };
}

// 处理 Agent 请求时：验证签名
async function handleAgentRequest(request) {
  const { eip712PublicKey } = await initializeConfig();
  
  // 验证 EIP-712 签名
  const isValid = verifyEIP712Signature(request.body, eip712PublicKey);
  if (!isValid) {
    return { error: "SLG-001", message: "身份验证失败" };
  }
  
  // 继续处理...
}

// 记录审计日志时：加密敏感字段
async function logAuditEvent(event) {
  const encryptedLog = await kms.encrypt({
    keyId: "alias/storylock-audit-dek",
    plaintext: JSON.stringify(event)
  });
  
  await cloudTable.put({
    partitionKey: "audit",
    sortKey: event.timestamp,
    logData: encryptedLog.ciphertext
  });
}

---

## 六、结论

| 问题 | 答案 |
|------|------|
| KMS 存储什么？ | 验证材料（公钥/证书）、配置加密密钥（DEK）、审计日志加密密钥 |
| KMS 不存储什么？ | 第二层核心密钥（masterSalt/rootKey/workKey/私钥）、用户故事内容、题集答案 |
| 验证材料为什么可公开？ | 公钥/证书天然可公开，用于验证而非保密 |
| 配置密钥为什么不可公开？ | 泄露可解密数据库密码/API 密钥，但本身受 IAM 权限保护 |
| 第二层密钥为什么不上云？ | "本地优先"设计哲学：核心秘密永远本地，不信任任何第三方 |
| KMS 存储多少密钥？ | 8-10 个（3-5 个验证材料 + 5-8 个加密密钥） |
| 月费用多少？ | 3-6 元（密钥托管 1 元/个 + API 调用 0.3 元/万次） |

一句话总结：KMS 存储"验证别人的材料"和"保护配置的钥匙"，共 8-10 个密钥，月费用 3-6 元，不存储任何第二层核心密钥或用户故事内容。