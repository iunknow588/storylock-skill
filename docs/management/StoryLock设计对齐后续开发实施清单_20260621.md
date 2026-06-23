# StoryLock 设计对齐后续开发实施清单

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 对应计划 | `StoryLock设计对齐后续开发计划_20260621.md` |
| 执行范围 | `skill/src`、`skill/scripts`、`skill/docs/design`、`skill/docs/test` |
| 当前结论 | P0/P1/P2/P3 代码侧已完成主要闭环；P4 Windows/Android/Linux 已完成自动化接线检查；导入校验错误码已按 `SL_*` 风格对齐；真机实测之外的内容已纳入 `npm run test:non-device-validation`，状态汇总以 `non_device_ready` 表示完成；当前不应归档，剩余工作是真机/真实桌面验收记录 |

2026-06-22 复核补充：`npm run test:linux-host` 已修复并通过。修复内容为 Linux 默认 `question-bank.json` 和测试导入夹具从 9 题补齐到 24 题，并按 challenge cell 的 `questionId` 提交答案，确保 `requestSignature` 高强度 12 格验证可以完成自动闭环。

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

补充说明：

1. Host 可读文件校验已覆盖 `package-manifest.json`、`resource-catalog.json` 和三类 templates，禁止出现 `canonicalAnswer`、`acceptedAnswers`、`correctOptions`、`password`、`privateKey`、`signingKeyBytes` 等明文字段。
2. `author-draft.json` 属于 StoryLock Core 本地作者态文件，允许保留编辑期内容，但不得由 Host 主界面直接编辑或远程读取。

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
- [x] 发布态只允许保存问题表达、校验策略、答案摘要或 digest，不保存明文答案。
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
- [x] 错误码与 `story-lock/doc/design/08-导入校验错误码与错误消息规范.md` 完全对齐。
- [x] JSON 输出可被 CI 和人工脚本稳定解析。

验收方式：

```powershell
npm run validate:storylock-package -- scripts\test\fixtures\storylock-package\valid
npm run inspect:storylock-package -- scripts\test\fixtures\storylock-package\valid
```

补充说明：

1. CLI issue 输出已同时包含 `level` 与 `severity`，其中 error 对应 `blocking`。
2. manifest/catalog/template 接线相关错误码已采用 `SL_*` 风格，例如 `SL_PKG_MISSING_MANIFEST`、`SL_CATALOG_INVALID_OBJECT_ID`、`SL_TEMPLATE_UNKNOWN_ROLE`。

## 3. P2 Windows StoryLock Core 持久化

### 3.1 Slint Core 子应用接入真实数据

- [x] `StoryLock Core` 子应用启动时加载本地 author draft。
- [x] `24 Questions` 作为第一主功能，支持 1-24 下拉选择和前后切换，保存 24 个问题，每题固定 9 个候选答案，并保存每个答案的正确/错误标记。
- [x] `Managed Objects` 作为第二主功能，保存待管理对象的 `resourceId`、`objectId`、对象类型、权限答对数量和本地秘密引用。
- [x] `Save & Confirm` 作为第三主功能，展示保存前学习训练、高阶授权确认频率和导出预检入口。
- [x] `Story Aids` 和 `Templates` 作为附属功能，只为 24 个问题和管理对象工作流提供参考和绑定。
- [x] 导出预览不直接把私密字段交给 Host 主窗口。
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

- [x] Windows 使用 `src/shared/storylock-package` 语义加载正式数据包并通过 Rust/JS 权限摘要契约测试。
- [x] Android 使用同一套 schema 和 permission summary。
- [x] Linux 使用同一套 schema 和 permission summary。
- [x] 三个平台都不能从 Host 主界面直接编辑底层故事和加密配置。
- [ ] 补齐 Windows、Android、Linux 真机或桌面验收记录。

补充说明：

1. Linux Host 新增 `STORYLOCK_LINUX_STORYLOCK_PACKAGE_DIR` 和 `GET /permission-summary`，通过 `src/shared/storylock-package` 加载正式包并返回脱敏权限摘要。
2. Windows Rust UI 已新增结构化 permission summary，并通过 `scripts/storylock-package/permission-summary-json.mjs` 与 JS 共享实现做契约测试；后续仍需把 Rust 侧预检/加载器进一步替换为正式共享生成物或生成代码。
3. Android 已新增 `storylock-resource-catalog.json` asset、`AndroidStoryLockPackageRepository` 和 `GET /permission-summary`，按共享 permission summary 语义返回脱敏摘要；Host 仍不得编辑底层 StoryLock Core 配置。

验收方式：

```powershell
npm run test:non-device-validation
npm run test:windows-package
npm run test:android-readiness
npm run test:linux-host
npm run diagnose:linux-secret-service:wsl
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

下一轮继续验收阶段：

1. 补齐 Windows、Android、Linux 真机或桌面验收记录。
2. 重点确认 Windows Slint UI、Android 通过 `adb forward` 暴露的 `/permission-summary`、Linux `/permission-summary` 和 Linux Secret Service 的实际运行表现。
3. 人工验收完成后，再把 20260621 阶段管理文档移动到 `docs/management/BACK/`。
