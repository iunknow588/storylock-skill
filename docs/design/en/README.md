# StoryLock Design Documents (English)

This directory contains the English translations of StoryLock design documents, synchronized with the Chinese versions in `../cn`.

## Documents

| English Document | Chinese Source | Description |
| --- | --- | --- |
| `README.md` | `README.md` | Overview and reading guide |
| `GLOSSARY.md` | `术语表.md` | Core terminology definitions |
| `SKILL_POSITIONING_AND_BOUNDARIES.md` | `Skill定位与边界.md` | Skill layering and responsibility boundaries |
| `SYSTEM_SKILL_TABLE.md` | `系统Skill表与能力边界.md` | System skill table and capability boundaries |
| `THREE_PACKAGE_CONTRACT.md` | `三包接口契约.md` | Inter-package interface contracts |
| `LOCAL_AGENT_GATEWAY.md` | `本地Agent网关设计.md` | Local agent gateway design |
| `THREE_LAYER_AGENT_DESIGN.md` | `三层Agent设计方法.md` | Three-layer Agent design method |
| `SESSION_AND_REPLAY_PROTECTION.md` | `Session与防重放策略.md` | Session and anti-replay strategy |
| `THREE_SKILL_PACKAGES.md` | `storylock_three_skill_packages_cn.md` | Three skill package split strategy |
| `EIP712_MINIMAL_REQUEST.md` | `EIP-712最小请求定义.md` | EIP-712 minimal request definition |
| `REDACTION_SPEC.md` | `脱敏规范.md` | Data redaction specification |
| `SECURITY_SPEC.md` | `安全规范.md` | Security specification |
| `OBJECT_ACCESS_POLICY.md` | `对象访问策略.md` | Object access policy |
| `CHALLENGE_ANSWER_STORAGE.md` | `挑战答案存储策略.md` | Challenge answer storage strategy |
| `PLATFORM_KEY_STORAGE.md` | `平台密钥存储适配指南.md` | Platform key storage adaptation guide |
| `QUICK_DECISION_GUIDE.md` | `快速决策指南.md` | Quick decision guide |
| `CHALLENGE_STATE_MACHINE.md` | `Challenge状态机.md` | Challenge state machine |
| `PROXY_SIGNATURE_PROTOCOL.md` | `代理签名机制协议参考.md` | Proxy signature mechanism reference |
| `DESIGN_DOC_OPTIMIZATION.md` | `设计文档优化建议.md` | Design document optimization suggestions |

## Historical And Analysis Documents

The following files are retained as historical analysis or transition notes. They are not the current design baseline unless explicitly referenced by one of the documents above.

| Document | Status |
| --- | --- |
| `storylock_object_access_policy.md` | Historical analysis; superseded by `OBJECT_ACCESS_POLICY.md` |
| `storylock_skill_pharos_alignment_analysis.md` | Historical Pharos alignment analysis |
| `storylock_skill_positioning_and_boundary.md` | Historical draft; superseded by `SKILL_POSITIONING_AND_BOUNDARIES.md` |
| `storylock_skill_positioning_and_boundary_analysis.md` | Historical positioning analysis |
| `storylock_story_skill_feasibility_analysis.md` | Historical feasibility analysis |

## Reading Order

1. `README.md`
2. `GLOSSARY.md`
3. `SYSTEM_SKILL_TABLE.md`
4. `THREE_PACKAGE_CONTRACT.md`
5. `LOCAL_AGENT_GATEWAY.md`
6. `THREE_LAYER_AGENT_DESIGN.md`
7. `SESSION_AND_REPLAY_PROTECTION.md`
8. `SKILL_POSITIONING_AND_BOUNDARIES.md`
9. `THREE_SKILL_PACKAGES.md`
10. `EIP712_MINIMAL_REQUEST.md`
11. `REDACTION_SPEC.md`
12. `SECURITY_SPEC.md`
13. `OBJECT_ACCESS_POLICY.md`

## Design Principles

1. **Layer 1** handles text, story drafts, story refinement, and question-set strength assessment. It does not perform local sensitive authorization.
2. **Layer 2** determines required strength based on the target object, generates grid challenges, and issues short-term local authorization.
3. **Layer 3** is only responsible for remote request packaging, redaction, and controlled delegation. It does not read story content or directly hold long-term keys.
4. Signature requests uniformly use the neutral `requestSignature` expression, not tied to narrow scenarios like "challenge signing".
5. Web2 password filling is initiated via `requestPasswordFill`, returning only minimal audit results by default, not plaintext passwords.

## Technical Baseline

| Item | Current Requirement |
| --- | --- |
| Runtime | Node.js 22 or above, due to current SQLite adapter using `node:sqlite` |
| Local state storage | SQLite |
| Sensitive material storage | Current in-memory implementation and adapter interface; production should use system keychain or equivalent secure storage |
| Anti-replay | `requestId`, `nonce`, expiry time, and SQLite state tables jointly constrain |
| Signature request structure | Refer to EIP-712 standard, using `StoryLockSignatureRequest` as the project's structured request type |
