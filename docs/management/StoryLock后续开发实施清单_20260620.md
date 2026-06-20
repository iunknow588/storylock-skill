# StoryLock 后续开发实施清单

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v1.0 |
| 日期 | 2026-06-20 |
| 对应计划 | `docs/management/StoryLock后续开发计划_20260620.md` |
| 用途 | 将阶段计划拆分为可执行任务清单、输出物清单与验收动作 |

## 1. 使用说明

本清单用于后续逐项推进开发，不再停留在“方向描述”层面。  
每项任务应至少回答四个问题：

1. 改哪里。
2. 改成什么。
3. 产出什么。
4. 怎么验收。

## 当前进度

1. `A1` 已完成。
2. `A2` 已完成。
3. `A3` 已完成。
4. `B1` 已完成。
5. `B2` 已完成，本次补齐了平台 SecretStore 启动期显式守门口径与 Linux 说明材料。
6. `C1` 进行中，已从 demo HMAC 推进到 Android KeyStore 非对称签名原型。
7. Linux 平台材料已从“适配 + 检查 + 目录预留”推进到本地宿主原型、原型包与桌面集成预留口径，但 Linux 正式发行版验收仍未完成。
8. `C2` 进行中，Android challenge 已从单题确认推进到按强度要求的多格输入与结构化失败返回。
9. `C2` 已进一步补入 Android 原型侧的失败计数与临时锁定语义，但仍未与第二层正式题集持久化完全对齐。
10. `C3` 进行中，已补 Android 宿主联调失败场景自测与真机检查模板。
11. `C2` 已继续推进到 asset-backed 本地题集加载，不再把 challenge 题集完全硬编码在 Android 运行时代码中。
12. `C2` 已补 Android 题集资源校验、宿主启动期 identityId 对齐检查，以及仓库侧题集校验脚本。
13. `D1` 已补 macOS Keychain SecretStore 适配、自测工厂断言、检查脚本与主文档口径同步。
14. `E3` 进行中，本次新增 `test:release`，用于校验网站 downloads 元数据、Android/Windows 二进制产物、文件大小与 SHA-256 checksum 一致。
15. `F1` 进行中，本次把发布元数据一致性纳入主测试链路，避免下载入口和实际产物脱节。
16. `F2` 进行中，本次新增平台验收矩阵与 `test:delivery`，把 Android 真机、Windows 本地宿主、发布元数据和交付前检查入口固定下来。
17. `E1/E2` 继续推进中，本次在平台验收矩阵中补齐 Android/Windows 发布前检查项，但正式证书签名、Android release 真机验收、Windows 生产发布仍未全部完成。
18. `E2/F2` 已继续推进 Windows 版本可运转验证：`check_windows_host_loop.ps1` 改为 UTF-8 JSON 请求并对 health、question-bank、verify、authorize、execute、revoke 关键字段做硬断言；Rust 宿主题库导入兼容 UTF-8 BOM；新增 `npm run test:windows-host` 入口。
19. Linux 已从“SecretStore 适配 + 目录预留”推进到最小本地宿主原型：新增 `src/host/linux-host/`，复用第二层 `local-story-access` 完成 health、question-bank、verify、authorize、execute、revoke，并新增 `npm run test:linux-host` 本地闭环自测；正式 Linux 签名包、Secret Service 真环境验收仍未完成。
20. Linux 发布链继续推进：新增 `npm run package:linux-host` 生成 Linux 原型归档与 manifest，`npm run build` 已能把 Linux 产物复制到 `release/web/public/downloads/` 并生成 JSON 元数据，`test:release` 已要求 Android/Windows/Linux 三平台元数据与二进制一致。
21. Linux 原型包继续加固：新增 `npm run test:linux-package`，会打开 Linux `tar.gz` 或 zip 归档校验 Linux host、题库、`local-story-access` 与 `shared` 模块，并在 `.deb` 存在时检查 Debian 包中的 executable、desktop entry 与 systemd user unit，避免只校验外层 checksum 的假闭环。
22. Linux 正式化继续推进：新增生产模式启动脚本、desktop entry、systemd user unit、Debian control/postinst/prerm、Debian staging 目录与 `npm run test:linux-desktop`；新增 `npm run package:linux-host:wsl`，可在 WSL 中检查 Node.js >=22 与 `dpkg-deb` 后生成 `.deb` 原型包。
23. WSL 打包脚本继续加固：已修复 Windows 中文路径传入 WSL 时的路径转换问题，并改为临时 shell 脚本执行多行检查；脚本会主动加载 WSL 内 `nvm`，选择已安装的最高 Node.js `>=22`，或使用 `STORYLOCK_WSL_NODE_BIN` 指定的 Node.js。当前已在 WSL 中使用 Node.js `v24.17.0` 成功生成 Linux `tar.gz` 与 `.deb` 原型包，manifest 已记录两个 artifact 及 SHA-256。
24. Linux 发布元数据继续完善：`npm run build` 已能同时复制 Linux `.deb` 与 `tar.gz` 到站点 downloads，并分别生成 `-deb.json` 与 `-tar-gz.json`，避免同版本多格式 metadata 互相覆盖；`npm run test:linux-package` 已纳入 `.deb` 内容检查。
25. Vercel 发布链继续加固：`preflight.ps1`、`publish_site_release.ps1` 与 `sync_env_file_to_vercel.ps1` 已增加 `.vercel/project.json` 与 `VERCEL_PROJECT_NAME` 一致性检查，避免本地构建正确但部署到错误 Vercel 项目，导致 `yian.cdao.online` 仍返回部署级 `404: NOT_FOUND`。
26. Vercel 首次修复 404 流程继续完善：`publish_site_release.ps1 -Preflight` 已改为部署前跳过线上 HTTP、只做本地环境和项目绑定检查；`vercel deploy --yes` 成功后再执行线上 HTTP preflight，避免旧生产环境 404 阻断新部署。
27. Vercel 部署诊断继续完善：`publish_site_release.ps1` 已增加部署前 `vercel whoami` 认证/网络检查、部署重试和 OIDC/TLS 失败提示；当前本机部署失败点是 Vercel CLI 无法访问 `https://vercel.com/.well-known/openid-configuration`，且未设置 `VERCEL_TOKEN`，不是站点构建或路由代码失败；脚本已能在真实部署前给出明确错误说明。
28. Vercel 发布通道继续完善：新增 `.github/workflows/vercel-production.yml`，可通过 GitHub Actions 使用 `VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID` 与 `STORYLOCK_ANDROID_SHARED_SECRET` 执行生产部署，绕开当前 Windows 本机 Vercel OIDC/TLS 访问受阻的问题；工作流会先跑 `npm test`、`npm run build`，部署后再检查 `yian.cdao.online` 关键端点。
29. 参考 `ming/scripts/deploy_vercel.sh` 后补齐 WSL 发布入口：新增 `scripts/vercel/publish_site_release_wsl.ps1` 与 `.cmd`，在 WSL 中加载 `nvm`、选择 Node.js `>=22`、优先使用 `VERCEL_TOKEN`，并在无全局 `vercel` 时使用 `npx --yes vercel@54.5.1`；该入口用于复用 WSL 中可用的 token/网络环境，绕过 Windows 本机 Vercel OIDC/TLS 失败。
30. 参考成功部署脚本后继续收敛 Vercel 项目与域名问题：`scripts/vercel/.env.example` 默认项目名已从旧的 `skill` 改为 `storylock-gateway`；`publish_site_release.ps1` 与 WSL 发布入口已增加 `VERCEL_CUSTOM_DOMAIN`、`VERCEL_BIND_CUSTOM_DOMAIN`、域名 inspect、部署 URL 提取和可选 alias 绑定，避免“部署成功但 `yian.cdao.online` 仍指向旧项目/旧部署”的 404 假成功。

## 2. 实施总顺序

建议按以下顺序执行：

1. 文档口径与当前状态收敛。
2. 平台 SecretStore 守门补齐。
3. Android 宿主安全链补强。
4. macOS 平台适配补齐。
5. Android / Windows 发布闭环补齐。
6. 测试与验收体系固化。

---

## 3. 第一批：文档口径与当前状态收敛

### A1. 修订 Android 宿主状态说明

**涉及目录**

1. `docs/ref/05-Android宿主实现规范.md`
2. `docs/ref/06-评审讲解与演示说明.md`
3. `docs/ref/10-Android真机闭环检查.md`
4. `docs/design/cn/易安与StoryLock技术说明书.md`

**实施内容**

1. 明确 Android 真机宿主原型已存在，不再写成“纯 future”。
2. 明确当前签名实现仍是 demo 级，不可对外表述为生产级签名能力。
3. 明确 challenge 流程已存在本地输入确认，但尚未完全等价于第二层正式高强度模型。
4. 明确 APK 下载入口已具备，但不等于正式移动端发布体系已完成。

**输出物**

1. 修订后的 Android 相关文档。
2. 一份统一表述口径，供评审和对外说明复用。

**验收**

1. `npm run test:docs`
2. 人工交叉检查：文档不再与 `src/host/android-host/` 当前实现冲突。

### A2. 修订 Windows 宿主与发布口径

**涉及目录**

1. `docs/design/cn/易安与StoryLock技术说明书.md`
2. `docs/ref/07-APK分发与安装说明.md`
3. `docs/ref/08-易安部署与域名说明.md`
4. `src/host/windows-host/README.md`

**实施内容**

1. 统一把 Windows 宿主描述为 prototype / 本地宿主实现方向。
2. 区分“已具备下载入口”和“已具备正式签名发布闭环”。
3. 明确当前 release 元数据能力与正式发布能力之间的差异。

**输出物**

1. 修订后的 Windows 宿主和发布说明。

**验收**

1. `npm run test:docs`
2. 人工核对 `src/host/windows-host/src/main.rs` 与文档描述一致。

### A3. 收敛管理文档基线

**涉及目录**

1. `docs/management/`
2. `docs/management/back/`

**实施内容**

1. 以后续计划和实施清单为当前正式管理文档。
2. 将历史意见稿继续保留在 `back/`，但不再作为当前执行依据。
3. 如有旧稿仍在 `docs/management/` 主目录，统一迁回 `back/` 或删除。

**输出物**

1. 清晰的管理文档主目录。
2. `back/README.md` 如有必要，补充“历史归档”说明。

**验收**

1. 主目录只保留当前有效管理文档。
2. 历史稿不再和现行稿并列混放。

---

## 4. 第二批：平台 SecretStore 守门补齐

### B1. 统一 Host 启动时的存储可用性检查

**涉及代码**

1. `src/shared/secret-store.js`
2. `src/skills/local-story-access/access-host.js`
3. `src/skills/local-story-access/scripts/check-secret-store.mjs`

**实施内容**

1. 在显式使用平台 SecretStore 时，启动阶段统一调用 `checkAvailable()`。
2. 对 Windows 缺失 CredentialManager、Linux 缺失 `secret-tool` 的场景给出明确错误。
3. 禁止生产路径模糊回退到 MemorySecretStore。
4. 保持开发模式与生产模式的告警语义一致。

**输出物**

1. 统一的启动守门逻辑。
2. 对应自测或脚本检查补充。

**验收**

1. `src/skills/local-story-access/scripts/selftest.mjs` 增加对应覆盖。
2. `node scripts/test/run-selftests.mjs`

### B2. 收敛平台存储文档

**涉及文档**

1. `docs/design/cn/平台密钥存储适配指南.md`
2. `docs/design/cn/安全规范.md`

**实施内容**

1. 把当前真实支持平台写清楚。
2. 把 development mode 与 production mode 的差异写清楚。
3. 增加“启动失败即显式报错”的守门原则。

**输出物**

1. 修订后的平台存储文档。

**验收**

1. 文档与 `secret-store.js` 行为完全一致。

---

## 5. 第三批：Android 宿主安全链补强

### C1. 替换 demo HMAC 签名

**涉及代码**

1. `src/host/android-host/app/src/main/java/org/storylock/androidhost/host/StoryLockAndroidHostService.kt`
2. `src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AndroidKeystoreSecretStore.kt`
3. `src/host/android-host/README.md`

**实施内容**

1. 梳理当前签名数据结构和本地 key material 生成逻辑。
2. 设计并落地基于 Android Keystore 的正式签名执行路径。
3. 保留对第三层的最小必要结果，不返回原始秘密材料。
4. 更新 README 中“当前已实现 / 当前未实现”边界。

**输出物**

1. 替换后的 Android 本地签名实现。
2. 对应说明文档。

**验收**

1. 本地签名路径不再依赖 demo HMAC key material。
2. 远程网关脱敏后响应仍保持兼容。

### C2. 补齐 challenge 正式化流程

**涉及代码**

1. `src/host/android-host/app/src/main/java/org/storylock/androidhost/host/LocalAuthorizationRuntime.kt`
2. `src/host/android-host/app/src/main/java/org/storylock/androidhost/host/StoryLockAndroidHostService.kt`
3. `src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AttachedActivityConfirmation.kt`
4. `src/host/android-host/app/src/main/java/org/storylock/androidhost/security/ChallengePromptLocalUserConfirmation.kt`

**实施内容**

1. 梳理当前 challenge 生成、展示、输入、验证链路。
2. 补齐高强度 challenge 所需的多格验证流程。
3. 明确 challenge 失败、取消、超时、BiometricPrompt 失败时的返回结构。
4. 保证 Android 本地模型与第二层正式边界尽量一致。

**输出物**

1. 更新后的 Android 本地 challenge 流程。
2. 对应错误码和失败路径说明。

**验收**

1. challenge 不再依赖内部补答案完成高强度验证。
2. 本地确认失败路径可重复触发并有稳定返回。

### C3. 补 Android 宿主测试

**涉及目录**

1. `src/host/android-host/`
2. `src/skills/remote-gateway/scripts/selftest-web-api-android.mjs`
3. `scripts/android/`

**实施内容**

1. 增加 Android 宿主失败场景联调测试。
2. 补充或修订真机检查脚本与报告模板。
3. 把宿主原型验证、自测验证、真机验证区分开。

**输出物**

1. 更新后的自测脚本。
2. 更新后的真机检查说明。

**验收**

1. `npm run selftest:web-api-android`
2. 真机检查文档可按步骤复现。

---

## 6. 第四批：macOS 平台适配补齐

### D1. 实现 macOS Keychain SecretStore

**涉及代码**

1. `src/shared/secret-store.js`
2. `src/skills/local-story-access/scripts/check-secret-store.mjs`

**实施内容**

1. 增加 macOS Keychain 适配器。
2. 保持 `getSecret / setSecret / deleteSecret / checkAvailable` 语义一致。
3. 明确是否支持 `listKeys`，不支持时给出一致错误语义。

**输出物**

1. `macOS` SecretStore 实现。
2. 对应的可用性检查逻辑。

**验收**

1. `createPlatformSecretStore()` 在 macOS 下不再直接报“未配置适配器”。
2. 平台文档同步更新。

### D2. 平台统一性回归

**涉及代码与文档**

1. `src/shared/secret-store.js`
2. `docs/design/cn/平台密钥存储适配指南.md`

**实施内容**

1. 核对 Windows、Linux、macOS 三平台接口一致性。
2. 统一错误文案、development mode 文案、回退原则。
3. 收敛 Linux 平台说明材料、检查入口与发布目录口径。

**输出物**

1. 平台适配回归结果记录。

**验收**

1. 三平台说明一致。
2. 文档不再出现缺失实现项与代码冲突。

---

## 7. 第五批：Android / Windows 发布闭环补齐

### E1. Android 发布链整理

**涉及目录**

1. `scripts/release/android/`
2. `release/app/android/`
3. `docs/ref/07-APK分发与安装说明.md`
4. `docs/ref/10-Android真机闭环检查.md`

**实施内容**

1. 补齐 debug / release 的构建与命名规范。
2. 固化 checksum、versionCode、releaseChannel 的生成与写入流程。
3. 梳理站点展示值与发布脚本输出值的一致性。
4. 形成发布前检查清单。

**输出物**

1. 更新后的 Android 发布脚本。
2. 更新后的发布说明文档。
3. 发布前检查清单。

**验收**

1. 发布脚本输出与站点展示元数据一致。
2. `GET /app/download/android` 返回的元数据可人工核对。

### E2. Windows 发布链整理

**涉及目录**

1. `scripts/release/windows/`
2. `release/app/windows/`
3. `src/host/windows-host/README.md`

**实施内容**

1. 梳理 build / sign / publish / upload 的脚本关系。
2. 固化 prototype、candidate、release 的命名与元数据差异。
3. 对齐下载入口、产物名称、checksum 与发布说明。

**输出物**

1. 更新后的 Windows 发布流程说明。
2. 完整的 Windows 产物发布链条。

**验收**

1. Windows 下载入口、产物文件名、元数据一致。
2. `scripts/release/windows/` 脚本职责清晰，无口径冲突。

### E3. 网站与 Web API 元数据对齐

**涉及代码**

1. `src/skills/remote-gateway/web-api-handler.js`
2. `src/yian-web/public/`
3. `release/web/public/downloads/`

**实施内容**

1. 梳理 Android/Windows 下载入口的优先级。
2. 确保网站显示值和 API 返回值使用同一套元数据来源。
3. 区分 debug/internal、candidate、release 的展示口径。

**输出物**

1. 对齐后的下载展示与 API 响应。

**验收**

1. 网站页面、API、实际产物三者一致。

---

## 8. 第六批：测试与验收体系固化

### F1. 补主线负面测试

**涉及目录**

1. `src/skills/local-story-access/scripts/`
2. `src/skills/remote-gateway/scripts/`
3. `scripts/test/`

**实施内容**

1. 增加 challenge 失败、重放冲突、过期请求、脱敏失败等负面测试。
2. 增加平台存储不可用、shared secret 不匹配、relay 超时等边界测试。

**输出物**

1. 扩展后的主线 selftest。

**验收**

1. `node scripts/test/run-selftests.mjs`
2. 关键失败路径都有稳定覆盖。

### F2. 固化真机与平台验收矩阵

**涉及目录**

1. `docs/ref/`
2. `docs/test/`
3. `scripts/android/`
4. `scripts/windows/`

**实施内容**

1. 输出 Android 真机验收矩阵。
2. 输出 Windows 本地宿主验收矩阵。
3. 输出统一交付前检查项。

**输出物**

1. 平台验收矩阵文档。
2. 检查脚本或检查模板。

**验收**

1. 任一版本交付前都能按矩阵逐项勾选。

---

## 9. 任务优先级映射

### P0

1. A1 修订 Android 宿主状态说明。
2. A2 修订 Windows 宿主与发布口径。
3. B1 统一 Host 启动时的存储可用性检查。
4. C1 替换 demo HMAC 签名。

### P1

1. C2 补齐 challenge 正式化流程。
2. C3 补 Android 宿主测试。
3. D1 实现 macOS Keychain SecretStore。
4. E1/E2/E3 补齐发布元数据与下载闭环。

### P2

1. F1/F2 固化测试与验收体系。
2. 其余 UI、演示与扩展能力补强。

---

## 10. 每周推进建议

### 第 1 周

1. 完成 A1、A2、A3。
2. 启动 B1。

### 第 2 周

1. 完成 B1、B2。
2. 启动 C1。

### 第 3 周

1. 完成 C1。
2. 启动 C2、C3。

### 第 4 周

1. 完成 C2、C3。
2. 启动 D1。

### 第 5 周

1. 完成 D1、D2。
2. 启动 E1、E2、E3。

### 第 6 周

1. 完成 E1、E2、E3。
2. 启动 F1、F2。

---

## 11. 阶段完成定义

### 阶段完成，不等于“代码写完”

每阶段至少同时满足：

1. 代码变更完成。
2. 相关文档同步完成。
3. 自测或验收动作可重复执行。
4. 管理文档记录已更新。

### 整体完成定义

以下四项同时满足，才可以认为当前阶段进入“交付级主线”：

1. 文档口径与代码状态完全一致。
2. Android / Windows / SecretStore 的平台安全边界闭合。
3. Android / Windows 下载与发布闭环可重复执行。
4. 真机检查、主线 selftest、文档检查可以稳定通过。
