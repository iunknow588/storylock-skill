# Host / StoryLock / Common 三域目录拆分路线

更新时间：2026-07-03

## 1. 结论

采用以下三个顶层目录承载后续代码、文档和自动化脚本是可行的：

```text
skill/
  host/
  storylock/
  common/
```

这个结构比单纯按 `apps/`、`crates/` 分类更适合当前项目，因为 Host 和 StoryLock 是两个清晰的产品域，而 `common` 是双方协议和共享基础设施的承载层。

核心原则：

1. `host` 承载 Yian Host 的代码、文档、测试、脚本和发布配置。
2. `storylock` 承载 StoryLock Core 的代码、文档、测试、脚本和包管理配置。
3. `common` 只承载真正公共的 contract、schema、共享测试夹具和跨域脚本。
4. Host 不读取 StoryLock 私有数据。
5. StoryLock 通过本地长连接或 IPC 注册到 Host，返回授权结果和脱敏摘要。

## 2. 目标目录结构

建议最终结构：

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

## 3. 三个目录的职责

### host

`host` 是 Yian Host 产品域。

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

`storylock` 是 StoryLock Core 产品域。

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

`common` 是公共协议和共享基础设施层。

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
- 任何“只是暂时不知道放哪”的杂项文件

## 4. Host 与 StoryLock 的关系

Host 已经有 Web 接口，因此 Host 可以独立存在。StoryLock 不应该被 Host 当作一个可读取的数据目录，而应该作为一个独立授权服务接入 Host。

推荐关系：

```text
StoryLock Core  --WebSocket / Named Pipe-->  Yian Host
```

运行流程：

1. Host 启动本地 Web/API 服务。
2. StoryLock Core 独立启动。
3. StoryLock Core 主动连接 Host 本地会话接口。
4. StoryLock Core 注册自身能力。
5. Host 收到远程授权请求。
6. Host 通过本地连接把请求转发给 StoryLock。
7. StoryLock 完成挑战和授权判断。
8. StoryLock 返回授权结果和脱敏摘要。
9. Host 把结果返回给远程 Agent 或 gateway。

Host 不读取 StoryLock 数据，只转发请求和接收结果。

## 5. common/contracts 建议内容

建议先建立：

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

禁止传输：

- 故事原文
- 故事摘要
- 问题文本
- 答案
- 密码
- 私钥
- vault 原始内容
- 明文签名密钥

允许传输：

- 在线状态
- 授权结果
- 授权 id
- 授权过期时间
- 脱敏对象引用
- 保密级别
- 访问等级
- 非敏感错误码

## 6. 迁移路线

### 阶段 0：目录基线建立

目标：建立目录基线，不搬动业务代码。

任务：

- 为 `host/`、`storylock/`、`common/` 增加 README。
- 在 `common/contracts/host-storylock-protocol/` 建立协议草案。
- 在 `docs/management/拆分计划/` 记录该三域目录路线。

验收：

- 三个目录职责清晰。
- 文档能解释哪些内容进入哪个目录。
- 当前代码仍保持可编译。

### 阶段 1：公共协议先行

目标：把 Host 与 StoryLock 的协作接口先沉淀到 `common`。

任务：

- 建立 `common/contracts/host-storylock-protocol/messages.md`
- 建立 JSON 示例
- 后续 Rust DTO 放入 `common/crates/storylock-contract`

验收：

- 协议中没有故事、答案、密码、私钥字段。
- Host 与 StoryLock 的交互可以只靠协议描述清楚。

### 阶段 2：Host 文档和脚本迁移

目标：先迁移低风险内容。

迁移候选：

```text
docs/design/cn/YianWindowsHost菜单配置说明_*.md -> host/docs/ui/
docs/management/long-polling升级任务/ -> host/docs/relay/
scripts/windows/ -> host/scripts/
src/host/windows-host/ -> 暂不移动，先记录映射
```

验收：

- 迁移后文档链接可追踪。
- 不影响编译。
- 不影响现有脚本入口。

### 阶段 3：StoryLock 文档和包脚本迁移

目标：把 StoryLock 包、挑战、对象策略文档迁移到 `storylock`。

迁移候选：

```text
docs/design/cn/StoryLock数据包与校验CLI说明_*.md -> storylock/docs/package/
docs/design/cn/Challenge状态机.md -> storylock/docs/challenge/
scripts/storylock-package/ -> storylock/scripts/package/
scripts/story-drafts/ -> storylock/scripts/package/
```

验收：

- StoryLock 文档不再散在 Host 管理目录下。
- 包校验脚本的位置和职责清晰。

### 阶段 4：公共 schema 和测试夹具迁移

目标：把真正被 Host 和 StoryLock 共用的内容迁移到 `common`。

迁移候选：

```text
src/shared/storylock-package/ -> common/schemas/storylock-package/
共享 JSON fixtures -> common/test-fixtures/
协议 DTO -> common/crates/
```

验收：

- `common` 中没有业务私有实现。
- Host 和 StoryLock 都可以依赖 `common`，但 `common` 不依赖二者。

### 阶段 5：代码迁移准备

目标：代码迁移前先建立映射，不立即大搬迁。

当前到目标的映射：

| 当前路径 | 目标路径 |
| --- | --- |
| `src/host/windows-host` | `host/src/windows-host` |
| `src/slint_ui/host_dashboard.slint` | `host/src/windows-host/src/ui/host_dashboard.slint` |
| `src/slint_ui/storylock_core.slint` | `storylock/src/storylock-core-desktop/src/ui/storylock_core.slint` |
| `src/slint_ui/storylock_core/` | `storylock/src/storylock-core-desktop/src/ui/storylock_core/` |
| `src/slint_ui/storylock/core_data` | `storylock/src/storylock-core-engine/src/core_data` |
| `src/slint_ui/storylock/resource_export` | `storylock/src/storylock-core-engine/src/resource_export` |
| Host 与 StoryLock DTO | `common/crates/storylock-contract` |

验收：

- 形成迁移清单。
- 每次迁移只移动一类文件。
- 每次迁移后都能编译。

### 阶段 6：代码正式迁移

目标：拆成三域工程结构。

顺序：

1. 迁移 `common/crates/storylock-contract`
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

## 7. 根目录 workspace 建议

最终可以在 `skill/Cargo.toml` 建立 workspace：

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

短期不建议马上改 workspace。先完成协议文档和目录 README，再做代码迁移。

## 8. 文档迁移规则

Host 文档进入：

```text
host/docs/
```

包括：

- Host Web API
- relay
- 本地 HTTP 服务
- Host UI
- Host 设置
- Host 发布和安装

StoryLock 文档进入：

```text
storylock/docs/
```

包括：

- StoryLock 包格式
- vault
- 挑战机制
- 故事模板
- 受控对象
- 学习策略
- StoryLock Core UI

公共文档进入：

```text
common/contracts/
common/schemas/
```

包括：

- Host / StoryLock 本地协议
- Gateway / Host 协议
- JSON schema
- 脱敏摘要结构
- 共享错误码

管理性总计划仍保留：

```text
docs/management/
```

## 9. 自动化脚本迁移规则

Host 专用脚本：

```text
host/scripts/
```

StoryLock 专用脚本：

```text
storylock/scripts/
```

公共脚本：

```text
common/scripts/
```

判断标准：

- 只服务 Host：放 `host/scripts`
- 只服务 StoryLock：放 `storylock/scripts`
- 两边都用，且不包含业务私有逻辑：放 `common/scripts`
- 发布总控脚本：可以保留 `release/` 或根 `scripts/release/`，但应调用三域内脚本

## 10. 风险与约束

| 风险 | 说明 | 处理 |
| --- | --- | --- |
| `common` 变成杂物目录 | 边界变模糊 | 只允许协议、schema、共享测试、跨域脚本 |
| 过早移动代码 | 编译和路径引用大面积失败 | 先文档、协议、README，再小步迁移 |
| Host 继续读取 StoryLock 数据 | 安全边界失败 | 用长连接协议替换直接读取 |
| StoryLock 依赖 Host runtime | StoryLock 无法独立运行 | Host runtime 只能留在 `host` |
| 文档链接失效 | 管理成本上升 | 迁移文档时保留索引和映射表 |

## 11. 完成定义

三域拆分完成时应满足：

- `host/` 可以独立说明、构建、测试、发布 Host。
- `storylock/` 可以独立说明、构建、测试、发布 StoryLock Core。
- `common/` 只包含公共协议、schema、测试夹具和共享脚本。
- Host 与 StoryLock 只通过本地长连接 / IPC / contract 协作。
- Host 不读取 StoryLock 私有数据。
- StoryLock 不承担 Host relay。
- 根目录保留统一 workspace 和总控文档。

