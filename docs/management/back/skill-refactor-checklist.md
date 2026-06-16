# StoryLock Skill 重构检查清单

## 项目目标

将当前 `story-lock` 仓库中偏 SDK / demo 形态的实现，重构为更贴近 Pharos Skill Engine 规范的 Skill 包。

当前目标不是立即重写底层 Rust 核心，而是按下面顺序推进：

1. 先把 Skill 包结构补完整
2. 再把上层 JS skill-layer 做成可独立运行、可独立测试
3. 最后再把 Rust / WASM 核心纳入 Skill 包的标准入口

参考依据：

1. `skill/docs/ref/storylock_skill_analysis_2026-06-15.md`
2. `https://docs.pharos.xyz/tooling-and-infrastructure/pharos-skill-engine-guide`
3. `https://docs.pharos.xyz/tooling-and-infrastructure/pharos-skill-engine-guide/piggybank-skill-guide`

## 当前状态判断

### 已完成

- [x] 已创建 `skill/src/storylock-skill-engine/`
- [x] 已具备 `SKILL.md + references + assets` 的基础目录结构
- [x] 已迁移 `story-assist.js`
- [x] 已迁移 `authorization-skills.js`
- [x] 已迁移 `recommended.js`
- [x] 已迁移 `strength-review.js`
- [x] 已补充比赛视角的中文说明文档
- [x] 已补迁移包自己的 `package.json`
- [x] 已补迁移包自己的 `index.js`
- [x] 已补迁移包自己的 `demo` / `selftest` 脚本
- [x] 已将 `errors.js`、`resource-catalog.js`、`constants.js`、`templates.js` 内聚进迁移包
- [x] 已补 Rust / WASM 构建脚本与 `dist/wasm` 产物校验入口
- [x] 已补 guide 侧 `SKILL.md`、`boundary.md`、`demo.md`、`invocation.md`

### 已知缺口

- [ ] `skill/src/storylock-skill-engine/` 仍未形成完全独立的 Rust 源码包
- [ ] JS skill-layer 与本地 WASM host 的正式接线路径仍未完成
- [ ] `assets/` 还没有形成完整的 schema / template 资源层
- [ ] 更完整的自动化测试仍待补齐，当前以 demo / selftest 为主
- [ ] 比赛提交文档还需要继续和最新代码状态保持同步

## 重构原则

1. 不破坏原始 `story-lock` 仓库的现有功能
2. 优先保证迁移包“可读、可演示、可测试”
3. 在没有把 Rust 核心完整纳入前，不夸大为“完全自包含”
4. 每完成一层，就补对应测试和文档

## 阶段一：Skill 包入口标准化

### 1.1 `SKILL.md`

- [x] frontmatter 只保留 `name` 和 `description`
- [x] description 明确覆盖：
  - [x] 这个 skill 做什么
  - [x] 什么时间触发
  - [x] 依赖什么前提
  - [x] 安全边界在哪里
- [x] 增补明确的 Capability Index
- [x] 增补 Asset / Reference 导航说明
- [x] 写明当前是否已经完成 Rust 封装

### 1.2 `references/`

- [x] 将能力说明按场景拆分，不再只依赖笼统入口页
- [x] 已形成以下参考文档：
  - [x] `story-assist.md`
  - [x] `password-fill.md`
  - [x] `challenge-sign.md`
  - [x] `strength-review.md`
- [x] 每个核心能力文档包含：
  - [x] Overview
  - [x] Invocation Template
  - [x] Parameters
  - [x] Output
  - [x] Error Handling
  - [x] Agent Guidelines

### 1.3 验收

- [x] 只读 `SKILL.md` 就能理解 skill 的范围与入口
- [x] AI 能从 Capability Index 定位到具体 reference
- [x] 文档不再混淆“Skill 层”和“Rust Core 层”

## 阶段二：上层代码独立化

### 2.1 模块结构

- [x] 为迁移包补自己的 `package.json`
- [x] 为迁移包补自己的 `index.js`
- [x] 为迁移包补自己的 `scripts/` 入口
- [x] 把最小运行依赖从 `story-lock/src/...` 中内聚进来

### 2.2 当前优先内聚的依赖

- [x] `errors.js`
- [x] `resource-catalog.js`
- [x] `constants.js`
- [x] `templates.js`

### 2.3 仍可暂缓的依赖

- [ ] `StoryLockService`
- [ ] Rust/WASM runtime 装配层
- [ ] browser host / cloud host 相关路径

### 2.4 验收

- [x] `StoryDraftAssistSkill` 可在迁移包内独立导入
- [x] `LocalPasswordFillSkill` 可在迁移包内独立导入
- [x] `ChallengeSigningAuthorizationSkill` 可在迁移包内独立导入
- [x] 不依赖 `story-lock/examples/04-skill-layer-demo.mjs` 也能运行本地 demo

## 阶段三：独立测试闭环

### 3.1 最小测试能力

- [x] 补本地 `demo` 脚本
- [x] 补最小 `selftest` 脚本
- [x] 验证本地 skill 包能直接运行

### 3.2 建议测试项

- [ ] draft assist
- [ ] password fill
- [ ] challenge signing
- [ ] 输入校验失败路径

### 3.3 验收

- [x] `npm run demo` 可运行
- [x] `npm run selftest` 可运行
- [x] demo 输出不再依赖原始 `story-lock/examples/`

## 阶段四：Rust / WASM 封装

这是当前真正还没完成的关键阶段。

### 4.1 需要补上的内容

- [x] 明确 Rust crate 来源
- [x] 明确 WASM build 命令
- [ ] 明确 JS skill-layer 如何调用 Rust / WASM bindings
- [ ] 明确哪些能力仍是 JS 参考实现，哪些已切到 Rust 核心

### 4.2 目录层面的目标

- [x] Skill 包中有明确的 Rust / WASM 资产入口
- [x] Skill 包中有构建说明
- [x] Skill 包中有最小运行说明

### 4.3 验收

- [x] 能明确回答“是否完成 Rust 代码封装”
- [x] 能明确回答“Skill 包本身是否可测试”

## 阶段五：比赛材料同步

- [ ] 更新 `skill/docs/ref/01-参赛概览.md`
- [ ] 更新 `skill/docs/ref/02-技术映射说明.md`
- [ ] 更新 `skill/docs/ref/03-演示与调用说明.md`
- [ ] 更新 `skill/docs/ref/04-提交材料清单.md`
- [ ] 保持与当前代码真实状态一致

## 当前建议优先级

### P0

- [x] 完成 Skill 包自己的 `package.json`
- [x] 完成 Skill 包自己的 `index.js`
- [x] 完成本地 demo 与 selftest
- [x] 清理最关键的跨仓库依赖

### P1

- [x] 标准化 references
- [ ] 补 schema / template 资源
- [x] 规范化错误处理说明
- [x] 补 Rust / WASM 构建与产物校验入口

### P2

- [ ] JS skill-layer 与 Rust / WASM bindings 的正式调用集成
- [ ] 更完整的 agent 配置
- [ ] 更完整的 examples

## 风险记录

- [ ] 风险：把“迁移版”误说成“完整自包含版”
- [ ] 风险：在未完成 Rust 封装前，过早声称已完成核心包装
- [ ] 风险：文档和代码状态不一致
- [ ] 风险：继续依赖原仓库路径，导致 demo 看起来能跑但 skill 包自身不可测

## 本轮完成记录

- [x] 更新 checklist 为按阶段推进的结构
- [x] 补齐最小独立代码入口
- [x] 补齐最小 demo / selftest
- [x] 记录哪些部分仍未完成 Rust 封装
- [x] 补齐 Rust / WASM 构建与产物校验入口
- [x] 补齐能力拆分后的 guide references
