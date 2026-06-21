# StoryLock 中文设计文档索引

本目录用于说明当前 `skill/src` 主线代码的设计边界、术语口径和实现约束。阅读时请以仓库当前代码为准，不再沿用旧的 `src/storylock-*-skill` 路径命名。

## 当前主线目录

1. `src/skills/local-story-processing`
2. `src/skills/local-story-access`
3. `src/skills/remote-gateway`
4. `src/engine`

其中前三者分别对应三层主线 Skill；`src/engine` 仅用于兼容和演示，不是新的安全边界层。

## 推荐阅读顺序

1. `术语表.md`
2. `Skill定位与边界.md`
3. `运行层级与Skill分层.md`
4. `三包接口契约.md`
5. `对象访问策略.md`
6. `Challenge状态机.md`
7. `Session与防重放策略.md`
8. `安全规范.md`
9. `平台密钥存储适配指南.md`
10. `StoryLock数据包与校验CLI说明_20260621.md`
11. `YianWindowsHost菜单配置说明_20260621.md`
12. `EIP-712最小请求定义.md`
13. `易安与StoryLock技术说明书.md`

## 和评审资料的关系

1. `docs/design/cn/` 偏设计约束和实现原则。
2. `docs/ref/` 偏参赛提交、评审讲解和演示执行。
3. `docs/test/` 偏测试方案和回归检查。
4. `docs/management/` 偏当前工作配置、后续计划和阶段任务，`docs/management/BACK/` 保存历史工作档案。

## 当前阅读注意事项

1. 对外讲解时，优先使用“易安远程入口”“私人智能助理”“StoryLock 本地核心”等术语。
2. 若文档提到 Android 宿主、Windows 宿主、Android host 等表述，应理解为不同阶段的工程名或平台侧宿主实现，不代表额外的新层级。
3. 当前仓库已经具备三层自测闭环、Web API 网关闭环、题集导入校验和文本一致性检查；真机级 Android App 仍属于后续增强项。
4. StoryLock Core 数据包第一版校验入口已经落在 `src/shared/storylock-package/` 和 `scripts/storylock-package/`，但完整导入导出、宿主持久化和跨平台统一加载仍属于后续开发。
5. 所有命令请优先从 `skill/` 根目录运行，推荐先执行 `npm run test`。
6. 阶段性路线与历史进展材料以 `docs/management/` 当前文件为准，历史档案不作为当前设计基线。

## 常用验证命令

```powershell
npm run test
```

```powershell
node scripts/test/run-selftests.mjs
```

```powershell
npm run test:storylock-package
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify/encoding-check.ps1
```
