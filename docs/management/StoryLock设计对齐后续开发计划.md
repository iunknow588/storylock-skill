# StoryLock 设计对齐后续开发计划

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 对齐来源 | `E:\2026OPC大赛\story-lock\doc\design` |
| 实现基线 | `E:\2026OPC大赛\skill\src` |
| 结论 | 当前代码已完成三层 Skill 和多平台宿主原型，但尚未完整落地 StoryLock Core 正式文件格式、资源目录、模板校验、导入导出和故事工作台持久化 |

## 1. 总体结论

`skill/src` 当前已经完成了以下主线能力：

1. 第一层本地故事处理：
   - `StoryDraftSkill`
   - `StoryRefineSkill`
   - `StrengthReviewSkill`
2. 第二层本地授权：
   - 对象强度策略
   - 九宫格验证
   - 本地授权 session
   - nonce / requestId 防重放
   - 失败窗口和撤销
   - SQLite 审计状态
3. 第三层远程网关：
   - `requestSignature`
   - `requestPasswordFill`
   - EIP-712 请求表达
   - 脱敏响应
4. 平台宿主原型：
   - Windows Rust / Slint 原生窗口
   - Android Host 原型
   - Linux Host 原型
   - 平台 SecretStore 适配方向

但 `story-lock/doc/design` 中设计的以下内容尚未全部完成：

1. 正式 `vault.stlk` / 单文件头信息模型。
2. `resource-catalog.json`、`package-manifest.json`、`templates/*` 的正式 schema、校验器和 CLI。
3. 作者稿级故事工作台数据模型的持久化和导入导出。
4. `6-of-24`、`12-of-24`、`22-of-24` 三类授权通道的完整产品化区分。
5. 资源目录与登录、签名、Agent 模板的正式运行时解析。
6. 多平台宿主对同一套 Core 数据包的统一读写。

因此当前项目不是“只等待真机测试”，而是处于：

> 三层 Skill 与宿主原型基本可用，但距离 `story-lock/doc/design` 所描述的完整 StoryLock Core 产品化形态仍需要继续开发。

## 2. 已实现对照

| 设计主题 | 设计来源 | 当前实现位置 | 完成度 |
| --- | --- | --- | --- |
| 三层能力边界 | `01`、`02`、`03` | `src/skills/local-story-processing`、`src/skills/local-story-access`、`src/skills/remote-gateway` | 已实现 |
| 本地故事草稿与润色 | `13` | `src/skills/local-story-processing/index.js` | 已实现基础能力 |
| 题集强度评估 | `01`、`03`、`13` | `StrengthReviewSkill`、`question-set-master.schema.json` | 部分实现 |
| 九宫格验证与本地授权 | `01`、`02`、`03` | `src/skills/local-story-access`、Windows / Linux / Android Host | 已实现原型 |
| session / nonce / 防重放 | `02`、`03` | `access-host.js`、`sqlite-schema.sql` | 已实现 |
| 失败节流与撤销 | `03` | `access-host.js`、`LocalRevocationSkill` | 已实现 |
| 平台 SecretStore | `03`、`04` | `src/shared/secret-store.js`、Android Keystore、Windows DPAPI 原型 | 部分实现 |
| 远程请求包装 | `01`、`02` | `src/skills/remote-gateway` | 已实现 |
| Windows 原生宿主 | `13` 的工作台方向 | `src/host/windows-host/src/slint_ui.rs` | 原型实现 |

## 3. 未完成差距

### 3.1 StoryLock Core 正式数据包

设计要求：

1. `vault.stlk`
2. `resource-catalog.json`
3. `package-manifest.json`
4. `templates/login-sites.json`
5. `templates/signing-actions.json`
6. `templates/agent-tasks.json`

当前状态：

1. `src/engine/assets/migrated/runtime/resource-catalog.js` 有历史迁移实现。
2. Windows Slint UI 已展示这些字段的原型界面。
3. 尚未形成 `src/shared` 或 `src/skills` 下的正式导出包读写模块。
4. 尚未形成跨 Windows / Android / Linux 共用的数据包加载器。

后续任务：

1. 新增 `src/shared/storylock-package/`。
2. 实现 package manifest、resource catalog、template bundle 的读写模型。
3. 明确 `vault.stlk` 当前阶段是占位文件、DPAPI 包装文件，还是复用现有 question bank / secret store。
4. 增加导入、导出、校验自测。

### 3.2 资源目录正式化

设计要求：

1. 四段式 `objectId`：
   - `<resourceKind>/<providerId>/<instanceSegment>/<role>`
2. `resourceId + role -> objectId` 解析。
3. `objectMeta` 标注 `objectKind`、`encoding`、`sensitivity`。
4. 支持网站、钱包、API、证书等资源类型。

当前状态：

1. 迁移包中存在 resource catalog 解析。
2. 当前主线三层 Skill 没有把它提升为正式共享模块。
3. Windows Host 只展示脱敏权限摘要，未保存资源目录。

后续任务：

1. 将 `resource-catalog` 从迁移目录提升到 `src/shared/resource-catalog.js`。
2. 新增 schema：
   - `resource-catalog.schema.json`
   - `package-manifest.schema.json`
   - `legacy-object-id-map.schema.json`
3. 增加语义校验：
   - `resourceId` 唯一
   - 同一资源内 `role` 唯一
   - `objectId` 四段式
   - `objectMeta` 覆盖所有绑定对象
4. 将登录、签名、Agent 模板统一通过 `resourceId + role` 解析。

### 3.3 模板文件正式化

设计要求：

1. `templates/login-sites.json`
2. `templates/signing-actions.json`
3. `templates/agent-tasks.json`
4. 统一顶层结构：
   - `version`
   - `items`
5. 模板只描述动作，不保存秘密值。

当前状态：

1. 迁移包有模板解析逻辑。
2. 远程网关和 engine 层有示例模板。
3. 未形成正式 schema 和 CLI。

后续任务：

1. 新增三类模板 schema。
2. 新增 `validate-templates` CLI。
3. 将模板解析结果映射到：
   - 登录字段绑定
   - 签名主材料与附件
   - Agent 任务 required roles
4. 增加反例测试：
   - 登录字段重复
   - 签名模板缺少 `primaryRole`
   - Agent 模板引用不存在 role
   - 跨文件 `resourceId` 不一致

### 3.4 导入校验错误码与 CLI

设计要求：

1. `validate-package`
2. `validate-catalog`
3. `validate-templates`
4. `inspect-package`
5. `preflight-export`
6. 稳定 JSON 输出。
7. 错误码、错误等级、字段路径和建议动作。

当前状态：

1. 已有题集导入脚本：
   - `generate-question-set-template`
   - `validate-question-set`
   - `import-question-set`
2. 尚未实现完整身份包 / 资源目录 / 模板校验 CLI。

后续任务：

1. 新增 `scripts/storylock-package/`。
2. 实现五个 CLI 命令。
3. 统一输出结构：
   - `status`
   - `command`
   - `errors`
   - `warnings`
   - `infos`
   - `summary`
4. 将错误码分层：
   - 包与清单问题
   - 资源目录问题
   - 资源结构问题
   - 模板问题
   - 旧对象名迁移问题

### 3.5 故事工作台正式化

设计要求：

1. 作者稿层：
   - 故事标题
   - 摘要
   - 记忆锚点
   - 8 个要素
   - 24 个节点
2. 节点层字段：
   - `nodeId`
   - `title`
   - `elementId`
   - `recommendedSelectionMode`
   - `recommendedCorrectCount`
   - `candidatePoolSize`
   - `recallPriority`
   - `verifyPolicy`
   - `editorNotes`
3. 运行时题目层：
   - 问题
   - 正确项
   - 干扰项
   - 答案摘要或 digest

当前状态：

1. `src/yian-web/public/storylock-app.html/js` 有 H5 原型。
2. Windows Slint `StoryLock Core` 有字段展示和编辑原型。
3. 尚未实现作者稿 JSON 的正式保存、导入、导出和校验。
4. 尚未接入第一层 `StoryDraftSkill` / `StoryRefineSkill` 的真实调用。

后续任务：

1. 新增作者稿 schema：
   - `story-author-draft.schema.json`
   - `story-node.schema.json`
2. 将 Windows `StoryLock Core` 的字段接入本地 JSON 存储。
3. 支持导入 / 导出作者稿。
4. 支持从作者稿生成当前 `question-set-master`。
5. 明确哪些字段只在作者态存在，哪些字段可进入发布态。

### 3.6 授权通道完整区分

设计要求：

1. 普通读取：`6-of-24` 九宫格原子授权。
2. 批量读取：`12-of-24` 全文回忆授权。
3. 故事修改：`22-of-24` 全文回忆或恢复材料。

当前状态：

1. 第二层已经支持 `low / medium / high` 和九宫格验证策略。
2. 当前 Windows / Linux / Android Host 主要是签名和密码填充路径。
3. 尚未把 `6-of-24`、`12-of-24`、`22-of-24` 做成产品级通道。
4. 故事修改仍未开放远程读写，符合安全边界，但本地修改流程尚未完整实现。

后续任务：

1. 在第二层增加明确的通道枚举：
   - `single_read`
   - `batch_read`
   - `story_edit`
2. 将通道映射到不同 challenge 策略。
3. 对 `story_edit` 增加本地二次确认和更高强度校验。
4. 保持远程故事读写关闭。

### 3.7 多平台正式宿主

当前状态：

1. Windows：Rust / Slint 原生窗口可运行，StoryLock Core 为原型界面。
2. Android：已有 Keystore、BiometricPrompt、Host service 原型。
3. Linux：已有 Secret Service 路径、Node Host、打包原型。

未完成：

1. 三个平台没有共用同一套正式 StoryLock package loader。
2. Windows StoryLock Core 未持久化作者稿和配置。
3. Android / Linux 尚未提供与 Windows 对等的底层 StoryLock Core 配置界面。
4. 真机验收仍需补齐。

## 4. 优先级计划

### P0：冻结正式数据模型

目标：

1. 确定 `identity-package/` 文件结构。
2. 确定 `resource-catalog.json`、`package-manifest.json`、`templates/*` schema。
3. 确定作者稿 JSON schema。

产出：

1. `src/shared/storylock-package/`
2. `src/shared/resource-catalog.js`
3. `src/shared/templates.js`
4. `src/shared/assets/schemas/*.schema.json`

验收：

1. schema contract 测试通过。
2. 合法样例通过。
3. 非法样例返回稳定错误码。

### P1：实现导入导出与校验 CLI

目标：

1. 支持 `validate-package`。
2. 支持 `validate-catalog`。
3. 支持 `validate-templates`。
4. 支持 `inspect-package`。
5. 支持 `preflight-export`。

产出：

1. `scripts/storylock-package/*.mjs`
2. `npm run validate:storylock-package`
3. `npm run inspect:storylock-package`

验收：

1. CLI 输出稳定 JSON。
2. 错误码与 `08-导入校验错误码与错误消息规范.md` 对齐。
3. CI 可运行。

### P2：Windows StoryLock Core 持久化

目标：

1. Windows `StoryLock Core` 不再只是字段原型。
2. 支持本地保存作者稿、资源目录、模板和导出包预览。
3. Host 主窗口继续只读脱敏权限摘要。

产出：

1. Windows Core 数据目录：
   - `author-draft.json`
   - `resource-catalog.json`
   - `templates/*.json`
   - `package-manifest.json`
2. Slint 保存、加载、校验按钮。
3. 权限摘要派生接口。

验收：

1. 关闭重启后配置仍在。
2. Host 不能直接读取故事原文、答案、密码、私钥。
3. Host 只能读取对象、动作、强度、九宫格数量。

### P3：授权通道产品化

目标：

1. 明确 `single_read`、`batch_read`、`story_edit`。
2. 将通道与不同 challenge 策略绑定。
3. 本地故事修改必须高强度确认。

产出：

1. 第二层策略更新。
2. Windows / Android / Linux Host 对齐。
3. 回归测试覆盖三类通道。

验收：

1. 普通签名 / 密码填充不被破坏。
2. 故事修改不能从远程直接触发。
3. 错误和拒绝路径可审计。

### P4：多平台统一落地

目标：

1. Android / Linux 使用同一套 package loader 和 schema。
2. Windows、Android、Linux 共享相同权限摘要格式。
3. 补齐真机验收。

产出：

1. Android package loader。
2. Linux package loader。
3. 三平台验收记录。

验收：

1. Windows 本机运行通过。
2. Android 真机运行通过。
3. Linux 桌面或 WSL 验收通过。

## 5. 当前不建议推进的事项

1. 不建议近期开放远程故事读写。
2. 不建议把题库答案、私钥、密码或 `signingKeyBytes` 暴露给远程 API。
3. 不建议让 Host 主窗口直接编辑底层故事和加密配置。
4. 不建议在 Core 中硬编码网站账号、钱包账号、API 凭据等固定领域类型。
5. 不建议先做复杂云同步，再补本地数据包校验。

## 6. 验收入口建议

新增回归命令建议：

```powershell
npm run test:storylock-package
npm run validate:storylock-package -- --input .temp/storylock-package
npm run inspect:storylock-package -- --input .temp/storylock-package
```

保留现有回归：

```powershell
npm run test
npm run test:windows-host-features
npm run test:windows-package
node scripts\test\docs-consistency.mjs
node scripts\verify\path-consistency.mjs
```

## 7. 下一步建议

建议下一轮开发先做 P0 + P1：

1. 把迁移目录中的 `resource-catalog` 和 `templates` 能力提升为正式共享模块。
2. 增加正式 JSON Schema。
3. 增加 package 校验 CLI。
4. 用 `story-lock/doc/usecase` 中的样例做合法样例和非法样例测试。

完成后再把 Windows `StoryLock Core` 界面接入真实读写。
