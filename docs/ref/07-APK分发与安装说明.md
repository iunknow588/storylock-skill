# APK 分发与安装说明

日期：2026-06-18

本文档说明易安第三层网站对 Android 宿主 APK 的分发、版本展示、安装流程与 debug/release 对外策略。

## 1. 产出路径

Android 宿主默认调试产物路径：

`android-host/app/build/outputs/apk/debug/app-debug.apk`

线上或本地网关优先读取环境变量：

`STORYLOCK_ANDROID_APK_PATH`

当 `STORYLOCK_ANDROID_APK_PATH` 指向的文件存在时，`GET /download/android-host` 直接返回该 APK 文件。当未配置本地文件但配置了 `STORYLOCK_ANDROID_APP_DOWNLOAD_URL` 时，下载入口使用 307 重定向到外部分发地址。

## 2. 命名规则

建议发布文件名：

`storylock-android-host-{versionName}-{versionCode}-{debug|release}.apk`

示例：

`storylock-android-host-0.1.0-1-debug.apk`

`storylock-android-host-0.1.0-1-release.apk`

## 3. 版本号规则

网站和 `GET /api/storylock-gateway` 使用以下环境变量展示版本信息：

`STORYLOCK_ANDROID_APK_VERSION`

`STORYLOCK_ANDROID_APK_VERSION_CODE`

`STORYLOCK_ANDROID_APK_CHECKSUM`

`STORYLOCK_ANDROID_PACKAGE_KIND`

`STORYLOCK_ANDROID_RELEASE_CHANNEL`

推荐约定：

`versionName` 使用语义化版本，例如 `0.1.0`。

`versionCode` 使用递增整数，每次对外发包递增。

`checksum` 优先使用 SHA-256，并在 release/candidate 包中必须提供。

## 4. 网站展示

易安网站运行态面板展示：

APK 版本：`versionName` 与 `versionCode`

APK 大小：本地文件存在时展示字节大小的友好格式

包类型：`debug` / `release` / `apk`

发布通道：`internal` / `candidate` / 自定义通道

校验值：可选 checksum

接口原始字段保留在 `appDistribution.artifact` 中，供开发者调试和自动化检查使用。

## 5. 安装流程

1. 在易安网站点击“下载 Android 宿主”。
2. 在 Android 设备上安装 APK，并允许来自当前来源的安装。
3. 回到易安网站打开绑定入口，生成一次性 binding token。
4. Android 宿主通过 deep link 写入 `gateway_url`、`binding_token`、`identity_id` 与 `preferred_mode`。
5. 宿主完成首次注册后，回到网站运行态面板查看连接模式、活跃宿主数和宿主登记状态。

## 6. Debug 与 Release 策略

debug 包仅用于评审演示、自测、真机联调和内部分发。debug 包可以走 `internal` 通道，但页面仍应展示版本、大小和包类型，避免误发。

release 包用于合作方验证或公开候选分发。release 包必须使用正式签名、递增 `versionCode`、带 SHA-256 校验值，并在 `STORYLOCK_ANDROID_RELEASE_CHANNEL` 中标记为 `candidate` 或更明确的发布通道。

在真实 APK 签名和真机编译验证完成前，易安网站可以展示下载入口和分发规则，但对外说明应保持“候选 / 内测”口径。
