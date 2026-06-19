import { createEip712DomainFromEnv } from '../index.js';

function readArg(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) {
    return null;
  }
  return process.argv[index + 1] ?? null;
}

const environment = readArg('--environment') ?? readArg('--env') ?? process.env.STORYLOCK_EIP712_ENVIRONMENT ?? process.env.STORYLOCK_EIP712_ENV ?? 'demo';
const prefix = readArg('--prefix') ?? 'STORYLOCK_EIP712_';

try {
  const domain = createEip712DomainFromEnv({
    ...process.env,
    [`${prefix}ENVIRONMENT`]: environment,
  }, { prefix });
  const productionReady = domain.environment === 'production';
  console.log(JSON.stringify({
    status: 'success',
    productionReady,
    domain,
    recommendation: productionReady
      ? 'Use this domain only with the matching deployed verifier and chain.'
      : 'Demo/test domains are not production chain verification evidence.',
  }, null, 2));
} catch (error) {
  console.error(JSON.stringify({
    status: 'error',
    productionReady: false,
    environment,
    prefix,
    reason: error instanceof Error ? error.message : String(error),
    recommendation: 'Set STORYLOCK_EIP712_VERSION, STORYLOCK_EIP712_CHAIN_ID, and STORYLOCK_EIP712_VERIFYING_CONTRACT for production.',
  }, null, 2));
  process.exitCode = 1;
}
