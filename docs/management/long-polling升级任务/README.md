# Windows Host long polling 升级任务

## 目标

把 Windows Host 与远程 gateway 的 relay 通信从短轮询升级为 long polling，降低远程授权申请到达本地的等待延迟，并减少空闲状态下的无效请求。

## 当前状态

1. Windows Host 启动本地 HTTP 服务，默认监听 `127.0.0.1:4510`。
2. 远程模式开启后，Windows Host 注册到 gateway，然后循环调用 `/local-host/relay/poll`。
3. 当前空闲时 gateway 立即返回 `idle`，Windows Host 等待约 `750ms` 后再次请求。
4. 本地 Slint Host 窗口默认展示；调试时可通过 `--server-only` 或 `STORYLOCK_WINDOWS_START_MODE=debug` 运行控制台模式。

## 重构方案

### 阶段 1：协议兼容升级

1. Windows Host poll 请求增加 long polling 元数据：
   - `transport: "long_poll"`
   - `waitMs`: 默认 `25000`
   - `clientTimeoutMs`: 默认 `35000`
2. Gateway 注册响应继续返回现有 `pollUrl` / `respondUrl`，保持旧客户端兼容。
3. Gateway status 中把 relay transport 从 `poll_respond` 更新为 `long_poll_respond`。

### 阶段 2：Gateway 等待队列

1. `/local-host/relay/poll` 在没有任务时不立即返回。
2. Gateway 最多等待 `waitMs`，期间如果有新任务入队，立即返回 `status: "ok"`。
3. 超时仍返回 `status: "idle"`，客户端收到后立刻发起下一次 poll。
4. 如果客户端不传 `waitMs`，保留短轮询行为，避免破坏旧 Android/Windows 客户端。

### 阶段 3：Windows Host 客户端

1. 将 reqwest 客户端超时调整到 long polling 可用的窗口。
2. 成功 `idle` 后不再固定等待 `750ms`，只做极短节流或立即重连。
3. 出错时仍保持退避，避免网络异常时打满 gateway。
4. 调试运行继续使用 `--server-only` 或 `STORYLOCK_WINDOWS_START_MODE=debug`，暂时不强制展示本地 Host 窗口。

### 阶段 4：验证

1. 运行 Rust 编译：
   - `cargo build`
2. 运行 gateway 自测：
   - `npm run selftest:web-api-android`
3. 检查空闲 poll 能等待后返回 `idle`，有任务时能提前返回。

## 验收标准

1. Windows Host release/debug 均能编译通过。
2. 开启远程模式后，relay 状态保持在线或 idle，不因 long polling 超时误判失败。
3. 远程授权申请入队后，等待中的 poll 能立即拿到任务。
4. 本地 Host 窗口可暂时不展示，控制台调试模式可以接管需要 Host 的任务。

## 已实施内容

1. Gateway relay 队列新增等待中的 poll waiter。
2. `/local-host/relay/poll` 支持 `waitMs`：
   - 未传 `waitMs`：保持旧短轮询，立即返回 `idle`。
   - 传入 `waitMs > 0`：启用 long polling，有任务立即返回，超时返回 `idle`。
3. Windows Host 远程 relay 请求默认改为：
   - `transport: "long_poll"`
   - `waitMs: 25000`
   - `clientTimeoutMs: 35000`
4. Windows Host 空闲重连节流从 `750ms` 降为 `25ms`，主要等待时间交给 gateway long poll。
5. Gateway 状态页 relay policy 标记为 `long_poll_respond`，并暴露 `waitingPollCount`。
6. Windows Host 增加调试入口：
   - `--debug-host`
   - `--runtime-debug`
   - `STORYLOCK_WINDOWS_START_MODE=debug-host`
7. Gateway 注册响应增加 `relay.pollPolicy`，Windows Host 会按 gateway 返回的策略调整 `waitMs` 与 `clientTimeoutMs`。
8. Gateway 限制单 host 同时等待中的 long poll 数量，默认 `1`；新的 long poll 会替换旧 waiter，旧 waiter 返回 `reason: "replaced_by_new_poll"`。
9. Gateway 在 long poll HTTP 连接关闭时会取消对应 waiter，避免断线请求占用内存。
10. Gateway 状态页增加 relay coordination / durability 信息，明确当前 relay 队列仍是进程内 volatile 队列。
11. Windows Host relay loop 增加周期性重新注册，默认约每 60 秒刷新一次注册状态和 gateway poll 策略。
12. Gateway 状态页增加 `relayPolicy.readiness`，明确当前单实例可用、多实例仍需要 Redis/KV/pub-sub 外部协调。
13. Gateway 状态页增加 long polling 观测指标：
   - `idleTimeoutCount`
   - `replacedPollCount`
   - `clientClosedPollCount`
   - 对应的最近发生时间戳

## 调试运行方式

不展示本地 Slint Host 窗口，仅启动本地 HTTP 服务和远程 relay 循环：

```powershell
$env:STORYLOCK_WINDOWS_REMOTE_ENABLED="1"
$env:STORYLOCK_GATEWAY_URL="https://yian.cdao.online"
$env:STORYLOCK_SHARED_SECRET="replace-with-shared-secret"
.\target\debug\yian-windows-host.exe --debug-host
```

如果只做本地接口调试，可以关闭远程模式：

```powershell
$env:STORYLOCK_WINDOWS_REMOTE_ENABLED="0"
.\target\debug\yian-windows-host.exe --debug-host
```

## 观察调试页使用说明

当前远端站点已经补充“观察调试”页，可直接用于 long polling 升级后的联调与验收观察。

### 入口

1. 打开远端站点主页。
2. 进入 `观察调试`。
3. 在页内二级菜单中切换：
   - `运行状态`
   - `Web 联调`
   - `Host 联调`

### 运行状态页怎么看

`运行状态` 顶部会先生成一条自动摘要，用一句话概括当前 relay 健康情况。

下面分为三组：

1. `当前策略`
   - 重点看：
     - `Relay 传输`
     - `默认等待`
     - `协调方式`
     - `生产就绪`
     - `Waiting Poll`
     - `Pending Response`
2. `最近事件`
   - 重点看：
     - `最近成功返回`
     - `最近超时`
     - `最近空闲超时`
     - `最近替换轮询`
     - `最近客户端断开`
3. `累计统计`
   - 重点看：
     - `累计请求`
     - `累计成功返回`
     - `累计超时`
     - `累计空闲超时`
     - `累计替换轮询`
     - `累计客户端断开`

### Host 联调页怎么看

`Host 联调` 顶部同样会生成一条 Host 摘要。

下面分为三组：

1. `本地接口`
   - 管理页 URL
   - 状态接口 URL
   - 诊断接口 URL
   - Gateway URL
2. `当前状态`
   - 本地 Host
   - Relay 状态
   - 运行模式
   - 最近错误
3. `最近错误`
   - 作为联调观察提示保留

同时提供：

1. `打开管理页`
2. `复制状态接口`
3. `复制诊断接口`
4. `刷新当前分区`

### 验收时建议关注的结论

满足下面这些现象时，可认为当前 long polling 加固链路基本符合预期：

1. `Relay 传输` 显示为 long poll 相关值。
2. `默认等待` 与 `client timeout` 已反映 gateway 返回的策略值。
3. `生产就绪` 明确提示：
   - 单实例可用；
   - 多实例仍需外部协调。
4. 空闲期间：
   - `Waiting Poll` 可短暂为 1；
   - `Pending Response` 应保持 0 或很低。
5. 正常请求到达时：
   - `最近成功返回` 会刷新；
   - `累计成功返回` 递增。
6. 若发生异常：
   - `最近超时` / `最近替换轮询` / `最近客户端断开` 会留下时间点；
   - 对应累计计数会增加。

### 颜色提示约定

观察调试页中的状态卡采用以下提示：

1. 绿色：当前值健康或符合预期。
2. 黄色：当前值可工作，但提示存在单实例边界、等待堆积或最近异常。
3. 红色：当前值表示明确错误或最近存在错误记录。

## 验证记录

1. Gateway 自测通过：

```powershell
npm run selftest:web-api-android --prefix src/skills/remote-gateway
```

2. Windows Host debug 编译通过：

```powershell
cargo build
```

当前仍有若干既有 Rust warning，主要是未使用函数和 import，不影响构建。

## 加固结论

当前版本已经完成单实例和本地调试场景下的 long polling 加固：

1. 策略可协商。
2. 等待请求可观测。
3. 单 host waiter 数量受控。
4. 客户端断连可清理。
5. 短轮询兼容仍保留。
6. Windows Host 会周期性刷新注册与策略。
7. Gateway 会显式输出生产就绪边界。
8. Gateway 会区分 idle 超时、重复 poll 替换、客户端断开三类等待结束原因。

如果部署到 Vercel 多实例或无状态函数，仍需要外部协调层：

1. Redis / Upstash / KV 保存 relay 队列与 pending response。
2. Pub/sub 或可唤醒队列负责跨实例通知等待中的 poll。
3. `relayRequestId` 响应表需要跨实例幂等解析。
