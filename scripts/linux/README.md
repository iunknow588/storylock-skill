# Linux Script Notes

This directory contains Linux-specific validation helpers.

Current mainline Linux verification command:

```powershell
node src/skills/local-story-access/scripts/check-secret-store.mjs
npm run test:linux-host
```

Current Linux package command:

```powershell
npm run package:linux-host
npm run package:linux-host:wsl
npm run build
npm run test:release
```

Current status:

1. Linux `SecretStore` verification is already available through the shared Node script above.
2. Linux local host loop verification is available through `scripts/linux/check_linux_host_loop.mjs`.
3. Linux prototype packaging is available through `scripts/release/linux/package_linux_host.mjs`.
4. WSL-assisted Debian packaging is available through `scripts/release/linux/package_linux_host_wsl.ps1`.
5. The generated `.deb`, when available, is still unsigned and should be treated as a prototype package.
