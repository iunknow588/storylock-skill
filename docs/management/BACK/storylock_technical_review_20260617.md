{
  "summary": "StoryLock技术评审报告：代码与文档一致性分析",
  "findings": [
    {
      "severity": "high",
      "type": "架构缺口",
      "description": "第二层LocalAuthorizationSkill签发的session默认readBudget=0/writeBudget=0，导致授权后的对象访问（如签名读取私钥）实际上无法通过readStoryObjectWithBudget的预算校验。文档中session应绑定读写预算，但代码中LocalAuthorizationSkill.run()硬编码budget为0，与对象访问策略中L4签名操作需要readBudget>=1的要求矛盾。",
      "location": "src/skills/local-story-access/index.js:LocalAuthorizationSkill.run()",
      "recommendation": "LocalAuthorizationSkill应根据allowedAction和对象类型设置合理的读写预算，或允许调用方通过policyHints传入预算参数。"
    },
    {
      "severity": "high",
      "type": "安全设计缺陷",
      "description": "第二层仍保留故事读写接口（readStoryObjectWithBudget/writeStoryObject），但文档明确第二层职责应收敛为'对象强度判断、九宫格验证与本地授权结果'，不应直接承担故事读取或写回。当前代码中第二层直接操作protected_story_objects表，模糊了第一层与第二层的边界。",
      "location": "src/skills/local-story-access/access-host.js:SqliteStore.readStoryObjectWithBudget()/writeStoryObject()",
      "recommendation": "将故事读写能力迁移至第一层或独立的持久化层，第二层仅负责授权结果返回，不直接操作故事对象内容。"
    },
    {
      "severity": "medium",
      "type": "命名不一致",
      "description": "第三层主接口已收敛为requestSignature/requestPasswordFill，但storylock-skill-engine的migrated层仍保留旧命名ChallengeSigningAuthorizationSkill和requestChallengeSign。文档要求将requestChallengeSign收敛为requestSignature，但migrated代码未同步更新。",
      "location": "src/engine/assets/migrated/skills/authorization-skills.js",
      "recommendation": "在migrated层新增SignatureAuthorizationSkill作为requestSignature的兼容实现，逐步废弃ChallengeSigningAuthorizationSkill。"
    },
    {
      "severity": "medium",
      "type": "功能缺失",
      "description": "文档要求第一层提供StrengthReviewSkill进行题集强度评估，但第一层storylock-local-story-processing-skill/index.js中未实现该Skill，仅在storylock-skill-engine的migrated层存在。第一层与migrated层能力未对齐。",
      "location": "src/skills/local-story-processing/index.js",
      "recommendation": "将StrengthReviewSkill从migrated层迁移至第一层，或确保第一层通过依赖引入该能力。"
    },
    {
      "severity": "medium",
      "type": "安全设计缺陷",
      "description": "第三层远程网关的transport函数直接透传完整请求对象，未对第二层返回结果做脱敏处理。文档要求第三层返回最小化结果，但当前代码中transport只是简单返回request，没有实现脱敏逻辑。",
      "location": "src/skills/remote-gateway/index.js:StoryLockRemoteGateway.invoke()",
      "recommendation": "在第三层增加结果脱敏层，确保返回给远程侧的结果不包含私钥、challenge答案等敏感字段。"
    },
    {
      "severity": "medium",
      "type": "实现不完整",
      "description": "EIP-712请求结构已定义，但第三层未实现实际的签名执行能力。requestSignature仅构造请求信封，未调用本地签名器完成实际签名。文档中第三层应'在本地完成授权与签名'，但当前代码缺少签名执行链路。",
      "location": "src/skills/remote-gateway/index.js",
      "recommendation": "第三层应集成第二层授权结果，并调用本地签名器完成签名，返回签名结果而非仅返回请求结构。"
    },
    {
      "severity": "low",
      "type": "配置风险",
      "description": "开发模式下MemorySecretStore允许明文存储masterSalt，但代码中developmentMode标志仅控制日志行为，未对安全存储降级做强制风险提示。文档要求开发模式必须有显式风险提示，但当前实现中风险提示不足。",
      "location": "src/shared/secret-store.js:MemorySecretStore",
      "recommendation": "MemorySecretStore初始化时应强制输出开发模式警告日志，并在文档中明确标注开发模式不可用于生产。"
    },
    {
      "severity": "low",
      "type": "文档与代码不一致",
      "description": "文档中错误码表定义SLG-008为REDACTION_REQUIRED，但代码中SLG-008被映射为replay_detected。错误码语义不一致。",
      "location": "src/skills/local-story-access/errors.js:ERROR_DEFS/ERROR_KEY_TO_CODE",
      "recommendation": "统一错误码定义，确保文档与代码一致。建议将replay_detected映射为新的错误码（如SLG-013），保留SLG-008给REDACTION_REQUIRED。"
    },
    {
      "severity": "low",
      "type": "性能与资源",
      "description": "SQLite nonce/request清理策略采用同步清理，在大量请求场景下可能阻塞主线程。文档建议单次清理批次不超过1000条，但代码中未实现批次限制。",
      "location": "src/skills/local-story-access/access-host.js:SqliteStore.cleanupExpired()",
      "recommendation": "为cleanupExpired添加批次限制参数，或改为异步后台清理。"
    },
    {
      "severity": "low",
      "type": "测试覆盖",
      "description": "第二层自测脚本覆盖了对象强度、九宫格、防重放、锁定等核心能力，但未覆盖session预算耗尽后的对象访问拒绝场景。readStoryObjectWithBudget的预算校验逻辑缺乏直接测试。",
      "location": "src/skills/local-story-access/scripts/selftest.mjs",
      "recommendation": "补充session预算耗尽后的对象访问拒绝测试用例。"
    }
  ],
  "conclusion": "StoryLock项目整体架构清晰，三层分离的设计思路正确，核心安全机制（AES-256-GCM、HKDF、HMAC、防重放、失败锁定）已实现。主要缺口在于：1）第二层session预算设置与对象访问策略不匹配；2）第二层仍保留故事读写职责，边界未完全收敛；3）第三层缺少实际签名执行能力；4）部分历史命名与文档新口径未同步。建议在M2-M3阶段优先修复session预算和职责边界问题，确保三层职责与文档定义严格一致。"
}
