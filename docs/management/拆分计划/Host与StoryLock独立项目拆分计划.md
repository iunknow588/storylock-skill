# Host 与 StoryLock 独立项目拆分计划（立即执行版）

更新时间：2026-07-03

## 1. 执行结论

从现在开始，Host 与 StoryLock 按两个独立项目推进。当前不再把拆分视为“后续可选优化”，而是当前阶段的主线任务。

拆分后的基本关系：

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

立即冻结的边界：

1. Host 是本地 Web/API 网关，不是 StoryLock 数据读取者。
2. StoryLock Core 是故事、答案、vault、对象策略、挑战验证的所有者。
3. Host 不读取 `vault.stlk`、draft、故事摘要、问题文本、答案、密码、私钥。
4. Host 收到远程请求后，只把授权请求转发给 StoryLock。
5. StoryLock 完成挑战和授权判断，只返回通过、拒绝、超时、脱敏摘要。
6. Host 和 StoryLock 之间必须通过本地长连接、IPC 或稳定 contract 协作。

## 2. 当前已完成基础

当前代码已经完成拆分前置条件：

- Host 配置文件独立为 `target/debug/config/host-config.json`
- StoryLock 配置文件独立为 `target/debug/storylock/config/storylock-config.json`
- StoryLock 模板文件回到 StoryLock 包内 `templates/`
- Host 设置页不再修改 StoryLock 包配置
- Host 语言配置只保存到 Host 配置
- StoryLock 包路径、导出路径只保存到 StoryLock 配置

这些改动说明：配置边界已经拆开，可以进入工程拆分。

## 3. 当前必须移除的耦合

第一批拆分必须处理以下耦合：

| 耦合点 | 当前问题 | 处理方式 |
| --- | --- | --- |
| `dashboard.rs` 直接创建 `StoryLockCoreApp` | Host 和 StoryLock UI 仍在同一进程 | 改为 Host 只启动或发现 StoryLock 独立进程 |
| Host 调用 `read_effective_author_draft(package_dir)` | Host 读取 StoryLock 私有 draft | 迁移到 StoryLock 内部，通过协议返回授权结果 |
| Host 调用 `host_learning_plan_status(&core_package_dir)` | Host 读取学习策略摘要 | 删除或改为 StoryLock 脱敏状态 |
| Slint 编译入口同时包含 Host 和 StoryLock | 两个 UI 无法独立构建 | 拆成两个 app |
| `catalog_resources.rs` 引用 `crate::host_runtime` | StoryLock engine 依赖 Host runtime | 改为 contract DTO 或注入参数 |

优先级最高的是：移除 Host 对 StoryLock draft、问题、答案的读取。

## 4. 独立项目目标

最终形成：

```text
skill/
  host/
    src/windows-host/
    docs/
    scripts/
    tests/

  storylock/
    src/storylock-core-desktop/
    src/storylock-core-engine/
    docs/
    scripts/
    tests/

  common/
    contracts/
    schemas/
    crates/
    scripts/
    test-fixtures/
```

项目职责：

| 项目 | 职责 | 不允许做的事 |
| --- | --- | --- |
| `host` | 本地 HTTP API、远程 relay、Host Dashboard、StoryLock 会话管理、授权请求转发 | 读取 StoryLock 私有数据 |
| `storylock` | StoryLock UI、包管理、vault、故事、答案、挑战、对象策略、授权判断 | 承担 Host relay 或修改 Host 配置 |
| `common` | Host 与 StoryLock 的协议、schema、共享 DTO、共享测试夹具 | 放业务私有逻辑或 UI |

## 5. 本地协作协议

立即采用“StoryLock 主动连接 Host”的设计。

首选：

```text
ws://127.0.0.1:<host_port>/storylock/session
```

备用：

```text
\\.\pipe\yian-host-storylock
```

StoryLock 启动后注册：

```json
{
  "type": "storylock.register",
  "sessionId": "sl-session-001",
  "app": "storylock-core-desktop",
  "version": "0.1.0",
  "capabilities": [
    "authorization.request",
    "authorization.challenge",
    "redacted.summary"
  ]
}
```

Host 转发授权请求：

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

StoryLock 返回授权结果：

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

协议禁止传输：

- 故事原文
- 故事摘要
- 问题文本
- 答案
- 密码
- 私钥
- vault 原始内容
- 明文签名密钥

## 6. 立即执行阶段

### 阶段 A：目录基线与协议文档

状态：立即执行。

任务：

- 使用 `skill/host`、`skill/storylock`、`skill/common` 作为最终三域目录。
- 在 `common/contracts/host-storylock-protocol` 建立协议文档和 JSON 示例。
- 在 `host/README.md`、`storylock/README.md`、`common/README.md` 固化职责边界。

验收：

- 三个顶层目录都有 README。
- `common/contracts` 描述 Host / StoryLock 本地协作协议。
- 文档明确 Host 不读取 StoryLock 私有数据。

### 阶段 B：建立 contract DTO

状态：立即执行。

先在当前工程内新增协议模块，迁入目标固定为 `common/crates/storylock-contract`。

目标内容：

```text
storylock_contract/
  session.rs
  authorization.rs
  redacted_summary.rs
  errors.rs
```

验收：

- DTO 不依赖 Slint。
- DTO 不依赖 Host runtime。
- DTO 不包含故事、答案、密码、私钥字段。

### 阶段 C：Host 移除 StoryLock 私有读取

状态：立即执行。

必须替换：

- `create_storylock_open_challenge`
- `storylock_open_expected_answers`
- `read_effective_author_draft(package_dir)`
- `host_learning_plan_status(&core_package_dir)`

迁移后：

- Host 发送 `authorization.request`
- StoryLock 生成挑战、验证答案
- StoryLock 返回 `authorization.result`
- Host 只展示连接状态和脱敏摘要

验收：

- Host 不再 import 或调用 `read_effective_author_draft`
- Host 不读取 `learning-policy.json`
- Host 不解析 StoryLock 问题或答案

### 阶段 D：拆 StoryLock Core 独立应用

状态：立即执行，但在阶段 B/C 完成后落地。

目标目录：

```text
storylock/src/storylock-core-desktop/
storylock/src/storylock-core-engine/
```

StoryLock Core 行为：

1. 加载 StoryLock 自己的配置。
2. 独立打开 StoryLock Core UI。
3. 尝试连接 Host `/storylock/session`。
4. 连接成功后注册能力。
5. 没有 Host 时仍可离线管理 StoryLock 包。

验收：

- StoryLock Core 可独立启动。
- StoryLock Core 不依赖 Host runtime。
- StoryLock Core 可离线编辑和管理本地包。

### 阶段 E：拆 Host 独立应用

状态：立即执行，但在阶段 D 后落地。

目标目录：

```text
host/src/windows-host/
```

Host 行为：

1. 启动本地 Web/API 服务。
2. 启动远程 relay。
3. 等待 StoryLock 注册。
4. 收到敏感操作请求时转发给 StoryLock。
5. StoryLock 未在线时返回明确错误。

验收：

- Host 可独立启动。
- Host 不编译 `StoryLockCoreApp`。
- Host 不读取 StoryLock 私有数据。

## 7. 第一批任务清单

第一批立即执行任务：

- [ ] 建立 `host/README.md`
- [ ] 建立 `storylock/README.md`
- [ ] 建立 `common/README.md`
- [ ] 建立 `common/contracts/host-storylock-protocol/README.md`
- [ ] 建立协议 JSON 示例
- [ ] 新增 `storylock_contract` DTO 草案
- [ ] 把 Host 中 StoryLock 授权挑战调用包成 facade
- [ ] 移除 Host 页面中的学习策略读取
- [ ] 把 Host StoryLock 页面改成连接状态和脱敏摘要
- [ ] 拆 StoryLock Core 独立启动入口
- [ ] 拆 Host 独立启动入口

## 8. 完成定义

满足以下条件后，认为 Host 与 StoryLock 独立项目拆分完成：

- Host 与 StoryLock Core 可以分别启动。
- Host 与 StoryLock Core 有各自配置文件。
- StoryLock Core 通过长连接或 IPC 注册到 Host。
- Host 不读取 StoryLock draft、vault、问题、答案、故事摘要。
- Host 收到授权请求后只转发给 StoryLock。
- StoryLock 返回授权结果和脱敏摘要。
- StoryLock 关闭不影响 Host Web API 和 relay。
- Host 关闭不破坏 StoryLock 包和 StoryLock 配置。
- `cargo build -p yian-windows-host` 通过。
- `cargo build -p storylock-core-desktop` 通过。
