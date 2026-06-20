# StoryLock 项目说明

版本：2026-06-20  
适用目录：`skill/`

## 1. 项目定位

StoryLock 是一个本地优先的授权访问 Skill 项目。它把故事处理、对象强度策略、九宫格验证、短时本地授权和远程请求包装拆成三层能力，使远程 Agent 可以请求本地完成签名或 Web2 密码填充，但不直接持有长期秘密。

一句话概括：

> StoryLock 通过三层 Skill 结构，把远程请求与本地挑战、授权和敏感执行连接起来，让长期秘密始终停留在本地边界内。

## 2. 当前三层结构

| 层级 | 代码目录 | 当前能力 |
| --- | --- | --- |
| 第一层：故事处理与强度分析 | `src/skills/local-story-processing` | `StoryDraftSkill`、`StoryRefineSkill`、`StrengthReviewSkill` |
| 第二层：本地访问授权 | `src/skills/local-story-access` | `ObjectStrengthPolicySkill`、`GridChallengeSkill`、`LocalAuthorizationSkill`、`LocalRevocationSkill` |
| 第三层：远程网关 | `src/skills/remote-gateway` | `requestSignature`、`requestPasswordFill` |
| 兼容演示包 | `src/engine` | 兼容与演示用入口 |

## 3. 当前关键状态

已完成或已稳定：

1. 三层主线目录和能力边界。
2. 自测闭环和端到端签名演示。
3. Web API 网关与 Android 宿主 mock 联通验证。
4. 题集模板、校验、导入与清理流程。
5. SecretStore 生产约束和 EIP-712 环境口径收紧。
6. 文本格式、文档一致性和路径一致性检查。
7. Android 真机宿主原型工程已经存在，包含本地 HTTP 宿主、Keystore 存储、challenge 输入、BiometricPrompt 与注册 / relay 原型代码路径。
8. Windows 宿主原型工程已经存在，具备本地执行与 relay 原型链路。
9. Linux 平台 SecretStore 适配、启动期可用性检查脚本、最小本地宿主原型、WSL 打包入口、`.deb` / `tar.gz` 原型包与桌面集成预留材料已经存在。
10. macOS 平台 Keychain SecretStore 适配与启动期可用性检查路径已经存在。

仍属于后续增强：

1. Android 宿主从当前原型工程推进到正式交付级 App。
2. Android 本地签名从当前 demo 实现推进到生产级 Keystore 签名闭环。
3. Android 高强度 challenge、BiometricPrompt 与本地确认 UI 进一步对齐第二层正式模型。
4. 更完整的 Android / Windows / Linux 多平台宿主正式发布链路。
5. Linux 真实桌面安装、Secret Service 真环境验收、正式签名包与发布闭环。
6. macOS 独立宿主交付材料与平台验收闭环。

## 4. 常用验证命令

```powershell
npm run test
```

```powershell
node scripts/test/run-selftests.mjs
```

```powershell
npm run selftest:web-api-android
```

```powershell
node scripts/android/validate_android_question_set.mjs
```

## 5. 文档入口

| 文档 | 路径 |
| --- | --- |
| 工作区根目录入口 | `README.md` |
| 中文设计入口 | `docs/design/cn/README.md` |
| 参赛与评审资料 | `docs/ref/README.md` |
| 测试方案 | `docs/test/StoryLock测试方案_v1.0.md` |
| Android 宿主说明 | `docs/ref/05-Android宿主实现规范.md` |
| 评审讲解与演示说明 | `docs/ref/06-评审讲解与演示说明.md` |
| Linux 平台密钥存储与检查说明 | `docs/ref/11-Linux平台密钥存储与检查说明.md` |
| 后续开发计划 | `docs/management/StoryLock后续开发计划_20260620.md` |
| 后续开发实施清单 | `docs/management/StoryLock后续开发实施清单_20260620.md` |

## 6. 对外表述建议

建议使用：

> StoryLock 是一个本地优先的授权访问 Skill 项目。它通过故事处理、本地访问授权、远程网关三层结构，把挑战验证、短时授权、签名请求和 Web2 密码填充包装为可调用能力，同时把长期秘密和授权判断保留在本地边界内。

补充说明时建议明确：

1. 当前仓库已经有 Android 与 Windows 宿主原型工程。
2. 当前仓库已经有 Linux 平台 SecretStore 适配、最小宿主原型、WSL 打包与 `.deb` / `tar.gz` 原型包，但还没有完成正式 Linux 签名发布与真实桌面验收。
3. 当前仓库已经有 macOS 平台 Keychain SecretStore 适配，但还没有单独的 macOS 宿主交付材料。
4. 当前主线已经验证三层闭环和平台宿主原型联调。
5. 当前仍未完成正式移动端发布与生产级本地签名交付闭环。

避免使用：

1. “已经完成完整多链多钱包生产系统”
2. “远程网关可以直接处理本地明文敏感内容”
3. “兼容演示包就是正式生产安全边界”
4. “Android 宿主已经是正式上架 App”
5. “Linux 平台已经完成正式宿主交付”
6. “macOS 平台已经完成正式宿主交付”
