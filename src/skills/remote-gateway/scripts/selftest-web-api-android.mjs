import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import gatewayHandler from '../../../../web-api/storylock-gateway.mjs';
import { createAndroidStoryLockHostServer } from '../../local-story-access/android-host-server.js';
import { StoryLockRemoteGateway } from '../index.js';
import { createHttpRemoteTransport } from '../http-transport.js';

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function startLocalHandlerServer(handler, host = '127.0.0.1', port = 0) {
  const server = createServer((req, res) => handler(req, res));
  return {
    async start() {
      await new Promise((resolve) => server.listen(port, host, resolve));
      const address = server.address();
      const resolvedPort = typeof address === 'object' && address ? address.port : port;
      return `http://${host}:${resolvedPort}`;
    },
    async stop() {
      await new Promise((resolve, reject) => server.close((error) => (error ? reject(error) : resolve())));
    },
  };
}

async function executeAgainstAndroid(executeUrl, sharedSecret, request) {
  const response = await fetch(executeUrl, {
    method: 'POST',
    headers: {
      'content-type': 'application/json; charset=utf-8',
      'x-storylock-shared-secret': sharedSecret,
    },
    body: JSON.stringify(request),
  });
  return {
    statusCode: response.status,
    body: await response.json(),
  };
}

async function executeAgainstAndroidBody(executeUrl, sharedSecret, request) {
  const response = await executeAgainstAndroid(executeUrl, sharedSecret, request);
  return response.body;
}

const sharedSecret = 'storylock-local-android-selftest';
const deviceId = 'android-device-001';
const appInstanceId = 'android-app-001';
const relayHosts = [
  { deviceId, appInstanceId },
  { deviceId: `${deviceId}-legacy`, appInstanceId: `${appInstanceId}-legacy` },
];
const androidHost = createAndroidStoryLockHostServer({ sharedSecret });
const originalEnv = {
  STORYLOCK_ANDROID_HOST_URL: process.env.STORYLOCK_ANDROID_HOST_URL,
  STORYLOCK_ANDROID_SHARED_SECRET: process.env.STORYLOCK_ANDROID_SHARED_SECRET,
  STORYLOCK_ANDROID_CONNECT_MODE: process.env.STORYLOCK_ANDROID_CONNECT_MODE,
  STORYLOCK_ANDROID_APP_DOWNLOAD_URL: process.env.STORYLOCK_ANDROID_APP_DOWNLOAD_URL,
  STORYLOCK_ANDROID_APK_PATH: process.env.STORYLOCK_ANDROID_APK_PATH,
  STORYLOCK_ANDROID_APK_VERSION: process.env.STORYLOCK_ANDROID_APK_VERSION,
  STORYLOCK_ANDROID_APK_VERSION_CODE: process.env.STORYLOCK_ANDROID_APK_VERSION_CODE,
  STORYLOCK_ANDROID_APK_CHECKSUM: process.env.STORYLOCK_ANDROID_APK_CHECKSUM,
  STORYLOCK_ANDROID_PACKAGE_KIND: process.env.STORYLOCK_ANDROID_PACKAGE_KIND,
  STORYLOCK_ANDROID_RELEASE_CHANNEL: process.env.STORYLOCK_ANDROID_RELEASE_CHANNEL,
  STORYLOCK_ANDROID_DEEP_LINK: process.env.STORYLOCK_ANDROID_DEEP_LINK,
  STORYLOCK_GATEWAY_PUBLIC_URL: process.env.STORYLOCK_GATEWAY_PUBLIC_URL,
  STORYLOCK_ANDROID_REGISTRY_FILE: process.env.STORYLOCK_ANDROID_REGISTRY_FILE,
};

const tempDir = await mkdtemp(join(tmpdir(), 'storylock-web-api-android-'));
const apkPath = join(tempDir, 'storylock-android-host.apk');
const registryPath = join(tempDir, 'storylock-registry.json');

try {
  const fakeApk = Buffer.from('fake-apk-binary-for-selftest', 'utf8');
  await writeFile(apkPath, fakeApk);
  const androidInfo = await androidHost.start();
  const health = await fetch(androidInfo.healthUrl, {
    headers: {
      'x-storylock-shared-secret': sharedSecret,
    },
  }).then((response) => response.json());
  assert.equal(health.status, 'ok');
  assert.equal(health.layer1.questionSetReady, true);
  const unauthorizedHealth = await fetch(androidInfo.healthUrl, {
    headers: {
      'x-storylock-shared-secret': 'wrong-secret',
    },
  });
  assert.equal(unauthorizedHealth.status, 401);
  assert.equal((await unauthorizedHealth.json()).message, 'unauthorized');

  delete process.env.STORYLOCK_ANDROID_HOST_URL;
  process.env.STORYLOCK_ANDROID_SHARED_SECRET = sharedSecret;
  process.env.STORYLOCK_ANDROID_CONNECT_MODE = 'relay_url';
  process.env.STORYLOCK_ANDROID_APK_PATH = apkPath;
  process.env.STORYLOCK_ANDROID_APK_VERSION = '0.1.0';
  process.env.STORYLOCK_ANDROID_APK_VERSION_CODE = '1';
  process.env.STORYLOCK_ANDROID_APK_CHECKSUM = 'sha256:selftest';
  process.env.STORYLOCK_ANDROID_PACKAGE_KIND = 'debug';
  process.env.STORYLOCK_ANDROID_RELEASE_CHANNEL = 'internal';
  process.env.STORYLOCK_ANDROID_DEEP_LINK = 'storylock-host://bind';
  process.env.STORYLOCK_ANDROID_REGISTRY_FILE = registryPath;

  const webApiServer = startLocalHandlerServer(gatewayHandler);
  const webApiBaseUrl = await webApiServer.start();
  const gatewayEndpoint = `${webApiBaseUrl}/api/storylock-gateway`;
  process.env.STORYLOCK_GATEWAY_PUBLIC_URL = webApiBaseUrl;

  let relayWorkerRunning = true;
  const relayWorker = (async () => {
    while (relayWorkerRunning) {
      for (const host of relayHosts) {
        const poll = await fetch(`${webApiBaseUrl}/local-host/relay/poll`, {
          method: 'POST',
          headers: {
            'content-type': 'application/json; charset=utf-8',
            'x-storylock-shared-secret': sharedSecret,
          },
          body: JSON.stringify({
            deviceId: host.deviceId,
            appInstanceId: host.appInstanceId,
          }),
        }).then((response) => response.json());

        if (poll.status === 'ok') {
          const relayResponse = await executeAgainstAndroidBody(
            androidInfo.executeUrl,
            sharedSecret,
            poll.request,
          );
          await fetch(`${webApiBaseUrl}/local-host/relay/respond`, {
            method: 'POST',
            headers: {
              'content-type': 'application/json; charset=utf-8',
              'x-storylock-shared-secret': sharedSecret,
            },
            body: JSON.stringify({
              relayRequestId: poll.relayRequestId,
              response: relayResponse,
            }),
          });
        }
      }

      await sleep(15);
    }
  })();

  try {
    const idlePoll = await fetch(`${webApiBaseUrl}/local-host/relay/poll`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json; charset=utf-8',
        'x-storylock-shared-secret': sharedSecret,
      },
      body: JSON.stringify({
        deviceId: 'unregistered-device',
        appInstanceId: 'unregistered-app',
      }),
    }).then((response) => response.json());
    assert.equal(idlePoll.status, 'idle');

    const platformDownloadResponse = await fetch(`${webApiBaseUrl}/app/download`);
    assert.equal(platformDownloadResponse.status, 200);
    assert.match(platformDownloadResponse.headers.get('content-type') ?? '', /application\/json/);
    const platformDownload = await platformDownloadResponse.json();
    assert.equal(platformDownload.status, 'ok');
    assert.equal(platformDownload.platforms.android.downloadUrl, `${webApiBaseUrl}/app/download/android`);
    assert.equal(platformDownload.platforms.windows.implementation, 'rust');

    const downloadResponse = await fetch(`${webApiBaseUrl}/app/download/android`);
    assert.equal(downloadResponse.status, 200);
    assert.equal(downloadResponse.headers.get('content-type'), 'application/vnd.android.package-archive');
    assert.equal(await downloadResponse.text(), 'fake-apk-binary-for-selftest');

    const legacyDownloadResponse = await fetch(`${webApiBaseUrl}/download/android-host`);
    assert.equal(legacyDownloadResponse.status, 200);
    assert.equal(legacyDownloadResponse.headers.get('content-type'), 'application/vnd.android.package-archive');

    const bindingInfo = await fetch(`${webApiBaseUrl}/app/bind?identityId=android-demo-001&preferredMode=relay_url`, {
      headers: {
        'x-storylock-shared-secret': sharedSecret,
      },
    }).then((response) => response.json());
    assert.equal(bindingInfo.status, 'ok');
    assert.equal(bindingInfo.binding.identityId, 'android-demo-001');
    assert.match(bindingInfo.binding.deepLink, /storylock-host:\/\/bind/);

    const invalidRegistration = await fetch(`${webApiBaseUrl}/local-host/register`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json; charset=utf-8',
      },
      body: JSON.stringify({
        bindingToken: 'invalid-token',
        identityId: 'android-demo-001',
        deviceId: 'bad-device',
        appInstanceId: 'bad-app',
      }),
    });
    assert.equal(invalidRegistration.status, 400);
    assert.equal((await invalidRegistration.json()).message, 'binding token is invalid or expired');

    const registration = await fetch(`${webApiBaseUrl}/local-host/register`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json; charset=utf-8',
      },
      body: JSON.stringify({
        bindingToken: bindingInfo.binding.token,
        identityId: 'android-demo-001',
        deviceId,
        appInstanceId,
        preferredMode: 'relay_url',
        host: {
          healthUrl: androidInfo.healthUrl,
          executeUrl: androidInfo.executeUrl,
        },
        install: {
          versionName: '0.1.0',
          versionCode: 1,
        },
        device: {
          model: 'selftest-android',
          manufacturer: 'storylock',
          sdkInt: 34,
        },
        reachability: {
          localHttp: true,
          relayPolling: true,
          deepLink: bindingInfo.binding.deepLink,
        },
      }),
    }).then((response) => response.json());
    assert.equal(registration.status, 'ok');
    assert.equal(registration.registration.identityId, 'android-demo-001');
    assert.equal(registration.registration.deviceId, deviceId);
    assert.match(registration.relay.pollUrl, /\/local-host\/relay\/poll$/);

    const legacyRegistration = await fetch(`${webApiBaseUrl}/android-host/register`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json; charset=utf-8',
        'x-storylock-shared-secret': sharedSecret,
      },
      body: JSON.stringify({
        identityId: 'android-demo-001',
        deviceId: `${deviceId}-legacy`,
        appInstanceId: `${appInstanceId}-legacy`,
        preferredMode: 'relay_url',
        host: {
          healthUrl: androidInfo.healthUrl,
          executeUrl: androidInfo.executeUrl,
        },
      }),
    }).then((response) => response.json());
    assert.equal(legacyRegistration.status, 'ok');
    assert.equal(legacyRegistration.registration.deviceId, `${deviceId}-legacy`);
    assert.match(legacyRegistration.relay.pollUrl, /\/local-host\/relay\/poll$/);
    assert.match(legacyRegistration.relay.respondUrl, /\/local-host\/relay\/respond$/);

    const status = await fetch(gatewayEndpoint, {
      headers: {
        'x-storylock-shared-secret': sharedSecret,
      },
    }).then((response) => response.json());
    assert.equal(status.status, 'ok');
    assert.equal(status.secondLayerConnection.activeOption.mode, 'relay_url');
    assert.equal(status.onlineStatus.websiteEntryUrl, `${webApiBaseUrl}/`);
    assert.equal(status.onlineStatus.gatewayEntryUrl, gatewayEndpoint);
    assert.equal(status.onlineStatus.downloadUrl, `${webApiBaseUrl}/app/download`);
    assert.equal(status.onlineStatus.androidDownloadUrl, `${webApiBaseUrl}/app/download/android`);
    assert.equal(status.onlineStatus.windowsDownloadUrl, `${webApiBaseUrl}/app/download/windows`);
    assert.equal(status.onlineStatus.bindingEntryUrl, `${webApiBaseUrl}/app/bind`);
    assert.equal(status.onlineStatus.activeConnectionMode, 'relay_url');
    assert.equal(status.onlineStatus.activeHostCount, 2);
    assert.equal(status.hostRegistry.activeHostCount, 2);
    assert.equal(status.appDistribution.androidAppDownloadUrl, `${webApiBaseUrl}/app/download/android`);
    assert.equal(status.appDistribution.artifact.available, true);
    assert.equal(status.appDistribution.artifact.fileName, 'storylock-android-host.apk');
    assert.equal(status.appDistribution.artifact.fileSizeBytes, fakeApk.length);
    assert.equal(status.appDistribution.artifact.versionName, '0.1.0');
    assert.equal(status.appDistribution.artifact.versionCode, '1');
    assert.equal(status.appDistribution.artifact.checksum, 'sha256:selftest');
    assert.equal(status.appDistribution.artifact.packageKind, 'debug');
    assert.equal(status.appDistribution.artifact.releaseChannel, 'internal');
    assert.equal(status.appDistribution.artifact.downloadStrategy, 'local_apk_file');
    assert.equal(status.appDistribution.platforms.android.downloadUrl, `${webApiBaseUrl}/app/download/android`);
    assert.equal(status.appDistribution.platforms.windows.downloadUrl, `${webApiBaseUrl}/app/download/windows`);
    assert.equal(status.appDistribution.platforms.windows.implementation, 'rust');

    const directUnsupported = await executeAgainstAndroid(androidInfo.executeUrl, sharedSecret, {
      requestId: 'req-unsupported-direct',
      capability: 'requestUnsupported',
      payload: {},
    });
    assert.equal(directUnsupported.statusCode, 400);
    assert.equal(directUnsupported.body.status, 'error');
    assert.equal(directUnsupported.body.error.code, 'SLG-001');

    const directUnauthorized = await executeAgainstAndroid(androidInfo.executeUrl, 'wrong-secret', {
      requestId: 'req-direct-unauthorized',
      capability: 'requestPasswordFill',
      payload: {},
    });
    assert.equal(directUnauthorized.statusCode, 401);
    assert.equal(directUnauthorized.body.message, 'unauthorized');

    const clientGateway = new StoryLockRemoteGateway({
      transport: createHttpRemoteTransport({
        endpoint: gatewayEndpoint,
        headers: {
          'x-storylock-shared-secret': sharedSecret,
        },
        timeoutMs: 30_000,
      }),
    });

    const signatureResult = await clientGateway.requestSignature({
      requestId: 'req-web-api-sign',
      nonce: '20001',
      eip712Nonce: '20001',
      expiry: Date.now() + 60_000,
      identityId: 'android-demo-001',
      keyId: 'wallet/main/private_key',
      algorithm: 'ed25519',
      payload: 'sign from web api gateway',
      resourceId: 'wallet/main/private_key',
    });
    assert.equal(signatureResult.status, 'success');
    assert.equal(signatureResult.executionLocation, 'remote_gateway');
    assert.equal(signatureResult.result.privateKey, '[redacted]');
    assert.equal(signatureResult.result.signingKeyBytes, '[redacted]');
    assert.equal(signatureResult.auditMeta.remoteGateway, 'web_api');

    const passwordResult = await clientGateway.requestPasswordFill({
      requestId: 'req-web-api-password',
      nonce: '20002',
      expiry: Date.now() + 60_000,
      identityId: 'android-demo-001',
      credentialRef: 'site/example',
      targetOrigin: 'https://example.com',
    });
    assert.equal(passwordResult.status, 'success');
    assert.equal(passwordResult.executionLocation, 'remote_gateway');
    assert.equal(passwordResult.result.password, '[redacted]');
    assert.equal(passwordResult.result.username, 'android-user');

    console.log('StoryLock Web API gateway + Android registration/relay/APK selftest passed.');
  } finally {
    relayWorkerRunning = false;
    await relayWorker;
    await webApiServer.stop();
  }
} finally {
  for (const [key, value] of Object.entries(originalEnv)) {
    if (value === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = value;
    }
  }
  await androidHost.stop();
  await rm(tempDir, { recursive: true, force: true });
}
