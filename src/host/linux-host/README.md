# Yian Linux Host

This directory contains the Linux local host prototype for Yian / StoryLock.

Current implemented scope:

1. Node.js local HTTP host.
2. `GET /health`.
3. `GET /question-bank/status`.
4. `POST /question-bank/import`.
5. `POST /verify`.
6. `POST /authorize`.
7. `POST /execute`.
8. `POST /revoke`.
9. StoryLock Layer 2 challenge/session logic reused from `src/skills/local-story-access`.

Current security boundary:

1. Development checks use `MemorySecretStore` so the loop can run on CI and Windows workstations.
2. Linux production mode should run with `STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE=1`, which uses `secret-tool` / Secret Service through `createPlatformSecretStore({ platform: "linux" })`.
3. The current Linux host is a prototype and does not yet include tray UI, desktop autostart, system package scripts, object-storage upload, or signed Linux packages.

Local check:

```powershell
npm run test:linux-host
```

Run manually:

```powershell
node src/host/linux-host/server.mjs
```

Useful environment variables:

1. `STORYLOCK_LINUX_DATA_DIR`
2. `STORYLOCK_LINUX_HOST_PORT`
3. `STORYLOCK_LINUX_IDENTITY_ID`
4. `STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE`
5. `STORYLOCK_LINUX_DEVELOPMENT_MODE`
