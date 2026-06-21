# StoryLock 管理文档

本目录保留当前有效的后续开发管理文档。

## 当前主文档

1. `StoryLock后续开发计划_20260620.md`
   - 用于说明当前状态、功能界面部署、功能设计、阶段规划和优先级。

2. `StoryLock后续开发实施清单_20260620.md`
   - 用于拆解具体任务、涉及文件、产出物和验收方式。

## 历史归档

`back/` 目录保存历史分析、旧版计划和评审意见，仅作为参考，不作为当前执行基线。

## 当前执行重点

1. Windows 本地 UI 继续优先推进：默认 host 已有 `/ui` 管理页、`/ui/status` 状态 JSON、`/diagnostics` 脱敏诊断、`/shutdown` 本机退出控制、`ui-tray` 托盘首版、Slint 状态窗编译入口、确认请求详情摘要和 `slint_dialog` 原生 Approve / Deny 确认窗口。
2. Windows 下载包明确为本地宿主原型，不冒充完整 StoryLock 桌面应用。
3. 远程网关继续只暴露 `requestSignature` 和 `requestPasswordFill` 两个安全出口。
4. 后续重点补托盘状态图标细化、九宫格挑战 Slint 化、发布验收和 StoryLock 工作台。

## 人工验收入口

1. Windows 托盘桌面体验：运行 `scripts\windows\start_windows_host_tray_manual_check.cmd`，并填写 `docs\test\Windows托盘人工验收记录_20260620.md`。
