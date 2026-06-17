# StoryLock Session 与防重放策略

## 当前实现基线

本文档以当前 `storylock-local-story-access-skill/access-host.js` 为准。

当前第二层使用 SQLite 保存：

1. `challenge_state`
2. `session_store`
3. `request_store`
4. `nonce_store`
5. `failure_window`
6. `answer_digest_set`
7. `audit_log`

第二层不再保存 `protected_story_objects`，也不再提供故事对象读写方法。

## Session 目标

Session 是九宫格验证通过后的短时授权结果，不是长期登录态。

每个 session 绑定：

1. `sessionId`
2. `challengeId`
3. `identityId`
4. `scope`
5. `resourceScope`
6. `sessionType`
7. `readBudget`
8. `writeBudget`
9. `issuedAt`
10. `expiresAt`
11. `status`

默认预算为 `0/0`。只有明确动作需要读取本地材料时才授予预算，例如 `signature` 与 `password_fill` 当前为 `readBudget=1, writeBudget=0`。

## 防重放字段

| 字段 | 作用 |
| --- | --- |
| `requestId` | 标识一次业务请求，支持幂等返回 |
| `nonce` | 标识一次授权或签名序列，防止重放 |
| `expiry` | 限制请求有效时间 |
| `sessionId` | 标识一次本地授权结果 |

当前实现：

1. 相同 `requestId` 且请求哈希一致，返回已保存响应。
2. 相同 `requestId` 但请求不同，拒绝。
3. 已使用 `nonce` 再次出现，拒绝。
4. 过期请求返回 `SLG-011`。
5. replay 冲突返回 `SLG-013`。

## Challenge 状态

当前代码实际使用的 challenge 状态包括：

1. `challenge_created`
2. `answers_submitted`
3. `verified`
4. `failed`
5. `locked`
6. `expired`
7. `idle`（用于锁定解除后的状态回收）

完整 8 状态模型可作为后续增强，但当前实现以可运行的 SQLite 事务状态为准。

## 失败锁定

当前策略：

1. 24 小时失败窗口。
2. 连续 3 次失败进入锁定。
3. 锁定时间 15 分钟。
4. 锁定到期后自动恢复。

## 清理策略

`cleanupExpired(now, { batchSize })` 会清理或标记：

1. 过期 `request_store`
2. 过期 `nonce_store`
3. 过期 active session
4. 过期 challenge

默认单次批次上限为 1000，避免长时间阻塞主线程。

当前会在 `createAccessHost()` 初始化时触发一次清理。长期运行宿主后续应增加定时调用或 CLI 清理命令。

## 错误码

| 错误码 | 类型 | 场景 |
| --- | --- | --- |
| `SLG-011` | `request_expired` | 请求过期 |
| `SLG-013` | `replay_detected` | requestId 或 nonce 冲突 |
| `SLG-003` | `challenge_failed` | 答案不匹配 |
| `SLG-004` | `challenge_locked` | 失败过多后锁定 |
| `SLG-005` | `session_invalid` | session 不存在、过期或状态无效 |

## 后续完善

1. 补充定时清理机制。
2. 明确多设备 nonce 策略。
3. 如后续恢复对象读写，需要在独立持久化层实现预算扣减与对象访问的原子事务。
