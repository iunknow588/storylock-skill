# StoryLock Challenge 状态机

本文定义第二层本地访问授权中的 challenge、session 与 authorization 的最小状态机。它服务于 `storylock-local-story-access-skill`，不再描述故事对象读取或写回流程。

## 当前职责

第二层负责：

1. 根据对象策略创建九宫格验证。
2. 接收并校验本地答案。
3. 记录失败窗口和答案摘要。
4. 验证通过后创建短时 session 或 authorization。
5. 写入本地审计日志。

第二层不负责：

1. 读取故事内容。
2. 写回故事内容。
3. 直接执行签名或密码填充。
4. 向远程侧返回长期秘密。

## 最小状态集合

| 状态 | 含义 |
| --- | --- |
| `created` | challenge 已创建，等待提交答案 |
| `verified` | 答案验证通过，可签发短时授权 |
| `failed` | 答案验证失败 |
| `expired` | challenge 已过期 |
| `locked` | 失败次数或策略条件达到阈值，暂时锁定 |

session 或 authorization 的状态建议单独记录：

| 状态 | 含义 |
| --- | --- |
| `active` | 短时授权有效 |
| `expired` | TTL 到期或预算耗尽 |
| `revoked` | 被用户或 Host 主动撤销 |

## 状态转换

### Challenge 转换

1. `created -> verified`：答案达到策略要求。
2. `created -> failed`：答案未达到策略要求。
3. `created -> expired`：超过 challenge TTL。
4. `failed -> created`：允许重试且未超过锁定阈值。
5. `failed -> locked`：达到失败次数阈值。
6. `locked -> created`：锁定窗口结束后重新创建。

### Authorization 转换

1. `verified -> active`：本地策略允许签发短时授权。
2. `active -> expired`：TTL 到期、读写预算耗尽或 nonce/requestId 失效。
3. `active -> revoked`：用户或 Host 主动撤销。

## Session 激活条件

只有同时满足以下条件时，才允许签发短时 session 或 authorization：

1. challenge 未过期。
2. 答案验证通过。
3. scope 与目标对象匹配。
4. requestId 与 nonce 未被使用。
5. 本地策略允许该能力被请求。

## 失败与锁定策略

当前建议：

1. 失败计数按 `identityId + 时间窗口` 统计。
2. 默认窗口为 24 小时。
3. 默认最大失败次数为 3 次。
4. 达到阈值后进入 `locked`。
5. 成功验证后可重置连续失败计数。

锁定期间：

1. 拒绝新的答案提交。
2. 返回明确错误。
3. 响应中包含 `retryAfter`。
4. 不泄露具体哪一题错误。

## SQLite 持久化

当前实现使用 SQLite 保存访问授权相关状态，主要包括：

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `question_set_item`
8. `audit_log`

其中：

1. `question_set_item` 是长期题集主档，保存题目引用、版本、状态与答案摘要。
2. `challenge_state.challenge_manifest_json` 是单次 challenge 题目绑定，不含答案原文。
3. `challenge_state.required_threshold` 明确本次通过阈值；当前 low/medium/high 分别为 3/6/9。

第二层不再保存 `protected_story_objects`。

## 并发控制

同一 `challengeId`、`requestId` 或 `nonce` 上应保证状态转换串行化。

建议：

1. 使用 SQLite 事务保护状态迁移。
2. 写入前校验当前状态是否符合预期。
3. 状态已变化时返回并发或重放错误。
4. replay 冲突统一使用 `SLG-013`。

## 默认时长建议

| 项目 | 默认值 |
| --- | --- |
| challenge TTL | 5 分钟 |
| one-shot authorization TTL | 3 分钟 |
| short session TTL | 10 分钟 |
| locked 窗口 | 15 分钟 |

这些值是保守默认值，后续可由 Host 策略覆盖。

## 结论

Challenge 状态机的目的不是支持故事读写，而是形成本地访问授权闭环：

1. 创建九宫格验证。
2. 校验答案。
3. 防止重放和暴力尝试。
4. 签发短时授权。
5. 记录审计。
