# StoryLock Challenge 状态机

## 目的

本文档用于定义第二层本地故事访问 Skill 中 challenge 与 session 的最小状态机。

它直接服务于：

1. `storylock-local-story-access-skill`
2. `本地Agent网关设计.md`
3. `对象访问策略.md`

## 为什么必须先定义状态机

当前第二层的核心职责不是“调用一个函数”，而是：

1. 创建 challenge
2. 接收答案
3. 验证答案
4. 建立 session
5. 用 session 去读写受保护故事对象

如果没有状态机，第二层就只有接口，没有安全边界。

## 状态列表

建议使用以下最小状态集合：

1. `idle`
2. `challenge_created`
3. `answers_submitted`
4. `verified`
5. `session_active`
6. `session_expired`
7. `failed`
8. `locked`

## 状态含义

| 状态 | 含义 |
| --- | --- |
| `idle` | 尚未创建 challenge |
| `challenge_created` | challenge 已创建，等待用户提交答案 |
| `answers_submitted` | 已收到答案，等待验证结果 |
| `verified` | challenge 验证通过，允许建立 session |
| `session_active` | session 有效，可进行受保护对象读写 |
| `session_expired` | session 已过期，不再允许读写 |
| `failed` | 本次 challenge 验证失败 |
| `locked` | 失败次数或策略条件达到阈值，暂时锁定 |

## 状态转换

### 基本流程

1. `idle -> challenge_created`
2. `challenge_created -> answers_submitted`
3. `answers_submitted -> verified`
4. `verified -> session_active`
5. `session_active -> session_expired`

### 失败流程

1. `answers_submitted -> failed`
2. `failed -> challenge_created`
3. `failed -> locked`
4. `locked -> idle`（锁定窗口结束后）

## 状态转换条件

| 从状态 | 到状态 | 触发条件 |
| --- | --- | --- |
| `idle` | `challenge_created` | 本地访问层调用 `createChallenge` |
| `challenge_created` | `answers_submitted` | 用户提交答案 |
| `answers_submitted` | `verified` | 答案达到要求阈值 |
| `answers_submitted` | `failed` | 答案未达到阈值 |
| `verified` | `session_active` | 本地访问层签发 session |
| `session_active` | `session_expired` | TTL 到期、读预算耗尽或主动失效 |
| `failed` | `challenge_created` | 允许重试且未超过锁定阈值 |
| `failed` | `locked` | 达到失败次数阈值或异常策略触发 |
| `locked` | `idle` | 锁定窗口结束或人工解锁 |

## Session 激活条件

只有在以下条件全部满足时，才允许 `verified -> session_active`：

1. challenge 未过期
2. 答案验证通过
3. scope 已确认
4. 本地策略允许创建对应会话

## Session 失效条件

建议定义以下几类失效条件：

1. **时间失效**
   - TTL 到期
2. **预算失效**
   - 读预算耗尽
3. **策略失效**
   - 对象访问范围发生变化
4. **主动失效**
   - 用户主动注销或本地 Agent 主动撤销

## 锁定策略建议

建议最小锁定策略如下：

1. challenge 连续失败达到阈值后进入 `locked`
2. `locked` 状态下不允许继续提交答案
3. 锁定窗口结束后回到 `idle`

当前阶段建议再补充：

1. `maxRetryCount` 默认值建议为 `3`
2. 达到 `maxRetryCount` 后，`failed -> locked`
3. `locked` 状态持续时间默认 15 分钟，可由 Host 策略覆盖

当前阶段不必先把锁定策略做得很复杂，但必须有状态。

## 失败计数策略

当前阶段建议明确冻结如下规则：

1. 失败计数默认按 `identityId + 时间窗口` 统计，而不是按 `challengeId`
2. 默认时间窗口建议为 `24 小时`
3. 默认允许失败次数为 `3 次 / 窗口`
4. 成功验证后可重置当前窗口内连续失败计数
5. 达到阈值后进入 `locked`，并返回带 `retryAfter` 的错误响应

这样做的原因：

1. 若按 `challengeId` 计数，攻击者可通过不断创建新 challenge 绕过限制
2. 若完全不重置，正常用户可能因偶发错误被长期锁死

## 锁定期间的拒绝规则

`locked` 状态下建议固定执行：

1. 拒绝新的答案提交
2. 返回统一错误码，例如 `SLG-003`
3. 响应中附带 `retryAfter`
4. 不静默忽略，不使用含糊提示

示例：

```json
{
  "errorCode": "SLG-003",
  "message": "challenge temporarily locked",
  "retryAfter": 900
}
```

## 状态持久化建议

当前阶段建议把状态分成两类：

1. **运行态**
   - `answers_submitted`
   - 校验中的短时上下文
   - 默认放内存
2. **持久态**
   - `challenge_created`
   - `verified`
   - `session_active`
   - `failed`
   - `locked`
   - 默认写入本地轻量存储

建议最小持久化字段：

1. `challengeId`
2. `identityId`
3. `scope`
4. `state`
5. `createdAt`
6. `expiresAt`
7. `failureCount`
8. `lockUntil`

设备重启后：

1. 已过期 session 不恢复
2. 未过期但可恢复的 `session_active` 可按策略恢复
3. `locked` 状态应保留到 `lockUntil`
4. 重启后若当前时间已超过 `lockUntil`，则允许状态回到 `idle`

## 并发控制建议

第二层在同一 `challengeId` 上应保证状态转换串行化。

当前阶段建议：

1. 以 `challengeId` 为粒度加本地锁
2. 每次状态迁移都校验“当前状态是否符合预期”
3. 若状态已变化，拒绝本次迁移并返回明确错误
4. 同一 `identityId` 下的答案提交也应避免并行穿透失败计数与锁定逻辑

实现建议：

1. 单进程场景可使用进程内互斥锁
2. 多进程场景优先使用 SQLite 事务锁
3. 若存在文件级运行态补充文件，可再辅以文件锁

推荐当前阶段优先顺序：

1. Node.js / TypeScript + SQLite 事务
2. 仅在确有跨进程运行态文件时再增加文件锁

SQLite 事务建议：

1. 优先使用 `BEGIN IMMEDIATE`
2. 这样可以在写入前就拿到保留锁，减少并发竞态
3. 若实现层支持隔离级别声明，则等效目标应为 `SERIALIZABLE`

示例流程：

```sql
BEGIN IMMEDIATE;
SELECT state, failure_count
FROM challenge_state
WHERE challenge_id = :challengeId;

-- 校验当前状态是否允许迁移
UPDATE challenge_state
SET state = :nextState,
    updated_at = :now,
    failure_count = :nextFailureCount
WHERE challenge_id = :challengeId
  AND state = :expectedState;

COMMIT;
```

若 `UPDATE` 影响行数为 0，则应：

1. 视为状态已变化
2. 回滚事务
3. 返回并发冲突错误

这样可以避免：

1. 重复提交答案
2. 同时签发多个 session
3. 过期后仍被旧请求激活

## 默认时长建议

当前阶段建议默认值如下：

| 项目 | 默认值 |
| --- | --- |
| challenge TTL | 5 分钟 |
| one_shot session TTL | 3 分钟 |
| short_session TTL | 10 分钟 |
| batch_session TTL | 20 分钟 |
| locked 窗口 | 15 分钟 |

这些值是当前阶段的保守默认值，后续可由 Host 策略覆盖。

## 与访问强度的关系

本状态机与 `对象访问策略.md` 中的访问强度分级联动：

1. `L2`
   - 更适合短时读会话
2. `L3`
   - 适合中敏读写会话
3. `L4-L5`
   - 适合高敏签名或高信任写操作

也就是说：

1. Challenge 不是独立存在
2. 它必须和对象分类、访问门槛、session 类型一起工作

## 当前阶段最小实现建议

当前阶段不建议再把 `failed` 与 `locked` 视为“后补功能”。

当前阶段最小可接受实现应直接包含：

1. `idle`
2. `challenge_created`
3. `answers_submitted`
4. `verified`
5. `session_active`
6. `session_expired`
7. `failed`
8. `locked`

原因：

1. 第二层本身就是安全边界
2. 若无 `failed` / `locked`，则无法形成最基本的防暴力破解闭环
3. 管理文档已将失败计数与锁定策略列为 M2 前必须冻结项

## 结论

第二层本地故事访问 Skill 的关键不是单个 API，而是这个状态机。

只有先把：

1. challenge 状态
2. 验证结果
3. session 生命周期

定义清楚，第二层的安全边界才算真正成立。
