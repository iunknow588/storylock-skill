# StoryLock 项目优化建议报告（修正版）

**日期**: 2026-06-19  
**分析范围**: `E:\2026OPC大赛\skill\docs` → `E:\2026OPC大赛\skill\src`  
**分析者**: 需求分析师  
**版本**: v1.1（修正版）

---

## 一、执行摘要

基于对 StoryLock 项目文档（docs）和源码（src）的深度分析，当前项目已具备**可运行的三层 Skill 架构基线**，但在**目录结构一致性、文档-代码同步、安全实现深度、测试覆盖、多平台宿主成熟度**等方面存在明显优化空间。

**核心结论**: 项目处于"最小可演示闭环"阶段，距离"可审计、可部署、可维护"的生产基线仍有差距。建议优先解决**文档与代码的命名不一致**、**目录结构冗余**、**安全实现中的占位符清理**、**测试覆盖不足**四个问题。

---

## 二、技术可行性评估

### 2.1 当前架构健康度

| 维度 | 评分 | 说明 |
|------|------|------|
| 三层边界清晰度 | ★★★★☆ | 职责划分明确，但存在历史命名残留 |
| 代码可运行性 | ★★★★☆ | 四个 selftest 入口均可运行 |
| 安全实现深度 | ★★★☆☆ | 框架到位，但存在 placeholder 和 mock |
| 文档-代码一致性 | ★★☆☆☆ | 命名不一致、路径未同步、术语混用 |
| 目录结构合理性 | ★★★☆☆ | 存在冗余前缀、重复目录、构建产物污染 |
| 测试覆盖度 | ★★★☆☆ | 有自测但缺乏单元测试和集成测试矩阵 |

### 2.2 关键技术债务

#### 债务1: 命名不一致（高优先级）

**问题**: 文档与代码中的 Skill 包名不一致。

| 文档中的名称 | 实际源码路径 | 状态 |
|-------------|-------------|------|
| `storylock-local-story-processing-skill` | `src/skills/local-story-processing/` | ❌ 已简化 |
| `storylock-local-story-access-skill` | `src/skills/local-story-access/` | ❌ 已简化 |
| `storylock-remote-gateway-skill` | `src/skills/remote-gateway/` | ❌ 已简化 |
| `storylock-skill-engine` | `src/engine/` | ❌ 已重命名 |

**风险**: 新开发者按文档找代码会迷路，文档中的路径引用大量失效。

**建议**: 
1. 统一使用简化后的目录名（`local-story-processing`、`local-story-access`、`remote-gateway`、`engine`）
2. 在 README 中显式声明"文档中的旧名称已重命名，请以 src 目录为准"
3. 全局搜索替换文档中的旧路径引用

#### 债务2: 安全实现中的 Placeholder（高优先级）

**问题**: EIP-712 domain 使用 placeholder 值，但代码中已增加 production 校验。

```javascript
// 当前代码中的校验逻辑（remote-gateway/index.js）
if (environment === 'production') {
  if (/placeholder/i.test(version)) {
    throw new Error('production EIP-712 domain must not use a placeholder version');
  }
  // ...
}
```

**矛盾点**: 文档明确说当前是 placeholder，但代码已拒绝 production 使用 placeholder。需要明确：
- 当前 demo 环境用什么 domain？
- 何时切换到 production domain？
- 切换流程是否文档化？

**建议**: 
1. 在 `docs/design/cn/EIP-712最小请求定义.md` 中补充"domain 切换检查清单"
2. 在代码中增加 `demo` / `test` / `production` 环境的显式配置入口
3. 为 production 环境提供 domain 注入模板（环境变量或配置文件）

#### 债务3: SecretStore 生产约束未完全落地（中优先级）

**问题**: 文档要求生产环境使用平台 SecretStore，但当前代码中 `MemorySecretStore` 仍是默认选项，且 `developmentMode` 的启用条件不够严格。

**当前代码逻辑**:
```javascript
// 需要显式传入 developmentMode=true 才允许使用 MemorySecretStore
// 但错误提示不够明确，可能导致生产环境误用
```

**建议**:
1. 默认拒绝 `MemorySecretStore`，必须显式传入 `developmentMode=true` 才允许
2. 启动时输出显式警告："当前使用 MemorySecretStore，仅适用于开发环境"
3. 在 `docs/design/cn/平台密钥存储适配指南.md` 中补充各平台的接入优先级和降级策略

#### 债务4: 九宫格题目来源仍为占位数据（中优先级）

**问题**: `docs/design/cn/开发落地路线与当前进展.md` 明确指出"真实九宫格题目选择作为后续增强"，当前使用 `demo-story-config.json` 和 `question-set-master.sample.json`。

**风险**: 演示时可能暴露"题目是硬编码的"这一事实，影响评审印象。

**建议**:
1. 提供最小可运行的真实题集导入流程（CLI 或脚本）
2. 在 `docs/ref` 中补充"如何生成 24 题题集"的操作指南
3. 将 `question-set-master.sample.json` 升级为可替换的模板，而非直接使用的数据源

---

## 三、市场与产品分析

### 3.1 当前定位清晰度

**优势**:
- 三层边界清晰，符合"本地优先授权"的安全叙事
- 术语体系完整（术语表、快速决策指南、能力边界表）
- 多平台宿主策略明确（Android + Windows + 未来 Linux）

**风险**:
- 功能描述存在"过度承诺"风险。例如 `docs/design/cn/易安与StoryLock技术说明书.md` 中提到的"多账号与多对象协作"当前尚未实现
- 对外介绍材料需要严格区分"已实现"和"后续探索"

### 3.2 竞品对标建议（已修正）

> **重要修正**：以下"本地宿主的安全边界"为**软件级隔离实现**，仅在架构原则上**类比**硬件安全模块（HSM）的"密钥不出本地边界"理念，**并非硬件安全模块本身**。

| 竞品/参考 | StoryLock 当前差距 | 优化建议 |
|-----------|-------------------|----------|
| Turnkey Agentic Wallet | 缺乏真实的密钥管理硬件集成 | 文档中明确说明"本地宿主的安全边界**类比**硬件安全模块（HSM），但当前实现为软件级隔离，非硬件方案" |
| Keyless Collective SDK | 缺乏云端协调层的实际部署 | 强调 StoryLock 的"本地优先"差异化定位 |
| Metaplex Agent Registry | 缺乏链上身份注册能力 | 将 Pharos 定位为可选锚定层，不作为核心依赖 |

**关键区分**：
- **类比关系**：StoryLock 的本地宿主在架构上**参考**了 HSM 的"密钥不出本地边界"原则
- **实际实现**：当前为**软件级**隔离（SQLite + SecretStore + 本地进程），无硬件安全元件（SE）、可信执行环境（TEE）或物理防篡改机制
- **未来扩展**：可在文档中注明"后续可接入硬件安全模块作为可选增强"，但不得作为当前能力

---

## 四、财务与资源评估

### 4.1 当前开发投入效率

**观察**: 文档投入明显大于代码实现深度。

| 类别 | 数量/规模 | 评估 |
|------|----------|------|
| 设计文档（cn+en） | 约 25 篇 | 过量，部分文档内容重复 |
| 管理文档（back） | 约 30 篇历史版本 | 归档价值高，但不应影响主线阅读 |
| 自测脚本 | 4 个入口 | 覆盖主线，但缺乏负面测试 |
| 真实 Android 宿主 | 骨架代码 | 尚未完成真机编译验证 |
| Windows 宿主 | Rust 原型 | 已实现注册/relay/execute 闭环，但签名仍是 HMAC 演示 |

### 4.2 资源优化建议

1. **文档瘦身**: 将 `docs/management/back/` 中的历史版本合并为"决策日志"，减少重复阅读成本
2. **测试补强**: 为每个 Skill 增加负面测试用例（无效输入、边界条件、并发冲突）
3. **宿主优先级**: Windows 宿主已比 Android 宿主走得更远，建议优先完成 Windows 真机闭环，再反哺 Android

---

## 五、风险评估

### 5.1 高风险项

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 文档-代码命名不一致导致评审误解 | 评审时按文档找不到对应代码 | 立即统一命名，在 README 中声明映射关系 |
| EIP-712 placeholder 被误认为生产就绪 | 安全评审质疑 | 增加显式环境检查，文档中标注"非生产状态" |
| MemorySecretStore 生产误用 | 密钥泄露 | 默认拒绝，强制显式启用开发模式 |
| 九宫格题目为占位数据 | 演示时暴露实现深度不足 | 提供最小真实题集导入流程 |

### 5.2 中风险项

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 多平台宿主并行开发资源分散 | 均无法达到演示标准 | 集中资源先完成 Windows 宿主 |
| 脱敏规范未在代码中强制校验 | 第三方集成时可能泄露敏感字段 | 在第三层增加脱敏校验中间件 |
| 缺乏定时清理机制 | SQLite 长期运行后膨胀 | 补充 cron 或定时任务调用 |

---

## 六、行动建议（优先级排序）

### 🔴 P0: 立即执行（阻塞评审）

1. **统一文档-代码命名**
   - 将所有文档中的 `storylock-*-skill` 替换为实际目录名
   - 在 `README.md` 顶部增加"命名对照表"

2. **清理 EIP-712 placeholder 歧义**
   - 在文档中明确标注当前 domain 状态
   - 在代码中增加 `demo` 环境的默认 domain 配置

3. **强化 SecretStore 生产约束**
   - 默认拒绝 `MemorySecretStore`
   - 启动时输出不可忽略的安全警告

### 🟡 P1: 短期完成（提升评审质量）

4. **补充真实题集导入流程**
   - 将 `question-set-master.sample.json` 改为模板
   - 提供 CLI 导入命令和验证脚本

5. **增加负面测试覆盖**
   - 为每个 Skill 补充 5-10 个负面测试用例
   - 重点覆盖：无效输入、重放攻击、过期请求、预算耗尽

6. **完成 Windows 宿主真机闭环**
   - 将当前 HMAC 演示签名替换为真实 DPAPI 密钥操作
   - 提供 Windows 宿主安装和运行指南

### 🟢 P2: 中期优化（提升可维护性）

7. **文档结构瘦身**
   - 将 `docs/management/back/` 合并为"决策日志.md"
   - 删除或归档已失效的历史评审意见

8. **增加脱敏校验中间件**
   - 在第三层增加自动脱敏检查
   - 对未脱敏的响应抛出 `SLG-008` (redaction_required)

9. **补充定时清理机制**
   - 在第二层增加 `setInterval` 或 cron 调用
   - 在文档中说明清理频率和策略

---

## 七、具体优化点清单

### 7.1 文档优化

| 文档 | 问题 | 建议 |
|------|------|------|
| `docs/readme.md` | 路径引用未更新 | 同步为 `src/skills/*` 路径 |
| `docs/design/cn/README.md` | 与 `docs/readme.md` 内容重复 | 合并或明确分工 |
| `docs/design/cn/易安与StoryLock技术说明书.md` | 功能描述存在过度承诺 | 增加"当前已实现 / 计划中"标注 |
| `docs/design/cn/EIP-712最小请求定义.md` | 未说明 placeholder 切换流程 | 补充切换检查清单 |
| `docs/design/cn/安全规范.md` | 未明确开发模式启用条件 | 补充强制确认和警告要求 |
| `docs/management/back/` | 历史版本过多 | 合并为决策日志 |

### 7.2 代码优化

| 文件 | 问题 | 建议 |
|------|------|------|
| `src/skills/remote-gateway/index.js` | EIP-712 校验逻辑分散 | 提取为独立校验模块 |
| `src/skills/local-story-access/index.js` | 缺乏输入长度限制 | 增加 `MAX_ANSWERS`、`MAX_ANSWER_LENGTH` 的显式校验 |
| `src/shared/secret-store.js` | 生产约束不够严格 | 默认拒绝 MemorySecretStore |
| `src/skills/remote-gateway/web-api-handler.js` | 脱敏逻辑未强制化 | 增加响应脱敏校验中间件 |
| `src/skills/local-story-access/access-host.js` | 缺乏定时清理 | 增加 `setInterval` 或导出定时清理函数 |

### 7.3 目录结构优化

| 问题 | 建议 |
|------|------|
| `src/yian-web/public/` 在源码目录下 | 提升为 `web/` 独立目录 |
| `.temp/` 存在已跟踪文件 | 清理并确认 `.gitignore` 生效 |
| `release/app/` 与 `src/host/` 可能存在重复 | 确认 `release/` 只放构建产物 |
| `scripts/` 子目录过多 | 按功能聚类（build/test/dev/utils） |

---

## 八、结论

StoryLock 项目具备**清晰的安全架构叙事**和**可运行的三层演示闭环**，这是其核心优势。但当前阶段的最大风险在于**文档与代码的脱节**——文档描述的是一个"接近完成"的系统，而代码中仍有大量 placeholder、mock 和演示实现。

**建议策略**: 
1. **诚实定位**: 在对外材料中明确区分"已实现"和"计划中"，避免过度承诺
2. **命名统一**: 立即解决文档-代码命名不一致问题，这是最低成本的信任修复
3. **测试补强**: 负面测试和边界测试是当前最薄弱的环节，直接影响评审印象
4. **宿主聚焦**: 优先完成 Windows 宿主的真实签名闭环，而非并行推进多平台

项目当前处于"可演示"阶段，通过上述优化可在短期内达到"可评审"状态。距离"可部署"仍需完成：真实题集管理、生产级 SecretStore 接入、多平台宿主真机验证。

---

## 附录：已修正内容说明

### 修正1：硬件安全模块类比表述（v1.1）

**原文（v1.0）**：
> "缺乏真实的密钥管理硬件集成 → 文档中明确说明'本地宿主可替换为硬件安全模块'"

**修正后（v1.1）**：
> "缺乏真实的密钥管理硬件集成 → 文档中明确说明'本地宿主的安全边界**类比**硬件安全模块（HSM），但当前实现为软件级隔离，非硬件方案'"

**修正原因**：
- 避免误导评审认为当前已实现硬件级安全
- 明确区分"类比"与"等同"
- 诚实披露当前为软件级隔离实现

### 关键区分（新增）

- **类比关系**：StoryLock 的本地宿主在架构上**参考**了 HSM 的"密钥不出本地边界"原则
- **实际实现**：当前为**软件级**隔离（SQLite + SecretStore + 本地进程），无硬件安全元件（SE）、可信执行环境（TEE）或物理防篡改机制
- **未来扩展**：可在文档中注明"后续可接入硬件安全模块作为可选增强"，但不得作为当前能力

---

*报告完成。如需针对某一具体优化点展开详细方案，可进一步分析。*

---

## 九、后续需完善工作清单（用于后续统一修正）

下面按照优先级与执行要点列出需要后续在仓库中统一修正的项，便于一次性批量替换与回归验证。

### P0（必须立即完成 — 阻塞评审）
- **统一目录与包引用**：把所有源代码/脚本中的旧路径 `src/storylock-*-skill` 替换为新的目录结构（`src/skills/local-story-processing`、`src/skills/local-story-access`、`src/skills/remote-gateway`、`src/engine`）。重点文件示例：
   - `scripts/git/commit.ps1`、`scripts/vercel/dev_local.ps1`
   - `web-api/storylock-gateway.mjs`、`src/ui/server.mjs`
   - 各 `scripts/test/*.mjs` 中的 schema/路径引用
- **更新 package.json 的 `name` 字段（可选策略见下）**：决定是否将各包的 `package.json` 中 `name` 字段同步为简化后名称（例如 `remote-gateway` 或 `@storylock/remote-gateway`），并在所有引用处同步校验（manifest、assets、测试脚本）。示例文件：
   - `src/skills/local-story-access/package.json`
   - `src/skills/local-story-processing/package.json`
   - `src/skills/remote-gateway/package.json`
   - `src/engine/package.json`
- **修正运行/自测脚本的工作目录与调用**：确保 `scripts/test/run-selftests.mjs`、`scripts/test/*` 能按新目录正常调用 `npm run selftest`（或用 `--prefix`）

### P1（短期完成 — 提升评审质量）
- **文档批量替换**：把 `docs/` 下所有旧路径替换为 `src/skills/...` 或 `src/engine`，并在 README 顶部添加“命名对照表 / 迁移说明”。重点目录：
   - `docs/management/`、`docs/design/`、`docs/ref/`、`docs/readme.md` 等
- **更新示例/引用中的资源路径**（包括 `SKILL.md`、references 文档、assets JSON）：例如 `src/skills/remote-gateway/assets/...` → `src/skills/remote-gateway/assets/...`
- **资产与 Manifest 校验**：将 `src/skills/remote-gateway/assets/agent-capabilities.json` 中的 `package` 字段与 `index.js` 暴露的方法保持一致，更新 `scripts/test/check-agent-capabilities.mjs` 中的断言（如果 package 名称发生变化）

### P2（中期完成 — 可提升可维护性）
- **编码/非 UTF-8 文档处理**：部分 `docs/management/` 文件名或内容包含非标准编码/Windows 特殊字符，使用编码感知工具（如 iconv、Notepad++ 或 PowerShell 的 `Get-Content -Encoding`）逐一修正并保存为 UTF-8。示例：`docs/management/` 下出现的乱码文件。
- **保留旧名的兼容映射**：若不修改 `package.json` 的 `name`，需添加映射说明（README）并在自测脚本中增加兼容判断逻辑，避免改名后失效的 CI/脚本。

### P3（可选/优化项）
- **文档瘦身与归档**：把 `docs/management/back/` 中的历史评审合并为 `docs/management/DECISIONS.md`（决策日志），减少主线阅读噪音
- **增强自动化校验**：在仓库中加入脚本：
   - `scripts/verify/path-consistency.mjs`：跑一遍全 repo 的路径替换断言
   - `scripts/verify/encoding-check.ps1`：检查并报告非 UTF-8 文档

### 验证与回归（建议在每次批量替换后执行）
- 在 `skill` 根目录运行（PowerShell）：

```powershell
Push-Location e:\2026OPC大赛\skill
# 1) 运行自测脚本
node scripts/test/run-selftests.mjs
# 2) 运行文档/路径一致性检查（若已添加）
node scripts/verify/path-consistency.mjs || echo 'path check failed'
# 3) 全仓 grep 检查残留旧路径
Get-ChildItem -Recurse -File -Include *.js,*.mjs,*.md,*.ps1 | Select-String -Pattern 'storylock-local-story-access-skill|storylock-local-story-processing-skill|storylock-remote-gateway-skill|storylock-skill-engine' -SimpleMatch
Pop-Location
```

### 责任分配建议与修改提议流程
- 先在分支上（如 `docs/rename-sweep`）完成批量替换并在 CI 上跑自测；确认无误后合并到主分支。
- 对于 `package.json` 的 `name` 字段改动，优先在单独 PR 中提出并由维护者确认命名策略（保持 package identity 与语义一致）。

---

请确认是否需要我：
1. 直接在仓库中把 `package.json` 的 `name` 字段批量替换为简化名称（我会先在新分支执行并运行自测）；或
2. 仅完成文档路径的替换、编码修正与脚本更新，保留 `package.json` 的原有 `name` 字段以保证包标识兼容性。

