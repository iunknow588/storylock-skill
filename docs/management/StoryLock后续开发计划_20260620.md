# StoryLock 后续开发计划

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.0 |
| 日期 | 2026-06-20 |
| 适用范围 | `skill/` 主线代码、平台宿主、发布与验证 |
| 依据 | 当前代码实现、现行设计文档、已完成一致性检查与自测结果 |

## 1. 目标

本计划用于明确 StoryLock 当前阶段之后的开发重点，避免继续把“架构设计”“原型代码”“正式交付能力”混写在一起。

当前项目已经具备：

1. 三层主线架构与自测闭环。
2. 第二层本地授权、问题集、session、防重放与撤销能力。
3. 第三层远程网关、EIP-712 请求包装、脱敏返回、下载入口、绑定与 relay 闭环。
4. Android 宿主与 Windows 宿主原型工程。
5. Linux 最小本地宿主原型、WSL 打包入口、`tar.gz` 与 `.deb` 原型包、桌面集成与 Debian staging 预留。
6. Windows / Linux / macOS 三个平台 SecretStore 适配器与启动期可用性守门。

当前项目尚未完全具备：

1. 生产级 Android 本地签名闭环。
2. Android 正式 release 真机验收与发布闭环。
3. Windows / Linux 正式签名包、安装验证与生产分发闭环。
4. Linux Secret Service 真桌面环境验收。
5. 以真机和正式构建为基准的交付级验收体系。

因此，后续开发目标不是继续扩写概念，而是把“主线可运行”推进到“平台可交付、文档可据实宣讲、构建可重复验证”。

## 2. 总体原则

1. 先补齐当前主线缺口，再扩展新能力。
2. 先解决“代码已存在但交付口径不清”的问题，再增加新设计。
3. 先完成平台安全边界和验收闭环，再谈多账户、多平台、多链扩展。
4. 设计文档只写当前已实现能力和明确的下一步，不把探索项写成已完成项。

## 3. 当前结论

### 3.1 已实现并应继续巩固的部分

1. `src/skills/local-story-processing` 的第一层主线能力。
2. `src/skills/local-story-access` 的第二层授权主线。
3. `src/skills/remote-gateway` 的第三层入口、脱敏和 Web API 链路。
4. 24 题模板生成、校验与导入流程。
5. 文档一致性检查、文本规范检查与主线 selftest。

### 3.2 部分实现、需要优先补齐的部分

1. Android 宿主已有真实工程、Keystore、BiometricPrompt、注册与 relay，但仍有 demo 级签名实现。
2. Android challenge 流程已有本地输入确认，但高强度多格验证尚未完全按第二层正式模型落地。
3. Windows 宿主已有 Rust 原型和本地执行闭环，但仍属于 prototype 发布口径。
4. Android 签名路径已从 demo HMAC 推进到 Android Keystore 非对称签名原型，但请求算法策略、认证绑定、审计与正式 release 验收仍需继续补齐。
5. Linux 平台已具备 SecretStore 适配、最小本地宿主、`npm run test:linux-host`、WSL 打包、`.deb` / `tar.gz` 原型包、desktop entry、systemd user unit 与站点 downloads 元数据；正式签名包、Secret Service 真环境验收和真实发行版安装验证尚未完成。
6. Vercel 本地构建、API 入口和下载路由已经对齐，但线上 `yian.cdao.online` 仍返回部署级 `404: NOT_FOUND`；当前本机执行生产部署时曾卡在 Vercel CLI 访问 `https://vercel.com/.well-known/openid-configuration` 的 TLS/OIDC 请求，且本地 `.vercel/project.json` 仍可能指向旧的 `skill` 项目。下一步应先补齐 Vercel token 或网络/TLS 访问条件，再把本地项目 link 到承载 `yian.cdao.online` 的 `storylock-gateway` 项目，并确认域名 alias 到最新生产部署。

### 3.3 尚未实现的部分

1. Android 正式 release 构建、签名、验签、发布闭环。
2. Windows 正式签名分发闭环。
3. Linux 正式签名包、真实 Linux 桌面安装、Secret Service 真环境验收与发布闭环。
4. macOS 独立宿主交付材料与平台验收记录。
5. 真机验证报告沉淀与固定化验收模板。

## 4. 开发阶段规划

## 第一阶段：主线补强与口径收敛

### 目标

把“当前真实状态”写清楚，把最影响评审和后续开发的偏差先收拢。

### 任务

1. 修订 Android 相关设计与评审文档口径：
   - 明确 Android 真机宿主原型已经存在。
   - 明确当前仍未完成生产级签名与正式发布闭环。
2. 修订平台宿主与发布说明：
   - Windows 宿主标注为 prototype。
   - Android APK 标注 debug/internal 与 release/candidate 的差异。
   - Linux 宿主标注为 prototype，区分 WSL 原型打包成功和真实 Linux 发行版验收完成。
3. 统一管理文档中的当前主线表述：
   - 三层主线。
   - Android/Windows/Linux 属于平台宿主实现方向。
   - 多账户、多平台、多链只保留为后续扩展方向。
4. 在第二层 Host 启动路径中补强平台 SecretStore 可用性检查与错误提示。

### 验收标准

1. 设计文档、评审文档、代码状态描述不再互相冲突。
2. 启动时缺失平台存储依赖会明确失败，而不是模糊降级。
3. `npm run test:docs` 与 `npm run test:text` 通过。

## 第二阶段：Android 宿主安全链补齐

### 目标

把 Android 宿主从“可运行原型”推进到“安全链条基本成立”。

### 任务

1. 将当前 demo HMAC 签名替换为真正的 Android Keystore 签名能力。
2. 将 challenge 交互从“局部演示式确认”补齐为与第二层一致的正式本地验证流程。
3. 梳理 Android 宿主内的长期材料、运行态材料与返回字段边界。
4. 明确 Android 本地返回结构中哪些字段仅供本地、哪些字段可交给第三层脱敏后返回。
5. 补充 Android 宿主的负面测试：
   - challenge 失败
   - BiometricPrompt 不可用
   - relay 超时
   - 无绑定状态
   - 错误 shared secret

### 验收标准

1. Android 本地签名不再依赖 demo HMAC key material。
2. challenge 验证流程与第二层正式模型一致，不再依赖内部补答案绕过。
3. Android 宿主具备可重复执行的真机闭环检查步骤。

## 第三阶段：平台能力补齐

### 目标

补齐当前明确缺口，使平台边界具备最小完整性。

### 任务

1. 固化 macOS Keychain SecretStore 适配器的验收口径。
2. 梳理 `createPlatformSecretStore(...)` 在 Windows、Linux、macOS 下的统一语义。
3. 为 Host 增加统一的 `checkAvailable()` 守门调用策略。
4. 补齐 Linux 平台材料、检查说明与目录口径。
5. 完善 Windows 宿主与第二层/第三层接口契合度检查。
6. 固化 Linux `.deb` / `tar.gz` 原型包、desktop entry、systemd user unit 与 WSL 打包脚本的回归检查。
7. 明确各平台 development mode 与 production mode 的限制与告警。

### 验收标准

1. 三个平台 SecretStore 接口行为一致。
2. 文档中的平台适配说明与代码完全一致。
3. 不再存在“文档声称支持、代码实际缺失”的平台能力。

## 第四阶段：发布与交付闭环

### 目标

把已有下载入口推进为真实的构建、签名、分发、验签闭环。

### 任务

1. 补齐 Android release 包构建与签名流程。
2. 固化 `release/app/android/` 的版本、校验值、渠道产物规范。
3. 补齐 Windows 包构建、签名、上传与元数据发布流程。
4. 补齐 Linux 正式包签名、真实发行版安装、Secret Service 真环境验收与回滚说明。
5. 将网站下载入口、Web API 元数据、发布脚本三者完全对齐。
6. 固化一份正式“发布前检查清单”：
   - 版本号
   - checksum
   - 渠道标识
   - 安装验证
   - 回滚策略

### 验收标准

1. 下载入口指向的产物、元数据和校验值一致。
2. Android/Windows/Linux 至少各有一条可重复执行的正式构建或候选构建流程。
3. 调试包、候选包、正式包的口径清晰，不混用。

## 第五阶段：测试与验收体系固化

### 目标

把现在的 selftest 扩展为更适合持续开发和评审交付的验证体系。

### 任务

1. 为第二层和第三层补充负面测试与边界测试。
2. 为 Android 宿主、Windows 宿主与 Linux 宿主补充平台级联调测试。
3. 固化真机检查报告模板。
4. 把文档一致性检查、文本规范检查、主线 selftest 纳入固定提交流程。
5. 对关键能力建立最小验收矩阵：
   - 本地授权
   - 防重放
   - 脱敏
   - 平台密钥存储
   - relay/poll/respond
   - 下载分发

### 验收标准

1. 主线能力具备正向与负向双向测试覆盖。
2. 平台宿主验证不再只依赖人工口头说明。
3. 任一版本在交付前都能跑通固定验收清单。

## 5. 优先级排序

### P0

1. 文档口径收敛到当前真实状态。
2. Android Keystore 签名原型继续推进到正式 release 验收口径。
3. Vercel 线上生产部署和 `yian.cdao.online` 404 问题闭环，优先补齐 `VERCEL_TOKEN` 或修复当前机器到 Vercel OIDC 端点的 TLS 访问，并确认 `.vercel/project.json`、`VERCEL_PROJECT_NAME=storylock-gateway` 与域名绑定项目一致。

### P1

1. Android challenge 正式化。
2. Linux Secret Service 真环境验收、正式签名包与真实发行版安装验证。
3. Windows/Android/Linux 发布脚本与元数据闭环。
4. macOS 独立宿主交付材料与验收说明固化。

### P2

1. 平台宿主更多 UI 与易用性补强。
2. 多账户、多对象、多链扩展能力。
3. 更完整的对外演示包装。

## 6. 交付物清单

后续每阶段建议至少产出以下交付物：

1. 代码提交与对应自测结果。
2. 更新后的设计/评审/管理文档。
3. 平台级操作说明或检查脚本。
4. 阶段验收记录。

## 7. 不建议当前阶段展开的事项

以下事项可以保留在设计参考中，但不应抢占当前主线资源：

1. 多链统一签名生态适配。
2. 多账户复杂授权编排。
3. 新的故事读写能力扩展。
4. 过度前置的产品包装与市场化表述。

## 8. 近期建议执行顺序

建议按以下顺序推进：

1. 先修正文档口径与计划基线，确保计划、清单、README 与验收矩阵一致。
2. 再处理 Vercel 线上生产部署，确认 `yian.cdao.online` 不再是部署级 404。
3. 再补 Android release 真机安全链、Windows 正式签名和 Linux 真桌面验收。
4. 再补 macOS 独立宿主交付材料。
5. 最后做扩展能力和更强的对外演示包装。

## 9. 结论

StoryLock 当前已经不是“只有概念”的阶段，而是“主线已跑通、平台原型已出现、但交付级闭环尚未完成”的阶段。

后续开发的核心不是继续扩写设想，而是把以下三件事做实：

1. 让代码状态、文档口径、评审表述完全一致。
2. 让 Android / Windows / Linux / SecretStore 的平台安全边界真正闭合。
3. 让构建、发布、验收流程具备可重复执行能力，并把线上部署状态纳入发布验收。
