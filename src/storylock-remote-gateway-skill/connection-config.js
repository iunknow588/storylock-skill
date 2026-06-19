function optionalString(value) {
  if (typeof value !== 'string') {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function ensureUrl(value) {
  const text = optionalString(value);
  if (!text) {
    return null;
  }
  try {
    return new URL(text).toString();
  } catch {
    return text;
  }
}

function buildConnectionOptions(env = process.env, {
  gatewayBaseUrl = null,
  bindingDeepLink = null,
} = {}) {
  const options = [];
  const directUrl = ensureUrl(env.STORYLOCK_ANDROID_HOST_URL);
  const relayUrl = ensureUrl(env.STORYLOCK_ANDROID_RELAY_URL)
    ?? (gatewayBaseUrl ? new URL('/android-host/relay', gatewayBaseUrl).toString() : null);
  const deepLink = optionalString(env.STORYLOCK_ANDROID_DEEP_LINK)
    ?? optionalString(bindingDeepLink);

  if (directUrl) {
    options.push({
      mode: 'direct_url',
      label: 'Direct Android Host URL',
      value: directUrl,
    });
  }
  if (relayUrl) {
    options.push({
      mode: 'relay_url',
      label: 'Relay URL',
      value: relayUrl,
    });
  }
  if (deepLink) {
    options.push({
      mode: 'deep_link',
      label: 'Android Deep Link',
      value: deepLink,
    });
  }
  return options;
}

export function resolveAndroidConnectionConfig(env = process.env, {
  gatewayBaseUrl = null,
  bindingDeepLink = null,
} = {}) {
  const preferredMode = optionalString(env.STORYLOCK_ANDROID_CONNECT_MODE) ?? 'direct_url';
  const options = buildConnectionOptions(env, {
    gatewayBaseUrl,
    bindingDeepLink,
  });
  const activeOption = options.find((option) => option.mode === preferredMode) ?? options[0] ?? null;
  return {
    preferredMode,
    activeOption,
    options,
  };
}

export function resolveAppDistributionConfig(env = process.env) {
  return {
    androidAppDownloadUrl: optionalString(env.STORYLOCK_ANDROID_APP_DOWNLOAD_URL),
    androidApkPath: optionalString(env.STORYLOCK_ANDROID_APK_PATH),
    androidApkVersion: optionalString(env.STORYLOCK_ANDROID_APK_VERSION) ?? '0.1.0',
    androidApkVersionCode: optionalString(env.STORYLOCK_ANDROID_APK_VERSION_CODE) ?? '1',
    androidApkSizeBytes: optionalString(env.STORYLOCK_ANDROID_APK_SIZE_BYTES) ?? '5836748',
    androidApkChecksum: optionalString(env.STORYLOCK_ANDROID_APK_CHECKSUM)
      ?? 'sha256:8150d8a09dc0b7357c001bd53ebd7788a0d211102d574268a9f720d49f34bdd1',
    androidPackageKind: optionalString(env.STORYLOCK_ANDROID_PACKAGE_KIND) ?? 'debug',
    androidReleaseChannel: optionalString(env.STORYLOCK_ANDROID_RELEASE_CHANNEL) ?? 'internal',
    androidUiDownloadUrl: optionalString(env.STORYLOCK_ANDROID_UI_DOWNLOAD_URL),
    androidInstallGuideUrl: optionalString(env.STORYLOCK_ANDROID_INSTALL_GUIDE_URL),
    windowsAppDownloadUrl: optionalString(env.STORYLOCK_WINDOWS_APP_DOWNLOAD_URL),
    windowsPackagePath: optionalString(env.STORYLOCK_WINDOWS_PACKAGE_PATH),
    windowsPackageVersion: optionalString(env.STORYLOCK_WINDOWS_PACKAGE_VERSION) ?? '0.1.0',
    windowsPackageVersionCode: optionalString(env.STORYLOCK_WINDOWS_PACKAGE_VERSION_CODE) ?? '1',
    windowsPackageSizeBytes: optionalString(env.STORYLOCK_WINDOWS_PACKAGE_SIZE_BYTES),
    windowsPackageChecksum: optionalString(env.STORYLOCK_WINDOWS_PACKAGE_CHECKSUM),
    windowsPackageKind: optionalString(env.STORYLOCK_WINDOWS_PACKAGE_KIND) ?? 'zip',
    windowsReleaseChannel: optionalString(env.STORYLOCK_WINDOWS_RELEASE_CHANNEL) ?? 'prototype',
  };
}
