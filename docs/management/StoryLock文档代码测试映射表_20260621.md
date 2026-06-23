# StoryLock 文档代码测试映射表

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 用途 | 将上游设计、当前代码和可执行测试放到同一张检查表中 |

## 1. 核心能力映射

| 能力 | 设计来源 | 代码路径 | 测试命令 | 状态 |
| --- | --- | --- | --- | --- |
| 第一层故事处理 | `story-lock/doc/design/01`、`02`、`13` | `src/skills/local-story-processing` | `npm run selftest` | 已实现基础能力 |
| 第二层本地授权 | `story-lock/doc/design/01`、`02`、`03` | `src/skills/local-story-access` | `npm run selftest` | 已实现 |
| 第三层远程网关 | `story-lock/doc/design/01`、`02` | `src/skills/remote-gateway` | `npm run selftest:web-api-android` | 已实现安全出口 |
| StoryLock package 加载与校验 | `story-lock/doc/design/05`、`07`、`08`、`10` | `src/shared/storylock-package` | `npm run test:storylock-package` | 第一版已实现 |
| JSON Schema 契约 | `story-lock/doc/design/07`、`10` | `src/shared/assets/schemas` | `npm run test:schemas` | 第一版已实现 |
| 校验 CLI | `story-lock/doc/design/08` | `scripts/storylock-package` | `npm run validate:storylock-package -- scripts\test\fixtures\storylock-package\valid` | 第一版已实现 |
| Windows 原生 Host | `story-lock/doc/design/13` | `src/host/windows-host` | `cargo test --manifest-path src\host\windows-host\Cargo.toml` | 原型已实现，需桌面验收 |
| Android Host | `story-lock/doc/design/03`、平台宿主设计 | `src/host/android-host` | `npm run test:android-readiness` | 原型已实现，需真机验收 |
| Linux Host | 平台宿主设计 | `src/host/linux-host` | `npm run test:linux-host` | 原型已实现，需桌面验收 |

2026-06-22 复核：Linux Host 自动检查已覆盖 `/permission-summary`，并完成 24 题题库导入、12 格 verify、authorize、execute、revoke 闭环；仍需 Linux 桌面或 WSL 环境的人工实测记录。

## 2. 用例夹具映射

| 用例主题 | 上游来源 | 当前 fixture | 状态 |
| --- | --- | --- | --- |
| 24 节点作者稿 | `story-lock/doc/usecase/01` | `scripts/test/fixtures/storylock-package/valid/author-draft.json` | 已有最小样例 |
| 网站账号资源 | `story-lock/doc/usecase/02` | `valid/resource-catalog.json`、`valid/templates/login-sites.json` | 已有最小样例 |
| 钱包签名资源 | `story-lock/doc/usecase/03` | `valid/resource-catalog.json`、`valid/templates/signing-actions.json` | 已有最小样例 |
| API 凭据 | `story-lock/doc/usecase/04` | 暂无独立 fixture | 后续扩展 |
| 证书与开发者身份 | `story-lock/doc/usecase/05` | 暂无独立 fixture | 后续扩展 |
| 模板文件样例 | `story-lock/doc/usecase/07` | `valid/templates/*.json` | 已有最小样例 |
| 完整导出包 | `story-lock/doc/usecase/08` | `valid/package-manifest.json` 等最小目录 | 缺少正式 `vault.stlk` |

## 3. 平台验收映射

| 平台 | 自动化检查 | 人工验收记录 | 当前状态 |
| --- | --- | --- | --- |
| Windows | `npm run test:windows-package`、`npm run test:windows-host-features` | `docs/test/Windows托盘人工验收记录_20260620.md` | 需要桌面实测补记录 |
| Android | `npm run test:android-readiness` | `docs/test/Android真机验收记录_20260622.md` | 记录模板已补；需要真机实测 |
| Linux | `npm run test:linux-host`、`npm run test:linux-package`、`npm run test:linux-desktop` | `docs/test/Linux桌面WSL验收记录_20260622.md` | 自动闭环已复核，记录模板已补；需要 Linux 桌面或 WSL 实测 |

## 4. 安全边界检查

| 边界 | 检查点 | 测试或文档入口 | 状态 |
| --- | --- | --- | --- |
| 远程网关不开放故事读写 | 只保留 `requestSignature`、`requestPasswordFill` | `scripts/test/docs-consistency.mjs` | 已检查 |
| Host 不读取私密字段 | 权限摘要不含答案、密码、私钥、签名字节 | `npm run test:storylock-package` | 已检查 |
| Story edit 本地化 | `story_edit` 只能走本地高强度通道 | `npm run selftest`、Windows cargo tests | 已检查 |
| 管理文档归档 | 当前文档与历史文档分离 | `docs/management/README.md` | 已说明 |

## 5. 发布前建议命令

```powershell
npm run test
cargo test --manifest-path src\host\windows-host\Cargo.toml
cargo test --manifest-path src\host\windows-host\Cargo.toml --features ui-slint
```

自动化通过后，再进行 Windows、Android、Linux 的人工或真机验收。
