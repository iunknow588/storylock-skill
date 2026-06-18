# StoryLock 中文设计文档

本目录用于说明 StoryLock 当前代码基线下的能力划分、运行边界、安全约束与开发落地进展。文档以 `src` 和 `android-host` 中已经落地的代码为准；历史评审意见只作为参考，不再作为当前设计口径。

## 当前代码基线

当前主线由三个 Skill 包和一个兼容演示包组成：

| 代码包 | 当前定位 | 主要能力 |
| --- | --- | --- |
| `src/storylock-local-story-processing-skill` | 第一层：本地故事处理与强度分析 | `StoryDraftSkill`、`StoryRefineSkill`、`StrengthReviewSkill` |
| `src/storylock-local-story-access-skill` | 第二层：对象强度、九宫格验证与本地授权 | `ObjectStrengthPolicySkill`、`GridChallengeSkill`、`LocalAuthorizationSkill` |
| `src/storylock-remote-gateway-skill` | 第三层：远程请求包装与代理授权入口 | `requestSignature`、`requestPasswordFill` |
| `src/ui` | 易安网站层：项目展示、APK 下载、首次绑定与运行态检查 | 首页、双语文案、APK 摘要、网关状态 |
| `src/storylock-skill-engine` | 兼容演示包 | `LocalPasswordFillSkill`、`SignatureAuthorizationSkill` 等本地执行示例 |

第二层当前不提供故事对象读取或写回接口。第三层也不再暴露 `requestStoryRead`、`requestStoryWrite`、`requestChallengeSign`、`requestCapabilityStatus` 等旧接口。

## 推荐阅读顺序

1. `系统Skill表与能力边界.md`
2. `三包接口契约.md`
3. `本地Agent网关设计.md`
4. `三层Agent设计方法.md`
5. `Session与防重放策略.md`
6. `Skill定位与边界.md`
7. `storylock_three_skill_packages_cn.md`
8. `EIP-712最小请求定义.md`
9. `脱敏规范.md`
10. `安全规范.md`
11. `对象访问策略.md`
12. `开发落地路线与当前进展.md`

如需了解历史整理思路，可以阅读 `设计文档优化建议.md`，但该文档不再代表当前实现基线。

## 设计原则

1. 第一层处理文本、故事草稿、故事润色和题集强度评估，不承担本地敏感授权。
2. 第二层根据要访问或使用的对象判断所需强度，生成九宫格验证，并签发短时本地授权。
3. 第三层只负责远程请求包装、脱敏与受控委托，不读取故事内容，也不直接持有长期密钥。
4. 签名请求统一使用中性的 `requestSignature` 表达，不绑定“挑战签名”这类过窄场景。
5. Web2 密码填充通过 `requestPasswordFill` 发起，默认只返回最小审计结果，不返回明文密码。
6. 易安网站层只负责项目展示、下载、绑定和状态检查；Android 宿主层仍是本地主执行边界。
7. PHAROS 是可选锚定层 / 可信协作层，不替代 Android 宿主中的本地授权执行。

## 技术基线

| 项目 | 当前要求 |
| --- | --- |
| 运行时 | Node.js 22 或以上，原因是当前 SQLite 适配使用 `node:sqlite` |
| 本地状态存储 | SQLite |
| 敏感材料存储 | 当前有内存实现与适配接口；生产环境应接入系统密钥链或等价安全存储 |
| 防重放 | `requestId`、`nonce`、过期时间与 SQLite 状态表共同约束 |
| 签名请求结构 | 参考 EIP-712 标准，使用 `StoryLockSignatureRequest` 作为本项目的结构化请求类型 |

## 当前不作为主线实现的内容

以下内容可以作为未来探索方向，但不作为当前已实现能力描述：

1. 完整链上验证。
2. EIP-1271 或 ERC-4337 的生产级集成。
3. 多链、多钱包、多签名主体管理。
4. 多平台内容发布和多账号凭据编排。
5. 把第三层网关作为故事读取或故事写回入口。
6. 把第二层实现成业务内容读写层。
