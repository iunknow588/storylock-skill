# StoryLock 管理文档

本目录保留当前有效的后续开发管理文档和进行中的任务配置。这里不是历史垃圾目录，后续新的任务计划、实施清单和验收记录仍放在这里。

## 当前主文档

1. `StoryLock设计对齐后续开发计划_20260621.md`
   - 当前执行基线。用于对齐 `story-lock/doc/design` 与 `skill/src` 的完成度，明确未完成的 StoryLock Core 正式数据包、资源目录、模板、校验 CLI、故事工作台持久化和多平台宿主任务。

2. `StoryLock设计对齐后续开发实施清单_20260621.md`
   - 当前执行清单。用于把 P0-P4 计划拆解成可编码、可测试、可验收的任务。

3. `StoryLock文档对齐后续开发计划_20260621.md`
   - 当前文档对齐总账。用于说明 `story-lock/doc` 与 `skill/docs` 的差异、已修复的文档入口问题，以及后续需要补齐的架构接线、usecase 映射、命名归档规则和文档-代码-测试映射表。

4. `StoryLock后续开发计划_20260620.md`
   - Windows 本地宿主、托盘、下载包和早期 StoryLock 工作台推进记录，作为当前计划的阶段性背景。

5. `StoryLock后续开发实施清单_20260620.md`
   - 20260620 阶段任务拆解，已被 20260621 设计对齐清单吸收的部分以后续清单为准。

## 历史归档

`BACK/` 目录保存历史分析、旧版计划和评审意见，仅作为参考，不作为当前执行基线。

## 当前执行重点

1. 先补齐 StoryLock Core 正式数据模型：`vault.stlk`、`resource-catalog.json`、`package-manifest.json`、`templates/*`、作者稿和 24 节点结构。
2. 把迁移目录里的 resource catalog / templates 能力提升为 `src/shared` 下的正式共享模块，供 Windows、Android、Linux 共用。
3. 增加 package / catalog / templates / author draft 的 JSON Schema、CLI 校验和稳定错误码输出。
4. 补齐 `skill/docs` 与 `story-lock/doc` 的文档对齐说明，优先补架构接线、usecase 到测试 fixture 映射、命名归档规则和文档-代码-测试映射表。
5. Windows Slint `StoryLock Core` 子应用继续作为底层配置入口，主 Host 只读取脱敏权限摘要，不能直接编辑故事原文、答案、密码、私钥或加密配置。
6. 远程网关继续只暴露 `requestSignature` 和 `requestPasswordFill` 两个安全出口，不开放远程故事读写。

## 人工验收入口

1. Windows 托盘桌面体验：运行 `scripts\windows\start_windows_host_tray_manual_check.cmd`，并填写 `docs\test\Windows托盘人工验收记录_20260620.md`。
2. StoryLock Core 设计对齐验收：完成 P0/P1 后运行 `npm run test:storylock-package`、`npm run validate:storylock-package` 和 `npm run inspect:storylock-package`。
3. 文档对齐验收：运行 `node scripts\test\docs-consistency.mjs` 和 `node scripts\verify\path-consistency.mjs`。
