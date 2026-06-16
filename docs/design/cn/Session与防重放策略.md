# StoryLock Session 与防重放策略

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.0 |
| 日期 | 2026-06-16 |
| 状态 | 当前阶段采用 |
| 适用层级 | 第二层与第三层交界 |

## 目的

本文档用于明确：

1. 第二层签发的 session 应如何绑定
2. 第三层请求如何避免被重放
3. `nonce`、`expiry`、`requestId` 分别解决什么问题

## 核心结论

当前阶段采用最小可实现方案：

1. session 只由第二层本地访问 Skill 签发
2. session 必须绑定 challenge、scope、对象范围和过期时间
3. 所有远程委托请求必须带 `requestId + nonce + expiry`
4. 已使用的 `nonce` 和已完成的 `requestId` 必须本地记录，防止重复执行
5. `expiry` 校验默认允许小范围时钟漂移容差

## Session 设计目标

Session 不是“方便复用的登录态”，而是：

1. 本次 challenge 成功后的受限访问凭证
2. 某个对象范围内的短时授权结果
3. 可被主动撤销、自动过期、预算耗尽后失效的运行令牌

## Session 绑定字段

建议最小 session 元数据如下：

```json
{
  "sessionId": "ses-001",
  "challengeId": "chl-001",
  "identityId": "identity-001",
  "scope": "story_read_basic",
  "resourceScope": ["story-001"],
  "maxReads": 1,
  "issuedAt": 1760000200,
  "expiresAt": 1760000500,
  "status": "active"
}
```

## 必须绑定的约束

每个 session 至少应绑定：

1. `challengeId`
2. `identityId`
3. `scope`
4. `resourceScope`
5. `expiresAt`
6. `readBudget` 或 `writeBudget`

不允许签发“无限资源范围、无限时长、无限预算”的 session。

## Session 类型建议

| 类型 | 适用场景 | 默认预算 | 默认 TTL |
| --- | --- | --- | --- |
| `one_shot` | 单次读取或单次写回 | 1 次 | 1-5 分钟 |
| `short_session` | 短时连续读取 | 3-10 次 | 5-15 分钟 |
| `batch_session` | 批量处理 | 按策略设定 | 15-30 分钟 |
| `privileged_session` | 高敏签名或高信任写入 | 1 次 | 1-3 分钟 |

## 防重放目标

需要分别防止三类重放：

1. **请求重放**
   - 相同远程请求被重复提交
2. **签名重放**
   - 旧的 EIP-712 请求被重复要求本地签名
3. **会话重放**
   - 已过期或已耗尽预算的 session 被再次使用

## 字段职责划分

| 字段 | 作用 | 防护对象 |
| --- | --- | --- |
| `requestId` | 标识一次业务请求 | 防止同一业务请求重复执行 |
| `nonce` | 标识一次签名 / 授权序列 | 防止签名消息重放 |
| `expiry` | 约束请求有效时间窗 | 防止旧请求长期可用 |
| `sessionId` | 标识一次访问会话 | 防止过期会话复用 |

## Nonce 策略

当前阶段建议采用本地单调序列或高强度随机数二选一：

### 方案 A：单调递增 nonce

适合：

1. 单设备
2. 单身份
3. 本地状态稳定可持久化

要求：

1. 按 `identityId + capability + resource` 维护最近 nonce
2. 新请求 nonce 必须严格大于已接收值

### 方案 B：随机 nonce

适合：

1. 多端并发
2. 需要降低时序耦合

要求：

1. 使用加密安全随机数
2. 在 TTL 窗口内记录已使用 nonce 集
3. 窗口结束后可清理

当前阶段推荐优先实现 **方案 B**，实现更简单，也更适合网关包装场景。

## RequestId 幂等策略

第二层和第三层都应支持幂等处理。

规则：

1. 相同 `requestId` 的请求若已成功执行，返回同一结果摘要或明确的重复请求错误
2. 相同 `requestId` 且负载不同，直接拒绝
3. 已失败的 `requestId` 是否允许重试，应由本地策略决定

建议错误码：

1. `SLG-009 DUPLICATE_REQUEST`
2. `SLG-010 NONCE_REPLAY_DETECTED`
3. `SLG-011 REQUEST_EXPIRED`
4. `SLG-012 SESSION_REVOKED_OR_EXPIRED`

## Session 校验顺序

每次使用 session 前建议按固定顺序校验：

1. session 是否存在
2. session 状态是否为 `active`
3. 当前时间是否早于 `expiresAt`
4. 请求 scope 是否包含在 session scope 内
5. 目标对象是否包含在 `resourceScope` 内
6. 读写预算是否仍充足

任一步失败都应拒绝执行。

## 预算扣减原子性

当前阶段要求：

1. `readBudget` / `writeBudget` 的扣减必须与对象读取或写回放在同一事务内
2. 不允许先读对象、后异步扣减预算
3. 当预算为 `0` 时必须直接拒绝

SQLite 示例：

```sql
BEGIN IMMEDIATE;

UPDATE session_store
SET read_budget = read_budget - 1
WHERE session_id = :sessionId
  AND status = 'active'
  AND read_budget > 0
  AND expires_at > :now;

SELECT changes() AS affected_rows;

-- 若 affected_rows = 0，则回滚并拒绝

SELECT encrypted_object
FROM protected_story_objects
WHERE story_object_id = :storyObjectId;

COMMIT;
```

要求：

1. 预算扣减与对象读取在同一事务中完成
2. 若任一步失败，必须整体回滚
3. 不允许出现 `SELECT budget -> 应用层判断 -> UPDATE budget` 的拆分竞态

若 SQLite 版本支持 `RETURNING`，也可采用：

```sql
UPDATE session_store
SET read_budget = read_budget - 1
WHERE session_id = :sessionId
  AND status = 'active'
  AND read_budget > 0
  AND expires_at > :now
RETURNING read_budget;
```

## 绑定远程请求的最小校验链

对第三层转入的请求，第二层建议按以下顺序校验：

1. `requestId` 是否已处理
2. `expiry` 是否过期
3. `nonce` 是否重放
4. `capability` 是否在白名单内
5. `scope` 是否与本地策略兼容
6. 若需要 challenge，则检查 challenge / session 是否满足

## 存储建议

当前阶段可采用本地轻量存储：

1. `session_store`
2. `nonce_store`
3. `request_store`

当前阶段建议明确采用：

1. SQLite 单文件数据库
2. 数据库名可统一为 `storylock_vault.db`
3. 所有会话、nonce、requestId 去重信息默认写入该数据库

推荐原因：

1. 单文件、零配置、跨平台
2. 事务能力足够支撑当前阶段并发控制
3. 后续可平滑迁移到嵌入式 KV 或更强存储

建议字段：

```json
{
  "requestId": "req-001",
  "nonce": "n-001",
  "capability": "requestChallengeSign",
  "status": "completed",
  "createdAt": 1760000000,
  "expiresAt": 1760000300
}
```

## Nonce 存储格式建议

为了同时兼容 EIP-712 与本地实现，建议：

1. 协议层 `nonce` 保持 `uint256` 语义
2. 本地存储层允许保存为十六进制字符串
3. 写入前统一归一化，避免同一 nonce 出现多种文本表示

## Nonce 清理策略

若采用随机 nonce 方案，建议使用：

1. 滑动时间窗口
2. 窗口大小默认设为 `2 x maxTTL`
3. 后台定期清理已过窗口的 nonce 记录

建议再增加上限：

1. 窗口大小不应无限随 `maxTTL` 扩大
2. 默认最大窗口建议不超过 `24 小时`
3. 超过上限时应截断为上限值

进一步建议:

1. Host 策略应给 `maxTTL` 设置硬上限,默认不超过 `2 小时`
2. 每次写入新 nonce 时,可异步触发一次批量清理
3. 单次清理批次建议不超过 `1000` 条
4. `nonce_store.createdAt` 或 `expiresAt` 必须建索引

这样可以在不引入复杂分布式协调的前提下，控制本地存储增长。

## RequestId 清理策略

当前阶段建议：

1. 仅保留有效时间窗内和最近一段历史窗口内的 `requestId`
2. 已完成记录可按时间分批清理
3. 可在 SQLite 中按 `createdAt` 或 `expiresAt` 建索引，定期删除过期记录

这样可以避免：

1. 本地数据库无限增长
2. 长期运行后幂等表变成性能瓶颈

## 时钟漂移容差

当前阶段建议：

1. `expiry` 校验默认允许 `+/- 30 秒` 容差
2. 容差只用于处理设备时钟轻微漂移
3. 不得把容差扩大成长期有效窗口

## 版本兼容窗口与 Session 关系

在单故事终身制下，题集版本与对象封装层允许平滑迁移，因此 Session 也需要配合处理：

1. 常规题集优化时，不强制撤销全部有效 session
2. 若 session 绑定的是已进入 deprecated 但仍在兼容窗口内的题集版本，可继续用到 TTL 结束
3. 根级重建时，旧 session 必须立即撤销
4. 旧题集摘要、旧对象密文、旧 session 的兼容窗口应分别记录，不得混成一个统一超时值

## 与 EIP-712 的关系

在第三层签名请求中：

1. `nonce` 放入 `value.nonce`
2. `expiry` 放入 `value.expiry`
3. `delegationContext` 记录请求来源链路

但要注意：

1. EIP-712 只负责请求结构表达
2. 防重放的判定逻辑仍由本地网关执行

## 当前阶段最小实现建议

建议先实现：

1. `requestId` 幂等
2. `nonce` 去重
3. `expiry` 校验
4. `session TTL` 校验
5. `readBudget` / `writeBudget` 校验

暂不要求先实现：

1. 多设备同步 nonce
2. 分布式 session 一致性
3. 跨主机全局幂等

## 结论

StoryLock 当前阶段的 session 不应被设计成宽松复用凭证，而应被设计成：

1. 绑定 challenge 的短时令牌
2. 绑定范围和预算的最小授权结果
3. 结合 `requestId + nonce + expiry` 的防重放执行单元
