# 易安、私人智能助理与 pharos 定位

日期：2026-06-18

本文档统一易安、第三方 Agent、三方云服务平台、私人智能助理、StoryLock 本地核心与 pharos 的对外表述。

## 1. 推荐总体链路

推荐对外表达为：

```text
第三方 Agent / pharos Agent / OpenClaw
  -> 通过 Skill 调用
三方云服务平台上的易安远程入口
  <-> 与用户本地设备双向通信
私人智能助理
  <-> 本地受控调用
StoryLock 本地核心（无网络服务）
```

这条链路把远程编排、云端入口、本地设备侧智能交互和本地敏感执行分开，避免把本地 App 误解成远程服务的一部分。

## 2. 第三方 Agent 与 pharos Agent

正式术语：

`第三方 Agent / pharos Agent / OpenClaw Agent`

职责：

1. 发起任务、编排流程、调用 Skill。
2. 通过易安远程入口请求用户本地确认。
3. 等待易安远程入口返回的本地设备侧确认状态后继续后续流程。

边界：

1. 不直接访问 StoryLock 本地核心。
2. 不读取故事存储、长期密钥、密码或九宫格答案。
3. 不绕过用户本地设备上的本地确认。

## 3. 三方云服务平台与易安远程入口

正式术语：

`三方云服务平台上的易安远程入口` / `Yian Remote Entry on Cloud Platform`

示例平台：

`AWS`、`华为云`、`Vercel`、企业自有云或其他云平台。

职责：

1. 承载易安网站、下载入口、绑定入口和状态接口。
2. 作为第三方 Agent 通过 Skill 访问用户本地能力的受控入口。
3. 管理绑定 token、设备登记、relay 轮询和脱敏返回。
4. 与用户本地设备上的私人智能助理进行双向通信。

边界：

1. 不保存长期密钥、密码、明文故事内容或九宫格答案。
2. 不替用户完成本地确认。
3. 没有通道直接访问 StoryLock 本地核心。
4. 不直接读写 StoryLock 本地核心的故事存储。

## 4. 私人智能助理

正式术语：

`私人智能助理` / `Private Assistant`

职责：

1. 运行在用户自己的本地设备上，可以具备网络访问能力；手机只是其中一种形态。
2. 与易安远程入口进行双向通信，接收请求并返回确认状态。
3. 向用户解释请求来源、请求内容和风险。
4. 调用本地设备的系统确认能力，例如解锁、生物识别或设备凭据。
5. 可以利用 AI 协助用户生成故事模板、整理提示词、解释确认问题。
6. 通过受控本地接口调用 StoryLock 本地核心，并接收 StoryLock 本地核心返回的最小结果。

边界：

1. 不直接接触故事存储服务。
2. 不持有长期密钥或直接读取秘密对象。
3. 不绕过 StoryLock 本地核心的授权策略。
4. AI 辅助功能只能处理模板、草稿、解释和交互引导，不读取已受保护的故事存储。

## 5. StoryLock 本地核心

正式术语：

`StoryLock 本地核心` / `StoryLock Local Core`

职责：

1. 作为无网络服务的本地敏感执行边界。
2. 保存故事存储、授权策略、密钥引用和审计记录。
3. 执行 challenge、会话授权、失败窗口和本地确认。
4. 在确认通过后生成必要结果，并只把最小结果返回给私人智能助理。

边界：

1. 不暴露远程网络接口。
2. 不直接接受第三方 Agent 或云平台调用。
3. 不与易安远程入口直接通信。
4. 不把故事存储、长期密钥、密码或九宫格答案交给远程入口。

## 6. pharos 定位

pharos 在当前方案中的定位是：

`可选锚定层 / 可信协作层 / Agent 平台侧能力`

它可以与第三方 Agent 运行在同一云端或 Agent 平台环境中，用于增强外部可信度、协作记录、链上锚定证明或后续生态连接。

pharos 不是本地主执行层，不替代私人智能助理，也不直接访问 StoryLock 本地核心。

推荐表述：

`pharos or other third-party agents call Yian Remote Entry through Skills. Yian Remote Entry communicates bidirectionally with the user's Private Assistant, while the offline StoryLock Local Core only exchanges minimal local results with that assistant.`

避免表述：

1. “pharos 直接执行本地授权。”
2. “易安网站保存用户长期密钥或密码。”
3. “三方云服务平台可以直接读取 StoryLock 故事存储。”
4. “私人智能助理可以绕过 StoryLock 本地核心。”
5. “本地 App 是远程服务节点。”
6. “易安远程入口可以直接访问 StoryLock 本地核心。”

## 7. 内部工程名说明

当前代码中仍可能保留 `android-host`、`Android Host` 等内部工程名，用于兼容路由、测试脚本和 Android 工程目录。

对外页面、产品说明和普通用户文案应使用：

1. `易安 App`
2. `私人智能助理`
3. `StoryLock 本地核心`
4. `本地设备确认`

不要在普通用户页面直接使用 `Android Host`。
