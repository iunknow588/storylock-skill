# StoryLock 平台验收矩阵

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.0 |
| 日期 | 2026-06-20 |
| 适用范围 | Android 宿主、Windows 宿主、发布元数据、交付前检查 |
| 目的 | 把平台交付前的自动检查、人工检查和记录材料固定为同一张矩阵 |

2026-06-22 自动化复核记录：`npm run test:android-readiness`、`npm run test:windows-package`、`npm run test:linux-host`、`node scripts\test\docs-consistency.mjs`、`node scripts\verify\path-consistency.mjs` 均已通过。其中 Linux host 闭环已从 9 题测试数据升级为 24 题数据，覆盖 `requestSignature` 的 12 格 challenge。

## 1. 使用原则

1. 自动检查必须能在仓库内重复执行。
2. 真机、证书、商店、对象存储等外部条件必须明确记录为人工检查。
3. debug/internal、prototype、candidate、release 不得混用口径。
4. 每次交付前都必须记录版本号、versionCode、checksum、packageKind、releaseChannel。

## 2. 自动检查矩阵

| 编号 | 范围 | 检查项 | 命令 | 通过标准 |
| --- | --- | --- | --- | --- |
| AUTO-01 | 主线 selftest | 三层主线、Android Web API mock、UI/engine 自测 | `npm run selftest` | 所有 selftest 通过 |
| AUTO-11 | Android 代码准备度 | Android 项目结构、Manifest、Keystore、Biometric、relay、题集和 APK 构建脚本 | `npm run test:android-readiness` | 不连接真机，仅验证 Android 代码与资源准备项 |
| AUTO-12 | 三端真测前准备度 | Android / Windows / Linux 非真机检查、包内容、桌面集成、发布元数据和交付入口 | `npm run test:platform-readiness` | 全部通过后进入真机、真实桌面和真实 Linux 环境验收 |
| AUTO-14 | 非真机验收闭环 | 生成 Windows 预检、Android 本机探测、Linux Secret Service 诊断，并运行 package/schema/CLI/平台/文档/路径检查 | `npm run test:non-device-validation` | 输出 `non_device_ready` 或 `ready_to_archive` 表示真机/真实桌面之外内容已完成 |
| AUTO-13 | 平台验收状态汇总 | 汇总 Windows 人工记录、Android 真机记录、Linux WSL/Secret Service 记录 | `npm run status:platform-validation` | 输出 `ready_to_archive` 前不得归档当前 management 文档；输出 `non_device_ready` 表示仅剩真机/真实桌面项 |
| AUTO-15 | 真实环境剩余待办 | 集中列出非真机闭环后的 Windows、Android、Linux 真实环境剩余项 | `docs\management\StoryLock真机真实桌面验收待办_20260622.md` | 所有 REAL 项完成后再尝试归档 |
| AUTO-02 | 文档一致性 | 当前文档不重新引入旧接口和旧路径 | `npm run test:docs` | 输出 `status=passed` |
| AUTO-03 | 路径一致性 | 仓库文本不引用旧 skill 包路径 | `npm run test:paths` | 输出 `status=passed` |
| AUTO-04 | 发布元数据 | downloads JSON 与 Android/Windows/Linux 产物大小、SHA-256 一致 | `npm run test:release` | 覆盖 `android`、`windows` 和 `linux` |
| AUTO-05 | 交付矩阵入口 | 本矩阵、真机文档、平台脚本和发布脚本存在 | `npm run test:delivery` | 输出 `status=passed` |
| AUTO-09 | Windows 包内容 | Windows prototype zip 内必须包含 exe、README 和启动脚本 | `npm run test:windows-package` | 输出 `status=passed` |
| AUTO-10 | Windows 托盘验收准备 | 托盘人工验收脚本、记录模板和 `ui-tray` 编译入口 | `npm run test:windows-tray-readiness` | 不启动桌面托盘，仅验证准备项与 feature 编译 |
| AUTO-08 | Linux 本地宿主闭环 | Linux 原型 host 的 health、题库、授权与执行路径 | `npm run test:linux-host` | 输出 `status=passed` |
| AUTO-06 | 文本编码 | 文档与脚本编码无异常 | `npm run test:text` | 编码检查通过 |
| AUTO-07 | 契约 Schema | 当前 JSON Schema 可解析且主接口枚举一致 | `npm run test:contract` | Schema 检查通过 |

## 3. Android 真机验收矩阵

| 编号 | 检查项 | 类型 | 入口 | 通过标准 |
| --- | --- | --- | --- | --- |
| AND-01 | APK 构建 | 自动/本地 | `scripts\release\android\build_apk.cmd -Variant debug` | 生成 `release\app\android\storylock-android-host-<version>-<versionCode>-debug.apk` |
| AND-02 | release 构建候选 | 自动/本地 | `scripts\release\android\build_apk.cmd -Variant release` | 生成 candidate APK，并写入 `.temp\vercel\android-package.env` 和 `.temp\vercel\output.json` |
| AND-03 | 元数据对齐 | 自动 | `npm run build` 后执行 `npm run test:release` | 网站 downloads JSON 与 APK 一致 |
| AND-04 | 下载入口 | 人工/接口 | `/app/download/android` | 返回 APK 或明确跳转到配置下载地址 |
| AND-05 | 绑定入口 | 人工/接口 | `/app/bind?identityId=...&preferredMode=relay_url` | 返回 binding token 或 deep link |
| AND-06 | 安装与唤起 | 真机 | `scripts\android\check_device_loop.ps1` | `adb install` 与 deep link 唤起成功 |
| AND-07 | relay 闭环 | 真机 | Android 宿主轮询网关 | 注册、poll、respond 均成功 |
| AND-08 | challenge 失败路径 | 真机 | 本地 challenge UI | 错误、取消、锁定返回结构化错误 |
| AND-09 | 脱敏返回 | 真机/接口 | `/api/storylock-gateway`，记录到 `docs\test\Android真机验收记录_20260622.md` | password、privateKey、signingKeyBytes 不明文返回 |

## 4. Windows 本地宿主验收矩阵

| 编号 | 检查项 | 类型 | 入口 | 通过标准 |
| --- | --- | --- | --- | --- |
| WIN-01 | 本地宿主闭环 | 自动/本地 | `npm run test:windows-host` 或 `scripts\windows\check_windows_host_loop.cmd` | health、question-bank、verify、authorize、execute、revoke 均成功且关键字段非空 |
| WIN-02 | prototype 包构建 | 自动/本地 | `scripts\release\windows\release_windows_host.cmd` | 生成 zip 和 release manifest |
| WIN-03 | MSI 候选 | 自动/本地 | `scripts\release\windows\release_windows_host.ps1 -BuildMsi` | 本地有 WiX 时生成 MSI 与 checksum |
| WIN-04 | 签名入口 | 人工/本地 | `scripts\release\windows\sign_windows_package.ps1` | 配置证书后签名成功 |
| WIN-05 | manifest 转 env | 自动/本地 | `scripts\release\windows\manifest_to_windows_env.ps1` | `.temp\vercel\windows-package.env` 与 `.temp\vercel\output.json` 字段完整 |
| WIN-06 | 发布准备 | 自动/本地 | `scripts\release\windows\publish_windows_release.cmd` | 生成发布摘要、上传清单和可复制产物目录 |
| WIN-07 | 下载入口 | 人工/接口 | `/app/download/windows` | 返回 Windows 包或明确跳转 |
| WIN-08 | 元数据对齐 | 自动 | `npm run build` 后执行 `npm run test:release` | Windows JSON 与 zip/msi/exe 一致 |
| WIN-09 | prototype 包内容 | 自动 | `npm run test:windows-package` | zip 内包含 `yian-windows-host.exe`、`README.md`、`start-yian-windows-host.cmd` |
| WIN-10 | 托盘桌面体验 | 人工/桌面 | `scripts\windows\start_windows_host_tray_manual_check.cmd`，记录到 `docs\test\Windows托盘人工验收记录_20260620.md` | 托盘图标可见，菜单可打开管理页、查看健康、复制脱敏诊断并退出释放端口 |

## 5. Linux 本地宿主验收矩阵

| 编号 | 检查项 | 类型 | 入口 | 通过标准 |
| --- | --- | --- | --- | --- |
| LIN-01 | 本地宿主闭环 | 自动/本地 | `npm run test:linux-host` | health、question-bank、verify、authorize、execute、revoke 均成功且关键字段非空 |
| LIN-02 | SecretStore 适配 | 自动/环境 | `node src/skills/local-story-access/scripts/check-secret-store.mjs` 或 `npm run diagnose:linux-secret-service:wsl` | Linux 真环境中 `secret-tool` 与 Secret Service 可用 |
| LIN-03 | 原型服务启动 | 自动/本地 | `node src/host/linux-host/server.mjs` | 服务监听 `127.0.0.1` 并返回 `/health` |
| LIN-04 | 题库导入 | 自动/本地 | `POST /question-bank/import` | 支持 UTF-8/BOM JSON，导入后 active 题数不少于 24，可支撑 12 格高强度 challenge |
| LIN-05 | 原型包生成 | 自动/本地 | `npm run package:linux-host` | 生成 Linux 原型归档与 manifest；Windows 下默认 zip，Linux/WSL 下默认 `tar.gz` |
| LIN-06 | 包内容校验 | 自动/本地 | `npm run test:linux-package` | `tar.gz` 或 zip 内包含 Linux host、题库、local-story-access 与 shared 模块；`.deb` 存在时同步检查 Debian 包内容 |
| LIN-07 | 元数据对齐 | 自动 | `npm run build` 后执行 `npm run test:release` | Linux downloads JSON 与 `.deb`、`tar.gz` 等实际产物分别一致 |
| LIN-08 | 桌面集成 | 自动/本地 | `npm run test:linux-desktop` | executable、desktop entry、systemd user unit、Debian control 与 manifest 一致 |
| LIN-09 | WSL 打包脚本检查 | 自动/本地 | `npm run test:linux-wsl` | WSL 包装脚本包含路径转换、Node.js >=22、`dpkg-deb` 与 Linux 打包脚本调用检查 |
| LIN-10 | WSL Debian 打包 | 自动/WSL | `npm run package:linux-host:wsl` | WSL 中 Node.js >=22 与 `dpkg-deb` 可用时生成 `.deb` 与 `tar.gz`，manifest 记录两个 artifact |
| LIN-11 | 正式包预留 | 人工/发布 | `release/app/linux/` | 后续 `.AppImage`、`.deb`、`.rpm` 或正式归档包落入该目录 |
| LIN-12 | 生产限制 | 人工/文档 | `docs/ref/11-Linux平台密钥存储与检查说明.md`，记录到 `docs\test\Linux桌面WSL验收记录_20260622.md` | 不把原型 host 宣称为正式 Linux 发布闭环 |

当前 Windows 本机已实际执行 `npm run package:linux-host:wsl`，脚本通过 WSL 内 `nvm` 选择最高可用 Node.js `>=22`，当前使用 Node.js `v24.17.0`，LIN-10 已生成 `.deb` 与 `tar.gz` 原型包。后续仍需真实 Linux 桌面环境安装、Secret Service 与签名发布验收。

## 6. 发布前检查清单

| 编号 | 检查项 | Android | Windows | Linux |
| --- | --- | --- | --- | --- |
| REL-01 | versionName / versionCode 已确认 | 必填 | 必填 | 必填 |
| REL-02 | packageKind 已确认 | `debug` / `release` | `zip` / `msi` / `exe` | `deb` / `tar.gz` / `AppImage` / `rpm` |
| REL-03 | releaseChannel 已确认 | `internal` / `candidate` / `release` | `prototype` / `candidate` / `release` | `prototype` / `candidate` / `release` |
| REL-04 | SHA-256 checksum 已生成 | 必填 | 必填 | 必填 |
| REL-05 | downloads JSON 已生成 | 必填 | 必填 | 必填 |
| REL-06 | 网站构建已同步产物 | `npm run build` | `npm run build` | `npm run build` |
| REL-07 | 发布元数据测试通过 | `npm run test:release` | `npm run test:release` | `npm run test:release` |
| REL-08 | 平台验收入口存在 | `npm run test:delivery` | `npm run test:delivery` | `npm run test:delivery` |
| REL-09 | 安装验证已记录 | 真机报告 | 本地宿主报告 | Linux 桌面报告 |
| REL-10 | 回滚策略已记录 | 保留上一 APK 与元数据 | 保留上一包与 manifest | 保留上一包与 manifest |
| REL-11 | 平台专项检查通过 | `scripts\android\check_device_loop.ps1` | `npm run test:windows-host` | `npm run test:linux-host`、`npm run test:linux-package`、`npm run test:linux-desktop` |

## 7. 当前阶段结论口径

如果本矩阵全部自动项通过，可以表述为：

> StoryLock 当前已经具备可重复执行的平台交付前检查入口，Android/Windows/Linux 产物元数据可以被自动核对。

仍然不要表述为：

1. Android 正式移动端发布闭环已经完成。
2. Windows 生产证书签名与自动升级已经完成。
3. Linux 正式签名发布与 Secret Service 真环境验收已经完成。
4. 真机验收可以被纯脚本完全替代。
