# StoryLock 三个 Skill 包拆分策略

本文说明 StoryLock 为什么拆成三个主包，以及每个包在当前代码中的实际责任。

## 结论

当前主线采用三个 Skill 包：

1. `storylock-local-story-processing-skill`
2. `storylock-local-story-access-skill`
3. `storylock-remote-gateway-skill`

另外，`storylock-skill-engine` 是兼容演示包，用于保留本地密码填充和签名授权示例，不作为新的安全层。

## 包一：本地故事处理 Skill 包

路径：

`src/skills/local-story-processing`

当前能力：

1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

责任：

1. 处理故事草稿生成。
2. 处理故事润色。
3. 评估题集或问题集合强度。

边界：

1. 不读取受保护对象。
2. 不签发授权。
3. 不使用签名密钥或密码凭据。

## 包二：本地访问授权 Skill 包

路径：

`src/skills/local-story-access`

当前能力：

1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`

责任：

1. 根据要访问或使用的对象判断所需强度。
2. 生成对应的九宫格验证。
3. 校验答案并签发短时本地授权。
4. 将 session、防重放、失败窗口、答案摘要和审计日志写入 SQLite。

边界：

1. 不再提供故事对象读取接口。
2. 不再提供故事对象写回接口。
3. 不直接执行远程请求。
4. 不返回长期秘密材料。

## 包三：远程网关 Skill 包

路径：

`src/skills/remote-gateway`

当前接口：

1. `requestSignature`
2. `requestPasswordFill`

责任：

1. 包装第三方或远程 Agent 的请求。
2. 对签名请求使用 `StoryLockSignatureRequest` 结构。
3. 调用可选的本地执行器。
4. 对结果做递归脱敏。
5. 返回结构化请求、响应与审计元信息。

边界：

1. 不读取故事内容。
2. 不直接持有私钥或密码。
3. 不暴露 `requestChallengeSign` 等旧接口。
4. 不把远程网关写成故事读写入口。

## 调用关系

推荐的主线调用关系是：

1. 第三层接收远程签名或密码填充请求。
2. 第二层完成对象强度判断、九宫格验证和短时授权。
3. 本地执行器在授权后完成签名或密码填充。
4. 第三层返回脱敏后的结构化结果。

第一层与第二层之间没有强制直接调用关系。第一层负责文本与题集质量，第二层负责访问授权。

## 关于 Pharos Skill Engine

当前项目可以参考 Pharos Skill Engine 的 Skill 包组织方式，但本地安全访问能力应由 StoryLock 自己实现。

更准确的划分是：

1. 第一层和第二层：StoryLock 自己实现，重点保护本地数据与授权边界。
2. 第三层：可面向 Pharos 或其他 Agent 生态做接口适配。
3. 兼容演示包：保留可运行示例，帮助验证本地密码填充和签名授权链路。

## 当前不再沿用的旧说法

以下旧说法已不适合作为当前文档口径：

1. “第二包负责受保护故事对象读写”。
2. “第三包调用第二包再调用第一包读取故事”。
3. “签名接口叫已废弃旧接口 `requestChallengeSign`”。
4. “当前可借鉴 `SigningAuthorizationSkill` 作为主线签名接口”。

当前主线签名口径统一为 `requestSignature` 与 `SignatureAuthorizationSkill`。
