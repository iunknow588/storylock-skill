# StoryLock 真机真实桌面验收待办

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-22 |
| 范围 | Windows 托盘/Slint、Android 真机、Linux Secret Service 真环境 |
| 当前状态 | 非真机验收已达到 `non_device_ready`；本文件只保留真实设备或真实桌面环境必须执行的剩余项 |

## 1. 当前总状态

先运行：

```powershell
npm run status:platform-validation
```

当前预期：

1. `status` 为 `non_device_ready`。
2. `nonDeviceReady` 为 `true`。
3. `canArchiveManagement` 仍为 `false`。
4. `requiredBeforeNonDeviceReady` 为空。
5. `requiredBeforeArchive` 只包含真实设备或真实桌面相关项。

## 2. 已完成的非真机闭环

以下内容不再作为开发阻塞项：

1. Windows 本地宿主 API 自动闭环。
2. Windows 托盘/Slint feature 编译与托盘手工入口守门。
3. Windows 托盘预检报告生成：`.temp/windows-tray-manual-report.local.md`。
4. Android 代码准备度、Manifest、Keystore、BiometricPrompt、题集和 permission summary asset 检查。
5. Android 本机探测报告生成：`.temp/android-device-loop-report.local.md`。
6. Linux host `/health`、`/permission-summary`、题库导入、verify、authorize、execute、revoke 自动闭环。
7. Linux WSL 打包、`.tar.gz` / `.deb` 原型包、包内容与桌面集成材料检查。
8. Linux Secret Service WSL 诊断报告生成：`.temp/linux-secret-service-wsl-report.local.md`。
9. StoryLock package、schema、validate CLI、inspect CLI、文档一致性和路径一致性检查。

统一入口：

```powershell
npm run test:non-device-validation
```

## 3. 剩余真实环境验收

| 编号 | 平台 | 剩余项 | 当前阻塞 | 验收入口 | 记录位置 |
| --- | --- | --- | --- | --- | --- |
| REAL-01 | Windows | 托盘图标、菜单、复制诊断、退出释放端口 | 需要人工桌面交互 | `scripts\windows\start_windows_host_tray_manual_check.cmd` | `docs\test\Windows托盘人工验收记录_20260620.md` |
| REAL-02 | Windows | Slint `StoryLock Core` 真实窗口体验 | 需要人工桌面交互 | Windows Host / StoryLock Core 子应用 | `docs\test\Windows托盘人工验收记录_20260620.md` |
| REAL-03 | Android | 真机安装、deep link 唤起、relay | 当前机器未连接 Android 设备 | `scripts\android\check_device_loop.ps1` | `docs\test\Android真机验收记录_20260622.md` |
| REAL-04 | Android | 通过 `adb forward` 实测 `/health` 与 `/permission-summary` | 当前机器未连接 Android 设备 | `scripts\android\check_device_loop.ps1` | `docs\test\Android真机验收记录_20260622.md` |
| REAL-05 | Linux | `secret-tool` 与 Secret Service 可用性 | 当前 WSL 未安装 `secret-tool`，且非交互 sudo 不可用 | `npm run diagnose:linux-secret-service:wsl` | `docs\test\Linux桌面WSL验收记录_20260622.md` |
| REAL-06 | Linux | 生产模式启动与 systemd user service / desktop entry 真实桌面表现 | 需要真实 Linux 桌面或配置完整的 WSL 用户会话 | `node src/host/linux-host/server.mjs` / systemd user service | `docs\test\Linux桌面WSL验收记录_20260622.md` |

## 4. 完成判定

只有以下条件全部满足，才可把 20260621 阶段 management 文档移动到 `BACK/`：

1. `docs/test/Windows托盘人工验收记录_20260620.md` 中 TRAY-01 到 TRAY-05 全部为通过。
2. `docs/test/Android真机验收记录_20260622.md` 中 AND-01 到 AND-09 全部为通过。
3. `docs/test/Linux桌面WSL验收记录_20260622.md` 中 LIN-05 到 LIN-10 的真实环境项完成或有明确替代验收记录。
4. `npm run status:platform-validation` 输出 `ready_to_archive`。

## 5. 不可替代边界

以下事项不得用纯自动化结果替代：

1. Windows 通知区域托盘图标是否可见。
2. Windows 托盘菜单项是否能被用户正常点击。
3. Android 真机上的 deep link、BiometricPrompt、本地 challenge 和 relay 表现。
4. Linux 桌面会话中的 Secret Service、systemd user service、desktop entry 行为。
