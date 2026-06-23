# StoryLock 管理文档

本目录保留当前有效的后续开发管理文档和进行中的任务配置。这里不是历史垃圾目录，后续新的任务计划、实施清单和验收记录仍放在这里。

## 当前主文档

1. `StoryLock设计对齐后续开发计划_20260621.md`
   - 当前执行基线。用于对齐 `story-lock/doc/design` 与 `skill/src` 的完成度，明确未完成的 StoryLock Core 正式数据包、资源目录、模板、校验 CLI、故事工作台持久化和多平台宿主任务。

2. `StoryLock设计对齐后续开发实施清单_20260621.md`
   - 当前执行清单。用于把 P0-P4 计划拆解成可编码、可测试、可验收的任务。

3. `StoryLock文档对齐后续开发计划_20260621.md`
   - 当前文档对齐总账。用于说明 `story-lock/doc` 与 `skill/docs` 的差异、已修复的文档入口问题，以及后续需要补齐的架构接线、usecase 映射、命名归档规则和文档-代码-测试映射表。

4. `StoryLock文档命名与归档规则_20260621.md`
   - 当前文档治理规则。用于说明 `docs/design`、`docs/management`、`docs/management/BACK`、`docs/test`、`docs/ref` 的边界和归档条件。

5. `StoryLock文档代码测试映射表_20260621.md`
   - 当前发布前检查总表。用于把上游设计、当前代码路径、测试命令和剩余验收项放到同一张表中。

6. `StoryLock真机真实桌面验收待办_20260622.md`
   - 当前剩余真实环境验收入口。用于在 `npm run test:non-device-validation` 已达到 `non_device_ready` 后，集中记录 Windows、Android、Linux 仍需真实设备或真实桌面完成的项目。

## 历史归档

`BACK/` 目录保存历史分析、旧版计划和评审意见，仅作为参考，不作为当前执行基线。20260620 阶段计划和实施清单已归档到 `BACK/`，当前工作以 20260621 文档为准。

## 当前执行重点

1. Windows Slint `StoryLock Core` 子应用继续作为底层配置入口，主 Host 只读取脱敏权限摘要，不能直接编辑故事原文、答案、密码、私钥或加密配置。
2. 补齐 Windows、Android、Linux 真机或桌面验收记录。
3. 远程网关继续只暴露 `requestSignature` 和 `requestPasswordFill` 两个安全出口，不开放远程故事读写。

## 当前归档判断

截至 2026-06-21，本目录尚未整体处理完毕，暂不移动当前主文档到 `BACK/`。

2026-06-22 复核：P0/P1/P4 相关自动化接线检查已重新执行；`npm run test:linux-host` 曾因 Linux host 默认/导入题库只有 9 个 active questions、无法支撑 `requestSignature` 高强度 12 格 challenge 而失败，现已将 Linux 默认题库和测试导入夹具补齐到 24 题，并修正测试答案按 `questionId` 映射提交。复核后 Linux host health、`/permission-summary`、题库导入、verify、authorize、execute、revoke 自动闭环通过。

当前仍不归档：缺少 Windows Slint/托盘桌面人工验收、Android 真机 `/permission-summary` 实测结果、Linux Secret Service 真环境验收结果。Android 与 Linux 的记录模板已补入 `docs/test`；2026-06-22 已完成 WSL 打包和 Linux 包/桌面材料自动复核，但 Android 当前机器未连接设备，Linux 当前 WSL 未安装 `secret-tool`。

2026-06-22 非真机闭环补充：`npm run test:non-device-validation` 已达到 `non_device_ready`，剩余项目已集中转入 `StoryLock真机真实桌面验收待办_20260622.md`。当前仍不归档，因为 `ready_to_archive` 需要真实设备或真实桌面验收完成。

已闭环：

1. StoryLock package 第一版共享加载、schema、校验 CLI 和权限摘要说明。
2. 平台宿主与数据包接线说明。
3. 用例样例到测试夹具映射说明。
4. 文档命名与归档规则。
5. 文档-代码-测试映射表。
6. Windows package、Android readiness、Linux host、文档一致性和路径一致性自动化检查。
7. 真机实测之外的验收材料生成和自动化平台检查，可通过 `npm run test:non-device-validation` 一次完成；该命令覆盖 Windows 预检、Android 本机探测、Linux Secret Service 诊断、StoryLock package、schema、CLI、平台 readiness、文档一致性和路径一致性。

仍待闭环：

1. Windows 桌面托盘和 Slint UI 人工验收记录。
2. Android 真机运行和 `/permission-summary` 实测结果。
3. Linux Secret Service 真环境运行验收结果。

完成以上人工验收后，可将 20260621 阶段计划、实施清单和对齐总账移动到 `BACK/`，并在本目录保留新的当前任务入口。

## 人工验收入口

0. 非真机验收闭环：运行 `npm run test:non-device-validation`，生成 Windows 预检报告、Android 本机探测报告、Linux Secret Service 诊断报告，并运行 package/schema/CLI/平台/文档/路径自动化检查；输出 `non_device_ready` 表示真机实测之外已完成。
1. 真实环境剩余待办：参见 `docs\management\StoryLock真机真实桌面验收待办_20260622.md`。
2. Windows 托盘桌面体验：运行 `scripts\windows\start_windows_host_tray_manual_check.cmd`，并填写 `docs\test\Windows托盘人工验收记录_20260620.md`；脚本会额外写入 `.temp\windows-tray-manual-report.local.md` 作为本地预检报告。
3. Android 真机体验：连接设备后运行 `scripts\android\check_device_loop.ps1`，通过 `adb forward` 实测 Android Host `/health` 与 `/permission-summary`，并填写 `docs\test\Android真机验收记录_20260622.md`。
4. Linux Secret Service：运行 `npm run diagnose:linux-secret-service:wsl` 生成 `.temp/linux-secret-service-wsl-report.local.md`；安装 `libsecret-tools` 和可用 Secret Service 后再复测并填写 `docs\test\Linux桌面WSL验收记录_20260622.md`。
5. 平台验收状态汇总：运行 `npm run status:platform-validation`；只有输出 `ready_to_archive` 后，才移动当前 management 文档到 `BACK/`。
6. StoryLock Core 设计对齐验收：完成 P0/P1 后运行 `npm run test:storylock-package`、`npm run validate:storylock-package` 和 `npm run inspect:storylock-package`。
7. 文档对齐验收：运行 `node scripts\test\docs-consistency.mjs` 和 `node scripts\verify\path-consistency.mjs`。
