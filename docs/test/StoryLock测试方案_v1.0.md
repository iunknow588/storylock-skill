# StoryLock 测试方案 v1.0

版本：v1.0  
日期：2026-06-20  
范围：`skill/`

## 1. 目标

本文档用于说明 StoryLock 当前仓库的测试基线、执行命令、覆盖范围与验收口径。  
当前主线以 `skill/src` 下三层运行时和一个兼容演示包为准：

1. `src/skills/local-story-processing`
2. `src/skills/local-story-access`
3. `src/skills/remote-gateway`
4. `src/engine`

## 2. 测试原则

1. 先保证主线运行时可自测通过。
2. 再验证跨层闭环、自定义题集、路径一致性、文本格式一致性。
3. 对外口径只描述当前仓库已验证的能力，不把历史接口当作现行能力。

## 3. 测试分层

| 分类 | 目标 | 入口 |
| --- | --- | --- |
| 运行时自测 | 验证三层运行时和兼容包可执行 | `node scripts/test/run-selftests.mjs` |
| 文本格式检查 | 验证文本文件为 UTF-8 无 BOM + LF | `npm run test:text` |
| Schema/契约检查 | 验证主要 schema 与能力契约 | `npm run test:contract` |
| 文档一致性检查 | 验证文档不回退到旧主线描述 | `npm run test:docs` |
| 路径一致性检查 | 验证主线文档不再引用旧目录路径 | `npm run test:paths` |

## 4. 一键验收

在 `skill/` 根目录执行：

```powershell
npm run test
```

该命令会依次执行：

1. `npm run selftest`
2. `npm run test:text`
3. `npm run test:contract`
4. `npm run test:docs`
5. `npm run test:paths`

## 5. 单项测试命令

### 5.1 工作区总自测

```powershell
node scripts/test/run-selftests.mjs
```

### 5.2 第一层：本地故事处理

```powershell
Push-Location src/skills/local-story-processing
npm run selftest
Pop-Location
```

覆盖重点：

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

### 5.3 第二层：本地授权与九宫格

```powershell
Push-Location src/skills/local-story-access
npm run selftest
Pop-Location
```

覆盖重点：

1. 对象强度策略
2. 九宫格 challenge 生成
3. 本地答案校验
4. 授权与撤销
5. 重放保护
6. 锁定与自动解锁
7. SQLite 清理
8. SecretStore 约束
9. 题集导入与模板生成

### 5.4 第三层：远程网关

```powershell
Push-Location src/skills/remote-gateway
npm run selftest
Pop-Location
```

覆盖重点：

1. `requestSignature`
2. `requestPasswordFill`
3. EIP-712 domain 生成与环境区分
4. 敏感字段递归脱敏

### 5.5 三层 E2E

```powershell
Push-Location src/skills/remote-gateway
npm run selftest:e2e
Pop-Location
```

覆盖重点：

1. 第三层签名请求包装
2. 第二层对象强度判断
3. 第二层九宫格验证
4. 第二层短时授权
5. 本地签名执行
6. 审计写入
7. 第三层脱敏返回

### 5.6 Web API + Android Host Mock

```powershell
Push-Location src/skills/remote-gateway
npm run selftest:web-api-android
Pop-Location
```

覆盖重点：

1. Web API 网关入口
2. Android host 注册与 relay
3. APK/下载元数据
4. 本地执行与脱敏

### 5.7 兼容演示包

```powershell
Push-Location src/engine
npm run selftest
Pop-Location
```

说明：

1. `src/engine` 是兼容与演示包。
2. 它不是新的第四层安全边界。

## 6. 文本格式一致性

### 6.1 检查

```powershell
npm run test:text
```

当前实际规则由 `scripts/text/` 定义：

1. UTF-8 without BOM
2. LF line endings
3. 发现格式偏差时，dry-run 也会返回失败

### 6.2 修复

```powershell
scripts\verify\encoding-check.ps1 -Fix
```

或：

```powershell
python scripts\text\normalize_text_files.py --root . --fix
```

## 7. 题集测试与导入

### 7.1 生成模板

```powershell
Push-Location src/skills/local-story-access
npm run generate:question-set-template
Pop-Location
```

默认生成：

`src/skills/local-story-access/assets/question-set-master.sample.json`

### 7.2 Dry-run 校验

```powershell
Push-Location src/skills/local-story-access
npm run validate:question-set -- --input assets/question-set-master.sample.json --require-min-active 24
Pop-Location
```

### 7.3 持久化导入

```powershell
Push-Location src/skills/local-story-access
npm run import:question-set -- --input assets/question-set-master.sample.json --db storylock.db --use-platform-secret-store
Pop-Location
```

### 7.4 题集验收要点

1. `identityId`
2. `questionSetVersion`
3. `normalizationVersion`
4. `status`
5. `questions`
6. 每题至少包含 `options` 或 `optionDigest`

对应 schema：

`src/skills/local-story-access/assets/schemas/question-set-master.schema.json`

## 8. 重点风险项测试清单

### 8.1 安全与授权

1. 过期请求拒绝
2. requestId/nonce 重放冲突
3. challenge 连续失败锁定
4. 未显式开发模式时拒绝 `MemorySecretStore`
5. production 持久化 host 不默认启用 legacy fallback

### 8.2 远程网关

1. 非法算法拒绝
2. 生产 EIP-712 domain 禁止 placeholder
3. 返回结果中 `password` / `privateKey` / `signingKeyBytes` 必须脱敏

### 8.3 仓库一致性

1. 文档不回退到旧 story read/write 主线
2. 主线文档不引用 `src/storylock-*` 旧目录
3. 文本文件格式统一

## 9. 通过标准

通过标准如下：

1. `npm run test` 全量通过
2. 关键自测输出包含 `passed`
3. 不存在 BOM / CRLF / 非 UTF-8 规范化差异
4. 文档主线与当前目录结构一致
5. 三层 E2E 与 Web API + Android host mock 闭环通过

## 10. 当前不纳入主线验收的内容

以下内容不作为当前主线通过标准：

1. 历史 `requestChallengeSign` 接口
2. 历史 story read/write 主线
3. 真实 Android App 真机工程闭环
4. 多链钱包与多账号产品化能力
5. 硬件级 HSM 能力

这些内容如需描述，只能作为历史背景、兼容说明或后续规划。
