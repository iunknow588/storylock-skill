import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createReadStream, existsSync } from 'node:fs';
import { stat } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import gatewayHandler from '../../web-api/storylock-gateway.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const publicDir = join(root, 'release', 'web', 'public');
const gatewayRoutes = [
  /^\/api\/storylock-gateway$/u,
  /^\/api\/site\/.*$/u,
  /^\/app\/download(?:\/(?:android|windows|linux))?$/u,
  /^\/app\/bind$/u,
  /^\/app\/registrations$/u,
  /^\/android-host\/.*$/u,
  /^\/local-host\/.*$/u,
  /^\/download\/(?:android|windows|linux)-host$/u,
];

const contentTypes = new Map([
  ['.html', 'text/html; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.css', 'text/css; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.png', 'image/png'],
  ['.apk', 'application/vnd.android.package-archive'],
  ['.zip', 'application/zip'],
]);

function routeToGateway(pathname) {
  return gatewayRoutes.some((pattern) => pattern.test(pathname));
}

async function serveStatic(req, res, pathname) {
  const relativePath = pathname === '/' ? 'index.html' : pathname.replace(/^\/+/u, '');
  const absolutePath = normalize(join(publicDir, relativePath));
  if (!absolutePath.startsWith(publicDir) || !existsSync(absolutePath)) {
    res.statusCode = 404;
    res.end('not found');
    return;
  }
  const info = await stat(absolutePath);
  res.statusCode = 200;
  res.setHeader('content-type', contentTypes.get(extname(absolutePath).toLowerCase()) ?? 'application/octet-stream');
  res.setHeader('content-length', String(info.size));
  createReadStream(absolutePath).pipe(res);
}

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? '/', 'http://127.0.0.1');
    if (routeToGateway(url.pathname)) {
      await gatewayHandler(req, res);
      return;
    }
    await serveStatic(req, res, url.pathname);
  } catch (error) {
    res.statusCode = 500;
    res.setHeader('content-type', 'application/json; charset=utf-8');
    res.end(JSON.stringify({ status: 'error', message: error.message }));
  }
});

await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const { port } = server.address();
const baseUrl = `http://127.0.0.1:${port}`;

async function check(path, {
  expectedStatus = 200,
  expectedContentType = null,
} = {}) {
  const response = await fetch(`${baseUrl}${path}`, {
    headers: {
      accept: String(expectedContentType ?? '').includes('json') ? 'application/json' : '*/*',
    },
    redirect: 'manual',
  });
  assert.equal(response.status, expectedStatus, `${path} status`);
  if (expectedContentType) {
    assert.match(response.headers.get('content-type') ?? '', expectedContentType, `${path} content-type`);
  }
  return response;
}

try {
  await check('/', { expectedContentType: /text\/html/u });
  await check('/main.js', { expectedContentType: /text\/javascript/u });
  await check('/styles.css', { expectedContentType: /text\/css/u });
  await check('/api/storylock-gateway', { expectedContentType: /application\/json/u });
  const download = await check('/app/download', { expectedContentType: /application\/json/u });
  const downloadBody = await download.json();
  assert.equal(downloadBody.platforms.android.platform, 'android');
  assert.equal(downloadBody.platforms.windows.platform, 'windows');
  assert.equal(downloadBody.platforms.linux.platform, 'linux');
  assert.equal(downloadBody.platforms.windows.packageKind, 'zip');
  assert.match(downloadBody.platforms.linux.packageKind, /^(deb|tar\.gz|zip)$/u);
  assert.ok(downloadBody.platforms.linux.fileName.startsWith('yian-linux-host-'));
  assert.match(downloadBody.platforms.windows.checksum, /^sha256:[0-9a-f]{64}$/u);
  assert.match(downloadBody.platforms.linux.checksum, /^sha256:[0-9a-f]{64}$/u);
  await check('/app/download/windows', { expectedContentType: /application\/zip/u });
  await check('/app/download/android', { expectedContentType: /application\/vnd\.android\.package-archive/u });
  await check('/app/download/linux', { expectedContentType: /application\/octet-stream|application\/zip/u });
  await check('/app/bind', { expectedContentType: /application\/json/u });
  await check('/app/registrations', { expectedContentType: /application\/json/u });
  console.log(JSON.stringify({
    status: 'passed',
    baseUrl,
    checks: 10,
  }, null, 2));
} finally {
  await new Promise((resolve) => server.close(resolve));
}
