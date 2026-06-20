# APK 分发与安装说明

日期：2026-06-20

本文档说明易安网站对 Android 宿主 APK 的当前分发方式、环境变量约定和安装演示口径。

## 1. 当前产物路径

Android 默认调试包产物路径：

`src/host/android-host/app/build/outputs/apk/debug/app-debug.apk`

对外统一分发目录：

`release/app/android/storylock-android-host-{versionName}-{versionCode}-{debug|release}.apk`

构建脚本：

`scripts/release/android/build_apk.ps1`

该脚本会在 Gradle 构建完成后，把 APK 复制到 `release/app/android/`，并更新供网关读取的路径变量。

## 2. 下载入口优先级

Web API 网关优先读取：

`STORYLOCK_ANDROID_APK_PATH`

当 `STORYLOCK_ANDROID_APK_PATH` 指向的文件存在时：

1. `GET /download/android-host` 直接返回 APK 文件。
2. 网站下载入口显示版本、大小、校验值和渠道信息。

当本地 APK 未配置，但配置了：

`STORYLOCK_ANDROID_APP_DOWNLOAD_URL`

则下载入口返回 307 重定向到外部分发地址。

若两者都未配置，网关会继续尝试：

1. `release/app/android/`
2. `release/web/public/downloads/`

中的候选 APK 文件。

## 3. 命名规范

建议文件名：

`storylock-android-host-{versionName}-{versionCode}-{debug|release}.apk`

示例：

1. `storylock-android-host-0.1.0-1-debug.apk`
2. `storylock-android-host-0.1.0-1-release.apk`

## 4. 版本元数据环境变量

网站和 `GET /api/storylock-gateway` 使用以下环境变量展示 APK 信息：

1. `STORYLOCK_ANDROID_APK_VERSION`
2. `STORYLOCK_ANDROID_APK_VERSION_CODE`
3. `STORYLOCK_ANDROID_APK_CHECKSUM`
4. `STORYLOCK_ANDROID_PACKAGE_KIND`
5. `STORYLOCK_ANDROID_RELEASE_CHANNEL`

推荐约定：

1. `versionName` 使用语义化版本，例如 `0.1.0`
2. `versionCode` 使用递增整数
3. `checksum` 优先使用 `sha256:...`
4. `PACKAGE_KIND` 使用 `debug` 或 `release`
5. `RELEASE_CHANNEL` 使用 `internal`、`candidate` 或更明确的渠道名

## 5. 页面展示要求

易安网站当前应至少展示：

1. APK 版本
2. versionCode
3. 文件大小
4. 包类型
5. 发布渠道
6. 校验值

原始响应中的下载信息保留在：

`appDistribution.artifact`

便于调试和自动化校验。

## 6. 安装演示流程

1. 打开易安网站。
2. 点击 Android 宿主下载入口。
3. 在 Android 设备上安装 APK。
4. 回到网站打开绑定入口。
5. 通过 deep link 将 `gateway_url`、`binding_token`、`identity_id` 和 `preferred_mode` 写入宿主。
6. 完成首次注册后，在网站查看注册状态和连接模式。

## 7. Debug 与 Release 口径

### Debug 包

用于：

1. 评审演示
2. 自测
3. 真机联调
4. 内部分发

要求：

1. 页面仍应展示完整版本信息。
2. 对外说明应明确为内部演示或候选态。

### Release 包

用于：

1. 候选交付
2. 合作方验证
3. 正式发布前检查

要求：

1. 使用正式签名。
2. 递增 `versionCode`。
3. 提供 SHA-256 校验值。
4. 在 `STORYLOCK_ANDROID_RELEASE_CHANNEL` 中明确标记渠道。

## 8. 当前真实边界

对外必须说明：

1. 当前仓库已经具备 APK 分发入口和元数据展示链路。
2. 当前仓库不应被误表述为已经完成完整移动端发布体系。
3. 如果展示的是 debug 包，必须讲清是演示安装链路，不是商店发布链路。
