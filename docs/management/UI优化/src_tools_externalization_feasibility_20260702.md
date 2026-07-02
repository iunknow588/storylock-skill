# tools 迁移可行性分析

日期：2026-07-02

## 当前状态

相关工具路径已经调整为：

1. `skill/tools/render-storylock-ui-docs`
2. `E:\2026OPC大赛\project-file-png-renderer`

这说明历史上的 `skill/src/tools` 已经退出当前工具承载路径。

## 结论

本次迁移方向是合理的，原因有三点：

1. `render-storylock-ui-docs` 属于文档截图工具，不属于 Windows Host 主运行时路径。
2. `project-file-png-renderer` 本身就是通用工具，独立到 `skill/` 外部更符合职责边界。
3. `skill` 仓库本身没有顶层 Cargo workspace，这些工具原本就不是主产物构建的一部分。

## 迁移后的边界

### 1. `render-storylock-ui-docs`

当前建议继续保留在 `skill/tools/` 下，而不是立即继续拆到仓库外。

原因：

1. 它仍然在编译期直接引用 `skill/src/host/windows-host/src/slint_ui` 下的 Slint 文件。
2. 它和 Windows Host UI 的关系仍然是“同仓协作工具”，不是完全独立产品。

当前代码落点：

- [main.rs](/e:/2026OPC大赛/skill/tools/render-storylock-ui-docs/src/main.rs)
- [README.md](/e:/2026OPC大赛/skill/tools/README.md)

### 2. `project-file-png-renderer`

当前迁到仓库外是合理的。

原因：

1. 它是通用文件渲染方向，不是 StoryLock UI 专用逻辑。
2. 它和 `skill` 主体没有直接运行时耦合。

当前外部路径：

- [README.md](/e:/2026OPC大赛/project-file-png-renderer/README.md)

## 本次清理需要同步的内容

迁移后必须保持下面几类引用一致：

1. `cargo run --manifest-path .\tools\render-storylock-ui-docs\Cargo.toml`
2. `tools/render-storylock-ui-docs/docs/*` 下的说明
3. `src/host/windows-host/tools/ui-doc-screenshots/README.md`
4. UI 优化总文档中的截图工具路径说明
5. `render-storylock-ui-docs` 自身的 Slint 相对导入与默认输出目录

## 后续建议

1. 保持 `render-storylock-ui-docs` 在 `skill/tools/` 下稳定一段时间。
2. 后续如果要把它彻底迁出仓库，先为 Windows Host UI 建稳定共享入口，避免继续依赖深层相对路径。
3. `project-file-png-renderer` 后续演进可完全按独立工具思路推进，不需要再放回 `skill/`。
