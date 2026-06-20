# 三层术语与 PHAROS 定位

日期：2026-06-20

本文档统一易安、第三方 Agent、私人智能助理、StoryLock 本地核心与 PHAROS 的对外表述，避免评审或对外沟通中出现职责混淆。

## 1. 推荐总体链路

推荐对外表述为：

```text
第三方 Agent / PHAROS Agent / OpenClaw
  -> 通过 Skill 调用
云端平台上的易安远程入口
  <-> 与用户本地设备双向通信
私人智能助理
  <-> 受控本地调用
StoryLock 本地核心（离线敏感执行边界）
```

这条链路强调：

1. 远程编排在云端。
2. 设备确认在本地。
3. 敏感执行在离线本地核心。

## 2. 第三方 Agent / PHAROS Agent

正式术语：

`第三方 Agent / PHAROS Agent / OpenClaw Agent`

职责：

1. 发起任务和流程编排。
2. 通过 Skill 访问易安远程入口。
3. 等待用户本地确认结果后继续后续流程。

边界：

1. 不直接访问 StoryLock 本地核心。
2. 不直接读取长期密钥、密码或 challenge 答案。
3. 不绕过用户设备上的本地确认。

## 3. 易安远程入口

正式术语：

`云端平台上的易安远程入口`

职责：

1. 承载网站、下载入口、绑定入口和状态接口。
2. 作为第三方 Agent 访问用户本地能力的受控网关。
3. 管理 binding token、设备注册、relay 轮询和最小状态返回。
4. 与用户本地设备上的私人智能助理双向通信。

边界：

1. 不保存长期密钥、密码、明文故事内容或九宫格答案。
2. 不替代用户完成本地确认。
3. 不直接访问 StoryLock 本地核心。

## 4. 私人智能助理

正式术语：

`私人智能助理` / `Private Assistant`

职责：

1. 运行在用户可控制的本地设备上。
2. 与易安远程入口双向通信。
3. 向用户解释请求来源、内容和风险。
4. 调用设备解锁、生物识别或设备凭据等本地确认能力。
5. 通过受控本地接口调用 StoryLock 本地核心。

边界：

1. 不替代 StoryLock 本地核心的授权策略。
2. 不应把受保护的本地秘密暴露给云端。
3. AI 辅助只用于解释、模板、草稿和交互引导。

## 5. StoryLock 本地核心

正式术语：

`StoryLock 本地核心` / `StoryLock Local Core`

职责：

1. 作为离线敏感执行边界。
2. 保存故事存储、授权策略、密钥引用和审计记录。
3. 执行 challenge、session、防重放和本地确认。
4. 只返回最小必要结果给私人智能助理。

边界：

1. 不暴露远程网络接口。
2. 不直接接受第三方 Agent 或云端平台调用。
3. 不把长期秘密、密码或 challenge 答案交给远程入口。

## 6. PHAROS 的定位

当前推荐定位：

`可选锚定层 / 可信协作层 / Agent 平台侧能力`

含义：

1. PHAROS 可以与第三方 Agent 运行在同一云端或协作平台中。
2. 它可以增强协作记录、可信证明或后续生态连接。
3. 它不是本地敏感执行层。
4. 它不替代私人智能助理。
5. 它不直接访问 StoryLock 本地核心。

推荐英文口径：

> PHAROS or other third-party agents call Yian Remote Entry through Skills. Yian Remote Entry communicates bidirectionally with the user's Private Assistant, while the offline StoryLock Local Core only returns minimal local results through that assistant.

## 7. 避免的表述

不要这样讲：

1. “PHAROS 直接执行本地授权。”
2. “易安网站保存用户长期密钥或密码。”
3. “云端平台可以直接读取 StoryLock 故事存储。”
4. “私人智能助理可以绕过 StoryLock 本地核心。”
5. “本地 App 就是远程服务节点。”
6. “易安远程入口可以直接访问 StoryLock 本地核心。”

## 8. 工程命名说明

当前代码中仍可能保留：

1. `android-host`
2. `Android Host`
3. `host registry`

这些多为工程目录名、兼容字段名或 mock 术语。对外页面、讲解材料和评审口径应优先使用：

1. `易安 App` 或 `易安远程入口`
2. `私人智能助理`
3. `StoryLock 本地核心`
4. `本地设备确认`
