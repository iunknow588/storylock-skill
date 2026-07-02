# Windows Host UI 优化总文档

日期：2026-07-02

## 文档说明

本文档合并并替代以下三份历史文档：

1. `windows-host_ui_optimization_suggestions_20250630.md`
2. `windows-host_ui_deployment_feasibility_20250630.md`
3. `windows-host_debug_test_refactor_task_list_20250702.md`

合并目的：

1. 将“建议、可行性、实施记录、专项重构任务”统一到一个入口。
2. 消除旧文档之间的命名漂移，尤其是 `Observation` 与“调试测试”的表述冲突。
3. 基于当前代码状态重新判断哪些优化已完成，哪些仍未完成。

## 一、总体结论

`windows-host` 的 UI 优化已经完成了大部分结构化改造和基础视觉优化，部署链路也已验证可行；但如果以最初的完整优化建议为标准，任务还没有全部完成。

当前状态更准确地说是：

1. **基础组件优化**：已完成。
2. **Host 主界面结构优化**：已完成。
3. **StoryLock Core 多页面分组与容器化**：大部分已完成。
4. **“其他观察”改名并拆分为“调试测试”多页**：已完成。
5. **原始建议中的折叠菜单、页面级进度条与长文本溢出治理**：已完成；Spinner、部分响应式与可访问性细化仍未完成。

## 二、已合并的原始目标

### 1. 原始建议类目标

最初的优化建议主要覆盖六类：

1. 侧边栏分组、子菜单折叠、选中态强化。
2. 主内容区卡片化、留白、对齐和 8px 网格。
3. 颜色、字体、标题层级、输入框和按钮状态统一。
4. 授权挑战、设置弹窗、对象页等组件级细化。
5. 加载、进度、确认弹窗等交互反馈。
6. 响应式、可访问性、文本溢出处理。

### 2. 可行性与发布类目标

已确认以下方向可沿用现有链路部署：

1. Slint UI 代码修改后继续随 Rust 编译进入 `yian-windows-host.exe`。
2. 现有 zip 打包链路可继续使用。
3. release 验证与截图验收可以继续沿用原脚本。

### 3. 调试测试专项重构目标

原 “Observation / 其他观察” 页已被确认为命名失真，需要改造为更明确的“调试测试”结构，并拆分为：

1. 调试概览
2. 连接联调
3. 九宫格测试

## 三、已完成的优化项

### 1. 基础共享组件

已完成：

1. `MenuButton`、`SubMenuButton`、`SideActionButton` 的 hover / pressed 反馈。
2. `ActionButton` 的 primary / secondary / danger / disabled 统一。
3. `SettingsIconButton` 的交互反馈。
4. `PathBrowseRow` 浏览按钮图标化。
5. `LargeEditableText` 白底输入区。
6. `LearningAnswerTile` 的选中态、hover、pressed 强化。
7. `RequestConfirmation` 的 Approve / Deny 主次区分。

### 2. HostDashboard 结构优化

已完成：

1. 侧边栏独立背景与分隔线。
2. 产品标识区 `Yian / StoryLock Host`。
3. 主内容白色工作区容器。
4. 页面级标题与辅助说明拆分。
5. Remote Web、StoryLock、Host 辅助页面的卡片化组织。
6. `SettingsDialog` 分组化。
7. `StoryLockAuthorizationDialog` 操作区压缩为两行。
8. 连接测试按钮增加 `Testing...` 禁用态与后台结果回写。

### 3. StoryLock Core 结构优化

已完成：

1. 侧边栏功能分组：`Content / Objects / Actions`。
2. `Protected Objects` 侧边栏改为默认收起的折叠菜单。
3. `Story Draft`、`24 Questions` 页面加入统一容器。
4. `QuestionEditorPage` 容器化。
5. `ProtectedObjectsPage` 重写与乱码清理。
6. `LearningPage` 拆分为参数、节奏、状态面板。
7. `LearningPage` 与 `ExportPage` 增加页面级可视化进度条。
8. `ExportPage` 增加 Learning Gate 等信息面板。
9. `StoryLockCoreSettingsDialog`、`ObjectEditorDialog`、`AnswerEditorDialog` 分组化。
10. `LearningTestDialog` 操作区压缩和统一化。

### 4. 调试测试重构

已完成：

1. `HostDashboard` 侧边栏将“其他观察 / Observation”改为“调试测试 / Debug And Tests”。
2. 原单页内容拆分为：
   - 调试概览
   - 连接联调
   - 九宫格测试
3. 页面标题与测试断言同步更新。
4. `cargo check` 与专项测试已通过。

## 四、当前未完成项

以下项目来自最初优化建议，但从当前代码与实施记录看，仍未完成或只完成了一部分。

### 1. 页面级 Tooltip

**状态：已按统一长文本溢出策略落地**

原因：

1. 当前没有单独的 Tooltip 浮层组件，但已通过共享 `StaticRow` 增加 `wrap-value` / `value-height`，形成统一的长文本查看策略。
2. `HostDashboard` 设置页中的 `StoryLock Status`、`Trigger`、`Note` 以及调试页中的 `Identity`、`Device` 已接入多行展示。
3. `LearningPage`、`ExportPage` 的进度说明文本已改为换行显示，不再完全依赖 `overflow: elide`。

### 2. 页面级 Spinner / 加载态反馈

**状态：部分完成**

原因：

1. `HostDashboard` 的连接测试按钮已经增加 `Testing...` 禁用态。
2. 但更广义的页面级 spinner 体系还没有在其他长操作里统一落地。

### 3. Learning / Export 页面的可视化进度条

**状态：已完成**

原因：

1. `LearningTestDialog` 中已有进度条。
2. `LearningPage` 和 `ExportPage` 现已复用 `learning-progress-percent` 增加页面级进度条。

### 4. StoryLock Core 侧边栏折叠化降噪

**状态：已完成**

原因：

1. 已完成功能分组。
2. “受保护对象”二级项已经改为默认收起，点击父项后展开。

### 5. Tooltip / 文本展开 / 统一溢出策略

**状态：已完成（2026-07-02）**

原因：

1. 已在 `skill/src/host/windows-host/src/slint_ui/common.slint` 为 `StaticRow` 增加统一的 `wrap-value` / `value-height` 能力。
2. 已在 `skill/src/host/windows-host/src/slint_ui/host_dashboard.slint` 将关键状态与说明信息切换为多行可读模式。
3. 已在 `skill/src/host/windows-host/src/slint_ui/storylock_core/pages_learning_export.slint` 将进度说明改为可换行文本。
4. 已补充 `storylock/tests.rs` 断言，并通过 `cargo check` 与专项测试验证。

### 6. 更进一步的响应式与可访问性细化

**状态：未完成**

原因：

1. 目前大量窗口仍是固定尺寸设计。
2. 原建议中的更小窗口适配、网格自适应、WCAG 进一步校验，没有看到完成记录。

### 7. 截图验收噪声与乱码链路彻底清理

**状态：已完成**

原因：

1. `skill/tools/render-storylock-ui-docs` 已完成迁移后的路径清理、样例文本清理和文档同步。
2. 已重新生成 `docs/management/流程图/ui-screenshots` 下的 01-07 截图，并清理旧的 `skill/src/docs/...` 影子输出目录。
3. 本轮 `cargo run` 与截图刷新过程中未再出现此前文档里记录的 ICU4X 警告。

### 8. MSI 打包链路专项验证

**状态：已完成验证，阻塞点已明确**

原因：

1. 当前记录已明确 zip 构建与产物校验通过。
2. 已于 2026-07-02 实际执行 `.\scripts\release\windows\build_windows_host.ps1 -BuildMsi -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype-ui`。
3. 构建脚本成功产出 zip：`release/app/windows/yian-windows-host-0.1.0-2-prototype-ui.zip`，SHA-256 为 `4896a9d6a99d5c3b6d499d9d2a0cd3741c6895667245a4bef23ad41e726b385e`。
4. MSI 未产出，直接原因已确认是当前环境缺少 `wix` CLI，脚本输出为 `WiX CLI was not found. Skipping MSI build.`，这属于打包工具链缺失，不是当前 UI 代码变更导致的失败。

## 五、完成度判断

### 1. 如果按“已经实施过的重构任务”来算

可以认为**已完成**：

1. 组件风格统一
2. Host 页面结构重组
3. StoryLock Core 多页面卡片化与分组
4. 调试测试专项拆页
5. zip 发布链路验证

### 2. 如果按“最初完整优化建议”来算

不能认为全部完成。

更准确的判断是：

1. **主体任务已完成**
2. **尾项与细化项仍有残留**

建议完成度口径：

- **主线完成度：约 88% - 92%**
- **未完成部分集中在更广义的 Spinner 体系、响应式/可访问性细化，以及 MSI 工具链补齐**

## 六、后续建议

建议后续只保留一条精简任务线，避免再次分裂成多份并行文档：

1. 清理截图工具的乱码样本与 ICU4X 警告。
2. 补跑 `build_windows_host.ps1 -BuildMsi`。
3. 视需要继续补齐更广义的 Spinner 与响应式细化。

相关结构分析补充：

1. `tools` / 外部工具迁移后的结构分析见 `src_tools_externalization_feasibility_20260702.md`。

## 七、建议执行顺序

为避免继续在多个零散尾项之间来回跳，建议按下面顺序推进：

### P0：优先补齐发布与验收闭环

1. 截图验收噪声与乱码样本清理

原因：

1. 当前主界面可读性问题已经收口，剩下更影响验收效率的是截图链路噪声。
2. 这项工作能直接降低后续截图对比与文档留档的干扰。

### P1：补齐发布验证

1. MSI 打包链路验证
2. 更广义的 Spinner 与响应式细化

原因：

1. 这两项更偏发布质量与长尾体验。
2. 不会改变当前主流程，但能补齐最终交付闭环。

### P2：补齐发布与适配验证

1. MSI 打包链路验证
2. 响应式与可访问性细化

原因：

1. 这两项重要，但不是当前 UI 主线体验的首要阻塞。
2. 更适合在主要界面形态稳定后再做。

## 八、尾项任务线

下面这条任务线是对“未完成项”的可执行展开版本，后续建议只维护这一节。

### 任务 1：Protected Objects 折叠导航

状态：已完成（2026-07-02）

目标：

1. 默认只显示“受保护对象”父项。
2. 点击后再展开“普通 / 私密 / 机密”。

代码落点：

1. `skill/src/host/windows-host/src/slint_ui/storylock_core.slint`
2. 如需复用展开按钮样式，再看 `skill/src/host/windows-host/src/slint_ui/common.slint`

验收标准：

1. 默认收起。
2. 当前组仍能高亮定位。
3. `cargo check` 通过。

### 任务 2：连接测试加载反馈

状态：已完成（2026-07-02）

目标：

1. `Test Local Host`
2. `Test Remote`

在测试进行中显示临时状态，例如：

1. 按钮禁用
2. 文案改为“测试中...”
3. 或增加轻量 spinner / status text

代码落点：

1. `skill/src/host/windows-host/src/slint_ui/host_dashboard.slint`
2. 对应回调绑定所在 Rust 文件

验收标准：

1. 连点不会重复触发同一测试。
2. 测试结束后恢复按钮状态。
3. 成功 / 失败状态仍正确回写。

### 任务 3：学习页 / 导出页页面级进度条

状态：已完成（2026-07-02）

目标：

1. 将当前文本进度升级为页面可视化进度条。
2. 保留现有摘要文本，不强行替换掉全部说明。

代码落点：

1. `skill/src/host/windows-host/src/slint_ui/storylock_core/pages_learning_export.slint`
2. 如需状态补齐，再看：
   - `skill/src/host/windows-host/src/slint_ui/storylock/editor_flow/learning.rs`
   - `skill/src/host/windows-host/src/slint_ui/storylock_core.slint`

验收标准：

1. 页面中可直接看到进度条。
2. 进度百分比和现有 `learning-progress-percent` 对齐。
3. `LearningTestDialog` 与主页面显示口径一致。

### 任务 4：Tooltip 与长文本查看策略

状态：已完成（2026-07-02）

目标：

1. 为易截断文本提供统一的查看方式。
2. 不再完全依赖 `overflow: elide`。

优先覆盖：

1. 日志区
2. 状态摘要
3. 可能被截断的路径和标题

验收标准：

1. 至少有一套统一策略，不再每页各做各的。
2. 长文本信息可读，不破坏现有布局。

完成记录：

1. `StaticRow` 已扩展为默认单行、按需多行的共享组件。
2. 设置页、调试页、学习/导出页的关键长文本已切换到统一策略。
3. 验证已完成：`cargo check`、`cargo test static_rows_support_wrapped_values_for_long_status_text -- --nocapture`。

### 任务 5：截图验收链路清理

状态：已完成（2026-07-02）

目标：

1. 清理乱码样本。
2. 追踪并尽量消除 `ICU4X` 警告。

代码落点：

1. `skill/tools/render-storylock-ui-docs`
2. 相关截图样本与输入文案

验收标准：

1. 截图工具输出更干净。
2. 验收截图不再混入明显伪数据或乱码噪声。

完成记录：

1. `skill/tools/render-storylock-ui-docs` 已迁移并完成路径清理，旧的 `src/tools/...` 引用已全部收口。
2. 截图样例文本与工具文档已清理，`render-storylock-ui-docs` 已恢复可编译、可出图状态。
3. 已执行 `cargo check` 与 `cargo run`，成功生成并刷新 `docs/management/流程图/ui-screenshots` 下的 01-07 截图。
4. 旧的 `skill/src/docs/management/流程图/ui-screenshots` 影子输出目录已清理。
5. 本轮工具运行未再出现此前文档中记录的 ICU4X 警告。

### 任务 6：MSI 打包验证

状态：已完成（2026-07-02）

目标：

1. 补全 `-BuildMsi` 的实际验证结论。

建议命令：

```powershell
cd E:\2026OPC大赛\skill
.\scripts\release\windows\build_windows_host.ps1 -BuildMsi -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype-ui
```

验收标准：

1. MSI 构建通过。
2. 产物路径明确。
3. 如失败，记录失败点和是否与当前 UI 变更相关。

完成记录：

1. 已执行底层打包脚本：
   - `.\scripts\release\windows\build_windows_host.ps1 -BuildMsi -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype-ui`
2. 验证结论：
   - release 编译通过
   - zip 产物成功生成
   - MSI 分支被实际执行，但当前环境缺少 `wix` CLI，因此脚本输出 `WiX CLI was not found. Skipping MSI build.`
   - 该阻塞点属于本机 MSI 工具链缺失，不是当前 UI 代码变更导致
3. 已进一步执行上层 release 包装脚本：
   - `.\scripts\release\windows\release_windows_host.ps1 -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype-ui`
4. 上层脚本验证结果：
   - 会再次调用底层 `build_windows_host.ps1`
   - 会补充生成 `release-manifest-0.1.0-2-prototype-ui.json`
   - 当前适合作为“正式 release 流程验证”，但不应替代底层打包能力验证
5. 当前可确认的有效发布产物：
   - `release/app/windows/yian-windows-host-0.1.0-2-prototype-ui.zip`
   - 最近一次 release 包装脚本生成的 SHA-256：`77389719244fd80f667dc951d639e198b61ef7f8751801ba905e42fbe1adb4da`

## 九、完成判定规则

后续建议按下面规则判断“是否真的完成”，避免因为只做了 UI 外观就提前关单。

### 可判定为完成

同时满足以下条件：

1. 代码已落地。
2. 至少跑过一次 `cargo check`。
3. 如涉及结构变化，相关测试断言已同步。
4. 文档中的状态从“未完成”更新为“已完成”。

### 不能直接判定为完成

以下情况不能算完成：

1. 只有建议，没有代码。
2. 只有代码，没有验证。
3. 只有局部 UI 改动，但文档与命名仍保留旧表述。

## 十、文档清理结论

原三份文档已被本文件取代，后续统一维护本文件：

1. `windows-host_ui_optimization_suggestions_20250630.md`
2. `windows-host_ui_deployment_feasibility_20250630.md`
3. `windows-host_debug_test_refactor_task_list_20250702.md`

## 十一、发布一致性验证补充（2026-07-02）

为避免将 Host UI 中的“调试测试”与发行链路校验混为一谈，补充如下口径：

1. `Debug And Tests` 页签负责运行时联调与功能测试，例如本地 Host、Remote Relay、九宫格授权等。
2. `scripts/release/windows/build_windows_host.ps1` 负责底层构建与打包能力验证。
3. `scripts/release/windows/release_windows_host.ps1` 负责正式 release 清单生成。
4. `scripts/release/windows/verify_windows_release_consistency.ps1` 负责“当前本地产物是否已经与线上发行版一致”的最终比对。

建议命令：

```powershell
cd E:\2026OPC大赛\skill
.\scripts\release\windows\verify_windows_release_consistency.ps1 `
  -ManifestPath .\release\app\windows\release-manifest-0.1.0-2-prototype-ui.json `
  -BaseUrl https://yian.cdao.online
```

本轮实际结论：

1. 本地最新 Windows 包为 `0.1.0 / versionCode 2 / prototype-ui`。
2. 2026-07-02 在线发行页 `https://yian.cdao.online/app/download` 仍返回 Windows 包 `0.1.0 / versionCode 1 / prototype`。
3. 差异字段包括 `versionCode`、`releaseChannel`、`fileName`、`fileSizeBytes`、`checksum`。
4. 因此当前状态不是“Host UI 缺少验证能力”，而是“线上发行站点尚未同步到本地刚生成的 Windows 发布包”。
