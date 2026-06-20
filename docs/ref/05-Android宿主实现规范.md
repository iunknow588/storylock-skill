# StoryLock Android 宿主实现规范

日期：2026-06-20

本文档用于说明当前仓库中 Android 宿主原型的真实边界、已落地能力和仍待补齐的正式交付缺口。

## 1. 当前真实状态

当前仓库已经包含一套可联调、可继续推进的 Android 真机宿主原型工程：

`src/host/android-host/`

当前已经落地的能力包括：

1. 本地 HTTP 宿主入口：`GET /health`、`POST /execute`
2. Android Keystore 保护下的本地 `SecretStore`
3. Android Keystore 非对称签名原型路径
4. 多格 challenge 输入与 BiometricPrompt 确认链路
5. challenge 失败计数与临时锁定语义
6. 基于本地资源文件加载的题集原型路径
7. deep link 绑定、注册、relay 轮询与回传原型代码路径

当前不能表述为：

1. Android 宿主已经是正式上架 App
2. Android 本地签名已经完成正式多算法生产闭环
3. Android challenge 已经与第二层正式实现完全等价
4. Android 正式 release 构建、签名、验签与发布闭环已经完成

## 2. Android 宿主职责

Android 宿主当前负责：

1. 持有本地题集可用状态
2. 加载本地 challenge 题集资源
3. 承载 challenge、BiometricPrompt、本地确认与本地执行
4. 执行本地签名或密码填充
5. 向第三层返回最小必要结果

Android 宿主当前不负责：

1. 直接向远程暴露第二层内部方法
2. 回传题集答案、私钥原文或 `SecretStore` 明文
3. 替第三层完成最终对外脱敏

## 3. 最小接口契约

当前最小宿主接口只有两个：

1. `GET /health`
2. `POST /execute`

### 3.1 `GET /health`

当前关键字段包括：

1. `status`
2. `layer1.mode`
3. `layer1.questionSetReady`
4. `layer1.strongestBasicChallengeBits`
5. `layer2.identityId`
6. `layer2.questionSetVersion`
7. `layer2.normalizationVersion`
8. `layer2.activeQuestionCount`
9. `stats.requestCount`

### 3.2 `POST /execute`

当前主线支持：

1. `requestSignature`
2. `requestPasswordFill`

## 4. 当前 challenge 与确认模型

当前 Android 原型已经不再是单题确认。

当前行为：

1. `requestSignature` 默认要求高强度 challenge
2. 高强度 challenge 当前需要 9 格答案
3. `requestPasswordFill` 默认要求中强度 challenge
4. 中强度 challenge 当前需要 6 格答案
5. challenge 通过后仍需继续经过 BiometricPrompt / 设备凭据确认
6. challenge 连续失败会累计失败计数，并在达到阈值后进入临时锁定
7. challenge 数据当前来自本地资源文件，而不是直接硬编码在运行时代码里

当前失败路径已区分：

1. `challenge_cancelled`
2. `challenge_failed`
3. `challenge_locked`
4. `biometric_unavailable`
5. `biometric_cancelled`
6. `biometric_failed`
7. `host_unavailable`

## 5. 当前题集来源

当前 Android 原型题集来源已经推进到：

1. 本地 asset 文件 `storylock-question-set.json`
2. 宿主启动时读取并构造 challenge 题集
3. 题集带有 `identityId`、`questionSetVersion` 与 `normalizationVersion`
4. 宿主启动期会对题集执行 fail-fast 检查，包括：
   - `identityId`、`questionSetVersion`、`normalizationVersion` 不能为空
   - active 题目 `questionId`、`promptText`、`answer` 不能为空
   - active `questionId` 不能重复
   - active 题目数量至少为 24
   - 题集 `identityId` 必须与宿主 `hostConfig.identityId` 一致
5. 仓库侧可通过以下命令做静态校验：

```powershell
node scripts/android/validate_android_question_set.mjs
```

当前仍未完成：

1. 与第二层正式 SQLite 题集持久化完全打通
2. 真机端题集导入、轮换和清理闭环
3. 正式 Android 题集数据管理工具链

## 6. 当前签名模型

当前 Android 原型签名路径已经从 demo HMAC 前进到：

1. 基于 Android Keystore 的本地 EC 密钥对
2. 本地执行签名
3. 返回签名结果和必要公钥元数据
4. 不再返回 demo `keyMaterial`

当前仍需继续补齐：

1. 请求算法和本地实际算法的正式策略映射
2. 更强的用户认证策略与签名键绑定
3. 更正式的审计与失败处理闭环

## 7. 当前可验证范围

当前仓库已经可以验证：

1. Web API 网关入口
2. Android 宿主 mock 注册
3. relay / poll / respond 闭环
4. APK 下载元数据入口
5. 共享密钥透传
6. 宿主执行结果脱敏返回
7. Android 原型工程中的本地宿主、Keystore、challenge、BiometricPrompt 与注册 / relay 代码路径
8. Android asset 题集的仓库侧结构校验与宿主启动期 fail-fast 检查

当前仓库尚未完成：

1. Android 正式交付级 UI 与完整产品化处理
2. 正式移动端 release 构建、签名、验签和发布闭环
3. 请求算法与本地签名策略最终对齐
4. 与第二层正式模型完全一致的题集持久化、失败窗口与撤销细节
5. Android 正式题集导入、轮换与持久化闭环

## 8. 建议演示口径

建议对外表述为：

> 当前仓库已经验证“云端第三层网关 + Android 本地宿主接口模型”的闭环，且仓库内已经存在 Android 真机宿主原型工程、Keystore、本地题集资源加载、challenge 与 BiometricPrompt 相关代码路径；但它仍处于可联调、可验证、未完成正式交付闭环的阶段。
