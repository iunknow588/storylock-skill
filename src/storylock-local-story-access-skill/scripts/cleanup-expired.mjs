import { MemorySecretStore, createPlatformSecretStore } from '../../shared/secret-store.js';
import { createAccessHost } from '../access-host.js';

function readArg(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) {
    return null;
  }
  return process.argv[index + 1] ?? null;
}

const dbPath = readArg('--db') ?? ':memory:';
const positionalBatchSize = process.argv.slice(2).find((arg) => /^\d+$/.test(arg));
const batchSize = Number(readArg('--batch-size') ?? positionalBatchSize ?? 1000);
const usePlatformSecretStore = process.argv.includes('--use-platform-secret-store');
const useDevelopmentMemoryStore = process.argv.includes('--development-memory-secret-store');

if (dbPath !== ':memory:' && !usePlatformSecretStore && !useDevelopmentMemoryStore) {
  console.error('Persistent cleanup requires --use-platform-secret-store or --development-memory-secret-store.');
  process.exit(1);
}

const secretStore = usePlatformSecretStore
  ? createPlatformSecretStore()
  : new MemorySecretStore({ developmentMode: true, suppressWarning: true });

const host = createAccessHost({
  dbPath,
  secretStore,
});

try {
  const result = host.cleanupExpired(Date.now(), { batchSize });
  console.log(JSON.stringify({
    status: 'success',
    dbPath,
    batchSize: Math.max(1, Math.min(Number(batchSize) || 1000, 1000)),
    result,
  }, null, 2));
} finally {
  host.close?.();
}
