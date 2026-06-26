# Yian Linux Host

This directory contains the Linux local host prototype for Yian / StoryLock.

Current implemented scope:

1. Node.js local HTTP host.
2. `GET /health`.
3. `GET /question-bank/status`.
4. `GET /story-template/status`.
5. `GET /story-templates`.
6. `POST /question-bank/import`.
7. `POST /verify`.
8. `POST /authorize`.
9. `POST /execute`.
10. `POST /revoke`.
11. `GET /permission-summary`.
12. StoryLock Layer 2 challenge/session logic reused from `src/skills/local-story-access`.
13. Optional StoryLock package loading through `src/shared/storylock-package`.
14. asset-backed `assets/story-drafts/manifest.json` plus exactly three unified story-draft files for external initialization templates.
15. the three built-in story drafts are examples only; users should rewrite them into more private and less guessable personal stories.

Current security boundary:

1. Development checks use `MemorySecretStore` so the loop can run on CI and Windows workstations.
2. Linux production mode should run with `STORYLOCK_LINUX_USE_PLATFORM_SECRET_STORE=1`, which uses `secret-tool` / Secret Service through `createPlatformSecretStore({ platform: "linux" })`.
3. `GET /permission-summary` returns only package metadata and redacted permission summary. It must not return story raw text, answers, passwords, private keys, or `signingKeyBytes`.
4. The current Linux host is a prototype and does not yet include tray UI, desktop autostart, system package scripts, object-storage upload, or signed Linux packages.

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
6. `STORYLOCK_LINUX_STORYLOCK_PACKAGE_DIR`
