import { createPlatformSecretStore } from '../../shared/secret-store.js';

const store = createPlatformSecretStore();

try {
  store.checkAvailable();
  console.log(`${store.constructor.name} is available.`);
} catch (error) {
  console.error(`${store.constructor.name} is not available: ${error.message}`);
  process.exitCode = 1;
}
