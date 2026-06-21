# StoryLock 后续开发计划

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.1 |
| 更新日期 | 2026-06-20 |
| 适用范围 | `skill/` 主线代码、易安站点、StoryLock 三层 Skill、Windows/Android/Linux 本地宿主、发布与验收 |
| 配套清单 | `docs/management/StoryLock后续开发实施清单_20260620.md` |

## 0. 2026-06-20 补充决策：Windows 原生 UI 采用 Slint 渐进落地

参考 Slint 的 Rust 原生桌面能力后，Windows 本地宿主 UI 的后续方向调整为“先网页原型、再 Slint 原生化、最后托盘与安装器产品化”。

核心决策：

1. Slint 只作为 Windows 本地宿主的原生 UI 层，不替代 StoryLock Local Core、安全存储、relay polling 和 localhost API。
2. `verify / authorize / execute / revoke`、DPAPI、题库、授权会话继续留在 Rust 核心内，UI 通过 Rust 状态和受控回调触发能力。
3. Slint 以可选 Cargo feature 方式接入，避免默认构建、命令行测试和 zip 原型被 GUI 依赖阻塞。
4. 当前已经上线的 `/storylock-app.html` 与 `/local-agent-app.html` 作为交互蓝图，后续迁移为 Slint 原生窗口。
5. 远程网关仍只暴露 `requestSignature` 和 `requestPasswordFill`，不因 UI 丰富化而扩大远程能力面。

推荐落地顺序：

| 顺序 | 形态 | 目标 |
| --- | --- | --- |
| 1 | 静态 Web 原型 | 快速验证 StoryLock 工作台和 Agent 控制台的信息架构 |
| 2 | Slint 只读总览窗口 | 在 Windows host 中显示状态、配置、能力边界和本地 API |
| 3 | Slint 请求队列窗口 | 展示待确认请求、来源、能力、对象、风险和过期时间 |
| 4 | Slint 九宫格挑战窗口 | 替代简单 `MessageBoxW`，承接本地授权确认 |
| 5 | 托盘 + MSI | 提供开机常驻、打开控制台、复制诊断、退出和正式安装体验 |

风险约束：

1. Slint 许可证需要在正式发布前复核，确保当前发布形态满足其 GPL / royalty-free / commercial 条款。
2. UI 事件循环不得阻塞本地 HTTP server 和 relay polling。
3. UI 不得显示题库答案、明文密码、私钥或可恢复故事原文。
4. 无头环境和 CI 默认不启用 Slint UI feature。

## 1. 当前结论

StoryLock / 易安当前已经从“概念验证”进入“主线可运行、交付体验待补齐”的阶段。代码层面已有三层 Skill、远程网关、本地授权、题库、平台宿主和下载发布链路；产品层面仍缺少足够清晰的本地 UI、安装体验、状态管理界面和正式签名发布闭环。

当前线上站点：

1. `https://yian.cdao.online/` 已部署到 Vercel `storylock-gateway` 项目。
2. 首页已改为同页分屏导航，包含：产品说明、安全方式、下载绑定、安装版本、使用说明、使用流程、常见问题、请求状态、帮助说明。
3. `帮助说明` 已并入首页 `#help` 面板，不再跳转到风格不同的 `help.html`。
4. Windows 下载直链已恢复并可返回 zip 包。

当前 Windows 包状态：

1. 当前包是 `Yian Windows Host` 本地宿主原型，不是完整 StoryLock 图形桌面应用。
2. 线上 zip 当前包含：
   - `yian-windows-host.exe`
   - `README.md`
   - `start-yian-windows-host.cmd`
3. 本地宿主提供 localhost API、relay polling、DPAPI 本地保护、题库、verify / authorize / execute / revoke 闭环。
4. 仍缺少系统托盘、请求列表、确认详情页、题库管理页、设备状态页和正式安装器体验。

当前远程能力边界：

1. 远程网关对第三方只暴露两个主能力：`requestSignature`、`requestPasswordFill`。
2. 远程层不暴露故事读取、故事写入、题库答案、私钥、密码、本地授权细节。
3. 本地验证、授权、撤销、秘密读取和执行必须留在本地宿主或第二层授权边界内。

## 2. 功能界面部署总结

### 2.1 易安首页

当前首页是统一的单页面板式产品入口：

| 页面 | 路由 | 当前作用 |
| --- | --- | --- |
| 首页 | `#home` | 品牌入口、下载按钮、绑定入口 |
| 产品说明 | `#overview` | 说明易安如何连接云端请求和本地确认 |
| 安全方式 | `#architecture` | 说明云端、私人助手、本地核心的边界 |
| 下载绑定 | `#binding` | 说明下载后如何绑定本地设备 |
| 安装版本 | `#apk` | 展示 Windows / Android / Linux 下载入口 |
| 使用说明 | `#user-guide` | 面向普通用户的使用说明 |
| 使用流程 | `#flow` | 从下载到完成一次确认的步骤 |
| 常见问题 | `#faq` | 绑定、安装、离线、换设备问题 |
| 请求状态 | `#runtime` | 调用网关状态与设备连接接口 |
| 帮助说明 | `#help` | 与其他页面一致的帮助说明入口 |

已完成部署项：

1. Vercel outputDirectory 使用 `release/web/public`。
2. `npm run build` 会复制 `src/yian-web/public` 到 `release/web/public`。
3. downloads 目录生成平台包和 metadata。
4. WSL 发布入口可成功部署生产环境并 alias 到 `yian.cdao.online`。

待完善项：

1. 首页需要显示平台包类型：Windows 当前是“本地宿主原型”，不是完整应用。
2. 请求状态页需要从“JSON 输出”升级为用户可读的状态面板。
3. 下载页需要展示文件大小、SHA-256、版本号、渠道、原型限制。
4. 需要给 Windows、Android、Linux 分别补“安装后下一步”界面说明。

### 2.2 Windows 本地宿主界面

当前 Windows 端可运行但 UI 不足：

| 能力 | 当前形态 | 问题 |
| --- | --- | --- |
| 启动 | `start-yian-windows-host.cmd` + 控制台 | 用户会看到命令行输出，产品感弱 |
| 本地确认 | Windows Yes/No MessageBox | 只能表达允许/拒绝，缺少详情说明 |
| 请求查看 | localhost API / JSON | 没有可视化请求列表 |
| 题库管理 | API / CLI | 没有题库导入、版本、状态 UI |
| 设备状态 | `/health` JSON、`/ui/status` 状态 JSON、`ui-tray` 托盘首版 | 已有浏览器管理页和托盘入口，仍缺在线/离线/待确认状态图标细化和完整设备页 |
| 日志诊断 | 控制台输出、`/diagnostics` 脱敏诊断 JSON、托盘复制诊断 | 已有复制诊断首版，仍缺日志页面 |

下一阶段应把 Windows 从“host service prototype”推进到“local assistant shell”：

1. 托盘图标：首版已提供打开管理页、查看健康、复制诊断和退出；后续细化在线、离线、请求待确认状态。
2. 本地管理页：优先采用 Slint 原生窗口，必要时再保留 `http://127.0.0.1:4510/ui` 浏览器页。
3. 请求确认页：展示请求来源、能力、对象、风险、过期时间。
4. 题库页：展示当前题库版本、题目数量、导入按钮。
5. 设备页：展示 identityId、deviceId、appInstanceId、gateway、relay 状态。
6. 日志页：展示最近注册、轮询、执行、错误记录。

### 2.3 远程网关界面

当前远程层有意保持窄接口：

| 远程能力 | 当前状态 | 设计理由 |
| --- | --- | --- |
| `requestSignature` | 已作为主线能力 | 高敏感操作，必须本地授权 |
| `requestPasswordFill` | 已作为主线能力 | 远程只拿审计结果，不拿明文密码 |

不建议近期新增远程故事读写能力。StoryLock 的故事内容、题库答案、私钥、密码等应继续留在本地边界内。

## 3. StoryLock 功能设计总结

### 3.1 三层 Skill

| 层 | 包 | 当前能力 | UI / 产品含义 |
| --- | --- | --- | --- |
| 第一层 | `local-story-processing` | 故事草稿、故事润色、题集强度评估 | 将来可做“故事工作台” |
| 第二层 | `local-story-access` | 对象强度、九宫格验证、本地授权、撤销、审计 | 本地安全确认核心 |
| 第三层 | `remote-gateway` | 远程签名、远程密码填充、脱敏返回 | 云端/第三方 Agent 入口 |

### 3.2 本地 Agent / Host 能力

Windows 本地宿主当前实际能力：

1. `GET /health`
2. `GET /question-bank/status`
3. `POST /question-bank/import`
4. `POST /verify`
5. `POST /authorize`
6. `POST /execute`
7. `POST /revoke`
8. `/local-host/register`
9. `/local-host/relay/poll`
10. `/local-host/relay/respond`

核心执行能力：

1. `requestSignature`
2. `requestPasswordFill`

### 3.3 产品层功能分组

后续产品功能应分成四个界面域：

| 域 | 面向用户 | 主要功能 |
| --- | --- | --- |
| 易安网站 | 普通用户、评审、第三方接入方 | 下载、绑定、说明、状态、帮助 |
| 本地宿主 UI | 普通用户 | 待确认请求、设备状态、题库、日志 |
| StoryLock 工作台 | 创作者 / 私人助手 | 故事草稿、润色、题集强度、保护对象管理 |
| 管理与发布后台 | 开发 / 运维 | 包版本、metadata、部署、验收报告 |

## 4. 后续开发阶段

### 阶段 A：功能界面补齐

目标：让当前可运行能力变成普通用户可理解的界面。

任务：

1. 易安首页下载区显示版本、大小、checksum、渠道和原型限制。
2. 请求状态页从 JSON 输出改成结构化状态卡片。
3. Windows 本地宿主增加最小 UI：
   - 托盘状态
   - 本地管理页
   - 请求确认页
   - 日志页
4. 帮助说明补充“Windows 当前是本地宿主原型，不是完整桌面应用”。
5. Android / Linux 下载说明也补充原型、debug、release 差异。

验收：

1. 普通用户下载 Windows 包后能知道下一步运行哪个文件。
2. 首页不再把原型包误写成正式桌面应用。
3. 请求状态页无需阅读原始 JSON 即可理解当前状态。

### 阶段 B：Windows 本地宿主产品化

目标：把 Windows 从 zip 原型推进到候选交付形态。

任务：

1. 修正旧配置中 `/android-host/*` 与当前 `/local-host/*` 的路径混用。
2. 保留 `/storylock-app.html` 与 `/local-agent-app.html` 作为站点级 UI 原型。
3. 为 Windows host 增加 `--slint-ui` 原型入口，默认构建不启用，使用 `--features ui-slint` 验证。
4. 将 Slint UI 从只读状态窗口扩展到请求队列、九宫格挑战和授权结果。
5. 在 Slint 请求确认稳定后，逐步替换当前 Windows Yes/No `MessageBoxW` fallback。
6. 增加 MSI 构建和桌面快捷方式。
7. 增加启动项/托盘/退出控制。
8. 增加日志文件和诊断导出。
9. 对 `start-yian-windows-host.cmd` 做用户友好提示。

验收：

1. `npm run test:windows-host` 或指定端口闭环通过。
2. zip / msi 包内容可被脚本检查。
3. 默认无 UI 构建不受 Slint 影响。
4. `cargo check --features ui-slint` 通过。
5. 双击启动后用户能看到状态和下一步，不只看到控制台。

### 阶段 C：安全链路正式化

目标：减少 demo 语义，保留明确的安全边界。

任务：

1. Android Keystore 签名从 prototype 推进到 release 验收路径。
2. Windows DPAPI 存储增加生产模式检查。
3. Linux Secret Service 做真实桌面环境验收。
4. 第二层题库、challenge、authorization 与各平台 host 对齐。
5. 远程层继续只暴露 `requestSignature`、`requestPasswordFill`。

验收：

1. 各平台不返回明文密码、私钥、答案。
2. replay、expiry、nonce、requestId 检查稳定。
3. 本地授权失败、过期、撤销路径都有测试覆盖。

### 阶段 D：发布与部署闭环

目标：让“构建、打包、部署、验收”可重复执行。

任务：

1. 将 `src/yian-web/public/downloads` 与 `release/app/*` 的关系固定为发布规则。
2. 每次 Windows 包重打后同步内置 downloads 或调整 Vercel 构建策略。
3. Vercel 发布继续优先使用 WSL 或 CI token 通道。
4. 对线上 zip、metadata、首页资源版本做部署后检查。
5. 建立 release checklist：版本、大小、checksum、包内容、直链、metadata。

验收：

1. `npm run build`
2. `npm run test:release`
3. `npm run test:site-http`
4. 线上 `/downloads/*.zip` 与 metadata checksum 一致。

### 阶段 E：StoryLock 工作台

目标：让第一层故事能力不只停留在 Skill API。

任务：

1. 增加故事草稿 UI。
2. 增加故事润色 UI。
3. 增加题集强度评估 UI。
4. 增加题集模板导入/导出。
5. 明确 StoryLock 工作台与本地授权边界：处理故事文本不等于读取受保护对象。

验收：

1. 第一层能力有可演示界面。
2. 受保护对象仍必须经第二层授权。
3. 远程网关不直接读取故事内容。

## 5. 优先级

### P0

1. 修正并固化 Windows 本地宿主原型说明。
2. 建立 Windows 最小 UI 或本地管理页。
3. 固化线上发布后检查：zip 内容、metadata、首页导航。
4. 修正 `/android-host/*` 与 `/local-host/*` 路径混用风险。

### P1

1. 请求状态页产品化。
2. Windows MSI / 快捷方式 / 托盘。
3. Android release 安全链路验收。
4. Linux 真桌面 Secret Service 验收。

### P2

1. StoryLock 工作台 UI。
2. 多账户、多对象、多设备管理。
3. 更完整的演示和评审模式。

## 6. 不建议近期扩展

1. 不建议近期开放远程故事读写。
2. 不建议把题库答案、私钥、密码暴露给远程 API。
3. 不建议在本地 UI 未补齐前继续扩展过多新 capability。
4. 不建议把 Windows 原型包包装成“正式 StoryLock 应用”。

## 7. 总结

后续开发重点不是继续增加远程函数数量，而是把当前“安全边界正确但用户界面偏弱”的能力产品化。远程保持窄接口，本地补 UI 和安装体验，StoryLock 工作台补故事处理界面，发布链路补可重复验收。
