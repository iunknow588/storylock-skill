# StoryLock 本地 Agent 网关设计

## 目的

本文档定义远程 Agent 如何通过受控接口访问本地 Agent 或本地 Host 运行时。

核心目标：

1. 第三方 Skill 不直接持有私钥
2. 第三方 Skill 不直接读取本地 secret store
3. 远程 Agent 通过包装接口委托本地能力执行
4. 本地 Agent 返回最小化、结构化结果

## 定位

本地 Agent 网关不是普通业务 Skill，而是本地敏感能力的受控代理层。

它的角色是：

1. 接收远程请求
2. 校验 capability 与 scope
3. 转发到本地执行能力
4. 返回最小结果
5. 阻止私钥、challenge answers、长期会话能力外泄

## 设计原则

1. 远程侧只见能力，不见秘密实现
2. 本地侧只暴露白名单接口
3. 返回结果优先于返回能力
4. 长期能力不应转授给远程侧
5. 高敏感对象默认一次性访问

## 第三层接口暴露边界

当前阶段必须明确区分：

1. **第三层可对外暴露的包装接口**
2. **第二层内部能力接口**

第三层负责“包装、校验、转发、收敛结果”，不负责直接暴露第二层全部能力。

### 当前阶段允许对外暴露

1. `requestStoryRead`
2. `requestStoryWrite`
3. `requestChallengeSign`
4. `requestStrengthReview`
5. `requestCapabilityStatus`
6. `queryStoryMetadata`

### 当前阶段不得对外暴露

1. `createChallenge`
2. `submitChallengeAnswers`
3. `activateSession`
4. `readSecretObject`
5. `deriveKey`
6. `consumeReadBudget`
7. `expireSession`

原因：

1. 这些接口属于第二层安全边界内部实现
2. 一旦直接暴露，远程侧将有机会拼接出越权访问链路
3. 第三方 Skill 应只看到“请求能力”，不应看到“安全执行细节”

## 网关角色划分

### 远程 Agent

负责：

1. 识别用户请求
2. 选择要调用的本地能力
3. 组装结构化请求
4. 接收本地执行结果
5. 包装结果给第三方 Skill 或最终用户

不负责：

1. 持有原始私钥
2. 保存 challenge answers
3. 直接读取本地 secret object

### 本地 Agent / 本地 Host

负责：

1. challenge 创建
2. answer 提交
3. secret object 读取
4. signing material 使用
5. story 私密输入本地处理
6. 本地策略判断与审计

## 推荐开放的网关接口

建议远程 Agent 只能通过以下白名单接口访问本地能力。

### 1. `requestStoryRead`

用途：

1. 请求本地读取受保护故事对象
2. 仅返回已授权、已脱敏或本地策略允许的内容

本地执行能力：

1. 第二层 session / challenge 校验
2. 本地对象读取与脱敏流程

返回：

1. `status`
2. `result`
3. `executionLocation`
4. `redactionLevel`
5. `retentionGranted`
6. `error`

### 2. `requestStoryWrite`

用途：

1. 请求本地对故事对象执行受控写回
2. 写回前必须经过第二层 challenge / session / scope 校验

本地执行能力：

1. 第二层写权限校验
2. 第一层故事处理结果落盘

返回：

1. `status`
2. `result`
3. `executionLocation`
4. `redactionLevel`
5. `retentionGranted`
6. `error`

### 3. `requestPasswordFill`

用途：

1. 请求本地完成登录字段填充

本地执行能力：

1. `LocalPasswordFillSkill`

返回：

1. `status`
2. `result`
3. `executionLocation`
4. `redactionLevel`
5. `error`

### 4. `requestChallengeSign`

用途：

1. 请求本地完成挑战签名

本地执行能力：

1. `ChallengeSigningAuthorizationSkill`

返回：

1. `status`
2. `result`
3. `executionLocation`
4. `redactionLevel`
5. `error`

禁止返回：

1. 私钥
2. challenge answers
3. 长期 session token

### 5. `requestLocalStoryAssist`

用途：

1. 请求本地处理带私密输入的故事草稿或润色任务

本地执行能力：

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`

返回：

1. 结构化草稿结果
2. 可选脱敏摘要

### 6. `requestStrengthReview`

用途：

1. 请求本地执行题集评估

本地执行能力：

1. `StrengthReviewSkill`

说明：

1. 若题集本身被视为敏感，则应优先本地执行

### 7. `requestCapabilityStatus`

用途：

1. 查询本地能力是否可用
2. 查询本地策略要求
3. 查询是否需要更高 challenge 强度

### 8. `queryStoryMetadata`

用途：

1. 查询不直接暴露正文的低敏元信息
2. 返回对象分类、访问等级、是否需本地执行等信息

说明：

1. 该接口默认只返回 `L1` 或经策略许可的低敏元数据
2. 不返回题集答案摘要、session material、secret object 内容

## 推荐请求结构

远程 Agent 发给本地网关的请求建议统一为：

1. `requestId`
2. `capability`
3. `scope`
4. `payload`
5. `policyHints`
6. `requestedRetention`
7. `nonce`
8. `expiry`

其中：

1. `capability` 指能力名
2. `scope` 指请求范围
3. `payload` 指业务参数
4. `policyHints` 指调用方对本地策略的提示
5. `requestedRetention` 指调用方希望保留结果还是能力
6. `nonce + expiry` 用于第三层到第二层的最小防重放链路

## 推荐响应结构

本地网关返回的结构建议统一为：

1. `requestId`
2. `status`
3. `capability`
4. `executionLocation`
5. `result`
6. `redactionLevel`
7. `retentionGranted`
8. `auditMeta`
9. `error`

其中：

1. `retentionGranted` 表示本地最终批准的保留级别
2. `auditMeta` 只返回最小审计元信息，不返回敏感实现细节

## 错误码规范

建议本地网关统一使用 `SLG` 前缀的错误码。

### 错误结构

```json
{
  "errorCode": "SLG-001",
  "errorType": "CAPABILITY_NOT_AVAILABLE",
  "message": "请求的本地能力当前不可用",
  "suggestedAction": "请检查本地Agent状态或稍后重试",
  "retryable": true
}
```

### 建议错误码表

| 错误码 | 错误类型 | 含义 | 可重试 | 建议处理 |
| --- | --- | --- | --- | --- |
| `SLG-001` | `CAPABILITY_NOT_AVAILABLE` | 请求的能力当前不可用 | 是 | 检查本地 Agent 是否已启动 |
| `SLG-002` | `CHALLENGE_REQUIRED` | 当前操作要求先完成挑战 | 是 | 引导用户进入 challenge 流程 |
| `SLG-003` | `CHALLENGE_FAILED` | 挑战答案未通过验证 | 是 | 允许重新提交答案 |
| `SLG-004` | `SESSION_EXPIRED` | 会话已过期 | 否 | 重新创建 challenge 与 session |
| `SLG-005` | `SCOPE_INSUFFICIENT` | 当前 scope 不足 | 否 | 请求更高访问级别 |
| `SLG-006` | `SECRET_OR_OBJECT_NOT_FOUND` | 目标对象不存在 | 否 | 检查对象标识或资源映射 |
| `SLG-007` | `HOST_NOT_IMPLEMENTED` | 宿主接口未实现 | 否 | 检查本地 Host 实现 |
| `SLG-008` | `REDACTION_REQUIRED` | 结果必须先脱敏 | 否 | 仅返回脱敏结果 |
| `SLG-009` | `DUPLICATE_REQUEST` | 相同 `requestId` 已被处理 | 否 | 查询已有结果或更换新的 `requestId` |
| `SLG-010` | `NONCE_REPLAY_DETECTED` | 检测到已使用的 `nonce` | 否 | 重新生成结构化请求 |
| `SLG-011` | `REQUEST_EXPIRED` | 请求已超过有效时间窗 | 否 | 重新发起新的请求 |
| `SLG-012` | `SESSION_REVOKED_OR_EXPIRED` | session 已撤销或已失效 | 否 | 重新完成 challenge 流程 |

## 分层校验责任

当前阶段建议明确采用“双层校验、职责不同”的规则：

### 第三层网关负责

1. 校验 `capability` 是否在对外白名单内
2. 校验请求结构是否完整
3. 校验 `requestId / nonce / expiry` 是否满足最小格式要求
4. 对调用来源、返回格式、错误结构做统一包装

### 第二层本地访问层负责

1. 校验 challenge / session / scope
2. 校验对象访问级别与资源范围
3. 校验算法是否合法、`keyId` 是否支持目标算法
4. 执行读预算、写预算与本地策略判断

这意味着：

1. 第三层不能替代第二层安全判断
2. 第二层也不应默认信任第三层传入请求

### HTTP 状态码映射建议

| 错误码 | 推荐 HTTP 状态码 |
| --- | --- |
| `SLG-001` | `503 Service Unavailable` |
| `SLG-002` | `401 Unauthorized` |
| `SLG-003` | `403 Forbidden` |
| `SLG-004` | `401 Unauthorized` |
| `SLG-005` | `403 Forbidden` |
| `SLG-006` | `404 Not Found` |
| `SLG-007` | `501 Not Implemented` |
| `SLG-008` | `422 Unprocessable Entity` |
| `SLG-009` | `409 Conflict` |
| `SLG-010` | `409 Conflict` |
| `SLG-011` | `400 Bad Request` |
| `SLG-012` | `401 Unauthorized` |

## 性能基准环境建议

当前阶段在讨论性能目标时，建议至少注明基准环境：

1. Node.js >= 18 LTS
2. 本地 SSD 存储
3. 8GB 以上内存
4. 单机场景

说明：

1. 这些目标默认不包含大型本地 LLM 推理
2. 若接入硬件密钥设备，签名耗时目标应单独评估

## 本地签名委托定位

如果第三方 Skill 需要签名能力，推荐通过本地签名委托模式实现。

其定位不是让第三方 Skill 持有私钥，而是：

1. 第三方 Skill 提交待签名请求
2. 远程 Agent 调用本地网关
3. 本地网关调用 `requestChallengeSign`
4. 本地执行签名
5. 返回结构化签名结果

因此：

1. 远程 Agent 不是私钥持有者
2. 第三方 Skill 不是私钥持有者
3. 本地侧是签名能力执行者

## 远程可保留内容

远程侧可以保留：

1. 签名结果
2. 脱敏故事结果
3. 评估结论
4. 显式批准下的短时不透明 handle

保留前提：

1. 必须由本地侧明确批准 `retentionGranted`
2. 默认批准“结果保留”，不批准“能力保留”
3. 高敏操作默认只返回一次性结果，不返回可复用 handle

远程侧不应保留：

1. 私钥
2. signing key bytes
3. challenge answers
4. 长期 secret read 权限
5. 可无限复用的签名能力 token
6. 第二层内部 session material

## 与前两份文档的关系

本文件与以下文档形成配套关系：

1. `Skill定位与边界.md`
   - 负责定义有哪些 Skill，以及边界归属
2. `对象访问策略.md`
   - 负责定义访问强度、会话与保留规则
3. `三包接口契约.md`
   - 负责定义三个 Skill 包之间的最小请求 / 响应结构
4. 本文件
   - 负责定义远程 Agent 如何通过受控接口调用本地能力

## 性能目标建议

当前阶段建议采用保守但可测的目标值：

| 操作 | 目标 |
| --- | --- |
| `requestCapabilityStatus` | < 50ms |
| challenge 创建 | < 100ms |
| session 校验 | < 50ms |
| 本地 challenge 签名授权 | < 500ms |
| 单次故事对象读取授权 | < 200ms |

说明：

1. 以上指标默认不包含大型本地模型推理耗时
2. 若第一层启用本地 LLM，则故事处理能力应单独统计
3. 第三层远程包装只应增加很薄的一层网络与序列化开销

## 结论

正确模型不是“第三方 Skill 直接持有私钥”。

正确模型是：

1. 第三方 Skill 提出能力请求
2. 远程 Agent 作为编排与包装层
3. 本地 Agent / 本地 Host 完成敏感执行
4. 最终只返回最小化、结构化结果
