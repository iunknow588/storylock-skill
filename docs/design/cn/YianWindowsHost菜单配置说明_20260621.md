# Yian Windows Host 菜单配置说明

| 项目 | 内容 |
| --- | --- |
| 文档版本 | v0.1 |
| 日期 | 2026-06-21 |
| 适用对象 | Windows 本地 Host 原生界面 |
| 当前状态 | 本地优先原型，默认不连接远程 |

## 1. 定位

Yian Windows Host 是 StoryLock 本地 Host 的 Windows 实现方向。它属于平台宿主层，不改变现有三层 Skill 边界：

1. 第一层：本地故事处理，只处理本地文本和题集质量。
2. 第二层：本地授权，负责强度判断、九宫格验证、授权、撤销和本地执行。
3. 第三层：远程网关，只包装 `requestSignature`、`requestPasswordFill` 等请求，并做脱敏返回。

当前 Windows Host 默认运行在 `local_only` 模式：只启动本地界面、本地 HTTP 调试接口和本地数据目录，不注册远程网关，不轮询 relay。

本说明与以下设计文档保持一致：

1. `docs/design/cn/系统Skill表与能力边界.md`
   - Windows Host 属于平台实现方向，不新增第四层安全边界。
   - 当前对外主线能力仍只保留 `requestSignature` 和 `requestPasswordFill`。
2. `docs/design/cn/三包接口契约.md`
   - 第一层负责本地故事处理。
   - 第二层负责本地授权、九宫格验证、撤销和审计。
   - 第三层负责远程请求包装和脱敏返回。
3. `docs/design/cn/平台密钥存储适配指南.md`
   - 长期敏感材料必须进入平台安全存储。
   - Windows 当前方向为系统受保护存储 / Credential Manager / DPAPI 类能力。
4. `docs/design/cn/代理签名机制协议参考.md`
   - 第三方 Skill 只提交结构化签名意图。
   - 本地 Host 在授权后执行签名。
   - 远程 Agent 不持有私钥、不返回长期敏感材料。

## 2. 总体配置原则

1. 远程能力必须显式启用。
2. 不建议近期开放远程故事读写。
3. 不允许把题库答案、私钥、密码、`signingKeyBytes` 或故事原文暴露给远程 API。
4. 长期敏感材料应进入平台安全存储。Windows 当前方向是 DPAPI / Credential Manager。
5. 运行状态、诊断信息和审计摘要可以展示，但必须脱敏。

## 3. 菜单总览

当前界面左侧菜单包括：

1. `Status`
2. `Local Core`
3. `Data`
4. `StoryLock Core`
5. `Diagnostics`

右侧顶部显示：

```text
Yian: StoryLock - 当前菜单名
```

右侧下方为配置表单区域。可修改项使用输入框风格展示；当前阶段部分字段仍是界面层展示，后续再接入写回配置文件或平台配置存储。

当前菜单的配置状态分为三类：

| 类型 | 含义 | 当前处理方式 |
| --- | --- | --- |
| 启动配置 | 启动前确定的端口、身份、数据目录、远程开关 | 通过环境变量配置 |
| 本地运行状态 | 程序启动后生成或读取的设备标识、能力状态、诊断摘要 | UI 展示，部分只读 |
| 后续可写配置 | 未来可在界面修改并保存的选项 | 当前先以输入框风格预留，不写回敏感材料 |

## 4. Status

### 含义

`Status` 展示当前 Host 的运行身份、本地设备标识、本地 API 和运行模式。

### 字段

| 字段 | 当前含义 | 配置方式 | 边界 |
| --- | --- | --- | --- |
| `Identity` | 当前本地身份标识 | 环境变量 `STORYLOCK_IDENTITY_ID` | 可展示，不包含私钥 |
| `Device` | 当前设备实例标识 | 环境变量 `STORYLOCK_DEVICE_ID`，未设置时自动生成 | 可展示，不作为长期密钥 |
| `Local API` | 本地 Host API 地址 | 环境变量 `STORYLOCK_WINDOWS_HOST_PORT` 控制端口 | 仅绑定 `127.0.0.1` |
| `Mode` | 当前运行模式 | 默认 `local_only` | 远程必须显式开启 |

### 推荐配置

本地验收阶段建议不设置远程相关变量，只确认界面和本地接口可用：

```powershell
$env:STORYLOCK_WINDOWS_HOST_PORT="4510"
.\yian-windows-host.exe
```

如需固定身份：

```powershell
$env:STORYLOCK_IDENTITY_ID="windows-demo-001"
$env:STORYLOCK_DEVICE_ID="windows-test-device-001"
```

## 5. Local Core

### 含义

`Local Core` 对应第二层本地授权与执行边界，展示本地能力、调用链和远程访问状态。

### 字段

| 字段 | 当前含义 | 配置方式 | 边界 |
| --- | --- | --- | --- |
| `Capabilities` | 本地可用能力 | 当前由程序根据模式生成 | 默认不包含 `relay_poll` |
| `Call Chain` | 本地执行链 | 固定为 `verify -> authorize -> execute -> revoke` | 不绕过本地授权 |
| `Boundary` | 本地安全边界 | 当前为 Windows DPAPI local only | 私钥不出本地 |
| `Remote Access` | 远程访问状态 | 默认 disabled | 远程只允许请求包装，不持有私钥 |

### 远程启用方式

默认不连接远程。仅当需要联调 relay 时显式启用：

```powershell
$env:STORYLOCK_WINDOWS_REMOTE_ENABLED="1"
$env:STORYLOCK_GATEWAY_URL="https://yian.cdao.online"
```

启用后仍需遵守：

1. 远程网关只发起结构化请求。
2. 本地 Host 负责确认、授权和执行。
3. 远程响应只返回脱敏结果或签名结果，不返回私钥、密码、题库答案。

## 6. Data

### 含义

`Data` 展示题集、数据目录和平台存储方式。

### 字段

| 字段 | 当前含义 | 配置方式 | 边界 |
| --- | --- | --- | --- |
| `Question Bank` | 当前本地题集版本和数量 | 启动时从内置题集初始化，也可后续导入 | UI 不展示答案 |
| `Data Directory` | 本地状态目录 | 环境变量 `STORYLOCK_WINDOWS_DATA_DIR` | 仅本地路径 |
| `Storage` | 平台安全存储方式 | 当前显示 Windows DPAPI | 长期敏感材料不写明文配置 |

### 推荐配置

本地测试可指定独立数据目录，避免污染正式数据：

```powershell
$env:STORYLOCK_WINDOWS_DATA_DIR="E:\2026OPC大赛\skill\.temp\runtime\windows-host-data"
```

当前本地数据包括：

1. 题集副本。
2. 验证记录。
3. 授权记录。
4. DPAPI 保护后的签名或凭据演示对象。

不应进入普通明文配置的内容：

1. 私钥原文。
2. 密码原文。
3. 题库答案明文。
4. 可重放 challenge material。

## 7. StoryLock Core

### 含义

`StoryLock Core` 是启动底层 StoryLock 应用的入口。Windows Host 主窗口只作为外部宿主和状态窗口，不在当前页面直接编辑故事或加密配置。

点击 `Open StoryLock Core` 后，会打开独立的原生底层应用窗口。故事编辑、题集策略、加密配置和受保护对象配置必须在这个底层应用中修改。

外部 Host 可以读取底层 StoryLock Core 返回的脱敏权限元数据，例如：

1. 受管理对象标识。
2. 请求动作类型。
3. 所需强度等级。
4. 对应九宫格数量。

外部 Host 不能读取或修改：

1. 故事原文。
2. 题库答案。
3. 私钥、密码、seed 或 `signingKeyBytes`。
4. 加密配置中的秘密材料。
5. 具体受保护对象的底层配置项。

### 字段

| 字段 | 当前含义 | 配置方式 | 边界 |
| --- | --- | --- | --- |
| `Application` | 打开底层 StoryLock Core 应用 | 点击 `Open StoryLock Core` | 新窗口运行，Host 不直接编辑配置 |
| `Managed object permission metadata` | 受管理对象的权限摘要 | 由底层 Core 派生后展示 | 只展示对象、动作、强度、九宫格数量 |

### 启动开关语义

`Open StoryLock Core` 表示打开底层本地应用，不代表启动远程服务，也不代表允许外部 Host 读取受保护故事对象。

底层应用当前包含五个菜单，字段口径参考 `E:\2026OPC大赛\story-lock\doc\usecase` 下的故事模板、资源目录样例和模板文件样例：

1. `Story`
   - 维护故事标题、摘要、记忆锚点和 8 个要素分组。
   - 默认要素为：时间、地点、人物、外部、起步、核心、过程、收束。
2. `24 Nodes`
   - 维护固定 24 节点作者稿。
   - 字段包括 `nodeId`、`title`、`elementId`、问题、标准答案、可接受答案、正确项、干扰项、选择模式、候选池大小、记忆优先级、校验策略和作者备注。
3. `Resources`
   - 维护 `resource-catalog.json` 口径的资源目录。
   - 字段包括 `resourceId`、`resourceKind`、`providerId`、`displayName`、`bindings` 和 `objectMeta`。
4. `Templates`
   - 维护 `templates/login-sites.json`、`templates/signing-actions.json`、`templates/agent-tasks.json` 口径的模板绑定。
   - 模板只描述动作，引用 `resourceId + role`，不保存秘密值。
5. `Export`
   - 预览导出包结构：`vault.stlk`、`resource-catalog.json`、`package-manifest.json` 和 `templates/*`。

底层故事处理输出会显式保留第一层边界：

```text
challengeCreated=false
sessionIssued=false
protectedObjectRead=false
remoteRetentionGranted=false
```

这表示第一层不会创建 challenge、不会签发 session、不会直接读取受保护对象，也不会给远程保留私有故事内容。

### 外部读取权限信息

外部 Host 只能读取如下示例形式的权限信息：

```text
wallet-key-main: action=signature, strength=high, gridCells=9
mailbox-primary: action=password_fill, strength=medium, gridCells=6
story-vault-local: action=story_edit, strength=local_core_only, gridCells=0
```

该信息用于判断远程请求或外部应用需要进入哪种本地授权流程，不能反推出具体故事、答案、密钥或配置秘密。

## 8. Diagnostics

### 含义

`Diagnostics` 用只读日志框展示脱敏诊断信息，类似本地 log view。

### 展示原则

诊断信息可以展示：

1. Host 是否运行。
2. 当前模式是否为 `local_only`。
3. 本地数据目录。
4. 题集路径。
5. 最近执行摘要。
6. 脱敏边界说明。

诊断信息不得展示：

1. 题库答案。
2. 明文密码。
3. 私钥或 seed。
4. `signingKeyBytes`。
5. 故事原文。
6. 远程共享密钥。

## 9. 当前环境变量清单

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `STORYLOCK_WINDOWS_HOST_PORT` | `4510` | 本地 API 端口 |
| `STORYLOCK_WINDOWS_DATA_DIR` | `%LOCALAPPDATA%\Yian\windows-host` 下的默认目录 | 本地状态目录 |
| `STORYLOCK_IDENTITY_ID` | `windows-demo-001` | 本地身份标识 |
| `STORYLOCK_DEVICE_ID` | 自动生成 | 设备标识 |
| `STORYLOCK_APP_INSTANCE_ID` | 自动生成 | 应用实例标识 |
| `STORYLOCK_WINDOWS_APPROVAL_MODE` | `windows_dialog` | 本地确认方式 |
| `STORYLOCK_WINDOWS_REMOTE_ENABLED` | `false` | 是否启用远程注册和 relay |
| `STORYLOCK_GATEWAY_URL` | `https://yian.cdao.online` | 远程启用后使用的网关地址 |

## 10. 本地启动建议

当前阶段先只验证本地窗口：

```powershell
.\yian-windows-host.exe
```

调试控制台模式：

```powershell
.\start-yian-windows-host.cmd
```

自动化本地检查：

```powershell
npm run test:windows-host
```

## 11. 后续完善方向

1. 将输入框中的可修改项接入配置写回。
2. 区分“只读静态字段”和“可保存配置字段”。
3. 增加保存、重置、验证配置按钮。
4. 给远程开关增加二次确认。
5. 将 Linux / Android Host 的菜单语义与 Windows 保持一致。
6. 继续保持远程故事读写关闭，敏感材料仅本地处理。
7. 将 `StoryLock Core` 从当前独立原生原型升级为可持久化作者稿、资源目录、模板文件和导出包的正式底层应用，并继续保持 Host 只读脱敏权限摘要。
