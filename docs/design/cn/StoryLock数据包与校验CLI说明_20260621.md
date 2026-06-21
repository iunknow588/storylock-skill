# StoryLock 数据包与校验 CLI 说明

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 适用目录 | `skill/` |
| 对齐来源 | `story-lock/doc/design`、`story-lock/doc/usecase` |
| 当前状态 | 第一版共享加载、结构校验、检查 CLI 已落地 |

## 1. 定位

本说明用于承接 `story-lock/doc` 中关于 StoryLock Core 正式数据包、资源目录、模板文件、导出包和校验 CLI 的设计，并说明当前 `skill/src` 中已经落地的实现入口。

当前实现目标不是开放远程故事读写，而是先把本地 StoryLock Core 数据包变成可检查、可导入、可导出、可被宿主读取权限摘要的正式结构。

## 2. 当前数据包结构

当前第一版数据包目录至少包含：

```text
storylock-package/
  package-manifest.json
  resource-catalog.json
  author-draft.json
  templates/
    login-sites.json
    signing-actions.json
    agent-tasks.json
```

后续导出态会继续补齐 `vault.stlk`。在当前阶段，`vault.stlk` 仍属于导出清单中的保留文件名，不作为远程可读的明文故事文件。

## 3. 字段边界

### 3.1 Host 可读取

Host 主窗口或外部应用只能读取派生后的权限摘要：

1. `resourceId`
2. `resourceKind`
3. `providerId`
4. `displayName`
5. `role`
6. `objectId`
7. `objectKind`
8. `sensitivity`

这些字段用于生成授权对象列表、九宫格数量和请求说明。

### 3.2 Host 不可读取

Host 主窗口和远程网关不得读取或展示：

1. 故事原文。
2. 题库答案。
3. `canonicalAnswer`。
4. `acceptedAnswers`。
5. 密码原文。
6. 私钥、seed、`signingKeyBytes`。
7. 可重放 challenge material。

底层故事和加密配置只能在 StoryLock Core 子应用或本地核心模块中修改。

## 4. 代码入口

| 能力 | 路径 |
| --- | --- |
| 共享模块入口 | `src/shared/storylock-package/index.js` |
| manifest 校验 | `src/shared/storylock-package/manifest.js` |
| resource catalog 校验 | `src/shared/storylock-package/resource-catalog.js` |
| template bundle 校验 | `src/shared/storylock-package/templates.js` |
| author draft 校验 | `src/shared/storylock-package/author-draft.js` |
| 权限摘要派生 | `src/shared/storylock-package/resource-catalog.js` |
| schema | `src/shared/assets/schemas/*.schema.json` |
| CLI | `scripts/storylock-package/*.mjs` |
| 测试 | `scripts/test/storylock-package.mjs` |
| 样例 | `scripts/test/fixtures/storylock-package/` |

## 5. CLI 命令

### 5.1 校验完整数据包

```powershell
npm run validate:storylock-package -- scripts\test\fixtures\storylock-package\valid
```

### 5.2 检查数据包摘要

```powershell
npm run inspect:storylock-package -- scripts\test\fixtures\storylock-package\valid
```

### 5.3 校验资源目录

```powershell
npm run validate:storylock-catalog -- scripts\test\fixtures\storylock-package\valid
```

### 5.4 校验模板文件

```powershell
npm run validate:storylock-templates -- scripts\test\fixtures\storylock-package\valid
```

### 5.5 导出前检查

```powershell
npm run preflight:storylock-export -- scripts\test\fixtures\storylock-package\valid
```

## 6. CLI 输出格式

所有 CLI 输出稳定 JSON：

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

错误对象包含：

```json
{
  "code": "SLP-202",
  "level": "error",
  "message": "objectId must be a four-segment objectId.",
  "path": "$.resources[0].bindings[0].objectId",
  "suggestion": "Use <resourceKind>/<providerId>/<instanceSegment>/<role>."
}
```

## 7. 当前限制

1. 当前是结构校验和最小语义校验，不是最终加密包实现。
2. 当前尚未把 Windows Slint `StoryLock Core` 的表单完全接入真实读写。
3. 当前尚未让 Android / Linux 宿主共用正式 package loader。
4. 当前 `vault.stlk` 尚未完成正式封装、加密、导入和导出闭环。
5. 当前不开放远程故事读写。

## 8. 验收命令

```powershell
npm run test:storylock-package
npm run test:contract
node scripts\test\docs-consistency.mjs
node scripts\verify\path-consistency.mjs
```
