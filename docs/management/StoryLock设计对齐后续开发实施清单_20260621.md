# StoryLock 设计对齐后续开发实施清单

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 对应计划 | `StoryLock设计对齐后续开发计划_20260621.md` |
| 执行范围 | `skill/src`、`skill/scripts`、`skill/docs/design`、`skill/docs/test` |
| 当前结论 | P0/P1 第一版已落地；P2 Windows StoryLock Core 已完成主要持久化闭环；P3 授权通道枚举、策略映射、Windows Host 执行链、batch/story_edit 闭环测试和本地审计覆盖已落地；P4 Linux 已接入共享 package loader 与 permission summary，Android 已增加共享契约 readiness 约束 |

## 1. P0 正式数据模型冻结

### 1.1 新增共享模块目录

- [x] 新增 `src/shared/storylock-package/`。
- [x] 新增 `src/shared/storylock-package/index.js` 作为统一入口。
- [x] 新增 `src/shared/storylock-package/manifest.js`。
- [x] 新增 `src/shared/storylock-package/resource-catalog.js`。
- [x] 新增 `src/shared/storylock-package/templates.js`。
- [x] 新增 `src/shared/storylock-package/author-draft.js`。
- [x] 新增 `src/shared/storylock-package/permission-summary.js`。

验收方式：

```powershell
npm run test:storylock-package
```

### 1.2 新增 Schema

- [x] 新增 `src/shared/assets/schemas/storylock-package-manifest.schema.json`。
- [x] 新增 `src/shared/assets/schemas/storylock-resource-catalog.schema.json`。
- [x] 新增 `src/shared/assets/schemas/storylock-template-bundle.schema.json`。
- [x] 新增 `src/shared/assets/schemas/storylock-author-draft.schema.json`。
- [x] 新增 `src/shared/assets/schemas/storylock-node.schema.json`。
- [x] 新增 `src/shared/assets/schemas/storylock-permission-summary.schema.json`。

验收方式：

```powershell
npm run test:schemas
```

`test:schemas` 已在 `package.json` 中定义为 `node scripts/test/schema-contract.mjs`。

### 1.3 固化字段边界

- [x] 作者稿允许保存故事标题、摘要、记忆锚点、8 要素、24 节点、编辑备注。
- [ ] 发布态只允许保存问题表达、校验策略、答案摘要或 digest，不保存明文答案。
- [x] Host 权限摘要只允许暴露 objectId、resourceId、role、objectKind、action、challengePolicy、requiredGridCount、displayName。
- [x] Host 权限摘要不得暴露故事原文、canonicalAnswer、acceptedAnswers、password、privateKey、signingKeyBytes。

验收方式：

```powershell
npm run test:storylock-package
```

## 2. P1 导入导出与校验 CLI

### 2.1 新增 CLI 目录

- [x] 新增 `scripts/storylock-package/validate-package.mjs`。
- [x] 新增 `scripts/storylock-package/validate-catalog.mjs`。
- [x] 新增 `scripts/storylock-package/validate-templates.mjs`。
- [x] 新增 `scripts/storylock-package/inspect-package.mjs`。
- [x] 新增 `scripts/storylock-package/preflight-export.mjs`。

### 2.2 新增 npm scripts

- [x] `validate:storylock-package`
- [x] `validate:storylock-catalog`
- [x] `validate:storylock-templates`
- [x] `inspect:storylock-package`
- [x] `preflight:storylock-export`
- [x] `test:storylock-package`

### 2.3 稳定输出结构

所有 CLI 输出统一为 JSON：

```json
{
  "status": "ok",
  "command": "validate-package",
  "errors": [],
  "warnings": [],
  "infos": [],
  "summary": {}
}
```

- [x] 错误对象包含 `code`、`level`、`message`、`path`、`suggestion`。
- [ ] 错误码与 `story-lock/doc/design/08-导入校验错误码与错误消息规范.md` 完全对齐。
- [x] JSON 输出可被 CI 和人工脚本稳定解析。

验收方式：

```powershell
npm run validate:storylock-package -- scripts\test\fixtures\storylock-package\valid
npm run inspect:storylock-package -- scripts\test\fixtures\storylock-package\valid
```

## 3. P2 Windows StoryLock Core 持久化

### 3.1 Slint Core 子应用接入真实数据

- [x] `StoryLock Core` 子应用启动时加载本地 author draft。
- [x] Story 菜单保存故事标题、摘要、记忆锚点和 8 要素。
- [x] `24 Nodes` 菜单保存完整 24 个节点的 nodeId、title、elementId、question、selectionMode、candidatePoolSize、verifyPolicy。
- [x] Resources 菜单保存 resource catalog 第一版资源。
- [x] Templates 菜单保存登录、签名和 Agent 三类模板。
- [x] Export 菜单生成导出预览，不直接把私密字段交给 Host 主窗口。
- [x] Export 页面执行本地结构预检，覆盖必需文件、24 节点数量、四段式 objectId、模板 resourceId/role 引用。
- [x] Host 主窗口读取真实 resource catalog 派生的权限摘要，不再使用硬编码 managed object 示例。
- [x] Windows Rust 侧权限摘要字段语义与 `src/shared/storylock-package/permission-summary.js` 保持一致。

### 3.2 数据目录

- [x] 明确 Windows 默认数据目录。
- [x] 数据目录下至少包含 `author-draft.json`、`resource-catalog.json`、`package-manifest.json`、`templates/login-sites.json`、`templates/signing-actions.json`、`templates/agent-tasks.json`。
- [x] 保存失败时给出本地错误提示，不静默丢失。

验收方式：

```powershell
cargo test --manifest-path src\host\windows-host\Cargo.toml
cargo test --manifest-path src\host\windows-host\Cargo.toml --features ui-slint
npm run test:windows-host-features
```

## 4. P3 授权通道产品化

- [x] 新增授权通道枚举：`single_read`、`batch_read`、`story_edit`。
- [x] `single_read` 对齐 `6-of-24` 九宫格原子授权策略。
- [x] `batch_read` 对齐 `12-of-24` 全文回忆授权策略。
- [x] `story_edit` 对齐 `22-of-24` 本地高强度修改授权策略。
- [x] 远程网关不得触发 `story_edit`。
- [x] Windows Host 接入 `authorizationChannel`，并将通道策略映射到 `requiredStrength`、`allowedAction` 和 grid policy。
- [x] Windows Host 覆盖 `batch_read` 本地执行路径测试，结果仅返回脱敏批量读取授权摘要。
- [x] Windows Host 覆盖 `story_edit` 本地执行路径测试，结果仅返回本地 Core 编辑授权确认。
- [x] 所有拒绝、失败、撤销路径写入本地审计。

补充说明：

1. `src/skills/local-story-access` 已覆盖远程 `story_edit` 策略拒绝、challenge 失败、撤销成功、重复撤销失败等审计事件。
2. Windows Host 已写入本地 `audit/local-audit.jsonl`，仅保存请求 ID、能力、对象引用、状态、错误码和脱敏上下文，不保存答案、密码、私钥或签名材料。

验收方式：

```powershell
npm run test
```

## 5. P4 多平台统一落地

- [ ] Windows 使用 `src/shared/storylock-package` 加载正式数据包。
- [ ] Android 使用同一套 schema 和 permission summary。
- [x] Linux 使用同一套 schema 和 permission summary。
- [ ] 三个平台都不能从 Host 主界面直接编辑底层故事和加密配置。
- [ ] 补齐 Windows、Android、Linux 真机或桌面验收记录。

补充说明：

1. Linux Host 新增 `STORYLOCK_LINUX_STORYLOCK_PACKAGE_DIR` 和 `GET /permission-summary`，通过 `src/shared/storylock-package` 加载正式包并返回脱敏权限摘要。
2. Android readiness 已检查共享 schema、`permission-summary.js` 和 Android README 的 Core 配置边界声明；Android 原生 package loader 仍需后续实现后再勾选。

验收方式：

```powershell
npm run test:windows-package
node scripts\test\docs-consistency.mjs
node scripts\verify\path-consistency.mjs
```

## 6. 当前禁止越界项

1. 不开放远程故事读写。
2. 不把题库答案、私钥、密码、`signingKeyBytes` 暴露给远程 API。
3. 不让 Host 主窗口管理底层故事配置，只允许启动底层 `StoryLock Core` 子应用。
4. 不把 H5 浏览器页面作为 Windows 主 UI。
5. 不使用 VBS 作为正式启动方案。

## 7. 下一步编码建议

下一轮继续 P4 多平台统一落地：

1. 推进 Windows Rust UI 侧与 Android 原生侧共用正式 package loader / schema / permission summary。
2. 对齐 `story-lock/doc/design/08-导入校验错误码与错误消息规范.md` 的导入校验错误码。
