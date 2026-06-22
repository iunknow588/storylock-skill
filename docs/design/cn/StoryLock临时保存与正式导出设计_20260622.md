# StoryLock 临时保存与正式导出设计

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-22 |
| 适用范围 | Windows Host / StoryLock Core 原生 Slint UI |
| 目标 | 区分临时保存、学习训练、正式导出，避免用户误以为普通保存已经替换正式密钥管理文件 |

## 1. 背景

StoryLock Core 的配置围绕 24 个问题、每题 9 个候选答案、每个答案的正确/错误标记展开。当前问题编辑页存在两个视觉上相似的 `Save` 文本：左侧行标题和右侧按钮都显示 `Save`，用户会误认为有两个保存动作。

同时，StoryLock 的安全边界要求：

1. 编辑阶段只能写入本地临时草稿。
2. 正式替换外部密钥管理文件必须经过学习训练。
3. 导出成功后，应删除临时草稿，避免临时文件被误认为正式配置。

## 2. 文件分层

| 文件类型 | 示例路径 | 用途 | 生命周期 |
| --- | --- | --- | --- |
| 正式本地草稿 | `identity-package/author-draft.json` | 当前 StoryLock Core 可加载的正式草稿基线 | 初始化、导出成功后更新 |
| 临时编辑草稿 | `identity-package/.tmp/author-draft.pending.json` | 问题编辑页和故事辅助页的临时保存结果 | 编辑中存在，导出成功后删除 |
| 外部导出包 | `storylock-managed-key-package/` | 第二层授权和外部 Host 可读取的脱敏权限包 | 仅学习训练通过后生成或替换 |

临时编辑草稿可以包含故事问题、答案和正确/错误标记，因此只能保留在 StoryLock Core 本地目录，不能被 Host 主窗口或远程 API 读取。

## 3. UI 规则

### 3.1 顶部全局暂存

`Save Temp Draft` 放在 StoryLock Core 顶部 LOGO / 标题区域右侧，作为全局暂存按钮：

```text
StoryLock Core - 当前菜单                         [Save Temp Draft]
```

含义：

1. `Save Temp Draft` 针对当前 StoryLock Core 窗口内存中的整体配置，而不是单独针对 24 Questions。
2. 保存内容包括故事辅助信息、当前问题、受管理对象配置、模板配置和保存/授权频率说明。
3. 故事与答案写入 `.tmp/author-draft.pending.json`。
4. 资源目录与模板仍属于本地 Core 配置文件，但状态提示必须说明它们只是 Core 本地配置，不代表正式导出。
5. 点击后必须重置学习训练进度，`Export` 保持不可用。

### 3.2 24 Questions

`24 Questions` 页面不再放置独立保存按钮，只负责编辑 24 个问题、每题 9 个候选答案，以及每个答案的 `correct` / `wrong` 标记。

### 3.3 Save Draft

`Save Draft` 菜单作为说明与策略区，展示临时保存目标、学习训练门禁和高阶授权频率设置，不再提供第二个保存按钮。实际保存统一使用顶部 `Save Temp Draft`。

### 3.4 Export

`Export` 是正式替换外部密钥管理文件的唯一入口。导出按钮必须满足：

1. 24 题学习训练全部通过。
2. 当前临时草稿通过包预检。
3. 导出时先将临时草稿提升为正式 `author-draft.json`。
4. 再复制生成 `storylock-managed-key-package/`。
5. 导出成功后删除 `.tmp/author-draft.pending.json`。

## 4. 行为流程

### 4.1 临时保存

```text
用户编辑任意 StoryLock Core 配置页
  -> 点击顶部 Save Temp Draft
  -> 写入 identity-package/.tmp/author-draft.pending.json
  -> 同步本地 Core 的 resource-catalog 与 templates
  -> 重置学习训练进度
  -> Export 保持不可用
```

### 4.2 学习训练

```text
用户进入 Export
  -> Start Test
  -> 按 24 个问题逐题匹配 9 个答案的 correct/wrong
  -> 24 / 24 通过
  -> Export 按钮启用
```

学习训练读取当前有效草稿：如果临时草稿存在，则优先读取临时草稿；否则读取正式 `author-draft.json`。

### 4.3 正式导出

```text
用户点击 Export
  -> 若临时草稿存在，先替换 author-draft.json
  -> 执行包预检
  -> 替换 storylock-managed-key-package/
  -> 写入 EXPORT_STATUS.txt
  -> 删除 identity-package/.tmp/author-draft.pending.json
```

## 5. 安全边界

1. Host 主窗口只能读取脱敏权限摘要。
2. 远程 API 不读取临时草稿、正式故事、答案正确性、密码、私钥或 `signingKeyBytes`。
3. 临时草稿与正式草稿都属于 StoryLock Core 本地私密数据。
4. 外部导出包只在学习训练通过后生成。

## 6. 实现检查项

1. `24 Questions` 页面不再出现保存按钮。
2. 顶部 LOGO / 标题区域显示唯一全局按钮 `Save Temp Draft`。
3. 临时保存写入 `.tmp/author-draft.pending.json`。
4. 加载、预览、学习训练优先使用临时草稿。
5. 导出成功后删除 `.tmp/author-draft.pending.json`。
6. 导出失败时保留临时草稿，便于继续修改。
7. 状态提示明确区分 `temporary draft` 和 `exported managed key package`。
