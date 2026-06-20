# StoryLock Android Host

This directory contains the Android-host application skeleton for StoryLock Layers 1 and 2.

Current implemented scope:

1. Android application project skeleton
2. local HTTP host entry at `GET /health` and `POST /execute`
3. Android Keystore-backed `SecretStore`
4. local multi-cell challenge prompt and BiometricPrompt confirmation flow
5. host registration store with stable `deviceId` and `appInstanceId`
6. deep-link based first-bind flow
7. relay polling client for server-side callback execution
8. `execute` path backed by Android Keystore asymmetric signature keys and Android Keystore-encrypted credential objects
9. asset-backed local question-set loading for Android challenge execution

Current network flow:

1. Layer 3 exposes `/local-host/bind`, `/local-host/register`, `/local-host/relay/poll`, `/local-host/relay/respond`
2. Android app can either receive a deep link such as `storylock-host://bind?...` or use the APK default gateway (`STORYLOCK_GATEWAY_URL`)
3. the app persists gateway info and registration token when a deep link is used
4. the app registers itself to the remote gateway
5. the app keeps polling relay requests and executes them through the local host runtime

Current non-goals:

1. full production mobile release hardening, attestation, and platform Credential Manager integration
2. full Layer 1 / Layer 2 runtime parity with the JS host
3. packaged APK build verification inside this workspace session

Recommended next steps:

1. add requested-algorithm policy mapping between Layer 3 requests and the Android Keystore signer
2. connect Android Credential Manager or an equivalent real credential provider
3. compile a real APK and point `STORYLOCK_ANDROID_APK_PATH` to the build artifact
4. replace the simplified authorization runtime with the production Layer 1 and Layer 2 bindings
5. move from asset-backed question-set loading to full second-layer persistent question-set management

Current signature behavior:

1. signature requests are executed by a local Android Keystore EC keypair generated per `keyId`
2. the host returns signature result and public-key metadata, not demo HMAC key material
3. the current prototype still does not guarantee parity with requested external algorithms such as `ed25519` or `secp256k1`

Current challenge behavior:

1. the Android prototype now asks for all required local challenge cells for the requested strength
2. signature requests require nine local challenge answers plus strong biometric confirmation
3. password-fill requests require six local challenge answers plus local confirmation
4. repeated challenge failures now increment a local failure counter and can trigger temporary lockout
5. failure responses now distinguish challenge cancellation, challenge mismatch, biometric unavailability, biometric cancellation, and challenge lockout
6. challenge data now comes from a versioned local asset file instead of a hard-coded question list inside the runtime

APK distribution notes:

1. default debug artifact path: `src/host/android-host/app/build/outputs/apk/debug/app-debug.apk`
2. recommended file name: `storylock-android-host-{versionName}-{versionCode}-{debug|release}.apk`
3. version display is controlled by `STORYLOCK_ANDROID_APK_VERSION` and `STORYLOCK_ANDROID_APK_VERSION_CODE`
4. release or candidate packages should provide `STORYLOCK_ANDROID_APK_CHECKSUM`, preferably SHA-256
5. debug packages are for judging, self-test, and internal device validation; release packages are for partner-facing or public candidate distribution

Build helper:

```powershell
Push-Location ..\..\..
scripts\release\android\build_apk.cmd -Variant debug
Pop-Location
```

The helper requires either a local `gradlew.bat` under `src/host/android-host/` or a system `gradle` command. It writes APK metadata to `scripts/vercel/.env.android-apk` for the Yian download entry.
