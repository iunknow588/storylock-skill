# StoryLock 三层 Agent 设计方法

## 目的

本文说明如何把 StoryLock 三层 Skill 结构表达为 Agent 可理解、可调用、可审计的能力体系。

这里的 Agent 化不是让远程 Agent 直接读取故事、密码、私钥或答案，而是把每一层的职责、输入、输出、禁止事项和安全前置检查写成明确的工具契约。

Pharos Skill Engine Guide 提供了一个可参考的方法：通过 `SKILL.md`、能力清单、前置检查、调用模板和安全约束，把底层工具整理成 Agent 可调用能力。StoryLock 可以参考这种组织方式，尤其用于第三层远程网关，但 StoryLock 的安全边界仍然以本地授权为核心。

## 三层 Agent 化原则

1. 第一层可以辅助 Agent 生成、整理和评估故事题集，但不接触 secret。
2. 第二层可以被本地 Agent 或宿主调用，用于对象强度判断、九宫格验证、本地授权和审计。
3. 第三层是远程 Agent 的唯一主入口，负责请求包装、委托执行和脱敏返回。
4. 远程 Agent 不直接调用第二层内部 API。
5. 任意 Agent 都不能绕过 challenge、session、防重放和 SecretStore 边界。

## 第一层：故事处理 Agent

第一层对应 `storylock-local-story-processing-skill`。

### Agent 角色

第一层 Agent 是故事与题集辅助 Agent，负责把用户提供的故事材料整理成可用于验证的问题候选。

### 可暴露能力

1. `StoryDraftSkill`：生成故事草稿。
2. `StoryRefineSkill`：润色和整理故事文本。
3. `StrengthReviewSkill`：评估问题或题集强度。

### 禁止事项

1. 不读取密码、私钥、签名 key 或 SecretStore。
2. 不签发授权。
3. 不创建 challenge。
4. 不把题集评估结果写成已经授权成功。

第一层 Agent 可以说“这个问题更适合作为 high 强度候选”，但不能说“该对象已经授权可访问”。

## 第二层：本地授权 Agent

第二层对应 `storylock-local-story-access-skill`。

### Agent 角色

第二层 Agent 是本地授权控制 Agent。它可以在本地宿主环境中执行，但不应作为远程 Agent 的直接入口。

### 可暴露能力

1. `ObjectStrengthPolicySkill`：根据对象类型和动作判断强度。
2. `GridChallengeSkill`：生成九宫格 challenge。
3. `LocalAuthorizationSkill`：校验答案并签发短时授权。
4. `LocalRevocationSkill`：撤销 challenge 或 authorization。

### 本地 Agent 前置检查

1. 题集是否存在且 active。
2. 题集数量是否满足所需强度。
3. `requestId` 和 `nonce` 是否重放。
4. challenge 是否过期。
5. identity 是否被失败窗口锁定。
6. SecretStore 是否符合当前运行模式。

### 禁止事项

1. 不把答案原文返回给远程侧。
2. 不把 SecretStore 中的明文 secret 返回给远程侧。
3. 不把第二层描述为故事读写层。
4. 不接受远程 Agent 绕过本地用户确认。

## 第三层：远程网关 Agent

第三层对应 `storylock-remote-gateway-skill`。

### Agent 角色

第三层 Agent 是远程请求进入本地授权链路的唯一主入口。它最适合参考 Pharos Skill Engine 的组织方法，把能力描述成远程 Agent 可调用的工具契约。

### 可暴露能力

1. `requestSignature`
2. `requestPasswordFill`

当前不暴露 `requestStoryRead`、`requestStoryWrite`、`requestChallengeSign`、`requestCapabilityStatus` 等旧接口。

### 第三层 Agent 前置检查

1. `capability` 是否在白名单内。
2. `requestId`、`nonce`、`expiry` 是否存在。
3. 签名算法是否在允许范围内。
4. EIP-712 production domain 是否拒绝 placeholder、零 chainId、零合约地址。
5. `requestedRetention` 是否允许。
6. 返回结果是否已递归脱敏。

### 禁止事项

1. 不持有私钥。
2. 不持有密码。
3. 不持有九宫格答案。
4. 不直接读取 SecretStore。
5. 不把本地执行器返回的敏感字段透传给远程 Agent。

## 推荐调用链

### 密码填充

```text
Remote Agent
  -> Layer 3 requestPasswordFill
  -> Layer 2 ObjectStrengthPolicySkill: credential + password_fill => medium
  -> Layer 2 GridChallengeSkill: 6 cells
  -> Local user answers challenge
  -> Layer 2 LocalAuthorizationSkill
  -> Local executor fills password locally
  -> Layer 3 returns audit metadata or redacted result
```

### 签名

```text
Remote Agent
  -> Layer 3 requestSignature
  -> Layer 2 ObjectStrengthPolicySkill: signature_key + signature => high
  -> Layer 2 GridChallengeSkill: 9 cells
  -> Local user answers challenge
  -> Layer 2 LocalAuthorizationSkill
  -> Local executor signs payload locally
  -> Layer 3 returns signature and redacted metadata
```

## Agent 工具契约写法

每个 Agent-facing Skill 文档应至少包含：

1. 能力名称。
2. 输入字段。
3. 输出字段。
4. 前置检查。
5. 禁止事项。
6. 错误码。
7. 敏感字段脱敏规则。
8. 示例调用。

示例：

```json
{
  "capability": "requestSignature",
  "identityId": "storylock-demo-user",
  "keyId": "wallet/main/private_key",
  "algorithm": "ed25519",
  "payload": "message to sign"
}
```

## 与 Pharos Skill Engine 的关系

Pharos Skill Engine 的价值在于提供 Agent 化组织方式：将 CLI、链上动作、配置检查和安全提醒整理成 Agent 可调用工具。

StoryLock 可以参考：

1. `SKILL.md` 作为 Agent 调用契约。
2. capability index。
3. preflight check。
4. 操作模板。
5. 敏感字段禁止泄露规则。

StoryLock 不应照搬：

1. 让 Agent 直接控制 secret。
2. 让第三层直接执行本地敏感读写。
3. 把链上执行器模型等同于本地授权模型。

## 当前实现状态

当前已经具备三层 Skill 包、第三层 `requestSignature` 与 `requestPasswordFill`、第二层对象强度/九宫格/本地授权/防重放/审计、第三层递归脱敏和三层 e2e selftest。

第三层已经提供机器可读 Agent 能力清单：`src/storylock-remote-gateway-skill/assets/agent-capabilities.json`。该清单通过 `npm run check:agent-capabilities` 校验，确保 Agent-facing capability 与 `StoryLockRemoteGateway` 当前主接口保持一致。

面向 Agent 的 HTTP 或 MCP host、自动生成 24 个问题的 Agent 工作流仍属于后续扩展。
