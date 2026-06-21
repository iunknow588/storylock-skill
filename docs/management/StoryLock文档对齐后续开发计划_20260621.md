# StoryLock 文档对齐后续开发计划

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 对比来源 | `E:\2026OPC大赛\story-lock\doc` |
| 当前仓库 | `E:\2026OPC大赛\skill\docs` |
| 结论 | `skill/docs` 已覆盖实现仓库的三层 Skill、平台宿主、测试和管理入口；架构接线、用例样例映射、文档编号治理和文档-代码-测试映射已补第一版，剩余主要是真机/桌面验收记录 |

## 1. 总体结论

`story-lock/doc` 是完整产品设计与用例文档集，包含 `20260pc/`、`architecture/`、`design/`、`management/`、`usecase/` 五条主线。

`skill/docs` 是实现仓库文档集，当前重点是说明 `skill/src` 的三层 Skill、平台宿主、StoryLock package 校验入口、测试入口和阶段计划。

因此不建议把 `story-lock/doc` 全量复制到 `skill/docs`。正确做法是：

1. `skill/docs/design` 只保留与当前代码直接相关的设计说明。
2. `skill/docs/management` 保存当前任务计划、实施清单、对齐差异和验收状态。
3. `story-lock/doc/usecase` 中的故事模板和完整用例作为上游产品资料，按需转化为测试 fixture 或用户操作说明。
4. `story-lock/doc/architecture` 中的云接线、备份和宿主组件映射，按需转化为 `skill/docs/design` 中的平台实现说明。

## 2. 已修复问题

### 2.1 根文档入口过旧

问题：

1. `docs/readme-cn.md` 仍以 2026-06-20 状态为主。
2. 没有列出 StoryLock package 第一版校验入口。
3. 后续计划仍主要指向 20260620 Windows 阶段文档。

修复：

1. 已把版本更新为 2026-06-21。
2. 已增加 `npm run test:storylock-package`、`validate:storylock-package`、`inspect:storylock-package` 入口。
3. 已把当前主计划切换到 `StoryLock设计对齐后续开发计划_20260621.md` 和本文件。

### 2.2 设计文档缺少数据包与 CLI 说明

问题：

`story-lock/doc/design` 已有资源目录、导出包、模板、JSON Schema、CLI 等完整设计，但 `skill/docs/design/cn` 原先没有对应的实现说明。

修复：

新增：

1. `docs/design/cn/StoryLock数据包与校验CLI说明_20260621.md`
2. `docs/design/cn/README.md` 推荐阅读入口。

### 2.3 管理目录缺少上游文档对齐总账

问题：

`docs/management` 有开发计划和实施清单，但缺少一份明确说明“相比 `story-lock/doc`，`skill/docs` 还缺什么、哪些不应复制、哪些需要后续转化”的文档。

修复：

新增本文件作为文档对齐总账。

## 3. 已补齐的问题

### 3.1 缺少 `architecture/` 等价索引

`story-lock/doc/architecture` 包含：

1. 总体技术方案。
2. 宿主接线。
3. 云备份包裹格式。
4. 华为云 KMS / OBS 接线。
5. 资源目录解析与组件映射。
6. 导入导出工作流。

`skill/docs` 当前没有单独 `docs/architecture`，相关内容分散在 `docs/design/cn`、`docs/ref` 和代码脚本中。

处理结果：

1. 不新建大量历史架构文档。
2. 已新增 `docs/design/cn/平台宿主与数据包接线说明_20260621.md`。
3. 已说明 Windows / Android / Linux 如何读取 StoryLock package 或 permission summary。
4. 云备份和 KMS / OBS 仍保持“后续增强”，未提前宣称完成。

### 3.2 缺少 `usecase/` 到测试 fixture 的映射

`story-lock/doc/usecase` 包含：

1. 24 节点故事模板。
2. 网站账号资源目录样例。
3. 钱包账号资源目录样例。
4. API 凭据资源目录样例。
5. 证书与开发者身份资源目录样例。
6. 模板文件样例集。
7. 完整导出包样例。
8. 故事工作台用户手册。

`skill/docs` 当前只有测试 fixture 的第一版最小样例，还没有说明这些上游样例如何转化。

处理结果：

1. 已新增 `docs/design/cn/StoryLock用例样例到测试夹具映射_20260621.md`。
2. 已说明网站账号、钱包、API、证书四类样例的当前 fixture 映射状态。
3. 已说明非法样例应覆盖重复 role、非法 objectId、缺失 template binding、节点数量错误和 Host 可读字段泄漏。

### 3.3 缺少文档编号与命名治理

`story-lock/doc/02-文档编号与命名规范.md` 有正式编号规范。

`skill/docs` 当前有中英文设计文档和日期后缀文档，但没有说明：

1. 哪类文档允许日期后缀。
2. 哪类文档应保持稳定名称。
3. `docs/management/BACK` 的归档规则。
4. `docs/ref/` 被排除 Git 管理后的使用边界。

处理结果：

1. 已新增 `docs/management/StoryLock文档命名与归档规则_20260621.md`。
2. 已明确 `docs/design` 放正式技术说明，`docs/management` 放当前任务配置，`docs/management/BACK` 放历史档案，`docs/ref` 为本地参赛参考资料且不进入 Git。

### 3.4 缺少中英文同步策略

`skill/docs/design/en` 有英文文档，但新增的中文文档不一定都有英文版本。

处理结果：

1. 当前开发阶段以中文设计为主。
2. 面向外部评审或交付的英文文档只翻译稳定版本。
3. 在 `docs/design/en/readme.md` 标注哪些中文文档尚无英文版。

### 3.5 缺少文档与代码能力的自动映射表

当前对齐依赖人工阅读，后续应补：

1. `story-lock/doc/design` 条目到 `skill/src` 模块的映射。
2. `story-lock/doc/usecase` 条目到 fixture / CLI / UI 的映射。
3. 每条能力的完成度、测试命令和验收方式。

处理结果：

1. 已在 `docs/management` 增加 `StoryLock文档代码测试映射表_20260621.md`。
2. 已把该表作为发布前检查和归档判断依据。

## 4. 优先级计划

### P0：完成当前文档入口修复

状态：已完成。

产出：

1. `docs/readme-cn.md`
2. `docs/readme.md`
3. `docs/design/cn/README.md`
4. `docs/design/cn/StoryLock数据包与校验CLI说明_20260621.md`
5. `docs/management/StoryLock文档对齐后续开发计划_20260621.md`

验收：

```powershell
node scripts\test\docs-consistency.mjs
node scripts\verify\path-consistency.mjs
```

### P1：补架构接线说明

状态：已完成第一版。

产出：

1. `docs/design/cn/平台宿主与数据包接线说明_20260621.md`
2. Windows / Android / Linux 共享 package loader 调用边界。
3. 云备份、KMS、OBS 标注为后续增强，不写成已完成。

### P2：补 usecase 到 fixture 映射

状态：已完成第一版。

产出：

1. `docs/design/cn/StoryLock用例样例到测试夹具映射_20260621.md`
2. 网站账号、钱包、API、证书四类合法 fixture。
3. 至少四类非法 fixture。

### P3：补文档命名与归档规则

状态：已完成第一版。

产出：

1. `docs/management/StoryLock文档命名与归档规则_20260621.md`
2. `docs/management/README.md` 更新归档规则。

### P4：补文档-代码-测试映射表

状态：已完成第一版。

产出：

1. `docs/management/StoryLock文档代码测试映射表_20260621.md`
2. 每条能力包含来源文档、代码路径、测试命令、完成状态、风险。

## 5. 当前不建议做的事

1. 不建议把 `story-lock/doc` 全量复制到 `skill/docs`。
2. 不建议把 `story-lock/doc/20260pc` 中的宣传材料混入 `skill/docs/design`。
3. 不建议把 `docs/ref/` 重新纳入 Git 管理。
4. 不建议在文档中宣称 Android / Linux / macOS 已达到正式交付级。
5. 不建议开放远程故事读写或在文档中暗示远程可读取故事原文、答案、密码或私钥。
