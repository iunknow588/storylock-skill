# Android 真机闭环检查

日期：2026-06-20

本文档用于执行当前仓库里的 Android 真机联调闭环检查：

`下载 -> 安装 -> 绑定 -> 注册 -> relay -> 本地 challenge -> 本地确认 -> 返回脱敏结果`

需要明确：

1. 当前检查目标是验证 Android 宿主原型链路是否打通。
2. 这不是正式 release 发布验收替代物。
3. 如果使用的是 `debug`、`internal` 或原型包，必须在记录中明确标注。

## 1. 前置条件

1. 本机可访问 Yian 网关。
2. Android 设备已连接，`adb devices` 可见目标设备。
3. 具备 Android SDK、JDK 17，以及 `gradle` 或本地 Gradle wrapper。
4. 若使用 relay 模式，设备可访问 `STORYLOCK_GATEWAY_PUBLIC_URL`。

## 2. 构建与产物确认

在 `skill/` 目录执行：

```powershell
scripts\release\android\build_apk.cmd -Variant debug
```

至少记录：

1. APK 路径
2. `versionName`
3. `versionCode`
4. SHA-256
5. `packageKind`
6. `releaseChannel`

## 3. 下载入口检查

至少检查以下入口：

1. `/app/download`
2. `/app/download/android`
3. `/android-host/bind`

期望：

1. 平台下载入口可访问。
2. Android 下载入口返回 APK 或明确跳转。
3. 绑定入口可返回带 `binding.deepLink` 的结果。

## 4. 安装与绑定

```powershell
adb install -r release\app\android\storylock-android-host-0.1.0-1-debug.apk
```

打开绑定入口：

```text
/app/bind?identityId=android-demo-001&preferredMode=relay_url
```

用浏览器或 adb 打开返回的 deep link：

```powershell
adb shell am start -a android.intent.action.VIEW -d "storylock-host://bind?gateway_url=...&binding_token=...&identity_id=android-demo-001&preferred_mode=relay_url"
```

## 5. 运行态状态检查

Android 宿主侧建议观察：

1. 是否已绑定
2. 是否已注册
3. relay 是否在线
4. 最近错误是否为空

网关侧建议观察：

1. active host 数量是否大于 0
2. 注册 ID 是否生成
3. relay poll / respond 是否持续成功

## 6. 本地 challenge 与确认检查

针对 `requestSignature`：

1. 是否弹出高强度多格 challenge
2. 是否要求强生物确认
3. challenge 连续失败后是否出现临时锁定

针对 `requestPasswordFill`：

1. 是否弹出中强度 challenge
2. 是否完成本地确认
3. 返回结果是否被第三层脱敏

## 7. 失败场景检查

至少建议覆盖以下失败路径：

1. deep link 未唤起
2. 宿主未注册
3. shared secret 不匹配
4. challenge 主动取消
5. challenge 输入错误
6. challenge 连续错误触发锁定
7. BiometricPrompt 不可用
8. BiometricPrompt 取消

当前期望错误类型至少包括：

1. `challenge_cancelled`
2. `challenge_failed`
3. `challenge_locked`
4. `biometric_unavailable`
5. `biometric_cancelled`
6. `biometric_failed`

## 8. 脱敏结果检查

针对网关最终返回结果，至少确认：

1. `executionLocation=remote_gateway`
2. `privateKey` 被替换为 `[redacted]`
3. `signingKeyBytes` 被替换为 `[redacted]`
4. `password` 被替换为 `[redacted]`

## 9. 推荐本地检查脚本

```powershell
scripts\android\check_device_loop.ps1 `
  -ApkPath release\app\android\storylock-android-host-0.1.0-1-debug.apk `
  -GatewayBaseUrl https://yian.cdao.online `
  -IdentityId android-demo-001 `
  -PackageKind debug `
  -ReleaseChannel internal `
  -DeepLink "storylock-host://bind?gateway_url=...&binding_token=...&identity_id=android-demo-001&preferred_mode=relay_url"
```

脚本当前会检查：

1. `adb devices`
2. APK 是否存在
3. `adb install`
4. deep link 唤起
5. `/`
6. `/api/storylock-gateway`
7. `/app/download`
8. `/app/download/android`
9. `/android-host/bind`
10. 带 `identityId` 的绑定请求

默认输出：

`%TEMP%/android-device-loop-report.local.md`

## 10. 检查记录模板

每次真机检查建议至少记录：

1. 设备型号
2. Android SDK 版本
3. APK 名称
4. `packageKind`
5. `releaseChannel`
6. deep link 是否成功
7. relay 是否成功
8. `requestSignature` 是否成功
9. `requestPasswordFill` 是否成功
10. 失败路径是否按预期返回

## 11. 当前阶段结论口径

如果本检查通过，推荐表述为：

> 当前 Android 宿主原型已经可以完成下载、安装、绑定、注册、relay、本地 challenge、本地确认和脱敏返回的真机联调闭环。

不要表述为：

1. Android 正式发布闭环已经完成
2. Android 正式 App 已经上架
3. Android 签名和 challenge 已达到完整生产交付级
