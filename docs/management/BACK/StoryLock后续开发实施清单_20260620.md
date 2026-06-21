# StoryLock 后续开发实施清单

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.1 |
| 更新日期 | 2026-06-20 |
| 对应计划 | `docs/management/StoryLock后续开发计划_20260620.md` |
| 用途 | 将功能界面、部署、Windows 本地宿主和 StoryLock 能力设计拆成可执行任务 |

## 1. 当前已完成基线

### 站点与部署

- [x] `yian.cdao.online` 已可访问首页。
- [x] Vercel 项目绑定为 `storylock-gateway`。
- [x] WSL 发布入口 `scripts/vercel/publish_site_release_wsl.ps1` 已可执行生产部署。
- [x] 首页菜单统一为同页面板切换。
- [x] 帮助说明已并入 `#help`，不再使用独立跳转风格。
- [x] Windows 下载直链已恢复。
- [x] Windows zip 线上验证包含：
  - `yian-windows-host.exe`
  - `README.md`
  - `start-yian-windows-host.cmd`

### 三层能力

- [x] 第一层：`local-story-processing` 包含故事草稿、润色、题集强度评估。
- [x] 第二层：`local-story-access` 包含对象强度、九宫格验证、本地授权、撤销、审计。
- [x] 第三层：`remote-gateway` 只暴露 `requestSignature`、`requestPasswordFill`。
- [x] Windows host 已有 `health / question-bank / verify / authorize / execute / revoke` 闭环。
- [x] `npm run test:release` 可检查下载元数据与包 checksum。

## 2. 必须保持的产品边界

- [x] 首页、文档、下载按钮不得把 Windows 原型包描述为完整 StoryLock 桌面应用。
- [x] 远程网关不得新增直接读取故事、私钥、密码、题库答案的 API。
- [x] 本地授权必须留在本地 host / 第二层边界内。
- [x] 远程返回必须继续脱敏，不返回 raw secret、answers、privateKey、password。
- [x] Android / Windows / Linux 的 prototype、debug、candidate、release 命名必须清晰区分。

## 3. A 组：易安首页与功能界面

### A1. 下载区展示包信息

**状态：已完成。**

**改哪里**

1. `src/yian-web/public/index.html`
2. `src/yian-web/public/main.js`
3. `src/yian-web/public/styles.css`
4. `src/skills/remote-gateway/web-api-handler.js`

**改成什么**

1. 在安装版本页展示每个平台：
   - versionName
   - versionCode
   - packageKind
   - releaseChannel
   - fileSizeBytes
   - checksum
2. Windows 显示为“Windows 本地宿主原型”。
3. Android 显示 debug/internal 或 release/candidate。
4. Linux 显示 deb / tar.gz 原型包。

**产出**

1. 下载信息卡片。
2. 用户可复制 checksum。
3. 下载按钮旁显示“原型限制”。

**验收**

1. `npm run build`
2. `npm run test:site-http`
3. 人工检查首页安装版本页。

### A2. 请求状态页产品化

**状态：已完成。**

**改哪里**

1. `src/yian-web/public/index.html`
2. `src/yian-web/public/main.js`
3. `src/yian-web/public/styles.css`

**改成什么**

1. 将 JSON 输出拆成状态卡片：
   - 网关状态
   - 当前通信模式
   - 在线设备数
   - 待处理请求数
   - 最近请求来源
   - Windows / Android / Linux host 状态
2. 保留“查看原始 JSON”折叠区。

**产出**

1. 普通用户可读的请求状态界面。
2. 调试人员仍可查看 JSON。

**验收**

1. `npm run test:site-http`
2. 线上检查 `https://yian.cdao.online/#runtime`

### A3. 帮助说明补充原型边界

**状态：已完成。**

**改哪里**

1. `src/yian-web/public/main.js`
2. `src/yian-web/public/index.html`

**改成什么**

1. 帮助说明明确：
   - Windows 当前是本地宿主原型。
   - 双击 `start-yian-windows-host.cmd` 启动。
   - 完整图形桌面应用尚未交付。
2. 增加“下载后看到哪些文件”的说明。

**验收**

1. `npm run build`
2. 人工检查 `#help`

## 4. B 组：Windows 本地宿主 UI 与产品化

### B0. Slint 原生 UI 接入原则

**状态：已完成首版。**

**改哪里**

1. `src/host/windows-host/Cargo.toml`
2. `src/host/windows-host/src/main.rs`
3. 新增 `src/host/windows-host/src/slint_ui.rs`

**改成什么**

1. 增加可选 feature：`ui-slint`。
2. 默认 `cargo build --release` 仍构建无 GUI 的宿主，保证现有测试和 zip 原型稳定。
3. `cargo run --features ui-slint -- --slint-ui` 打开 Slint 原生状态窗口。
4. UI 首版只显示：
   - host 状态
   - gateway
   - identityId / deviceId
   - localhost API
   - 受控能力边界
   - StoryLock Local Core 调用链
5. UI 不显示题库答案、密码、私钥和故事原文。

**验收**

1. `cargo check`
2. `cargo check --features ui-slint`
3. `cargo run --features ui-slint -- --slint-ui` 可在 Windows 桌面打开窗口。
4. 未启用 feature 时，`--slint-ui` 给出明确提示而不是破坏 host 主流程。

### B1. 修正路径混用

**改哪里**

1. `src/host/windows-host/src/main.rs`
2. `src/host/windows-host/README.md`
3. `scripts/windows/check_windows_host_loop.ps1`

**改成什么**

1. 默认配置统一使用：
   - `/local-host/register`
   - `/local-host/relay/poll`
   - `/local-host/relay/respond`
2. 检查旧输出是否仍出现 `/android-host/*`。
3. 如果需要兼容旧路径，在文档中明确“旧路径仅兼容，不是主线”。

**验收**

1. `npm run test:windows-host`
2. 运行 exe，配置输出不再显示 `/android-host/*` 主路径。

### B2. 本地管理页 / Slint 状态窗口

**状态：已完成首版。**

**改哪里**

1. `src/host/windows-host/src/main.rs`
2. 新增 `src/host/windows-host/src/slint_ui.rs`
3. 后续如保留浏览器页，再新增 `src/host/windows-host/ui/`

**改成什么**

1. 短期：Slint 状态窗口显示：
   - host 健康状态
   - relay 状态
   - 题库版本
   - 最近执行结果
   - 数据目录
2. 中期：如需要浏览器访问，再增加 `GET /ui` 和 `GET /ui/status`。
3. 当前默认 host 已内置 `/ui` 与 `/ui/status`，用于展示 host、relay、题库、最近执行摘要和能力边界。

**产出**

1. `--slint-ui` 可打开原生状态窗口。
2. 控制台启动时继续提示 localhost API 地址。
3. 浏览器可打开 `http://127.0.0.1:4510/ui`。
4. `/ui/status` 不展示题库答案、密码、私钥、签名 key bytes 或故事原文。

**验收**

1. `powershell -File scripts/windows/check_windows_host_loop.ps1 -Port 4511`
2. `cargo check --features ui-slint`
3. 手动打开 `/ui`

### B3. Slint 请求确认页

**状态：已完成首版。**

**改哪里**

1. `src/host/windows-host/src/main.rs`
2. `src/host/windows-host/src/slint_ui.rs`

**改成什么**

1. 将 Windows Yes/No MessageBox 升级为 Slint 请求详情确认窗口。
2. 展示字段：
   - capability
   - objectRef
   - requester / origin
   - requiredStrength
   - expiry
   - 风险说明
3. 明确 Approve / Deny 行为。
4. 九宫格挑战进入 Slint 窗口，验证通过后才签发短时授权会话。
5. 当前已在默认 host 中生成确认详情摘要，并通过 `/ui/status` 和 `/ui` 展示最近确认请求。
6. 当前已支持 `STORYLOCK_WINDOWS_APPROVAL_MODE=slint_dialog`，在 `ui-slint` feature 构建中打开原生 Approve / Deny 窗口；无 feature 时退回详细 Windows 确认弹窗。

**验收**

1. 本地触发 `POST /execute`。
2. 用户能看到请求详情，而不是只有 Yes/No。
3. UI 不展示题库答案或敏感原文。
4. `npm run test:windows-host` 验证最近确认请求摘要包含 requestId、capability、requiredStrength 和风险说明。
5. `cargo check --features ui-slint` 验证 Slint 确认窗口可编译。

### B4. 托盘与退出控制

**状态：已完成首版。**

**改哪里**

1. `src/host/windows-host/Cargo.toml`
2. `src/host/windows-host/src/main.rs`
3. 新增 `src/host/windows-host/src/tray_ui.rs`
4. `scripts/windows/check_windows_host_loop.ps1`
5. 新增 `scripts/windows/check_windows_host_features.ps1`
6. `scripts/release/windows/build_windows_host.ps1`
7. `src/host/windows-host/start-yian-windows-host.cmd`

**改成什么**

1. 当前已增加 `GET /diagnostics`，输出仅含本地 host、UI、relay、题库和最近请求的脱敏诊断信息。
2. 当前已增加 `POST /shutdown`，用于本机管理页或后续托盘菜单安全退出 host，并释放 4510 端口。
3. 当前已增加可选 feature：`ui-tray`。
4. `cargo run --features ui-tray -- --tray` 启动本地 server、后台 relay 和系统托盘图标。
5. 托盘菜单项：
   - 打开本地管理页
   - 查看健康状态
   - 复制诊断信息
   - 退出
6. Windows release 构建默认带 `ui-tray` feature，启动脚本可用 `STORYLOCK_WINDOWS_START_MODE=tray` 切换托盘模式。

**验收**

1. `npm run test:windows-host` 已覆盖 `/diagnostics`、脱敏字段、`/shutdown` 和端口释放。
2. `npm run test:windows-host-features` 已覆盖默认构建、`ui-slint`、`ui-tray`、`ui-slint ui-tray` 组合编译。
3. 后续人工验收 Windows 运行后托盘可见、菜单可打开本地管理页、复制诊断和退出；入口为 `scripts\windows\start_windows_host_tray_manual_check.cmd`，记录到 `docs\test\Windows托盘人工验收记录_20260620.md`。

### B5. Windows 包内容检查

**状态：已完成。**

**改哪里**

1. 新增 `scripts/test/windows-package-contents.mjs`
2. `package.json`

**改成什么**

1. 检查 zip 至少包含：
   - `yian-windows-host.exe`
   - `README.md`
   - `start-yian-windows-host.cmd`
2. 未来 MSI 存在时检查 installer metadata。

**验收**

1. `npm run test:windows-package`
2. `npm test` 纳入该检查。

## 5. C 组：StoryLock 工作台功能

### C1. 故事草稿界面

**改哪里**

1. 新增或扩展 `src/ui/`
2. 调用 `src/skills/local-story-processing/index.js`

**改成什么**

1. 表单字段：
   - objective
   - audience
   - tone
   - constraints
   - source
2. 输出结构化故事草稿。

**验收**

1. 本地 UI 可调用 `StoryDraftSkill`。
2. 不触碰受保护对象。

### C2. 故事润色界面

**改哪里**

1. `src/ui/`
2. `src/skills/local-story-processing`

**改成什么**

1. 输入已有草稿。
2. 选择润色目标。
3. 输出修改建议和润色结果。

**验收**

1. `StoryRefineSkill` 可从 UI 触发。

### C3. 题集强度评估界面

**改哪里**

1. `src/ui/`
2. `src/skills/local-story-processing`
3. `src/skills/local-story-access/assets/schemas/question-set-master.schema.json`

**改成什么**

1. 上传/粘贴问题集。
2. 展示：
   - 题目数量
   - 每题候选项情况
   - 强度判断
   - 缺口建议

**验收**

1. `StrengthReviewSkill` 可从 UI 触发。
2. 问题集可进入第二层导入前检查流程。

## 6. D 组：远程能力与安全链路

### D1. 远程能力清单固化

**改哪里**

1. `src/skills/remote-gateway/assets/agent-capabilities.json`
2. `src/skills/remote-gateway/SKILL.md`
3. `docs/design/cn/Skill定位与边界.md`

**改成什么**

1. 明确允许：
   - `requestSignature`
   - `requestPasswordFill`
2. 明确禁止：
   - story read/write
   - raw secret read
   - raw private key read
   - submit grid answers remotely

**验收**

1. `npm run selftest --prefix src/skills/remote-gateway`
2. `npm run check:agent-capabilities --prefix src/skills/remote-gateway`

### D2. Replay / expiry / nonce 负面测试

**改哪里**

1. `src/skills/remote-gateway/scripts/selftest.mjs`
2. `src/skills/local-story-access/scripts/selftest.mjs`

**改成什么**

1. 增加缺失 nonce。
2. 增加过期 expiry。
3. 增加重复 requestId。
4. 增加重复 nonce。
5. 增加脱敏失败。

**验收**

1. `node scripts/test/run-selftests.mjs`

## 7. E 组：Android / Linux 平台推进

### E1. Android release 安全链路

**改哪里**

1. `src/host/android-host/`
2. `scripts/android/`
3. `scripts/release/android/`

**改成什么**

1. Keystore 签名 release 验收。
2. BiometricPrompt 失败路径。
3. relay 超时路径。
4. debug / release 包差异说明。

**验收**

1. `npm run selftest:web-api-android`
2. Android 真机检查清单。

### E2. Linux 真环境验收

**改哪里**

1. `src/host/linux-host/`
2. `scripts/linux/`
3. `scripts/release/linux/`

**改成什么**

1. Secret Service 真桌面环境检查。
2. `.deb` 安装、启动、卸载验证。
3. systemd user unit 验证。

**验收**

1. `npm run test:linux-host`
2. `npm run test:linux-package`
3. 真 Linux 桌面人工验收记录。

## 8. F 组：发布与部署闭环

### F1. 发布后线上检查

**改哪里**

1. `scripts/vercel/preflight.ps1`
2. `scripts/test/site-http-smoke.mjs`

**改成什么**

1. 检查首页是否含 `#help`。
2. 检查 `/downloads/yian-windows-host-0.1.0-1-prototype.zip` 返回 200。
3. 下载 zip 并检查包内容。
4. 检查 metadata checksum 与 zip 一致。

**验收**

1. `npm run test:site-http`
2. 线上 preflight。

### F2. downloads 来源规则固化

**改哪里**

1. `scripts/vercel/build_yian_web.mjs`
2. `.vercelignore`
3. `src/yian-web/public/downloads/`
4. `release/app/`

**改成什么**

1. 明确 Vercel 远程构建时不能依赖被 `.vercelignore` 排除的 `release/app`。
2. 若发布包需要进入线上，必须：
   - 同步到 `src/yian-web/public/downloads/`，或
   - 调整 CI / Vercel 上传策略。
3. 构建脚本继续生成 metadata。

**验收**

1. 新 Windows zip 线上下载内容与本地一致。
2. `npm run test:release` 通过。

### F3. 部署入口稳定化

**改哪里**

1. `scripts/vercel/publish_site_release.ps1`
2. `scripts/vercel/publish_site_release_wsl.ps1`
3. `.github/workflows/vercel-production.yml`

**改成什么**

1. Windows 本机入口继续保留 OIDC/TLS 错误提示。
2. WSL 入口作为当前可用发布入口。
3. CI token 入口作为正式发布入口。

**验收**

1. WSL 可部署。
2. CI 可手动触发。
3. 部署日志记录 deployment URL。

## 9. 文档同步清单

- [ ] `docs/ref/08-易安部署与域名说明.md` 更新当前线上已恢复状态。
- [x] `src/host/windows-host/README.md` 强调原型包和完整 UI 的区别。
- [x] `docs/test/StoryLock平台验收矩阵_20260620.md` 增加 Windows zip 内容检查。
- [ ] `docs/design/cn/Skill定位与边界.md` 保持远程能力只有两个主线出口。
- [ ] `docs/management/README.md` 作为管理文档入口。

## 10. 优先执行顺序

1. A1 / A3：先修正网页说明，避免用户误解。
2. B1 / B5：修正 Windows 路径与包内容检查。
3. A2：请求状态页产品化。
4. B2 / B3：Windows 本地管理页和请求确认页。
5. F1 / F2：发布后线上检查和 downloads 来源规则。
6. D2：安全负面测试。
7. E1 / E2：Android / Linux 真环境验收。
8. C1 / C2 / C3：StoryLock 工作台。

## 11. 每次交付前最小验收

```powershell
npm run build
npm run test:release
npm run test:site-http
npm run test:scripts
```

涉及 Windows host 时追加：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\windows\check_windows_host_loop.ps1 -Port 4511
```

涉及完整主线时追加：

```powershell
npm run selftest
```

部署后追加线上检查：

1. 首页 200。
2. `#help` 存在。
3. Windows zip 200。
4. Windows zip 内容包含启动脚本。
5. Windows metadata checksum 与 zip 一致。
