# 拆分计划目录

更新时间：2026-07-03

本目录保存 Host 与 StoryLock 立即拆分的执行计划。当前拆分已经从“可行性分析”切换为“立即执行任务”。

## 当前基线

1. [Host 与 StoryLock 独立项目拆分计划（立即执行版）](Host与StoryLock独立项目拆分计划.md)
2. [Host / StoryLock / Common 三域目录拆分路线（立即执行版）](Host_StoryLock_Common三域目录拆分路线.md)

## 执行原则

- Host 是本地 Web/API 网关，不读取 StoryLock 私有数据。
- StoryLock Core 是故事、答案、vault、对象策略和挑战验证的所有者。
- Host 与 StoryLock 通过本地长连接、IPC 或 contract 协作。
- `host/`、`storylock/`、`common/` 是立即采用的三域目录。
- 每次迁移只移动一类文件，每次迁移后必须编译验证。

