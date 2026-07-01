# Windows Host UI 部署可行性分析

日期：2026-06-30

## 结论

`windows-host` 的 UI 优化可以按现有部署链路落地。当前 Slint UI 文件在 Rust 编译阶段被打进 `yian-windows-host.exe`，用户侧不需要额外安装 UI 资源、Node、Python、WebView 或脚本运行时。

因此 UI 优化后的发布方式仍然是：

1. 修改 `src/slint_ui/*.slint` 和必要的 Rust 绑定代码。
2. 执行 release build。
3. 重新生成 zip / MSI。
4. 用现有发布脚本产出 manifest、checksum、env 文件。
5. 通过现有下载页或对象存储分发新包。

## 当前 UI 部署结构

项目入口：

```text
skill/src/host/windows-host
```

关键文件：

```text
src/slint_ui/host_dashboard.slint
src/slint_ui/common.slint
src/slint_ui/storylock_core.slint
src/slint_ui/storylock_core/*.slint
src/slint_ui/request_confirmation.slint
src/slint_ui/mod.rs
src/slint_ui/dashboard.rs
```

`src/slint_ui/mod.rs` 通过 `slint::slint!` 引入 `.slint` 文件。编译完成后，UI 定义随 Rust 程序一起进入 exe。

当前部署产物：

```text
target/release/yian-windows-host.exe
release/app/windows/yian-windows-host-<version>-<versionCode>-<channel>.zip
release/app/windows/yian-windows-host-<version>-<versionCode>-<channel>.msi
```

zip 包当前包含：

```text
yian-windows-host.exe
README.md
start-yian-windows-host.cmd
identity-package/
config/
templates/
```

MSI 当前主要安装：

```text
yian-windows-host.exe
README.md
```

## 部署可行性

| 优化项 | 是否可直接部署 | 说明 |
| --- | --- | --- |
| 侧边栏分组、折叠、active/hover 状态 | 可行 | 主要改 `.slint` 组件和状态属性，不影响发布脚本 |
| 8px 网格、间距、卡片层级 | 可行 | 纯 Slint 布局改动，重新编译 exe 即可 |
| 输入框、按钮、状态 badge 样式 | 可行 | 优先改 `common.slint`，影响面集中 |
| SettingsDialog 分组 | 可行 | 改 `host_dashboard.slint` 和 `storylock_core/dialogs.slint` |
| 授权九宫格卡片化 | 可行 | 改 `LearningAnswerGrid` / 授权弹窗布局，需重点回归 |
| loading / progress 状态 | 部分可行 | 如果只是展示已有进度，Slint 可直接做；如果需要真实异步状态，需要 Rust 回调补状态 |
| tooltip | 部分可行 | Slint 可实现 hover 区域，但需要统一 Tooltip 组件和全局定位，建议后置 |
| 图标化按钮 | 可行但需谨慎 | 目前未引入图标库；建议先用 Slint 文本/符号或本地 raster 资产，避免新增复杂依赖 |
| 响应式布局 | 部分可行 | Slint 固定窗口较多，建议先做最小宽高和内容不溢出，再做复杂自适应 |

## 推荐部署策略

### 第一阶段：只改 UI 视觉和布局，不改业务链路

目标是降低风险，确保部署仍是“单 exe + 现有 zip/MSI”。

建议先做：

1. `common.slint` 中统一按钮、输入框、行组件、卡片/面板样式。
2. `host_dashboard.slint` 中优化 Host 首页和 StoryLock 入口页。
3. `storylock_core.slint` 中优化侧边栏分组和主内容容器。
4. `request_confirmation.slint` 中区分 Approve / Deny 主次按钮。
5. 保持现有回调、数据结构、文件格式、local API 不变。

这一阶段不需要调整部署脚本。

### 第二阶段：补部署验收

建议在每次 UI 发布前执行：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo test
cargo build --release
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

然后执行打包：

```powershell
cd E:\2026OPC大赛\skill
.\scripts\release\windows\build_windows_host.ps1 -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype
```

如果需要 MSI：

```powershell
.\scripts\release\windows\build_windows_host.ps1 -BuildMsi -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype
```

### 第三阶段：发布

完整 release flow：

```powershell
cd E:\2026OPC大赛\skill
.\scripts\release\windows\release_windows_host.ps1 -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype
```

发布产物会生成：

```text
release/app/windows/release-manifest-0.1.0-2-prototype.json
release/app/windows/yian-windows-host-0.1.0-2-prototype.zip
```

后续可继续走：

```powershell
.\scripts\release\windows\publish_windows_release.ps1 -ManifestPath <manifest> -CopyArtifacts
```

## 是否需要改部署脚本

短期不需要。

原因：

1. Slint UI 已编译进 exe。
2. zip 包已经分发 exe。
3. MSI 已经从 `target/release/yian-windows-host.exe` 安装。
4. 用户双击 exe 即进入 Slint UI，不需要额外 UI 文件。

建议后续小改：

1. MSI 可增加 `start-yian-windows-host.cmd`，方便命令行启动用户排查问题。
2. release flow 可加入 `cargo run --bin render_storylock_ui_docs`，自动更新 UI 截图作为视觉验收。
3. 如果未来引入外部图片、字体或图标资源，需要确认它们是 `include_bytes!` 打进 exe，还是加入 zip/MSI 清单。

## 主要风险

1. 当前 UI 文案和截图工具里存在部分乱码，需要单独处理编码问题，否则视觉验收会受干扰。
2. Slint fixed-size 布局较多，卡片化和分组后可能导致小窗口内容溢出，需要逐页截图检查。
3. `common.slint` 是共享组件，改按钮/行组件会影响 Host、StoryLock、授权弹窗多个界面，需要集中回归。
4. loading/progress 如果要真实反映耗时任务，需要 Rust 侧暴露状态，不应只做静态样式。
5. 如果引入图标或字体资源，部署清单可能需要同步调整。

## 建议的落地顺序

1. 先优化 `common.slint`：按钮、输入框、静态行、面板、状态标记。
2. 再优化 `host_dashboard.slint`：侧边栏、Host 首页、设置弹窗、授权弹窗。
3. 然后优化 `storylock_core.slint` 和分页：故事、问题、对象、学习、导出。
4. 最后补截图验收和 release manifest。

这个顺序的好处是部署链路不变，风险集中在 UI 层；每一步都可以通过重新编译 exe 和截图比对验证。

## 2026-06-30 已完成的第一批 UI 基础优化

本轮先完成低风险、部署链路不变的共享组件优化：

1. `MenuButton` 增加 hover / pressed 反馈，保持原有选中态左侧色条。
2. `SubMenuButton` 增加 hover / pressed 反馈。
3. `SideActionButton` 增加 hover / pressed 反馈。
4. `PathBrowseRow` 的浏览按钮从纯文本 `...` 改为更像工具按钮的深色图标按钮。
5. `LargeEditableText` 的编辑区改为白底、边框输入区域，减少和页面背景混在一起的问题。
6. `LearningAnswerTile` 的九宫格选项增加选中 tint、hover 边框和 pressed 反馈。
7. `ActionButton` 增加 `danger` 属性，并统一 primary / secondary / danger / disabled 的边框、背景、文字颜色和 hover / pressed 状态。
8. `SettingsIconButton` 增加 hover / pressed 反馈。
9. `RequestConfirmation` 的 Approve / Deny 从原生 `Button` 换成统一 `ActionButton`，Approve 为 primary，Deny 为 danger。
10. `StoryLockAuthorizationDialog` 的 Cancel 使用 danger 样式。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
```

结果：通过。当前仅保留既有 unused / dead_code 警告，没有新增阻断问题。

## 下一批建议

下一批建议继续控制风险，优先做视觉结构，不改业务逻辑：

1. Host 首页区域卡片化：把身份/设备、连接测试、状态日志拆成更清晰的面板。
2. StoryLock Host 页卡片化：路径选择、边界说明、九宫格测试分组。
3. SettingsDialog 分组：通用设置、路径配置、说明信息。
4. StoryLockAuthorizationDialog 操作区压缩：上一题/下一题并排，授权/取消并排，右侧信息更紧凑。
5. 运行 `render_storylock_ui_docs` 生成截图，对比每页是否有文字溢出或乱码。

## 2026-06-30 已完成的第二批 UI 结构优化

第二批继续保持部署链路不变，只调整 Host 主窗口结构：

1. `HostDashboard` 侧边栏改为更深的独立背景，并增加右侧 1px 分隔线。
2. 侧边栏增加产品标识区：`Yian` 与 `StoryLock Host`，让 exe 双击后的首屏更像正式桌面入口。
3. 主内容背景从整页灰底调整为白色工作区，外围保留浅色页面背景。
4. 标题区拆为主标题 + 辅助说明，降低标题过重的问题。
5. 三个 Host 页面共用一个浅色内容容器，增加边框和 6px 圆角。
6. Remote Web、StoryLock、Observation 三页分别增加页面级小标题，强化信息分组。
7. `FormRow`、`StaticRow` 增加可配置宽度，默认值收窄以适配内容容器。
8. `PathBrowseRow` 增加可配置输入宽度，避免路径选择行顶出卡片边界。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
```

结果：通过。当前仍只有既有 unused / dead_code 警告。

## 下一批建议更新

后续建议从“结构”进入“页面细化”：

1. 继续拆 Host 首页内部小面板：身份设备、连接测试、状态日志。
2. StoryLock Host 页内部小面板：包路径、边界说明、强度说明、九宫格测试。
3. SettingsDialog 增加分组标题和分隔线。
4. StoryLockAuthorizationDialog 右侧操作区压缩为上下两组按钮，减少空白。
5. 修复 UI 截图工具和文档中的乱码问题，再生成截图做视觉验收。

## 2026-06-30 已完成的第三批页面细化

第三批进入 Host 页面内部细化，继续不改业务逻辑：

1. 新增共享组件 `SectionTitle`，用于页面内部小标题。
2. 新增共享组件 `SectionPanel`，统一小面板白底、边框、圆角。
3. Remote Web 页拆成三个内部面板：
   - Identity And Device
   - Connection And Runtime
   - Local Package Status
4. StoryLock 页拆成三个内部面板：
   - Current Package
   - Boundary And Strength Rules
   - Nine-Grid Test
5. 保留原有字段、按钮、回调和数据绑定，仅调整视觉分组。
6. 继续使用可配置宽度，避免路径行、状态行顶出内容容器。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

结果：

1. `cargo check` 通过。
2. UI 截图工具执行完成并更新截图。
3. 截图工具仍输出 `ICU4X data error: No segmentation model for language: ja` 警告；当前不阻断截图生成，但建议后续单独处理。

## 下一批建议更新 2

建议下一批处理：

1. SettingsDialog 分组：通用设置、路径配置、状态说明。
2. StoryLockAuthorizationDialog 右侧操作区压缩，减少当前 180px 操作卡片里的空白。
3. Observation 页也拆成 Statistics / Diagnostics 两个内部面板。
4. 处理截图工具中的乱码样本文案和 ICU4X 警告，保证视觉验收更干净。

## 2026-06-30 已完成的第四批弹窗和观察页优化

第四批继续完善 Host 主 UI 周边窗口和观察页：

1. Observation 页拆成两个内部面板：
   - Management Statistics
   - Diagnostics And Boundaries
2. SettingsDialog 拆成三个设置面板：
   - General
   - Package Path
   - Open And Unlock
3. SettingsDialog 保留原有语言切换、路径选择、状态说明和回调逻辑。
4. StoryLockAuthorizationDialog 右侧操作区从四个纵向按钮压缩为两行并排按钮：
   - Prev / Next
   - Authorize / Cancel
5. Cancel 继续使用 danger 样式，Authorize 继续使用 primary 样式。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

结果：

1. `cargo check` 通过。
2. UI 截图工具执行完成并更新截图。
3. 截图工具仍输出 `ICU4X data error: No segmentation model for language: ja` 警告；当前不阻断截图生成。

## 下一批建议更新 3

建议下一批进入 StoryLock Core 主窗口：

1. `storylock_core.slint` 侧边栏分组：内容管理、保护对象、安全学习、导出/设置。
2. `storylock_core/pages_story.slint` 中故事草稿和 24 问题页增加更清晰的内容容器。
3. `storylock_core/pages_learning_export.slint` 中学习/导出页增加进度和结果面板。
4. 单独处理 `render_storylock_ui_docs.rs` 中的乱码样本文案，减少截图验收噪声。

## 2026-06-30 已完成的第五批 StoryLock Core 壳层优化

第五批进入 StoryLock Core 主窗口，仍保持业务逻辑和数据绑定不变：

1. `storylock_core.slint` 侧边栏改为独立背景，并增加右侧分隔线。
2. StoryLock Core 侧边栏增加产品标识区：`StoryLock` / `Local Core`。
3. 侧边栏菜单按功能分组：
   - Content
   - Objects
   - Actions
4. 主内容背景改为白色工作区，页面外层保留浅色背景。
5. 标题区拆成主标题 + 当前包状态说明。
6. 主页面容器增加白底、边框、圆角。
7. `pages_story.slint` 的 Story Draft 页增加 `SectionPanel` 和 `SectionTitle`，把故事标题、摘要、完整故事统一收进内容面板。
8. `pages_story.slint` 的 24 Questions 页增加 `SectionPanel` 和 `SectionTitle`，问题总览不再直接贴边。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

结果：

1. `cargo check` 通过。
2. UI 截图工具执行完成并更新截图。
3. 截图工具仍输出 `ICU4X data error: No segmentation model for language: ja` 警告；当前不阻断截图生成。

## 下一批建议更新 4

建议下一批继续 StoryLock Core 内页：

1. `QuestionEditorPage` 增加内容面板和更清晰的返回按钮区域。
2. `ProtectedObjectsPage` 拆成对象列表 / 当前对象详情面板。
3. `LearningPage` 拆成预学习参数 / 留存计划 / 状态摘要面板。
4. `ExportPage` 拆成学习门禁 / 导出状态 / 日志输出面板。

## 2026-06-30 已完成的第六批 StoryLock Core 内页优化

第六批继续完善 StoryLock Core 内页，仍保持业务回调和数据绑定不变：

1. `QuestionEditorPage` 增加 `SectionPanel` 和 `SectionTitle`，问题答案编辑区域不再直接贴边。
2. `ProtectedObjectsPage` 重写为干净的 Slint 文件，保留全部输入属性和 `select-object` 回调。
3. `ProtectedObjectsPage` 清理原有乱码可见文案，改为 Unicode 转义中文和英文双语。
4. `ProtectedObjectsPage` 增加统一白底对象列表面板。
5. `LearningPage` 拆成三个面板：
   - Pre-Learning Parameters
   - Retention Schedule
   - Status
6. `ExportPage` 增加 Learning Gate 面板，导出页的学习门禁信息更清楚。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

结果：

1. `cargo check` 通过。
2. UI 截图工具执行完成并更新截图。
3. 截图工具仍输出 `ICU4X data error: No segmentation model for language: ja` 警告；当前不阻断截图生成。

## 下一批建议更新 5

建议下一批处理：

1. 清理 `render_storylock_ui_docs.rs` 中的乱码样本文案，避免截图验收被假数据污染。
2. 检查 `dialogs.slint`：Settings、AnswerEditor、ObjectEditor、LearningTestDialog 是否需要同样分组。
3. 根据新截图做一次人工视觉回归，确认 StoryLock Core 侧边栏没有挤压、文字没有明显溢出。

## 2026-06-30 已完成的第七批弹窗和乱码清理

第七批继续完善 StoryLock Core 弹窗，并清理对象页乱码：

1. `storylock_core/dialogs.slint` 引入 `SectionPanel` 和 `SectionTitle`。
2. `StoryLockCoreSettingsDialog` 拆成三个面板：
   - General
   - Workspace
   - Package Files
3. `ObjectEditorDialog` 增加 Object Details 面板，URI、Username、Password 进入统一内容容器。
4. `AnswerEditorDialog` 增加 Question And Answers 面板，答案编辑区不再直接贴边。
5. `ProtectedObjectsPage` 已重写为干净 Slint 文件，原有可见乱码文案已替换为 Unicode 转义中文和英文。
6. 保留原有属性、回调和数据绑定，不改变业务行为。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

结果：

1. `cargo check` 通过。
2. UI 截图工具执行完成并更新截图。
3. 截图工具仍输出 `ICU4X data error: No segmentation model for language: ja` 警告；当前不阻断截图生成。

## 下一批建议更新 6

建议下一批处理：

1. `LearningTestDialog` 右侧操作区继续压缩，和授权挑战弹窗保持一致。
2. 修复 `render_storylock_ui_docs.rs` 中剩余乱码样本文案。
3. 检查 `ProtectedObjectTableRow` 在 668px 宽度下的列宽是否需要进一步微调。

## 2026-06-30 已完成的第八批学习测试弹窗优化

第八批继续完善弹窗体验和截图验收噪声：

1. `LearningTestDialog` 右侧操作区从四个纵向按钮压缩为两行并排按钮：
   - Prev / Next
   - Restart / Close
2. `LearningTestDialog` 的按钮布局与前面的授权挑战弹窗保持一致。
3. 重新扫描 `src/slint_ui` 和 `render_storylock_ui_docs.rs` 的明显乱码。当前 UI 可见文案中的主要乱码已清理。
4. 剩余命中位于 `storylock/core_data/vault.rs`，属于坏编码检测样本，不是 UI 可见文案。

验证结果：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo check
cargo run --bin render_storylock_ui_docs -- E:\2026OPC大赛\skill\docs\management\流程图\ui-screenshots
```

结果：

1. `cargo check` 通过。
2. UI 截图工具执行完成并更新截图。
3. 截图工具仍输出 `ICU4X data error: No segmentation model for language: ja` 警告；当前不阻断截图生成。

## 下一批建议更新 7

建议下一批处理：

1. 做一次 release build 验证：`cargo build --release`。
2. 运行 Windows host 打包脚本，确认 UI 优化后的 exe 仍能进入 zip 产物。
3. 如需继续视觉优化，下一步进入更细的表格列宽、进度条和滚动区域微调。

## 2026-07-01 已完成的发布链路验证

本轮验证 UI 优化后的 release 构建和 zip 打包链路。

执行：

```powershell
cd E:\2026OPC大赛\skill\src\host\windows-host
cargo build --release

cd E:\2026OPC大赛\skill
.\scripts\release\windows\build_windows_host.ps1 -Version 0.1.0 -VersionCode 2 -ReleaseChannel prototype-ui
```

结果：

1. `cargo build --release` 通过。
2. Windows host zip 打包通过。
3. 生成产物：

```text
E:\2026OPC大赛\skill\release\app\windows\yian-windows-host-0.1.0-2-prototype-ui.zip
```

4. zip 大小：

```text
7082046 bytes
```

5. SHA-256：

```text
23b6fb775b045f1580660e06b36480e8ee83f8e4a2db1516ee760b118bec8981
```

6. zip 内容检查通过，包含：

```text
yian-windows-host.exe
start-yian-windows-host.cmd
README.md
identity-package/
config/
templates/
```

注意：

1. release build 仍有既有 unused / dead_code 警告，不阻断构建。
2. `.temp\vercel\windows-package.env` 中中文路径在当前控制台输出里显示为乱码，但实际 zip 产物路径存在，打包成功。后续如果该 env 文件要被自动发布系统读取，建议单独验证 UTF-8 读写链路。

## 下一批建议更新 8

建议下一批处理：

1. 如需 MSI，执行 `build_windows_host.ps1 -BuildMsi` 验证 WiX 打包。
2. 做一次人工截图检查，重点看：
   - StoryLock Core 侧边栏是否拥挤。
   - Protected Objects 表格列宽是否合适。
   - Learning / Export 页滚动区域是否顺畅。
3. 如果发布到下载页，继续执行 `release_windows_host.ps1` 或 `publish_windows_release.ps1` 生成 manifest / publish summary。
