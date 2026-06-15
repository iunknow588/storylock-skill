# StoryLock Password Fill

## Overview

`LocalPasswordFillSkill` 面向“完成本地挑战授权后输出登录字段”的产品化模式。

它是 `LoginAuthorizationSkill` 的包装层，目标是把：

1. 创建挑战
2. 提交答案
3. 读取 secret object
4. 还原登录字段

组织成更接近登录填充场景的统一输出。

## Invocation Template

```js
import {
  LocalPasswordFillSkill,
  LOGIN_BINDING_MODE,
} from "./index.js";

const host = {
  async createChallenge(identityId, scope) {
    return { challengeId: "challenge-1", identityId, scope };
  },
  async submitChallengeAnswers(identityId, challengeId, answers) {
    return {
      sessionId: "session-1",
      identityId,
      challengeId,
      answerCount: answers.length,
    };
  },
  async readSecretObject(_identityId, _sessionId, secretObjectId) {
    return new TextEncoder().encode(`value-for:${secretObjectId}`);
  },
};

const skill = new LocalPasswordFillSkill({ host });

const result = await skill.run({
  identityId: "demo-user",
  siteId: "github",
  resourceId: "github-login",
  resourceCatalog: {
    resources: [],
  },
  bindings: [
    { fieldName: "username", secretObjectId: "secret-user" },
    { fieldName: "password", secretObjectId: "secret-pass" },
  ],
  bindingMode: LOGIN_BINDING_MODE.TEMPLATE_WITH_OVERRIDES,
  answers: ["answer-1", "answer-2"],
});
```

## Parameters

### `LocalPasswordFillSkill.run(input)`

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `identityId` | string | 是 | - | 身份标识 |
| `siteId` | string | 是 | - | 站点标识 |
| `resourceId` | string \| null | 否 | `null` | 资源配置标识 |
| `resourceCatalog` | object \| null | 否 | `null` | 资源目录 |
| `bindings` | array | 否 | `[]` | 直接传入的字段绑定 |
| `bindingMode` | string | 否 | `template_with_overrides` | 字段绑定模式 |
| `answers` | array | 否 | `[]` | 用户提交的挑战答案 |

### Constructor dependencies

| 依赖 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `host.createChallenge` | function | 是 | 创建本地挑战 |
| `host.submitChallengeAnswers` | function | 是 | 提交答案并换取会话 |
| `host.readSecretObject` | function | 是 | 在有效会话中读取秘密对象 |

## Output

返回对象结构：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `mode` | string | 固定为 `local_password_fill` |
| `siteId` | string | 当前站点标识 |
| `scope` | string | `vault_read_basic` 或 `vault_read_batch` |
| `challenge` | object | 挑战对象 |
| `authorization` | object | 授权结果，需包含 `sessionId` |
| `fields` | array | 登录字段列表 |

`fields` 中每项至少包含：

| 字段 | 说明 |
| --- | --- |
| `fieldName` | 表单字段名 |
| `value` | 解码后的字符串值 |
| `secretObjectId` | 对应 secret object 引用 |

## Error Handling

| 错误 | 触发条件 | 修复建议 |
| --- | --- | --- |
| `ValidationError: host.createChallenge must be a function` | 宿主缺少 challenge 接口 | 完整实现 host |
| `ValidationError: identityId must be a non-empty string` | 身份为空 | 提供有效 `identityId` |
| `ValidationError: siteId must be a non-empty string` | 站点为空 | 提供有效 `siteId` |
| `ValidationError: authorization result must include a sessionId` | 宿主返回值不完整 | 修正 `submitChallengeAnswers()` 返回结构 |
| `ValidationError: secretBytes must be a Uint8Array` | secret object 返回类型错误 | 让 `readSecretObject()` 返回 `Uint8Array` |

## Agent Guidelines

1. 先确认用户要的是“登录字段填充”，不是“签名授权”。
2. 预检宿主是否具备 `createChallenge / submitChallengeAnswers / readSecretObject`。
3. 若用户没有给 `bindings`，再看是否有 `resourceCatalog + resourceId` 可推导字段。
4. 输出时优先解释 `scope`、`fields` 和 `authorization.sessionId` 的含义。
5. 不要持久化 `fields[].value`，也不要把这一层描述成 StoryLock 的最终安全边界。
