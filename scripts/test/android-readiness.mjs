import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../../', import.meta.url));

function read(relativePath) {
  return readFileSync(join(root, relativePath), 'utf8');
}

function required(relativePath) {
  assert.ok(existsSync(join(root, relativePath)), `${relativePath} must exist`);
}

const requiredFiles = [
  'src/host/android-host/settings.gradle.kts',
  'src/host/android-host/build.gradle.kts',
  'src/host/android-host/app/build.gradle.kts',
  'src/host/android-host/app/src/main/AndroidManifest.xml',
  'src/host/android-host/app/src/main/assets/storylock-question-set.json',
  'src/host/android-host/app/src/main/assets/storylock-resource-catalog.json',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/StoryLockHostApplication.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/MainActivity.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AndroidKeystoreSecretStore.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AndroidKeystoreSigner.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/security/BiometricPromptLocalUserConfirmation.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/security/ChallengePromptLocalUserConfirmation.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AttachedActivityConfirmation.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/host/HostBindingStore.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/host/HostGatewayClient.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/host/HostRegistrationManager.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/host/LocalAuthorizationRuntime.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/host/AndroidStoryLockPackageRepository.kt',
  'src/host/android-host/app/src/main/java/org/storylock/androidhost/host/StoryLockAndroidHostService.kt',
  'src/shared/assets/schemas/storylock-resource-catalog.schema.json',
  'src/shared/assets/schemas/storylock-permission-summary.schema.json',
  'src/shared/storylock-package/permission-summary.js',
  'scripts/release/android/build_apk.ps1',
  'scripts/release/android/build_apk.cmd',
  'scripts/android/validate_android_question_set.mjs',
  'docs/ref/10-Android真机闭环检查.md',
];

for (const file of requiredFiles) {
  required(file);
}

const buildGradle = read('src/host/android-host/app/build.gradle.kts');
for (const token of [
  'applicationId = "org.storylock.androidhost"',
  'minSdk = 26',
  'targetSdk = 34',
  'versionCode = 1',
  'versionName = "0.1.0"',
  'STORYLOCK_GATEWAY_URL',
  'STORYLOCK_CONNECT_MODE',
  'androidx.biometric:biometric',
  'org.nanohttpd:nanohttpd',
]) {
  assert.ok(buildGradle.includes(token), `Android Gradle config must include ${token}`);
}

const manifest = read('src/host/android-host/app/src/main/AndroidManifest.xml');
for (const token of [
  'android.permission.INTERNET',
  'android:name=".StoryLockHostApplication"',
  'android:allowBackup="false"',
  'android:launchMode="singleTask"',
  'android:scheme="storylock-host"',
  'android:host="bind"',
]) {
  assert.ok(manifest.includes(token), `Android manifest must include ${token}`);
}

const keystoreStore = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AndroidKeystoreSecretStore.kt');
assert.match(keystoreStore, /AndroidKeyStore/u, 'Android SecretStore must use AndroidKeyStore');
assert.match(keystoreStore, /AES/u, 'Android SecretStore must use encrypted local storage');
assert.match(keystoreStore, /GCM/u, 'Android SecretStore must use authenticated encryption');

const signer = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/security/AndroidKeystoreSigner.kt');
assert.match(signer, /KeyPairGenerator/u, 'Android signer must generate platform key pairs');
assert.match(signer, /AndroidKeyStore/u, 'Android signer must use AndroidKeyStore');
assert.match(signer, /SHA256withECDSA/u, 'Android signer must sign with the current ECDSA prototype');

const biometric = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/security/BiometricPromptLocalUserConfirmation.kt');
assert.match(biometric, /BiometricPrompt/u, 'Android confirmation must use BiometricPrompt');
assert.match(biometric, /biometric_unavailable/u, 'Android confirmation must report biometric unavailable');
assert.match(biometric, /biometric_cancelled/u, 'Android confirmation must report biometric cancellation');

const challengePrompt = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/security/ChallengePromptLocalUserConfirmation.kt');
assert.match(challengePrompt, /challenge_failed/u, 'Android challenge prompt must report failed answers');
assert.match(challengePrompt, /challenge_cancelled/u, 'Android challenge prompt must report cancellation');

const authorizationRuntime = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/host/LocalAuthorizationRuntime.kt');
assert.match(authorizationRuntime, /maxFailureCount/u, 'Android authorization runtime must track failure limits');
assert.match(authorizationRuntime, /ChallengeLockedException/u, 'Android authorization runtime must support temporary lockout');
assert.match(authorizationRuntime, /requestSignature/u, 'Android authorization runtime must distinguish signature strength');

const hostService = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/host/StoryLockAndroidHostService.kt');
for (const token of [
  'requestSignature',
  'requestPasswordFill',
  'permissionSummary',
  'storyLockPackage',
  'challenge_locked',
  'challenge_failed',
  'biometric_unavailable',
  'secretMaterial',
  'audit_meta_only',
]) {
  assert.ok(hostService.includes(token), `Android host service must include ${token}`);
}
assert.doesNotMatch(hostService, /\.put\("password"\s*,\s*credential/u, 'Android password-fill response must not return raw password');
assert.doesNotMatch(hostService, /\.put\("privateKey"/u, 'Android signature response must not expose privateKey fields');
assert.doesNotMatch(hostService, /\.put\("signingKeyBytes"/u, 'Android signature response must not expose signingKeyBytes fields');

const androidPackageRepository = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/host/AndroidStoryLockPackageRepository.kt');
for (const token of [
  'storylock-resource-catalog.json',
  'permissionAction',
  'permissionChallengePolicy',
  'permissionRequiredGridCount',
  'password_fill',
  'signing_key',
]) {
  assert.ok(androidPackageRepository.includes(token), `Android package repository must include ${token}`);
}
assert.doesNotMatch(androidPackageRepository, /canonicalAnswer|acceptedAnswers|privateKey|signingKeyBytes/u, 'Android permission summary repository must not expose sensitive story/key fields');

const androidServer = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/host/AndroidHostServer.kt');
assert.match(androidServer, /\/permission-summary/u, 'Android local server must expose permission summary endpoint');

const sharedPermissionSummary = read('src/shared/storylock-package/permission-summary.js');
for (const token of [
  'createPermissionSummary',
  'permissionAction',
  'permissionChallengePolicy',
  'permissionRequiredGridCount',
  'signingKeyBytes',
]) {
  assert.ok(sharedPermissionSummary.includes(token), `Shared permission summary must include ${token}`);
}

const androidReadme = read('src/host/android-host/README.md');
assert.match(androidReadme, /permission summary/u, 'Android README must mention permission summary alignment');
assert.match(androidReadme, /must not edit StoryLock Core configuration/u, 'Android README must preserve Core configuration boundary');

const gatewayClient = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/host/HostGatewayClient.kt');
for (const token of [
  '/local-host/register',
  '/local-host/relay/poll',
  '/local-host/relay/respond',
  'x-storylock-shared-secret',
]) {
  assert.ok(gatewayClient.includes(token), `Android gateway client must include ${token}`);
}

const bindingStore = read('src/host/android-host/app/src/main/java/org/storylock/androidhost/host/HostBindingStore.kt');
for (const token of ['binding_token', 'registration_id', 'relay_poll_url', 'relay_respond_url']) {
  assert.ok(bindingStore.includes(token), `Android binding store must include ${token}`);
}

const buildApkScript = read('scripts/release/android/build_apk.ps1');
for (const token of [
  'ValidateSet("debug", "release")',
  'assemble$($Variant.Substring(0, 1).ToUpper())$($Variant.Substring(1))',
  'STORYLOCK_ANDROID_APK_CHECKSUM',
  'STORYLOCK_ANDROID_PACKAGE_KIND',
  'STORYLOCK_ANDROID_RELEASE_CHANNEL',
]) {
  assert.ok(buildApkScript.includes(token), `Android APK build helper must include ${token}`);
}

const questionSetCheck = spawnSync(process.execPath, ['scripts/android/validate_android_question_set.mjs'], {
  cwd: root,
  encoding: 'utf8',
});
assert.equal(questionSetCheck.status, 0, questionSetCheck.stderr || questionSetCheck.stdout);

const questionSetResult = JSON.parse(questionSetCheck.stdout);
assert.equal(questionSetResult.status, 'passed');
assert.ok(questionSetResult.activeQuestionCount >= 24, 'Android question set must have at least 24 active questions');

const catalogSummaryCheck = spawnSync(process.execPath, [
  'scripts/storylock-package/permission-summary-json.mjs',
  'src/host/android-host/app/src/main/assets/storylock-resource-catalog.json',
], {
  cwd: root,
  encoding: 'utf8',
});
assert.equal(catalogSummaryCheck.status, 0, catalogSummaryCheck.stderr || catalogSummaryCheck.stdout);
const catalogSummary = JSON.parse(catalogSummaryCheck.stdout);
assert.equal(catalogSummary.items.length, 4, 'Android resource catalog must expose four permission summary items');
assert.equal(
  catalogSummary.items.find((item) => item.objectKind === 'password').action,
  'password_fill',
);
assert.equal(
  catalogSummary.items.find((item) => item.objectKind === 'private_key').action,
  'sign',
);
assert.equal(JSON.stringify(catalogSummary).includes('signingKeyBytes'), false);

console.log(JSON.stringify({
  status: 'passed',
  filesChecked: requiredFiles.length,
  questionSet: {
    file: relative(root, join(root, 'src/host/android-host/app/src/main/assets/storylock-question-set.json')).replaceAll('\\', '/'),
    activeQuestionCount: questionSetResult.activeQuestionCount,
    questionSetVersion: questionSetResult.questionSetVersion,
  },
  permissionSummary: {
    file: relative(root, join(root, 'src/host/android-host/app/src/main/assets/storylock-resource-catalog.json')).replaceAll('\\', '/'),
    items: catalogSummary.items.length,
  },
}, null, 2));
