import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import gatewayHandler from '../../../api/storylock-gateway.mjs';
import { createAndroidStoryLockHostServer } from '../../storylock-local-story-access-skill/android-host-server.js';
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
  return response.json();
}

const sharedSecret = 'storylock-local-android-selftest';
const deviceId = 'android-device-001';
const appInstanceId = 'android-app-001';
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

const tempDir = await mkdtemp(join(tmpdir(), 'storylock-vercel-android-'));
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

  const vercelServer = startLocalHandlerServer(gatewayHandler);
  const vercelBaseUrl = await vercelServer.start();
  const gatewayEndpoint = `${vercelBaseUrl}/api/storylock-gateway`;
  process.env.STORYLOCK_GATEWAY_PUBLIC_URL = vercelBaseUrl;

  let relayWorkerRunning = true;
  const relayWorker = (async () => {
    while (relayWorkerRunning) {
      const poll = await fetch(`${vercelBaseUrl}/android-host/relay/poll`, {
        method: 'POST',
        headers: {
          'content-type': 'application/json; charset=utf-8',
          'x-storylock-shared-secret': sharedSecret,
        },
        body: JSON.stringify({
          deviceId,
          appInstanceId,
        }),
      }).then((response) => response.json());

      if (poll.status === 'ok') {
        const relayResponse = await executeAgainstAndroid(
          androidInfo.executeUrl,
          sharedSecret,
          poll.request,
        );
        await fetch(`${vercelBaseUrl}/android-host/relay/respond`, {
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
        continue;
      }

      await sleep(15);
    }
  })();

  try {
    const downloadResponse = await fetch(`${vercelBaseUrl}/app/download`);
    assert.equal(downloadResponse.status, 200);
    assert.equal(downloadResponse.headers.get('content-type'), 'application/vnd.android.package-archive');
    assert.equal(await downloadResponse.text(), 'fake-apk-binary-for-selftest');

    const legacyDownloadResponse = await fetch(`${vercelBaseUrl}/download/android-host`);
    assert.equal(legacyDownloadResponse.status, 200);
    assert.equal(legacyDownloadResponse.headers.get('content-type'), 'application/vnd.android.package-archive');

    const bindingInfo = await fetch(`${vercelBaseUrl}/app/bind?identityId=android-demo-001&preferredMode=relay_url`, {
      headers: {
        'x-storylock-shared-secret': sharedSecret,
      },
    }).then((response) => response.json());
    assert.equal(bindingInfo.status, 'ok');
    assert.equal(bindingInfo.binding.identityId, 'android-demo-001');
    assert.match(bindingInfo.binding.deepLink, /storylock-host:\/\/bind/);

    const registration = await fetch(`${vercelBaseUrl}/android-host/register`, {
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
    assert.match(registration.relay.pollUrl, /\/android-host\/relay\/poll$/);

    const status = await fetch(gatewayEndpoint, {
      headers: {
        'x-storylock-shared-secret': sharedSecret,
      },
    }).then((response) => response.json());
    assert.equal(status.status, 'ok');
    assert.equal(status.secondLayerConnection.activeOption.mode, 'relay_url');
    assert.equal(status.onlineStatus.websiteEntryUrl, `${vercelBaseUrl}/`);
    assert.equal(status.onlineStatus.gatewayEntryUrl, gatewayEndpoint);
    assert.equal(status.onlineStatus.androidDownloadUrl, `${vercelBaseUrl}/app/download`);
    assert.equal(status.onlineStatus.bindingEntryUrl, `${vercelBaseUrl}/app/bind`);
    assert.equal(status.onlineStatus.activeConnectionMode, 'relay_url');
    assert.equal(status.onlineStatus.activeHostCount, 1);
    assert.equal(status.hostRegistry.activeHostCount, 1);
    assert.equal(status.appDistribution.androidAppDownloadUrl, `${vercelBaseUrl}/app/download`);
    assert.equal(status.appDistribution.artifact.available, true);
    assert.equal(status.appDistribution.artifact.fileName, 'storylock-android-host.apk');
    assert.equal(status.appDistribution.artifact.fileSizeBytes, fakeApk.length);
    assert.equal(status.appDistribution.artifact.versionName, '0.1.0');
    assert.equal(status.appDistribution.artifact.versionCode, '1');
    assert.equal(status.appDistribution.artifact.checksum, 'sha256:selftest');
    assert.equal(status.appDistribution.artifact.packageKind, 'debug');
    assert.equal(status.appDistribution.artifact.releaseChannel, 'internal');
    assert.equal(status.appDistribution.artifact.downloadStrategy, 'local_apk_file');

    const clientGateway = new StoryLockRemoteGateway({
      transport: createHttpRemoteTransport({
        endpoint: gatewayEndpoint,
        headers: {
          'x-storylock-shared-secret': sharedSecret,
        },
      }),
    });

    const signatureResult = await clientGateway.requestSignature({
      requestId: 'req-vercel-sign',
      nonce: '20001',
      eip712Nonce: '20001',
      expiry: Date.now() + 60_000,
      identityId: 'android-demo-001',
      keyId: 'wallet/main/private_key',
      algorithm: 'ed25519',
      payload: 'sign from vercel gateway',
      resourceId: 'wallet/main/private_key',
    });
    assert.equal(signatureResult.status, 'success');
    assert.equal(signatureResult.executionLocation, 'remote_gateway');
    assert.equal(signatureResult.result.privateKey, '[redacted]');
    assert.equal(signatureResult.result.signingKeyBytes, '[redacted]');
    assert.equal(signatureResult.auditMeta.remoteGateway, 'vercel');

    const passwordResult = await clientGateway.requestPasswordFill({
      requestId: 'req-vercel-password',
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

    console.log('StoryLock Vercel gateway + Android registration/relay/APK selftest passed.');
  } finally {
    relayWorkerRunning = false;
    await relayWorker;
    await vercelServer.stop();
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
