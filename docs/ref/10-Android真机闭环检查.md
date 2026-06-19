# Android 真机闭环检查

日期：2026-06-18

本文档用于执行清单中的真机闭环：

`下载 -> 安装 -> 绑定 -> 注册 -> 回连 -> 本地授权 -> 返回结果`

## 1. 前置条件

1. 本机安装 Android SDK、JDK 17 与 Gradle，或在 `src/host/android-host/` 下加入 Gradle wrapper。
2. 易安本地或线上网关可访问。
3. Android 设备已连接，`adb devices` 能看到目标设备。
4. 若使用 relay 模式，设备能访问 `STORYLOCK_GATEWAY_PUBLIC_URL`。

当前仓库未提交 Gradle wrapper；如果系统没有 `gradle` 命令，需要先安装 Gradle 或在 Android Studio 中为 `android-host` 生成 wrapper。

## 2. 构建 APK

从 `skill/` 目录执行：

```powershell
scripts\release\android\build_apk.cmd -Variant debug
```

脚本会：

1. 运行 `assembleDebug` 或 `assembleRelease`。
2. 查找 `src/host/android-host/app/build/outputs/apk/{variant}/*.apk`。
3. 计算 SHA-256。
4. 复制 APK 到 `release/app/android/`，作为后续安装、上传服务器和站点构建的统一来源。
5. 写出 `scripts/vercel/.env.android-apk`，供易安下载入口读取。

## 3. 接入易安下载入口

将脚本输出的环境变量同步到本地 `.env` 或 Vercel 项目环境变量：

`STORYLOCK_ANDROID_APK_PATH`

`STORYLOCK_ANDROID_APK_VERSION`

`STORYLOCK_ANDROID_APK_VERSION_CODE`

`STORYLOCK_ANDROID_APK_CHECKSUM`

`STORYLOCK_ANDROID_PACKAGE_KIND`

`STORYLOCK_ANDROID_RELEASE_CHANNEL`

然后访问：

`/download/android-host`

预期结果：返回 APK 文件，或在使用外部分发地址时返回 307 重定向。

## 4. 安装与绑定

```powershell
adb install -r release\app\android\storylock-android-host-0.1.0-1-debug.apk
```

打开易安绑定入口：

`/android-host/bind?identityId=android-demo-001&preferredMode=relay_url`

将返回的 `binding.deepLink` 通过浏览器或 adb 打开：

```powershell
adb shell am start -a android.intent.action.VIEW -d "storylock-host://bind?gateway_url=...&binding_token=...&identity_id=android-demo-001&preferred_mode=relay_url"
```

## 5. 状态检查

Android 宿主界面应显示：

1. `not bound`
2. `bound, not registered`
3. `registering`
4. `online`
5. `offline`

注册完成后，易安网站运行态面板应显示活跃宿主数量大于 0。

## 6. 回连与本地授权

执行远程能力请求：

1. `requestSignature`
2. `requestPasswordFill`

预期流程：

1. 网关通过 relay 向 Android 宿主派发请求。
2. Android 宿主弹出本地 challenge 与 BiometricPrompt / 设备凭据确认。
3. 本地执行后返回脱敏结果。
4. 网关返回 `executionLocation=remote_gateway`，敏感字段为 `[redacted]`。

## 7. 记录结果

检查完成后记录：

1. APK 路径、版本、大小和 SHA-256。
2. 设备型号、Android SDK 版本。
3. 网关 URL 与连接模式。
4. deep link 是否成功唤起。
5. 注册 ID 与 relay poll/respond 是否成功。
6. `requestSignature` 与 `requestPasswordFill` 返回结果。

可使用脚本生成本地记录：

```powershell
scripts\android\check_device_loop.cmd `
  -ApkPath release\app\android\storylock-android-host-0.1.0-1-debug.apk `
  -GatewayBaseUrl https://yian.cdao.online `
  -DeepLink "storylock-host://bind?gateway_url=...&binding_token=...&identity_id=android-demo-001&preferred_mode=relay_url"
```

脚本会检查 `adb devices`、APK 是否存在、安装命令、deep link 唤起、易安关键端点，并生成：

`%TEMP%/android-device-loop-report.local.md`

如需保存到项目文档目录，可显式传入 `-ReportPath docs/management/android-device-loop-report.local.md`。该文件是本机检查产物，默认不提交。
