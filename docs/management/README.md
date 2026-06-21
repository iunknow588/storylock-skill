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

## 历史归档

`BACK/` 目录保存历史分析、旧版计划和评审意见，仅作为参考，不作为当前执行基线。20260620 阶段计划和实施清单已归档到 `BACK/`，当前工作以 20260621 文档为准。

## 当前执行重点

1. Windows Slint `StoryLock Core` 子应用继续作为底层配置入口，主 Host 只读取脱敏权限摘要，不能直接编辑故事原文、答案、密码、私钥或加密配置。
2. 补齐 Windows、Android、Linux 真机或桌面验收记录。
3. 远程网关继续只暴露 `requestSignature` 和 `requestPasswordFill` 两个安全出口，不开放远程故事读写。

## 当前归档判断

截至 2026-06-21，本目录尚未整体处理完毕，暂不移动当前主文档到 `BACK/`。

已闭环：

1. StoryLock package 第一版共享加载、schema、校验 CLI 和权限摘要说明。
2. 平台宿主与数据包接线说明。
3. 用例样例到测试夹具映射说明。
4. 文档命名与归档规则。
5. 文档-代码-测试映射表。
6. Windows package、Android readiness、Linux host、文档一致性和路径一致性自动化检查。

仍待闭环：

1. Windows 桌面托盘和 Slint UI 人工验收记录。
2. Android 真机运行和 `/permission-summary` 实测记录。
3. Linux 桌面或 WSL 环境运行验收记录。

完成以上人工验收后，可将 20260621 阶段计划、实施清单和对齐总账移动到 `BACK/`，并在本目录保留新的当前任务入口。

## 人工验收入口

1. Windows 托盘桌面体验：运行 `scripts\windows\start_windows_host_tray_manual_check.cmd`，并填写 `docs\test\Windows托盘人工验收记录_20260620.md`。
2. StoryLock Core 设计对齐验收：完成 P0/P1 后运行 `npm run test:storylock-package`、`npm run validate:storylock-package` 和 `npm run inspect:storylock-package`。
3. 文档对齐验收：运行 `node scripts\test\docs-consistency.mjs` 和 `node scripts\verify\path-consistency.mjs`。
