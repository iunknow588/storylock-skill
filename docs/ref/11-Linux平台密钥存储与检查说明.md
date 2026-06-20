# Linux 平台密钥存储与检查说明

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.0 |
| 日期 | 2026-06-20 |
| 当前状态 | 当前主线参考材料 |
| 适用范围 | Linux 平台 SecretStore、第二层 Host 启动检查、Linux 本地宿主原型、发布目录预留 |

## 1. 目的

本文档用于补齐 Linux 平台在当前仓库中的真实材料口径，避免把“已经有 Linux SecretStore 适配”误写成“已经有 Linux 正式宿主交付”。

当前应明确区分：

1. 已经实现 Linux `SecretStore` 适配与检查脚本。
2. 已经具备最小 Linux 本地宿主原型和本地 loop 自测。
3. 已经预留 Linux 安装包目录，并可生成原型归档包。
4. 已经具备原型级 desktop entry、systemd user unit、Debian staging 与 WSL 打包入口。
5. 尚未完成 Linux 签名发布链路、Secret Service 真环境验收与真实发行版验收。

## 2. 当前已实现内容

当前仓库中，Linux 相关主线实现包括：

1. `src/shared/secret-store.js`
   - `LinuxSecretServiceStore`
   - `createPlatformSecretStore({ platform: "linux" })`
2. `src/skills/local-story-access/access-host.js`
   - Host 启动时对平台 SecretStore 显式执行 `checkAvailable()`
3. `src/skills/local-story-access/scripts/check-secret-store.mjs`
   - 平台存储检查报告脚本
4. `release/app/linux/`
   - Linux 安装包目录预留
   - 当前可生成 Linux 原型归档
   - 在 Linux/WSL 依赖满足时可生成 `tar.gz` 与 `.deb` 原型包
5. `scripts/vercel/build_yian_web.mjs`
   - 可识别 Linux 安装包扩展名与平台名
6. `src/host/linux-host/`
   - Node.js Linux 本地宿主原型
   - 复用 `src/skills/local-story-access` 的题库、challenge、session 与撤销逻辑
7. `scripts/linux/check_linux_host_loop.mjs`
   - 本地闭环自测：`health -> question-bank -> verify -> authorize -> execute -> revoke`
8. `src/host/linux-host/bin/yian-linux-host`
   - 生产模式启动脚本，默认启用 Linux 平台 SecretStore
9. `src/host/linux-host/desktop/` 与 `src/host/linux-host/systemd/`
   - 原型级桌面入口与 systemd user unit

## 3. Linux SecretStore 当前行为

当前 Linux 适配使用：

1. `secret-tool`
2. Secret Service / libsecret 兼容桌面密钥环

当前支持语义：

1. `getSecret`
2. `setSecret`
3. `deleteSecret`
4. `checkAvailable`

当前限制：

1. `listKeys` 不支持。
2. 需要用户会话中存在可访问的 Secret Service。
3. 没有 Secret Service 的纯服务器环境不能视为当前生产就绪路径。

## 4. 启动期检查口径

当前主线要求：

1. 当第二层 Host 显式使用平台 SecretStore 时，启动阶段必须先执行 `checkAvailable()`。
2. 如果 `secret-tool` 不存在，必须直接失败。
3. 如果 Secret Service 不可访问，也必须直接失败。
4. 不允许在生产路径静默回退到 `MemorySecretStore`。

当前 Linux 失败提示重点包括：

1. `secret-tool is required but was not found in PATH`
2. `Secret Service is unavailable: ...`

## 5. 推荐检查命令

在 `skill/` 目录下可直接执行：

```powershell
node src/skills/local-story-access/scripts/check-secret-store.mjs
```

检查结果重点看：

1. `platform`
2. `adapter`
3. `status`
4. `productionReady`
5. `recommendation`

## 6. Linux 安装包目录口径

当前目录：

1. `release/app/linux/`

当前用途：

1. 预留 Linux `.AppImage`
2. 预留 Linux `.deb`
3. 预留 Linux `.rpm`
4. 预留压缩归档包

当前不能对外表述为：

1. 已经完成 Linux 正式宿主
2. 已经完成 Linux 正式安装包发布
3. 已经完成 Linux 真机或正式发行版交付验收

## 7. Linux 本地宿主原型

当前 Linux 原型入口：

1. `src/host/linux-host/server.mjs`
2. `src/host/linux-host/assets/question-bank.json`
3. `npm run test:linux-host`
4. `npm run package:linux-host`
5. `npm run package:linux-host:wsl`

当前原型支持：

1. `GET /health`
2. `GET /question-bank/status`
3. `POST /question-bank/import`
4. `POST /verify`
5. `POST /authorize`
6. `POST /execute`
7. `POST /revoke`

当前原型默认使用 development memory SecretStore，以便在 Windows 开发机与 CI 环境中验证本地闭环。Linux 真环境应改用：

```powershell
$env:STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE="1"
node src/host/linux-host/server.mjs
```

使用平台 SecretStore 时，仍需满足 `secret-tool` 与 Secret Service 可用。

WSL 打包入口：

```powershell
npm run package:linux-host:wsl
```

该入口会在 WSL 中检查 Node.js 与 `dpkg-deb`，并复用 `scripts/release/linux/package_linux_host.mjs` 生成 Linux 原型包与 Debian 包。若 WSL 内缺少 Node.js >=22 或 `dpkg-deb`，脚本会直接失败并提示补齐环境。

## 8. 建议对外口径

建议使用：

1. 当前仓库已经具备 Linux 平台 SecretStore 适配与检查材料。
2. 当前仓库已经具备 Linux 本地宿主最小原型与可重复本地 loop 自测。
3. 当前仓库已经预留 Linux 安装包目录与站点元数据识别入口。
4. Linux 原型级桌面集成与 Debian staging 已具备，正式签名包、Secret Service 真环境验收与发布闭环仍属于后续增强项。
5. 当前 Windows 本机已实际调用 `npm run package:linux-host:wsl`，脚本会加载 WSL 内 `nvm` 并选择已安装的最高 Node.js `>=22`；当前实测使用 Node.js `v24.17.0`，已成功生成 Linux `tar.gz` 与 `.deb` 原型包。
6. 当前站点构建会为 Linux `.deb` 与 `tar.gz` 分别生成 downloads 元数据，避免同版本多格式包覆盖同一个 JSON。

避免使用：

1. 已经支持 Linux 正式宿主 App。
2. Linux 平台已经完成交付级签名与发布。
3. Linux 与 Android / Windows 一样已经完成正式交付闭环。

## 9. 当前阶段结论

当前 Linux 这条线的真实状态是：

1. 平台密钥存储适配已存在。
2. 启动期依赖守门已纳入主线。
3. 最小 Linux 本地宿主原型已存在，并可通过 `npm run test:linux-host` 验证本地闭环。
4. 发布目录与站点识别材料已预留。
5. 原型归档包、`.deb` 原型包、desktop entry、systemd user unit、Debian staging 与 WSL 打包入口已存在。
6. WSL 打包闭环已验证到 `tar.gz`、`.deb`、manifest artifact 与站点 downloads metadata。
7. 正式 Linux 签名包、Secret Service 真环境验收与发布闭环尚未完成。
