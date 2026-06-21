# StoryLock 文档命名与归档规则

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 适用目录 | `docs/design`、`docs/management`、`docs/test`、`docs/ref` |

## 1. 目录边界

| 目录 | 用途 | Git 状态 |
| --- | --- | --- |
| `docs/design` | 正式技术说明、设计边界、实现接线 | 纳入 Git |
| `docs/test` | 测试方案、人工验收记录、平台验收矩阵 | 纳入 Git |
| `docs/management` | 当前任务计划、实施清单、对齐差异和进行中的工作配置 | 纳入 Git |
| `docs/management/BACK` | 历史工作档案、旧版计划、旧评审意见 | 纳入 Git，但不作为当前执行基线 |
| `docs/ref` | 本地参赛参考资料、外部材料、临时评审素材 | 不纳入 Git |

`docs/management` 不是垃圾目录，也不应整体删除。新的任务计划、实施清单和阶段验收配置仍放在这里；完成或过期后再移动到 `docs/management/BACK`。

## 2. 命名规则

正式技术文档优先使用稳定主题名：

```text
docs/design/cn/平台宿主与数据包接线说明_20260621.md
docs/design/cn/StoryLock数据包与校验CLI说明_20260621.md
```

阶段性管理文档必须带日期：

```text
docs/management/StoryLock设计对齐后续开发计划_20260621.md
docs/management/StoryLock设计对齐后续开发实施清单_20260621.md
```

人工验收记录也应带日期：

```text
docs/test/Windows托盘人工验收记录_20260620.md
```

## 3. 归档规则

满足以下条件时，可以从 `docs/management` 移动到 `docs/management/BACK`：

1. 任务已经在代码、正式设计文档或测试记录中闭环。
2. 文档不再作为当前执行基线。
3. README 中已不再把它列为当前主文档。
4. 未完成项已经转移到新的计划、实施清单或验收记录。

不满足以上条件时，不要归档。

## 4. 当前管理文档状态

截至 2026-06-21：

1. `StoryLock设计对齐后续开发计划_20260621.md` 仍用于记录代码与上游设计差距。
2. `StoryLock设计对齐后续开发实施清单_20260621.md` 仍用于记录 P0-P4 完成度。
3. `StoryLock文档对齐后续开发计划_20260621.md` 的文档可补齐项已转入正式设计文档和本规则文档，但仍可保留到真机/桌面验收完成后统一归档。

## 5. 不建议操作

1. 不把 `story-lock/doc` 全量复制到 `skill/docs`。
2. 不把 `docs/ref` 重新纳入 Git。
3. 不把历史评审意见当作当前设计基线。
4. 不在正式文档中宣称尚未完成的 Android、Linux、macOS 交付能力。
5. 不在文档中暗示远程可以读取故事原文、答案、密码或私钥。
