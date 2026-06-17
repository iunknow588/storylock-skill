# StoryLock 产品说明文档

## 1. 项目基本信息

| 项目 | 内容 |
|------|------|
| 产品名称 | StoryLock |
| 适用场景 | Agent 场景下的本地私钥管理与权限授权 |
| 项目定位 | 面向 Agent 场景的本地私钥管理与授权控制 Skill 方案 |
| 目标用户 | 个体开发者、小型团队、Agent / Skill 开发者 |
| 仓库主目录 | `skill/` |
| 文档版本 | 2026-06-17 |

---

## 2. 项目简介

StoryLock 是一个面向 Agent 场景的本地私钥与敏感对象授权管理工具。本地 Agent 负责管理私钥和文件密钥，远程在线运行的 Agent 不直接持有用户私钥。文件内容以密钥加密存储，只有知晓文件密钥并具备对应权限等级的本地 Agent 才能读取。StoryLock 在授权决策中引入联想记忆线索，把“访问本地敏感对象需要对应权限等级”和“由人类可理解、可回忆的线索参与授权判断”沉淀为一套可复用的 Skill 能力。

围绕这个目标，项目拆分为三层能力：

1. 第一层：本地故事处理能力  
   负责故事草稿生成、故事润色、题集强度评估等本地处理逻辑。
2. 第二层：本地受控访问能力  
   负责根据目标对象判断所需的密码强度，并生成对应的九宫格验证。对象需要使用低强度、中强度或高强度验证，都属于第二层定义和执行的对象级访问策略。
3. 第三层：远程网关与代理授权能力  
   负责把外部调用包装成统一请求结构，并以代理授权技能的方式把签名授权、密码填充等本地能力受控暴露给远程 Agent。它连接远程在线 Agent 与本地 Agent：远程侧发起结构化请求，本地侧完成权限校验、本地授权、签名或密码填充，再返回最小化结果。通过这一层，远程 Agent 可以使用本地能力，但不会直接持有用户私钥或文件密钥。

一句话概括：

> StoryLock 是一个由本地 Agent 管理私钥与文件密钥、通过联想记忆线索参与授权决策，并在三层 Skill 协同下让远程 Agent 安全使用本地能力的 Agent 安全能力包。

---

## 3. 代码结构

### 3.1 三包结构

| 包名 | 目录 | 作用 |
|------|------|------|
| `storylock-local-story-processing-skill` | `skill/src/storylock-local-story-processing-skill` | 第一层，本地故事处理 |
| `storylock-local-story-access-skill` | `skill/src/storylock-local-story-access-skill` | 第二层，本地受控访问 |
| `storylock-remote-gateway-skill` | `skill/src/storylock-remote-gateway-skill` | 第三层，远程请求包装与统一网关 |

### 3.2 汇总入口

| 模块 | 目录 | 说明 |
|------|------|------|
| `storylock-skill-engine` | `skill/src/storylock-skill-engine` | 汇总导出、示例脚本、自测脚本、WASM 构建脚本 |
| `shared` | `skill/src/shared` | 共享加密、SQLite、SecretStore 适配代码 |

---

## 4. 核心能力

### 4.1 第一层：本地故事处理

技能包括：

- `StoryDraftAssistSkill`
- `StoryRefineAssistSkill`
- `StrengthReviewSkill`

对应代码：

- `skill/src/storylock-local-story-processing-skill/index.js`
- `skill/src/storylock-skill-engine/assets/migrated/skills/story-assist.js`
- `skill/src/storylock-skill-engine/assets/migrated/skills/strength-review.js`

能力：

- 故事草稿生成
- 故事内容润色
- 题集或问题集合强度评估

### 4.2 第二层：本地受控访问

技能包括：

- `ObjectStrengthPolicySkill`
- `GridChallengeSkill`
- `LocalAuthorizationSkill`

对应代码：

- `skill/src/storylock-local-story-access-skill/index.js`
- `skill/src/storylock-local-story-access-skill/access-host.js`

能力：

- 统一请求校验
- 对象级强度策略，可根据目标对象决定所需的密码强度
- 根据密码强度生成对应的九宫格验证
- `requestId` 幂等与 `nonce` 防重放
- 本地验证创建与答案校验
- 短时会话签发
- 授权结果返回

### 4.3 第三层：远程网关

接口包括：

- `requestSignature`
- `requestPasswordFill`

对应代码：

- `skill/src/storylock-remote-gateway-skill/index.js`

能力：

- 统一请求信封结构
- 统一能力名与 scope
- 远程调用包装
- 签名授权请求包装
- Web2 网站登录表单的密码填充请求包装
- EIP-712 最小签名请求结构包装

第三层是远程 Agent 使用本地能力的统一代理授权入口。它负责把远程请求转换为结构化、可校验、可审计的本地调用，并将本地执行结果以最小化形式返回。

- 远程 Agent 通过第三层发起授权请求，而不直接接触私钥
- 本地 Agent 通过第三层执行权限校验、本地授权、签名或密码填充
- 第三层将“能否请求”与“如何在本地执行”分离开来
- 返回结果遵循最小化原则，避免敏感内容向远程侧泄露

---

## 5. 安全机制

### 5.1 安全能力

| 能力 | 说明 |
|------|------|
| 对象级密码强度与九宫格校验 | 根据目标对象确定所需密码强度，并生成对应的九宫格验证 |
| 防重放 | `requestId` 幂等、`nonce` 去重、`expiry` 校验 |
| 会话模型 | 基于短时会话控制访问窗口 |
| 自动锁定 | 连续失败后进入 `locked` |
| 自动解锁 | 锁定窗口到期后恢复可用 |
| 对象加密 | AES-256-GCM |
| 密钥派生 | HKDF-SHA256 |
| 答案摘要 | HMAC-SHA256 摘要存储 |

补充说明：

- 私钥与文件密钥由本地 Agent 管理
- 远程 Agent 只请求调用能力，不直接持有私钥
- 授权判断引入人类联想记忆线索，而不是单纯依赖机械口令

### 5.2 能力范围

主线能力包括：

1. 统一的三层 Skill 结构
2. 面向对象的密码强度策略、九宫格验证与短时会话
3. 防重放、本地答案校验与授权结果返回
4. 远程请求包装到本地执行的受控链路
5. 签名授权与 Web2 密码填充请求的本地受控执行能力

扩展能力包括：

- 题集模型与渐进式九宫格验证策略
- 更细粒度的对象访问分级与策略引擎
- 跨平台 SecretStore 适配
- 本地签名执行、审计与恢复机制
- 标准化的 HTTP / 宿主集成层

本文档内容以仓库中的实际代码、接口与脚本为准。

---

## 6. 设计与实现对应关系

结合现有代码与 `skill/docs/management/code_doc_consistency_review.md`，这一节用于说明当前设计在代码中的落点，以及后续扩展的主要方向。

### 6.1 当前实现映射

- 三包目录结构与职责分层
- 第二层的对象强度判断与九宫格验证生成链路
- `locked` 状态自动恢复
- 参考 EIP-712 标准，定义项目内使用的 `StoryLockSignatureRequest` 请求结构
- 远程网关聚焦签名授权与密码填充请求包装
- `REQUEST_EXPIRED -> SLG-011` 错误码映射修正
- 本地授权状态更新改为更稳妥的原子更新方式

### 6.2 后续扩展方向

- 继续围绕本地授权、权限控制与代理执行能力完善底层机制
- 面向不同应用领域探索典型使用场景，验证方案的通用性与可迁移性
- 结合具体业务需求逐步沉淀可复用的集成模式与工程实践

---

## 7. 运行与验证

### 7.1 Skill Engine 演示

目录：

`skill/src/storylock-skill-engine`

命令：

```powershell
cd skill/src/storylock-skill-engine
npm run demo
npm run selftest
npm run build:wasm
npm run selftest:wasm
```

内容：

- `demo`：草稿生成、密码填充、签名授权的示例链路
- `selftest`：验证核心导出能力可调用
- `build:wasm`：验证 Rust/WASM 产物构建流程
- `selftest:wasm`：验证 WASM dist 产物可加载

### 7.2 本地受控访问自测

目录：

`skill/src/storylock-local-story-access-skill`

命令：

```powershell
cd skill/src/storylock-local-story-access-skill
node scripts/selftest.mjs
```

自测覆盖：

- 对象强度判断
- 九宫格验证生成
- 防重放
- 幂等请求
- 锁定与自动解锁
- 请求过期错误码
- 本地授权结果返回

### 7.3 远程网关自测

目录：

`skill/src/storylock-remote-gateway-skill`

命令：

```powershell
cd skill/src/storylock-remote-gateway-skill
node scripts/selftest.mjs
```

自测覆盖：

- 远程请求包装
- `requestSignature`
- `requestPasswordFill`
- EIP-712 结构
- 默认 `policyHints`

---

## 8. 产品特性

### 8.1 特性一：不仅存储秘密，也控制秘密的使用

很多工具主要解决“秘密放在哪儿”的问题。StoryLock 进一步关注秘密的使用边界：

- 谁可以用
- 什么时候可以用
- 能用多久
- 能返回什么
- 远程调用方最多能拿到什么

这使它不只是存储工具，而是一个面向 Agent 场景的本地授权与密钥管理框架。

### 8.1.1 联想记忆参与授权

StoryLock 的授权设计强调由人类联想记忆线索参与决策，不把授权过程简化为固定密码的机械输入。

这种设计更贴近日常使用习惯：

- 人更容易回忆与自身经历、故事和线索相关的信息
- 本地 Agent 负责保管密钥并执行权限控制
- 远程 Agent 负责发起请求，不直接接触私钥

这种设计把本地密钥管理与基于可理解线索的授权判断结合起来，更适合长期使用，也更适合 Agent 参与但不越权的场景。

### 8.2 特性二：三层结构清晰，分工明确，支撑安全应用构建

项目不是单点脚本，而是拆分为：

- 本地处理包
- 本地访问包
- 远程网关包

三层之间分工明确、彼此协同：

- 第一层负责本地处理与记忆线索相关能力
- 第二层负责对象强度判断、九宫格验证与本地授权
- 第三层负责签名授权、密码填充等远程请求包装与代理授权

这种结构把处理、授权、代理执行三类工作明确拆开，使系统在扩展能力时仍能保持清晰边界，并支撑更安全的应用构建。

### 8.3 特性三：仓库具备文档、代码、自测与示例脚本

现有仓库包含：

- `SKILL.md`
- `references/`
- `assets/schemas/`
- `assets/templates/`
- `demo/selftest/build` 脚本

仓库同时提供文档、代码、自测与示例脚本，便于理解、验证与复用。

### 8.4 特性四：基于 Rust 与 Pharos 构建统一安全执行架构

当前架构以 Rust 与 Pharos 作为上下两层的技术支点，仓库已经具备以下基础：

- `storylock-skill-engine` 统一入口
- `dist/wasm` 构建产物
- WASM 构建与产物自测脚本

在这一架构中：

- 底层使用 Rust / WASM 提升关键执行路径的安全性与稳定性
- 高层通过 Pharos 适配不同链、不同签名流程和不同宿主环境
- StoryLock 在本地授权、密钥管理和代理执行之间形成统一的安全执行框架

---

## 9. 商业与应用场景

### 9.1 典型应用场景

| 场景 | StoryLock 的作用 |
|------|------|
| 策略探索型自动化交易 | 可探索远程 Agent 发起策略请求、本地 Agent 分级控制账户密钥、签名权限与操作强度的应用模式 |
| 自动化内容生成与发布 | 可探索远程 Agent 编排内容流程、本地 Agent 控制平台凭据、素材使用与发布权限的应用模式 |
| 多账号运营场景 | 可探索面向多个站点、多个账号和多个发布通道的本地凭据管理与高敏感操作授权模式 |
| 多对象代理执行 | 可探索同一流程涉及多个钱包、云环境、网站账号或凭据对象时的本地授权、签名与访问控制模式 |

---

## 10. 仓库内容

| 类型 | 路径 | 说明 |
|------|------|------|
| 产品说明文档 | `skill/docs/usecase/00-参赛说明文档.md` | 项目说明 |
| 一致性评审文档 | `skill/docs/management/code_doc_consistency_review.md` | 代码与设计文档对照评审 |
| 第一层代码 | `skill/src/storylock-local-story-processing-skill` | 本地故事处理 |
| 第二层代码 | `skill/src/storylock-local-story-access-skill` | 本地受控访问 |
| 第三层代码 | `skill/src/storylock-remote-gateway-skill` | 远程网关 |
| 汇总入口 | `skill/src/storylock-skill-engine` | 示例、自测、WASM 构建 |
| 共享模块 | `skill/src/shared` | 加密、SQLite、SecretStore |

---

## 11. 扩展方向

### 11.1 短期

- 打通网关层、访问层、处理层之间的完整处理流程
- 梳理远程请求、本地授权、本地处理与结果返回之间的链路
- 让 demo、自测与说明文档围绕同一条主流程对齐

### 11.2 中期

- 形成更稳定的能力接口与请求结构
- 完善对象访问分级、强度策略与权限控制的标准化表达
- 推进 SecretStore、签名执行与宿主接入接口的标准化适配

### 11.3 长期

- 面向不同应用领域探索典型应用场景
- 在具体场景中复用本地授权、密钥管理与代理执行能力
- 逐步沉淀可扩展的 Agent 安全应用模式

---

## 12. 总结

StoryLock 可以概括为：

1. 一个三层协同的 Skill 方案；
2. 一个由本地 Agent 管理私钥和文件密钥、通过联想记忆线索参与授权决策，并按权限等级执行授权的 Agent 安全方案；
3. 一个具备代码、文档、示例脚本与自测脚本的可验证项目。

三层 Skill 分别承担以下职责：

- 第一层负责本地处理、故事整理和记忆线索相关能力
- 第二层负责根据目标对象确定所需密码强度，按对象级策略生成对应九宫格验证，并返回本地授权结果
- 第三层负责远程请求包装、代理授权与最小化结果返回

项目能力围绕以下边界展开：

- 本地 Agent 管理私钥与文件密钥
- 远程 Agent 只请求调用能力，但不直接持有私钥
- 授权判断引入联想记忆线索，而不是仅依赖机械口令
- 敏感对象访问遵循权限等级、短时会话与最小化返回原则
