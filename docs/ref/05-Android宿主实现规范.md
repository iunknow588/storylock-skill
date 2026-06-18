# StoryLock Android 宿主实现规范

## 1. 目标

本文档用于把当前 `Android 宿主 mock` 收敛为后续真实 Android 宿主实现的最小规范。

当前目标不是定义完整 App 产品，而是定义“第一层和第二层如何在 Android 侧承载，并与第三层云端入口对接”。

## 2. 当前架构定位

建议采用以下部署关系：

1. 第三层 `storylock-remote-gateway-skill` 通过 Vercel 风格入口对外暴露。
2. 第一层 `storylock-local-story-processing-skill` 在 Android 本地执行。
3. 第二层 `storylock-local-story-access-skill` 在 Android 本地执行。
4. Android 宿主对第三层只暴露最小接口：`GET /health`、`POST /execute`。

## 3. Android 宿主必须承载的能力

### 3.1 第一层

至少需要：

1. 题集强度检查。
2. 本地故事题集准备状态检查。

对应当前代码能力：

1. `StrengthReviewSkill`

### 3.2 第二层

至少需要：

1. 对象强度策略判定。
2. 九宫格 challenge 创建。
3. 本地答案校验。
4. 短时授权签发。
5. 防重放、失败锁定、审计落库。
6. 撤销接口预留。

对应当前代码能力：

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`
4. `LocalRevocationSkill`

## 4. Android 宿主最小接口

### 4.1 `GET /health`

用途：

1. 给第三层检查 Android 宿主是否在线。
2. 返回第一层题集就绪状态。
3. 返回第二层 active question set 的概要状态。

当前 schema：

`src/storylock-remote-gateway-skill/assets/schemas/android-host-health.schema.json`

### 4.2 `POST /execute`

用途：

1. 接收第三层标准请求。
2. 在 Android 本地完成本地授权链路。
3. 返回标准 remote gateway response。

当前输入输出 schema：

1. `src/storylock-remote-gateway-skill/assets/schemas/remote-gateway-request.schema.json`
2. `src/storylock-remote-gateway-skill/assets/schemas/remote-gateway-response.schema.json`

## 5. Android 侧本地安全要求

### 5.1 密钥与长期秘密

真实 Android 宿主应优先接入：

1. Android Keystore
2. 必要时结合 EncryptedSharedPreferences 或等价安全存储保存非密钥元数据

当前必须满足：

1. 不把长期私钥明文写入普通文件。
2. 不把 challenge answers 明文长期持久化。
3. 不把 masterSalt 明文暴露给第三层或远端。

### 5.2 challenge answers

真实 Android 宿主必须遵守当前挑战答案存储策略：

1. 原始答案只允许短时存在于内存或受控输入流程。
2. 持久层只保存摘要、challenge manifest 和 session 元数据。
3. challenge 完成后立即清理运行态原始答案。

### 5.3 生物识别与本地确认

建议真实 Android 宿主增加以下本地确认能力：

1. BiometricPrompt 二次确认。
2. 应用前后台切换时自动失效待授权状态。
3. 高强度签名请求可要求“题集验证 + 生物识别”双条件。

注意：  
生物识别是 Android 宿主的本地加固机制，不应替代题集 challenge 的主契约，除非未来文档另行升级定义。

## 6. 第三层与 Android 宿主的连接要求

### 6.1 第三层云端入口

当前第三层入口：

1. `api/storylock-gateway.mjs`
2. `src/storylock-remote-gateway-skill/vercel-handler.js`
3. `GET /download/android-host`

第三层职责：

1. 接收远程请求。
2. 保持主接口 allowlist。
3. 转发标准 envelope 给 Android 宿主。
4. 对返回结果做脱敏。
5. 向外提供 Android 本地宿主的下载地址与第二层连接元数据。

### 6.2 宿主鉴权

当前最小方案：

1. `x-storylock-shared-secret`

后续真实实现建议增加：

1. 设备标识 `deviceId`
2. App 实例标识 `appInstanceId`
3. 宿主注册记录
4. 可选设备证明或签名挑战

## 7. 当前仓库已经验证的部分

当前仓库已经验证：

1. 第三层 Vercel 风格入口可运行。
2. Android 宿主 mock 可承载第一层与第二层能力。
3. `selftest:vercel-android` 能跑通端到端请求链路。

当前仓库尚未提供：

1. 完整 Android App 工程。
2. Android Keystore 实现。
3. BiometricPrompt 集成。
4. 真机网络暴露方案。

## 8. 推荐对外口径

建议使用：

> 当前仓库已经验证“第三层云端入口 + Android 本地宿主”的最小链路，其中第三层负责请求包装与脱敏，第一层和第二层保留在本地宿主；真实 Android App 与 Android Keystore 集成属于下一阶段实现工作。
