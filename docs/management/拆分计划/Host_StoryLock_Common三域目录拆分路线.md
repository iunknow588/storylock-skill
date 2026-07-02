# Host / StoryLock / Common 三域目录拆分路线（立即执行版）

更新时间：2026-07-03

## 1. 执行结论

从现在开始，项目按以下三域目录推进：

```text
skill/
  host/
  storylock/
  common/
```

这不是远期规划，而是当前代码、文档、脚本迁移的执行基线。

三域原则：

1. `host` 承载 Yian Host 的代码、文档、测试、脚本和发布配置。
2. `storylock` 承载 StoryLock Core 的代码、文档、测试、脚本和包管理配置。
3. `common` 只承载真正公共的 contract、schema、共享测试夹具和跨域脚本。
4. Host 不读取 StoryLock 私有数据。
5. StoryLock 通过本地长连接或 IPC 注册到 Host，返回授权结果和脱敏摘要。

## 2. 立即采用的目标结构

```text
skill/
  host/
    README.md
    docs/
      api/
      relay/
      ui/
      release/
    src/
      windows-host/
    scripts/
      build/
      run/
      release/
      test/
    tests/
      integration/
      fixtures/
    release/

  storylock/
    README.md
    docs/
      core/
      package/
      challenge/
      objects/
      release/
    src/
      storylock-core-desktop/
      storylock-core-engine/
    scripts/
      build/
      run/
      package/
      release/
      test/
    tests/
      engine/
      desktop/
      fixtures/
    packages/
      templates/
      examples/

  common/
    README.md
    contracts/
      host-storylock-protocol/
      gateway-host-protocol/
    schemas/
      storylock-package/
      host-status/
      authorization/
    crates/
      storylock-contract/
      yian-host-contract/
      shared-test-support/
    scripts/
      verify/
      format/
      ci/
    test-fixtures/
      authorization/
      storylock-package/
      host-status/
```

## 3. 三域职责

### host

职责：

- 本地 Web/API 服务
- `/health`、`/verify`、`/authorize`、`/execute`、`/ui/status`
- 远程 gateway relay
- Host Dashboard
- Host 自己的配置文件
- StoryLock 连接状态管理
- 授权请求转发
- 脱敏授权状态展示

禁止：

- 读取 StoryLock draft
- 读取故事摘要
- 读取问题文本或答案
- 读取 `vault.stlk` 私有内容
- 读取密码、私钥、签名密钥
- 直接依赖 StoryLock UI

### storylock

职责：

- StoryLock Core 桌面应用
- 故事、问题、答案管理
- vault 与 StoryLock 包管理
- 受控对象策略
- 挑战生成与验证
- 学习策略与导出
- StoryLock 自己的配置文件
- 主动连接 Host 并注册能力

禁止：

- 承担 Host 的远程 relay
- 直接管理 Host 的远程 gateway 配置
- 修改 Host 私有配置
- 依赖 Host Dashboard

### common

允许放入：

- Host 与 StoryLock 的本地协作协议
- Gateway 与 Host 的协议 DTO
- JSON schema
- 脱敏授权摘要格式
- 共享错误码
- 共享测试夹具
- 跨域验证脚本

禁止放入：

- StoryLock 私有引擎逻辑
- Host runtime 逻辑
- UI 代码
- 平台密钥实现
- 私有配置文件
- 临时杂项文件

## 4. 立即建立的公共协议目录

第一批建立：

```text
common/contracts/host-storylock-protocol/
  README.md
  messages.md
  security-boundary.md
  examples/
    storylock-register.json
    authorization-request.json
    authorization-result.json
    redacted-summary-result.json
```

协议消息：

| 消息 | 方向 | 说明 |
| --- | --- | --- |
| `storylock.register` | StoryLock -> Host | 注册 StoryLock 能力 |
| `storylock.heartbeat` | 双向 | 保活 |
| `storylock.status` | StoryLock -> Host | 脱敏状态 |
| `authorization.request` | Host -> StoryLock | 请求授权 |
| `authorization.result` | StoryLock -> Host | 授权结果 |
| `authorization.cancel` | Host -> StoryLock | 取消请求 |
| `redacted.summary.request` | Host -> StoryLock | 请求脱敏摘要 |
| `redacted.summary.result` | StoryLock -> Host | 返回脱敏摘要 |
| `storylock.error` | 双向 | 协议错误 |

## 5. 立即迁移路线

### 阶段 A：目录和 README

立即执行：

- 建立 `host/README.md`
- 建立 `storylock/README.md`
- 建立 `common/README.md`
- 建立 `common/contracts/host-storylock-protocol/README.md`

验收：

- 三个目录职责明确。
- README 明确 Host 不读取 StoryLock 私有数据。

### 阶段 B：协议文档和示例

立即执行：

- 建立 `messages.md`
- 建立 `security-boundary.md`
- 建立注册、授权请求、授权结果、脱敏摘要 JSON 示例

验收：

- 协议中没有故事、答案、密码、私钥字段。
- Host 与 StoryLock 的交互可以只靠协议描述清楚。

### 阶段 C：低风险文档迁移

立即执行：

Host 文档迁移候选：

```text
docs/design/cn/YianWindowsHost菜单配置说明_*.md -> host/docs/ui/
docs/management/long-polling升级任务/ -> host/docs/relay/
```

StoryLock 文档迁移候选：

```text
docs/design/cn/StoryLock数据包与校验CLI说明_*.md -> storylock/docs/package/
docs/design/cn/Challenge状态机.md -> storylock/docs/challenge/
```

验收：

- 迁移后保留索引或映射表。
- 不影响当前代码编译。

### 阶段 D：脚本迁移

立即执行：

```text
scripts/windows/ -> host/scripts/
scripts/storylock-package/ -> storylock/scripts/package/
scripts/story-drafts/ -> storylock/scripts/package/
跨域验证脚本 -> common/scripts/
```

验收：

- Host 专用脚本只在 `host/scripts`
- StoryLock 专用脚本只在 `storylock/scripts`
- 双方共用脚本只在 `common/scripts`

### 阶段 E：代码迁移准备

立即执行：

先建立映射，不一次性大搬迁：

| 当前路径 | 目标路径 |
| --- | --- |
| `src/host/windows-host` | `host/src/windows-host` |
| `src/slint_ui/host_dashboard.slint` | `host/src/windows-host/src/ui/host_dashboard.slint` |
| `src/slint_ui/storylock_core.slint` | `storylock/src/storylock-core-desktop/src/ui/storylock_core.slint` |
| `src/slint_ui/storylock_core/` | `storylock/src/storylock-core-desktop/src/ui/storylock_core/` |
| `src/slint_ui/storylock/core_data` | `storylock/src/storylock-core-engine/src/core_data` |
| `src/slint_ui/storylock/resource_export` | `storylock/src/storylock-core-engine/src/resource_export` |
| Host / StoryLock DTO | `common/crates/storylock-contract` |

验收：

- 每次迁移只移动一类文件。
- 每次迁移后都能编译。

### 阶段 F：代码正式迁移

执行顺序：

1. 建立 `common/crates/storylock-contract`
2. 迁移 StoryLock engine
3. 迁移 StoryLock desktop
4. 迁移 Host runtime 和 Host UI
5. 建立根 workspace
6. 更新构建脚本
7. 更新 CI / 自动化测试

验收：

- `cargo build -p yian-windows-host` 通过。
- `cargo build -p storylock-core-desktop` 通过。
- Host 不编译 StoryLock UI。
- StoryLock 不依赖 Host runtime。

## 6. 根 workspace 目标

最终在 `skill/Cargo.toml` 建立 workspace：

```toml
[workspace]
members = [
  "host/src/windows-host",
  "storylock/src/storylock-core-desktop",
  "storylock/src/storylock-core-engine",
  "common/crates/storylock-contract",
  "common/crates/yian-host-contract",
]
resolver = "2"
```

workspace 在第一批目录和协议落地后开始启用。

## 7. 风险与强制约束

| 风险 | 处理 |
| --- | --- |
| `common` 变成杂物目录 | 只允许协议、schema、共享测试、跨域脚本 |
| 代码移动导致编译失败 | 小步迁移，每步编译 |
| Host 继续读取 StoryLock 数据 | 用长连接协议替换直接读取 |
| StoryLock 依赖 Host runtime | Host runtime 只能留在 `host` |
| 文档链接失效 | 迁移时保留索引和映射表 |

## 8. 完成定义

三域拆分完成时应满足：

- `host/` 可以独立说明、构建、测试、发布 Host。
- `storylock/` 可以独立说明、构建、测试、发布 StoryLock Core。
- `common/` 只包含公共协议、schema、测试夹具和共享脚本。
- Host 与 StoryLock 只通过本地长连接、IPC 或 contract 协作。
- Host 不读取 StoryLock 私有数据。
- StoryLock 不承担 Host relay。
- 根目录保留统一 workspace 和总控文档。

