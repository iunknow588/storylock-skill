# 易安与 StoryLock 完善清单

日期：2026-06-18

> 归档说明：该文件原始内容因历史编码转换出现乱码。本文按原章节结构和可识别上下文重写为可读归档，保留已完成项、待完成项和关键进展。

## 1. 第三层网站：易安

- [x] 完整检查中英文切换覆盖范围：页面标题、导航、按钮、运行状态、提示、错误提示、页脚文案均保持双语一致。
- [x] 为网站补充更清晰的项目简介区块：第三方 Agent / pharos Agent 通过 Skill 访问部署在第三方云服务平台上的易安远程入口；易安远程入口与用户本地设备上的私人智能助理双向通信；私人智能助理可联网并可用 AI 辅助用户生成故事模板和理解请求，但不接触故事存储服务；最终敏感存储与授权确认由无网络的 StoryLock 本地核心完成。
- [x] 增加“下载后如何绑定”的三步说明：下载 App、打开绑定链接、完成首次注册与回连。
- [x] 补充线上可用的项目介绍内容：面向普通最终用户说明下载、安装、绑定、确认请求、查看连接状态和处理常见注意事项。
- [x] 检查首页在桌面与移动端的显示效果：标题换行、按钮宽度、结构图显示、状态面板不溢出。

## 2. 第三层网关

- [x] 完善 `GET /api/storylock-gateway` 的线上状态输出，明确网站入口、下载入口、绑定入口、当前连接模式和活跃宿主数量。
- [x] 继续补强宿主注册机制：处理重复注册、设备更新、宿主失效和过期清理。
- [x] 补充共享密钥与绑定 token 的轮换策略。
- [x] 为 relay 轮询链路增加更明确的超时、重试、失败记录与恢复策略。
- [x] 明确多宿主场景下的路由策略：identity 选择、最近在线优先和 preferred mode 优先。

## 3. 私人智能助理与 StoryLock 本地核心

- [ ] 完成真实 Android 工程编译验证。
- [ ] 产出真实 APK 构建文件，并接入第三层下载入口。
- [x] 完善首次安装后的绑定引导界面：未绑定、已绑定、注册中、在线、离线等状态可见。
- [x] 将 `execute` 从 placeholder 结果接入真实授权链路。
- [x] 将真实 Keystore 对象接入签名流程。
- [x] 将真实 credential object 接入密码填充流程。
- [x] 继续对齐 Android `health` 输出与现有 schema。
- [ ] 增加真机网络可达性验证：本地 HTTP、relay 轮询、deep link 唤起、回连状态。

进展：当前 `android-host/` 是内部工程名，对外产品口径应表达为“私人智能助理 + StoryLock 本地核心”。已有 `scripts/release/android/build_apk.cmd` 与真机闭环检查文档；当前机器未安装 `gradle`，且 `android-host/` 尚未提交 Gradle wrapper，因此真实 APK 编译仍需安装 Gradle 或补充 wrapper 后执行。

进展：`execute` 已通过本地 challenge / BiometricPrompt 确认后读取 Android Keystore 加密保存的签名 key 与 credential 对象。当前签名算法仍是 HMAC 演示实现，后续生产化应替换为 Android Keystore 非对称签名与 Credential Manager。

边界：私人智能助理可以联网、与易安远程入口双向通信、辅助解释请求、生成故事模板草稿；StoryLock 本地核心无网络，负责故事存储、密钥隔离、本地确认和敏感执行。远程入口没有通道直接访问 StoryLock 本地核心，智能助理不得直接读取故事存储服务或绕过本地核心。

## 4. APK 分发与安装

- [x] 明确 APK 文件产出路径、命名规则与版本号规则。
- [x] 在网站展示当前 APK 版本信息。
- [x] 增加 APK 摘要信息：文件大小、版本号、可选校验值。
- [x] 增加安装说明：下载、安装、绑定、注册、查看状态。
- [x] 明确 release 包与 debug 包的对外策略。

## 5. 部署与域名

- [ ] 将 `yian.cdao.online` 正式绑定到当前 Vercel 项目。
- [ ] 配置线上环境变量：
  - `STORYLOCK_GATEWAY_PUBLIC_URL`
  - `STORYLOCK_ANDROID_CONNECT_MODE`
  - `STORYLOCK_ANDROID_APK_PATH` 或下载 URL
  - `STORYLOCK_ANDROID_DEEP_LINK`
  - `STORYLOCK_ANDROID_REGISTRY_FILE`
- [x] 核对网站首页、网关接口、下载接口、绑定接口的同域访问行为。
- [x] 确认 Vercel 路由与本地路由一致。
- [x] 补充一份可执行部署步骤说明。

进展：已有 `scripts/vercel/preflight.cmd`，可检查域名、线上环境变量、APK 来源、首页、静态资源、网关、绑定和下载入口。实际域名绑定与 Vercel 环境变量写入仍需在 Vercel/DNS 控制台完成。

## 6. 文档同步

- [x] 将易安网站对外文案与 `docs/design/cn` 保持一致。
- [x] 将第三层网站定位同步到 `docs/ref` 与相关 README。
- [x] 补充“网站层、网关层、本地设备承载层”的正式术语说明。
- [x] 明确 pharos 在当前方案中的定位：pharos / OpenClaw / 第三方 Agent 可运行在 Agent 平台或云环境中，通过 Skill 访问第三方云服务平台上的易安远程入口；pharos 可作为可选锚定层 / 可信协作层，而不是本地主执行层，也不直接访问 StoryLock 本地核心。

## 7. 验证与测试

- [x] 继续保留并扩展 `selftest:web-api-android`。
- [x] 增加网站首页最小自检：首页可打开、双语切换可用、状态接口可读。
- [x] 增加 APK 下载自检：能正确返回文件或重定向。
- [x] 增加宿主注册与 relay 回连端到端验证。
- [ ] 在真机环境下补一次完整闭环检查：下载 -> 安装 -> 绑定 -> 注册 -> 回连 -> 本地授权 -> 返回结果。

进展：已有 `scripts/android/check_device_loop.cmd`，可记录 adb、APK 安装、deep link 唤起和易安端点检查；当前未连接真机且未产出真实 APK，因此完整闭环仍待真机执行。

## 8. 当前建议优先顺序

1. 完成易安网站双语与内容细化。
2. 产出真实 APK，并接入下载入口。
3. 完成 Android 真机编译与首次绑定验证。
4. 将 `execute` 接入真实 Keystore 与 credential 对象。
5. 完成 Vercel 域名与线上环境变量配置。
