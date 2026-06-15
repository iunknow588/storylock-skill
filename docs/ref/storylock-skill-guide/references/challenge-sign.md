# StoryLock Challenge Sign

## Overview

这一页覆盖两类相关能力：

1. `SigningAuthorizationSkill`：完成挑战授权后取回签名相关材料
2. `ChallengeSigningAuthorizationSkill`：完成挑战授权后直接生成签名结果

比赛和产品说明里，通常优先引用 `ChallengeSigningAuthorizationSkill`，因为它更接近“挑战签名授权”的完整链路。

## Invocation Template

### Direct signing

```js
import { ChallengeSigningAuthorizationSkill } from "./index.js";

const host = {
  async createChallenge(identityId, scope) {
    return { challengeId: "challenge-1", identityId, scope };
  },
  async submitChallengeAnswers(identityId, challengeId, answers) {
    return { sessionId: "session-1", identityId, challengeId, answers };
  },
  async readSecretObject() {
    return new Uint8Array([1, 2, 3, 4]);
  },
};

const signer = ({ algorithm, payload, secretReference }) => ({
  kind: "demo_signature",
  algorithm,
  secretReference,
  hex: Array.from(payload).map((x) => x.toString(16).padStart(2, "0")).join(""),
});

const skill = new ChallengeSigningAuthorizationSkill({ host, signer });

const result = await skill.run({
  identityId: "demo-user",
  keyId: "key-main",
  algorithm: "ed25519",
  challengeCode: "login-challenge-123",
  secretObjectId: "secret-signing-key",
  answers: ["answer-1"],
});
```

### Authorization material only

```js
import { SigningAuthorizationSkill } from "./index.js";

const skill = new SigningAuthorizationSkill({ host });

const result = await skill.run({
  identityId: "demo-user",
  keyId: "key-main",
  algorithm: "ed25519",
  payload: "hello",
  secretObjectId: "secret-signing-key",
  includeKeyMaterial: false,
  answers: ["answer-1"],
});
```

## Parameters

### Shared inputs

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `identityId` | string | 是 | - | 身份标识 |
| `keyId` | string | 是 | - | 业务侧 key 标识 |
| `algorithm` | string | 是 | - | 签名算法名 |
| `secretObjectId` | string | 条件必填 | - | 直接指定主签名密钥引用 |
| `resourceId` | string \| null | 否 | `null` | 通过资源目录间接解析时使用 |
| `primaryRole` | string \| null | 否 | `null` | 资源角色 |
| `resourceCatalog` | object \| null | 否 | `null` | 资源目录 |
| `attachments` | array | 否 | `[]` | 附带材料声明 |
| `answers` | array | 否 | `[]` | 挑战答案 |

### `SigningAuthorizationSkill` extra inputs

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `payload` | Uint8Array \| string \| number[] | 是 | - | 待签名内容 |
| `includeKeyMaterial` | boolean | 否 | `false` | 是否把原始 key material 带回给上层 |

### `ChallengeSigningAuthorizationSkill` extra inputs

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `payload` | Uint8Array \| string \| number[] \| null | 否 | `null` | 原始负载 |
| `challengeCode` | string \| null | 否 | `null` | 当未直接传 `payload` 时可作为签名输入 |

### Constructor dependencies

| 依赖 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `host.createChallenge` | function | 是 | 创建挑战 |
| `host.submitChallengeAnswers` | function | 是 | 提交答案并换取会话 |
| `host.readSecretObject` | function | 是 | 读取主密钥或附件材料 |
| `signer` | function / object with `sign()` | `ChallengeSigningAuthorizationSkill` 必填 | 真正执行签名 |

## Output

### `SigningAuthorizationSkill`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `challenge` | object | 挑战对象 |
| `authorization` | object | 授权结果 |
| `keyId` | string | 业务 key 标识 |
| `algorithm` | string | 算法名 |
| `payload` | mixed | 原始负载 |
| `scope` | string | 授权范围 |
| `secretReference` | string | 主密钥引用 |
| `signingKey` | Uint8Array \| null | 仅在 `includeKeyMaterial=true` 时返回 |
| `attachments` | array | 附件材料结果 |

### `ChallengeSigningAuthorizationSkill`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `mode` | string | 固定为 `challenge_signing_authorization` |
| `challenge` | object | 挑战对象 |
| `authorization` | object | 授权结果 |
| `keyId` | string | 业务 key 标识 |
| `algorithm` | string | 算法名 |
| `payload` | Uint8Array | 归一化后的签名输入 |
| `scope` | string | 授权范围 |
| `secretReference` | string | 主密钥引用 |
| `signature` | mixed | `signer` 返回的签名对象 |
| `attachments` | array | 仅回显附件引用，不回显材料内容 |

## Error Handling

| 错误 | 触发条件 | 修复建议 |
| --- | --- | --- |
| `ValidationError: signer must be a function or expose sign()` | signer 注入不合法 | 传函数或带 `sign()` 的对象 |
| `ValidationError: keyId must be a non-empty string` | 未给 key 标识 | 提供业务 keyId |
| `ValidationError: algorithm must be a non-empty string` | 未给算法 | 传入算法名 |
| `ValidationError: challengeCode must be a Uint8Array, string, or byte array` | 未传 `payload` 且 `challengeCode` 类型错误 | 用字符串或字节数组 |
| `ValidationError: signingKeyBytes must be a Uint8Array` | `readSecretObject()` 返回类型错误 | 让宿主返回 `Uint8Array` |

## Agent Guidelines

1. 若用户要“完成挑战后直接签名”，优先使用 `ChallengeSigningAuthorizationSkill`。
2. 若用户只是要授权材料供上层自签，使用 `SigningAuthorizationSkill`。
3. 有 `resourceCatalog` 时，可通过 `resourceId` / `primaryRole` 解析 secret reference；没有时就要求明确传 `secretObjectId`。
4. 解释输出时，区分“拿到授权材料”和“已完成签名”。
5. 文档表述里应明确：这层只是技能封装，正式安全语义仍归 StoryLock Core。
