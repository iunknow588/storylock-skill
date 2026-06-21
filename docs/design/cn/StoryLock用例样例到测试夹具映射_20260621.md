# StoryLock 用例样例到测试夹具映射

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 上游用例目录 | `story-lock/doc/usecase` |
| 当前测试夹具目录 | `scripts/test/fixtures/storylock-package` |

## 1. 定位

`story-lock/doc/usecase` 是完整产品用例资料，包含 24 节点故事模板、网站账号、钱包账号、API 凭据、证书身份、模板文件和完整导出包样例。

`skill` 仓库不应全量复制这些产品资料，而应把其中稳定的结构转成可执行测试夹具，用于校验当前代码是否符合设计边界。

## 2. 当前已落地夹具

| 夹具 | 用途 |
| --- | --- |
| `scripts/test/fixtures/storylock-package/valid/package-manifest.json` | 合法数据包清单 |
| `scripts/test/fixtures/storylock-package/valid/resource-catalog.json` | 合法资源目录，覆盖登录凭据和钱包签名对象 |
| `scripts/test/fixtures/storylock-package/valid/author-draft.json` | 合法 24 节点作者稿 |
| `scripts/test/fixtures/storylock-package/valid/templates/login-sites.json` | 登录模板 |
| `scripts/test/fixtures/storylock-package/valid/templates/signing-actions.json` | 签名模板 |
| `scripts/test/fixtures/storylock-package/valid/templates/agent-tasks.json` | Agent 任务模板 |
| `scripts/test/fixtures/storylock-package/invalid/*` | 非法结构与越界字段反例 |

当前 `npm run test:storylock-package` 已验证：

1. 合法夹具可以加载、校验和 inspect。
2. 合法权限摘要不会出现密码、私钥或 `signingKeyBytes`。
3. 非法夹具可以触发稳定错误码。
4. Host 可读模板禁止出现 `canonicalAnswer` 等私密字段。

## 3. 上游用例映射

| 上游文档 | 当前映射状态 | 后续处理 |
| --- | --- | --- |
| `01-24节点故事模板（时间地点人物主题冲突情节结论）.md` | 已映射到 `valid/author-draft.json` 的 24 节点结构 | 后续增加更多真实故事案例 |
| `02-网站账号资源目录样例.md` | 已映射到 `valid/resource-catalog.json` 的登录凭据对象 | 后续增加多站点、多角色样例 |
| `03-钱包账号资源目录样例.md` | 已映射到签名对象与 `signing-actions.json` | 后续增加多链钱包和算法策略 |
| `04-API凭据资源目录样例.md` | 尚未单独形成 fixture | 后续加入 API key、token、endpoint role |
| `05-证书与开发者身份资源目录样例.md` | 尚未单独形成 fixture | 后续加入 certificate、developer identity role |
| `07-模板文件样例集（登录签名与Agent）.md` | 已映射到三类 `templates/*.json` | 后续扩展字段覆盖率 |
| `08-完整导出包样例（含清单资源目录与模板）.md` | 已映射到最小完整 package fixture | 后续补 `vault.stlk` 封装和导出包校验 |
| `12-StoryLock故事工作台用户使用说明书.md` | 已映射到 Windows StoryLock Core 子应用方向 | 后续补真实操作验收记录 |

## 4. 当前反例覆盖

非法夹具应持续覆盖以下风险：

1. 缺失 `package-manifest.json` 必需文件声明。
2. 非四段式 `objectId`。
3. 重复 `resourceId` 或重复 role。
4. 模板引用不存在的 `resourceId` 或 role。
5. 作者稿节点数量不是 24。
6. Host 可读文件出现答案、密码、私钥或签名材料字段。

## 5. 后续扩展顺序

建议按以下顺序继续扩展 fixture：

1. API 凭据合法样例。
2. 证书与开发者身份合法样例。
3. 旧对象名迁移样例。
4. 多资源协同工作流样例。
5. `vault.stlk` 导出态封装样例。

每增加一类样例，都应同步补：

1. 合法 fixture。
2. 至少一个非法 fixture。
3. `scripts/test/storylock-package.mjs` 中的断言。
4. 文档中的映射说明。

## 6. 验收命令

```powershell
npm run test:storylock-package
npm run validate:storylock-package -- scripts\test\fixtures\storylock-package\valid
npm run inspect:storylock-package -- scripts\test\fixtures\storylock-package\valid
```
