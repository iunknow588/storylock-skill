# Host 与 StoryLock 独立项目拆分计划

更新时间：2026-07-03

## 1. 核心结论

Yian Host 已经提供本地 Web 接口，因此从架构上已经具备和 StoryLock 独立的基础。后续拆分不应该再让 Host 读取 StoryLock 的故事、答案、vault 或对象私有配置，而应该建立一个明确的本地协作通道：

```text
远程 Agent / Gateway
        |
        v
Yian Host 本地 Web/API 服务
        |
        | 本地长连接 / IPC，只传请求与脱敏结果
        v
StoryLock Core 独立应用
```

拆分后的原则：

1. Host 是本地网关和 Web API 服务，不是 StoryLock 数据读取者。
2. StoryLock Core 是故事、答案、vault、对象策略、挑战验证的所有者。
3. Host 和 StoryLock 之间通过本地长连接或 IPC 协作。
4. Host 只接收通过、拒绝、超时、脱敏授权摘要等结果。
5. Host 不直接读取 `vault.stlk`、draft、故事摘要、问题文本、答案、密码、私钥。

所以，拆分方向是可行的，而且当前 Host 已有 Web 接口，使拆分基础比之前更清晰。

## 2. 当前代码状态

已经完成的边界收紧：

- Host 配置文件独立为 `target/debug/config/host-config.json`
- StoryLock 配置文件独立为 `target/debug/storylock/config/storylock-config.json`
- StoryLock 模板文件回到 StoryLock 包内 `templates/`
- Host 设置页不再修改 StoryLock 包配置
- Host 语言配置只保存到 Host 配置
- StoryLock 包路径、导出路径只保存到 StoryLock 配置

仍需移除的耦合：

- `src/slint_ui/dashboard.rs` 直接创建 `StoryLockCoreApp`
- Host 仍调用 `read_effective_author_draft(package_dir)` 生成和验证 StoryLock 开锁挑战
- Host 仍调用 `host_learning_plan_status(&core_package_dir)` 读取学习策略摘要
- Host 与 StoryLock UI 仍在同一个 Slint 编译入口
- `src/slint_ui/storylock/resource_export/catalog_resources.rs` 仍引用 `crate::host_runtime`

其中最需要优先处理的是：Host 读取 StoryLock draft 来做挑战。这是单体原型遗留，不符合拆分后的安全边界。

## 3. 目标架构

目标不是让 Host “能读 StoryLock 数据”，而是让 Host “能请求 StoryLock 完成授权判断”。

建议形态：

```text
apps/
  yian-windows-host/
    - 本地 HTTP API
    - Gateway relay
    - Host Dashboard
    - StoryLock client registry
    - 授权请求转发

  storylock-core-desktop/
    - StoryLock Core UI
    - 包管理
    - 故事编辑
    - 问题与答案管理
    - 受控对象策略
    - 授权挑战生成与验证

crates/
  yian-host-contract/
    - Host Web API DTO
    - relay request / response
    - health / status schema

  storylock-contract/
    - StoryLock 本地协作协议 DTO
    - 脱敏授权摘要
    - 授权请求与授权结果
    - 心跳、注册、错误码

  storylock-core-engine/
    - StoryLock 包读写
    - vault / template / learning-policy
    - challenge engine
    - object policy engine
```

## 4. Host 与 StoryLock 的连接方式

推荐采用“StoryLock 主动连接 Host”的本地长连接模式。

```text
StoryLock Core  --WebSocket / Named Pipe-->  Yian Host
```

原因：

- Host 已经有本地 Web 服务，扩展 WebSocket 最自然。
- StoryLock 主动注册，Host 不需要扫描进程或读取 StoryLock 配置。
- Host 可以在没有 StoryLock 时继续运行 Web API 和远程 relay。
- StoryLock 可以随时启动、断开、重连，不影响 Host 基础服务。

### 首选方案：WebSocket

建议新增本地接口：

```text
ws://127.0.0.1:<host_port>/storylock/session
```

StoryLock Core 启动后连接 Host，并发送注册消息：

```json
{
  "type": "storylock.register",
  "sessionId": "sl-session-001",
  "app": "storylock-core-desktop",
  "version": "0.1.0",
  "capabilities": [
    "package.status",
    "authorization.request",
    "authorization.challenge",
    "redacted.summary"
  ]
}
```

Host 收到远程请求后，不读取 StoryLock 数据，而是通过长连接转发请求：

```json
{
  "type": "authorization.request",
  "requestId": "req-001",
  "objectRef": "windows-credential-ref",
  "action": "password_fill",
  "requiredLevel": "high",
  "requester": "remote-agent"
}
```

StoryLock 完成挑战和授权判断后返回：

```json
{
  "type": "authorization.result",
  "requestId": "req-001",
  "approved": true,
  "authorizationId": "auth-001",
  "expiresInSeconds": 300,
  "redactedObject": {
    "objectRef": "windows-credential-ref",
    "secrecyLevel": "confidential",
    "accessLevel": "12/12"
  }
}
```

Host 只保存授权结果和脱敏摘要。

### 备用方案：Named Pipe

Windows 上可增加 Named Pipe 作为更强本机边界：

```text
\\.\pipe\yian-host-storylock
```

适用场景：

- 后续需要更强的本机进程身份校验
- 不希望 StoryLock 协作通道暴露为 HTTP/WebSocket
- Windows 桌面端优先，不追求跨平台统一实现

建议短期先用 WebSocket，后续再加 Named Pipe。

## 5. 本地协作协议

建议协议类型：

| 消息类型 | 方向 | 作用 |
| --- | --- | --- |
| `storylock.register` | StoryLock -> Host | 注册 StoryLock Core 能力 |
| `storylock.heartbeat` | 双向 | 保活与在线状态 |
| `storylock.status` | StoryLock -> Host | 返回脱敏状态 |
| `authorization.request` | Host -> StoryLock | 请求 StoryLock 完成授权 |
| `authorization.result` | StoryLock -> Host | 返回授权结果 |
| `authorization.cancel` | Host -> StoryLock | 请求取消授权 |
| `redacted.summary.request` | Host -> StoryLock | 请求脱敏摘要 |
| `redacted.summary.result` | StoryLock -> Host | 返回脱敏摘要 |
| `storylock.error` | 双向 | 返回协议错误 |

禁止通过协议传输：

- 故事原文
- 问题文本
- 答案
- 密码
- 私钥
- vault 原始内容
- 明文签名密钥
- 用户的完整 StoryLock 包路径，除非用户明确选择用于本机打开

允许通过协议传输：

- StoryLock 是否在线
- 包是否已加载
- 包是否需要解锁
- 授权是否通过
- 授权过期时间
- 受控对象脱敏引用
- 保密级别
- 访问等级
- 错误码和非敏感错误说明

## 6. Host 不需要读取的信息

Host 不需要：

- 故事摘要
- 故事节点
- 问题文本
- 正确答案
- 学习计划细节
- vault 文件
- 资源目录里的私密字段
- 受控对象的真实值

Host 只需要：

- StoryLock 在线状态
- 授权请求是否被批准
- 授权 id
- 授权有效期
- 脱敏对象摘要
- 可展示的最小对象清单

因此，当前 Host 中调用 `read_effective_author_draft(package_dir)` 的逻辑应该迁移到 StoryLock Core 内部。

## 7. 分阶段拆分计划

### 阶段 0：计划冻结

目标：先不做大规模代码迁移，只冻结边界和协议。

任务：

- 明确 Host 不读取 StoryLock 私有数据。
- 明确 StoryLock 通过长连接注册到 Host。
- 明确 Host Web API 继续作为远程入口和本地网关。
- 明确 StoryLock Core 是授权判断所有者。

验收：

- 本文档作为拆分基线。
- 后续代码迁移不得新增 Host 直接读取 StoryLock draft/vault 的逻辑。

### 阶段 1：建立协议 DTO

目标：先在当前项目内新增协议类型，暂不移动文件。

建议新增：

```text
src/storylock_contract/
  mod.rs
  session.rs
  authorization.rs
  redacted_summary.rs
  errors.rs
```

内容：

- `StoryLockRegister`
- `StoryLockHeartbeat`
- `StoryLockAuthorizationRequest`
- `StoryLockAuthorizationResult`
- `StoryLockRedactedSummary`
- `StoryLockProtocolError`

验收：

- DTO 不依赖 Slint。
- DTO 不依赖 Host runtime。
- DTO 不包含故事、答案、密码、私钥字段。

### 阶段 2：Host 增加 StoryLock 会话管理

目标：Host 能接受 StoryLock 长连接注册。

任务：

- 在 Host 本地 Web 服务中增加 `/storylock/session`
- 保存当前 StoryLock 连接状态
- 增加 heartbeat 与断线处理
- Host Dashboard 只显示 StoryLock 在线/离线/忙碌/授权中

验收：

- 不启动 StoryLock 时，Host 仍可正常运行。
- 启动 StoryLock 后，Host 显示 StoryLock 在线。
- StoryLock 断开后，Host 自动切回离线状态。

### 阶段 3：迁移授权挑战

目标：移除 Host 对 StoryLock draft 的读取。

当前应替换的点：

- `create_storylock_open_challenge`
- `storylock_open_expected_answers`
- Host 直接调用 `read_effective_author_draft(package_dir)`

迁移后：

- Host 收到授权请求。
- Host 通过长连接发送 `authorization.request`。
- StoryLock Core 内部生成挑战、显示挑战、验证答案。
- StoryLock Core 返回 `authorization.result`。

验收：

- Host 不再 import 或调用 `read_effective_author_draft`。
- Host 不解析 StoryLock 问题和答案。
- 授权流程仍可完成。

### 阶段 4：移除 Host 读取学习摘要

目标：Host 不再读取 `learning-policy.json`。

迁移后：

- 如果 Host 页面需要显示 StoryLock 状态，由 StoryLock 返回 `redacted.summary.result`。
- 如果 StoryLock 未连接，Host 显示“StoryLock 未连接”即可。
- Host 不再读取 learning policy 细节。

验收：

- Host 不调用 `host_learning_plan_status`。
- Host 页面只展示脱敏状态。

### 阶段 5：拆出 StoryLock Core 独立应用

目标：StoryLock Core 独立编译、独立启动。

目标目录：

```text
apps/storylock-core-desktop/
  Cargo.toml
  src/main.rs
  src/ui/
  src/session_client/
```

StoryLock Core 启动行为：

1. 加载 StoryLock 自己的配置。
2. 打开 StoryLock Core UI。
3. 尝试连接 Host `/storylock/session`。
4. 连接成功后注册能力。
5. 连接失败时仍可作为离线 StoryLock 管理工具运行。

验收：

- StoryLock Core 可单独运行。
- 没有 Host 时也能编辑和管理本地包。
- 有 Host 时自动注册，接受授权请求。

### 阶段 6：拆出 Host 独立应用

目标：Host 不编译 StoryLock UI。

目标目录：

```text
apps/yian-windows-host/
  Cargo.toml
  src/main.rs
  src/host_runtime/
  src/ui/
  src/storylock_session/
```

Host 行为：

1. 启动本地 HTTP 服务。
2. 启动远程 relay。
3. 等待 StoryLock Core 注册。
4. 收到远程敏感操作请求时，转发给 StoryLock。
5. 若 StoryLock 未在线，返回需要本地 StoryLock 在线的错误。

验收：

- Host 可独立运行。
- Host 不依赖 `StoryLockCoreApp`。
- Host 不读取 StoryLock 包私有数据。

## 8. 最终工程结构

建议最终结构：

```text
skill/
  Cargo.toml

  crates/
    storylock-contract/
    yian-host-contract/
    storylock-core-engine/

  apps/
    yian-windows-host/
    storylock-core-desktop/

  docs/
    management/
      拆分计划/
```

短期可先保留当前 `src/host/windows-host`，在内部新增模块模拟未来结构，避免一次性大搬迁。

## 9. 是否需要两个 git 仓库

短期不建议拆仓库。

原因：

- 当前仍处于比赛和快速演示阶段。
- 协议 DTO、长连接行为、StoryLock 授权流程还需要迭代。
- 拆仓会放大同步成本。

建议先做到：

- 两个独立可执行程序
- 一个 workspace
- 独立配置
- 独立测试
- 只通过协议通信

等协议稳定 2-3 个迭代后，再考虑拆成两个仓库。

## 10. 完成定义

满足以下条件，认为 Host 与 StoryLock 拆分完成：

- Host 与 StoryLock Core 可以分别启动。
- Host 与 StoryLock Core 有各自配置文件。
- StoryLock Core 通过长连接或 IPC 注册到 Host。
- Host 不读取 StoryLock draft、vault、问题、答案、故事摘要。
- Host 收到授权请求后，只转发给 StoryLock。
- StoryLock 返回授权结果和脱敏摘要。
- StoryLock 关闭不影响 Host Web API 和 relay。
- Host 关闭不破坏 StoryLock 包和 StoryLock 配置。
- `cargo build -p yian-windows-host` 通过。
- `cargo build -p storylock-core-desktop` 通过。

## 11. 下一步建议

下一步不要马上搬文件，先做协议和边界补丁：

1. 在当前项目内新增 `storylock_contract` 协议模块。
2. 把 Host 中 StoryLock 授权挑战相关调用包一层 facade。
3. 设计 `/storylock/session` 长连接接口。
4. 移除 Host 页面中的学习策略读取。
5. 把 Host 的 StoryLock 页面改成“连接状态 + 授权对象脱敏摘要”。
6. 等协议跑通后，再拆独立 app。

