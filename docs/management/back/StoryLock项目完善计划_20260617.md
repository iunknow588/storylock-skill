# StoryLock 项目完善计划

日期：2026-06-17

本文基于 `docs/design/cn`、`docs/design/en` 与 `src` 的一致性检查结果编写，用于指导 StoryLock 从“最小可演示安全闭环”继续完善到“评审口径清晰、实现边界准确、可持续迭代”的参赛版本。

## 一、当前结论

StoryLock 当前已经完成三层主线的最小可运行实现：

1. 第一层：本地故事处理与题集强度评估。
2. 第二层：对象强度策略、九宫格验证、本地授权、SQLite 状态与审计。
3. 第三层：远程请求包装、EIP-712 风格结构化请求、可选本地执行器、结果脱敏。

当前项目定位应表述为：

> StoryLock 已具备可自测、可演示的最小安全授权闭环；当前重点不是继续扩大能力宣称，而是收敛文档口径、补齐真实九宫格题集绑定、完善生产密钥存储和版本化策略。

## 二、主要问题

| 编号 | 问题 | 影响 | 优先级 |
| --- | --- | --- | --- |
| P0-01 | 中英文设计目录数量和历史文档不完全一致 | 评审时可能认为文档未同步 | 高 |
| P0-02 | 第二层源码 `SKILL.md` 仍残留故事读写描述 | 与当前设计边界冲突 | 高 |
| P0-03 | 部分英文文档含历史分析或旧命名副本 | 主线口径不够清晰 | 高 |
| P1-01 | 九宫格仍是占位式 cell 生成 | 授权验证完整性不足 | 中 |
| P1-02 | 题集版本、答案摘要版本、单故事终身制未完全落地 | 生产迁移与长期维护不足 | 中 |
| P1-03 | SecretStore 生产级适配仍偏接口和雏形 | 生产安全边界不足 | 中 |
| P1-04 | EIP-712 仍使用 placeholder domain | 不适合正式链上验证 | 中 |
| P2-01 | EIP-1271、ERC-4337、多链多钱包仍是后续探索 | 不能作为当前已实现能力宣传 | 低 |

## 三、P0：文档与边界立即收口

### 3.1 统一中英文设计目录

目标：

1. 为 `docs/design/cn` 与 `docs/design/en` 建立一一对应清单。
2. 英文目录中多出的历史分析文档移入 `back/` 或标注为 `archive`。
3. 重复命名文档只保留当前主线版本。

建议处理：

| 当前英文文档 | 建议 |
| --- | --- |
| `storylock_skill_pharos_alignment_analysis.md` | 移入 `en/back/` 或标注历史分析 |
| `storylock_skill_positioning_and_boundary.md` | 与 `SKILL_POSITIONING_AND_BOUNDARIES.md` 合并或归档 |
| `storylock_object_access_policy.md` | 与 `OBJECT_ACCESS_POLICY.md` 合并或归档 |
| `storylock_story_skill_feasibility_analysis.md` | 移入 `en/back/`，避免作为当前设计基线 |

验收标准：

1. `en/README.md` 中的“同步中文版本”与实际目录一致。
2. 当前主线文档不存在互相冲突的接口描述。
3. 历史文档均有明确 `archive`、`historical analysis` 或 `非当前基线` 标识。

### 3.2 修正源码 Skill 描述

目标：

1. 修正 `src/storylock-local-story-access-skill/SKILL.md` 的 description。
2. 不再写 `read access`、`write access`、`protected story object access` 作为第二层职责。
3. 改为 `object strength policy, grid verification, local authorization`。

验收标准：

1. 第二层 `SKILL.md` 与 `docs/design/cn/系统Skill表与能力边界.md` 口径一致。
2. 第二层不再被描述成故事读取或写回层。

### 3.3 更新管理文档覆盖度

目标：

1. 将管理文档中“设计文档 15 份”的统计更新为真实数量或改为“主线设计文档 + 历史分析文档”。
2. 区分中文设计、英文翻译、历史分析、管理文档。

验收标准：

1. `StoryLock项目评审意见_20260617.md` 不再出现过期统计。
2. `readme-cn.md` 与 `readme.md` 的文档入口与实际目录一致。

## 四、P1：核心能力完善

### 4.1 接入真实九宫格题集

现状：

当前 `GridChallengeSkill` 已能生成验证、登记 requestId/nonce、防重放并签发授权，但 grid cell 仍主要基于 seed 和序号生成，占位性质较强。

目标：

1. 从题集主档或本地题集存储中选择真实问题。
2. 每个 cell 绑定 `questionId`、`versionTag`、`promptRef`、候选项摘要。
3. 根据 `low`、`medium`、`high` 强度选择不同 requiredCells。
4. 验证时按 cell 逐项校验，而不是只判断任一答案命中。

验收标准：

1. high 强度至少要求 9 格完整验证。
2. medium、low 有明确题目数量和通过规则。
3. 远程侧不能获得答案原文。
4. 自测覆盖正确答案、错误答案、部分答案、重复提交和过期 challenge。

### 4.2 完善题集与答案摘要版本化

目标：

1. 为题集问题增加 `active`、`deprecated`、`pending` 状态。
2. 为答案摘要增加 `normalizationVersion` 和 `questionSetVersion`。
3. 支持迁移窗口内的新旧版本双校验。
4. 校验通过后可异步迁移旧摘要。

验收标准：

1. 旧题不会被新 challenge 优先选中。
2. 历史 session 和旧答案摘要有明确过期策略。
3. 文档中的“单故事终身制”不再只停留在设计描述。

### 4.3 强化 SecretStore 生产适配

目标：

1. 明确 `MemorySecretStore` 只用于开发和测试。
2. Windows Credential Manager、Linux Secret Service 至少完成可用性检查与失败提示。
3. 补充 macOS Keychain 适配计划或明确当前不支持。
4. 为持久化 SQLite 场景强制要求平台 SecretStore 或显式 development mode。

验收标准：

1. 生产模式不能静默使用内存密钥。
2. `check-secret-store` 能输出平台、适配状态、失败原因。
3. README 中写清各平台运行要求。

### 4.4 完善 EIP-712 生产升级路径

现状：

当前 `StoryLockSignatureRequest` 已作为结构化签名请求存在，但 domain 使用 `1-placeholder` 和占位合约地址。

目标：

1. 将 placeholder domain 明确限制为 demo/test。
2. 增加 production domain 配置入口。
3. 记录 `chainId`、`verifyingContract`、`version` 的切换策略。
4. 将 EIP-1271、ERC-4337 保持为后续扩展，不写成当前已支持。

验收标准：

1. 测试环境与生产环境 domain 配置可区分。
2. 生产配置缺失时拒绝标记为 production。
3. 文档与代码都避免“完整链上账户抽象已实现”的误导。

## 五、P2：质量与参赛呈现

### 5.1 增加文档一致性检查脚本

目标：

1. 检查 `requestChallengeSign` 是否被写成当前主接口。
2. 检查第二层是否被写成故事读取/写回层。
3. 检查 `docs/design/cn` 与 `docs/design/en` 是否存在未标注的孤立文档。

验收标准：

1. 根目录 `npm run test` 或独立脚本能执行该检查。
2. 检查失败时输出具体文件和行号。

### 5.2 形成评审演示脚本

目标：

1. 准备 3 分钟演示路径。
2. 准备 8 分钟技术说明路径。
3. 准备常见质疑回答：为什么第二层不读写故事、为什么 EIP-712 不是完整链上账户、为什么 SecretStore 当前仍需生产适配。

验收标准：

1. 演示脚本与 e2e selftest 对齐。
2. 演示中不展示明文密码、私钥、答案原文。
3. 能清楚说明“已实现”和“后续增强”的边界。

## 六、推荐执行顺序

| 顺序 | 任务 | 预计产出 |
| --- | --- | --- |
| 1 | 修正第二层 `SKILL.md` 旧描述 | 代码包文档与设计一致 |
| 2 | 清理英文历史/重复文档 | 中英文设计目录收口 |
| 3 | 更新管理文档和 README 文档统计 | 评审入口准确 |
| 4 | 增加文档一致性检查脚本 | 防止旧口径回流 |
| 5 | 接入真实九宫格题集 | 授权链路更完整 |
| 6 | 增加题集与答案摘要版本化 | 支持长期演进 |
| 7 | 强化 SecretStore 平台适配 | 提升生产可信度 |
| 8 | 完善 EIP-712 production 配置 | 支持后续链上验证 |

## 七、完成定义

一个完善项只有同时满足以下条件，才视为完成：

1. 代码实现完成。
2. 自测或检查脚本覆盖完成。
3. 中文设计文档同步完成。
4. 英文设计文档同步完成或明确标注暂未同步。
5. README 或管理文档入口同步完成。
6. 不扩大当前能力宣传，不把后续探索写成已实现。

## 八、近期里程碑

### M1：文档收口版

目标时间：1 个工作日内。

完成内容：

1. 修正第二层 `SKILL.md`。
2. 清理或标注英文历史文档。
3. 更新管理文档覆盖度统计。
4. 补充文档一致性检查。

### M2：真实九宫格增强版

目标时间：3 到 5 个工作日。

完成内容：

1. 接入题集问题来源。
2. 按强度选择问题。
3. 按 cell 校验答案。
4. 增加状态机和错误路径自测。

### M3：生产边界增强版

目标时间：5 到 10 个工作日。

完成内容：

1. SecretStore 平台适配强化。
2. EIP-712 production domain 配置。
3. 题集版本和答案摘要版本化。
4. 完整评审演示脚本。

## 九、风险提醒

1. 不应把兼容演示包 `storylock-skill-engine` 描述为第四层安全边界。
2. 不应把远程网关描述为可以直接读取本地故事或秘密对象。
3. 不应把 placeholder EIP-712 domain 描述为生产链上验证能力。
4. 不应把当前占位九宫格描述为已接入真实题集。
5. 不应把内存 SecretStore 描述为生产安全存储。

