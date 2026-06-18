import { createPlatformSecretStore } from '../../shared/secret-store.js';

function recommendationFor({ platform, status, reason }) {
  if (status === 'available') {
    return 'Use --use-platform-secret-store for persistent StoryLock hosts on this platform.';
  }
  if (platform === 'win32') {
    return 'Install and enable the CredentialManager PowerShell module, then rerun this check.';
  }
  if (platform === 'linux') {
    return 'Install libsecret secret-tool and ensure a Secret Service provider is available in the user session.';
  }
  if (platform === 'darwin') {
    return 'macOS Keychain is not supported in this phase; use an injected SecretStore adapter or keep this host out of production.';
  }
  return `Provide a custom persistent SecretStore adapter before production use.${reason ? ` Last error: ${reason}` : ''}`;
}

function inspectSecretStore() {
  const platform = process.platform;
  try {
    const store = createPlatformSecretStore({ platform });
    const adapter = store.constructor?.name ?? 'UnknownSecretStore';
    try {
      store.checkAvailable();
      return {
        platform,
        adapter,
        status: 'available',
        available: true,
        productionReady: true,
        reason: null,
        recommendation: recommendationFor({ platform, status: 'available' }),
      };
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error);
      return {
        platform,
        adapter,
        status: 'unavailable',
        available: false,
        productionReady: false,
        reason,
        recommendation: recommendationFor({ platform, status: 'unavailable', reason }),
      };
    }
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    return {
      platform,
      adapter: null,
      status: 'unsupported',
      available: false,
      productionReady: false,
      reason,
      recommendation: recommendationFor({ platform, status: 'unsupported', reason }),
    };
  }
}

const report = inspectSecretStore();
console.log(JSON.stringify(report, null, 2));

if (report.available) {
  console.log(`${report.adapter} is available on ${report.platform}.`);
} else {
  console.error(`${report.status} on ${report.platform}${report.adapter ? ` (${report.adapter})` : ''}: ${report.reason}`);
  console.error(report.recommendation);
  process.exitCode = 1;
}
