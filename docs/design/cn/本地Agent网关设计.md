# StoryLock 本地 Agent 网关设计

## 当前定位

本地 Agent 网关对应第三层 `storylock-remote-gateway-skill`。它不是普通业务 Skill，而是远程请求进入本地敏感能力前的包装、校验、转发与脱敏层。

当前对外主接口只有：

1. `requestSignature`
2. `requestPasswordFill`

旧接口 `requestStoryRead`、`requestStoryWrite`、`requestChallengeSign`、`requestCapabilityStatus`、`requestLocalStoryAssist`、`queryStoryMetadata` 已不再作为当前设计基线。

## 网关职责

第三层负责：

1. 校验请求字段。
2. 校验 `nonce` 与 `expiry` 的基本格式。
3. 生成 `StoryLockSignatureRequest` EIP-712 请求结构。
4. 通过 `transport` 或可选本地执行器转交本地 Host。
5. 对返回结果做统一脱敏。
6. 保证只暴露白名单能力。

第三层不负责：

1. 保存私钥或密码。
2. 校验九宫格答案。
3. 签发 session。
4. 直接读取 secret object。
5. 暴露第二层内部 API。

## 执行方式

当前代码支持两种执行方式。

### 方式 A：transport 包装

`StoryLockRemoteGateway({ transport })` 将标准化后的请求对象交给宿主环境。宿主环境负责把请求接入第二层或其他本地执行链路。

### 方式 B：可选本地执行器

`StoryLockRemoteGateway` 可注入：

1. `signatureExecutor`
2. `passwordFillExecutor`

此时网关仍先构造标准请求，再调用本地执行器，并对执行结果做脱敏。

## 签名请求

`requestSignature` 必填字段：

1. `requestId`
2. `nonce`
3. `expiry`
4. `identityId`
5. `keyId`
6. `algorithm`
7. `payload`

支持算法：

1. `ed25519`
2. `secp256k1`

网关生成的 EIP-712 类型为 `StoryLockSignatureRequest`，字段包括：

1. `action`
2. `resource`
3. `scope`
4. `expiry`
5. `nonce`
6. `requestedBy`
7. `delegationContext`

## 密码填充请求

`requestPasswordFill` 必填字段：

1. `requestId`
2. `nonce`
3. `expiry`
4. `identityId`
5. `credentialRef`
6. `targetOrigin`

返回给远程侧的结果不得包含明文密码。

## 脱敏规则

网关会递归替换以下字段：

1. `answers`
2. `signingKey`
3. `signingKeyBytes`
4. `secretBytes`
5. `secretValue`
6. `password`
7. `privateKey`
8. `mnemonic`
9. `seed`
10. `rawSecret`
11. `keyMaterial`

允许保留安全引用字段，例如 `secretReference`、`secretObjectId`。

## 错误码

第三层应透传或包装本地 `SLG` 错误码。当前错误码表以 `三包接口契约.md` 为准，其中 replay 冲突使用 `SLG-013`。

## 性能目标

当前阶段建议只保留可测试目标：

| 操作 | 目标 |
| --- | --- |
| 请求结构校验 | < 50ms |
| EIP-712 请求构造 | < 50ms |
| 网关脱敏 | < 50ms |
| 本地签名执行 | 由本地 signer 或硬件设备决定 |

## 后续完善

1. 增加完整三层 demo：第三层请求 -> 第二层授权 -> 本地签名执行。
2. 为 HTTP 宿主补充状态码映射。
3. 增加来源校验与本地用户确认策略。
