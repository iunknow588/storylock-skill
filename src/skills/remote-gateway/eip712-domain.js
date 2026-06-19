function ensureString(value, fieldName) {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return value.trim();
}

function isZeroAddress(value) {
  return /^0x0{40}$/i.test(value);
}

export function normalizeEip712Domain(domain = {}, defaults = {}) {
  const input = domain ?? {};
  const fallback = defaults ?? {};
  const version = ensureString(input.version ?? fallback.version ?? '1-placeholder', 'eip712.domain.version');
  const chainId = Number(input.chainId ?? fallback.chainId ?? 1);
  if (!Number.isInteger(chainId) || chainId < 0) {
    throw new Error('eip712.domain.chainId must be a non-negative integer');
  }
  const verifyingContract = ensureString(
    input.verifyingContract ?? fallback.verifyingContract ?? '0x0000000000000000000000000000000000000000',
    'eip712.domain.verifyingContract',
  );
  if (!/^0x[0-9a-fA-F]{40}$/.test(verifyingContract)) {
    throw new Error('eip712.domain.verifyingContract must be a 20-byte hex address');
  }
  const environment = ensureString(
    input.environment ?? fallback.environment ?? (version.includes('placeholder') ? 'demo' : 'production'),
    'eip712.domain.environment',
  );
  if (!['demo', 'test', 'production'].includes(environment)) {
    throw new Error('eip712.domain.environment must be demo, test, or production');
  }
  if (environment === 'production') {
    if (/placeholder/i.test(version)) {
      throw new Error('production EIP-712 domain must not use a placeholder version');
    }
    if (chainId === 0) {
      throw new Error('production EIP-712 domain must use a non-zero chainId');
    }
    if (isZeroAddress(verifyingContract)) {
      throw new Error('production EIP-712 domain must not use the zero verifyingContract');
    }
  }
  return {
    name: ensureString(input.name ?? fallback.name ?? 'StoryLock', 'eip712.domain.name'),
    version,
    chainId,
    verifyingContract,
    environment,
  };
}

export function createEip712DomainFromEnv(env = process.env, {
  prefix = 'STORYLOCK_EIP712_',
  defaultEnvironment = 'demo',
} = {}) {
  const environment = ensureString(
    env[`${prefix}ENVIRONMENT`] ?? env[`${prefix}ENV`] ?? defaultEnvironment,
    `${prefix}ENVIRONMENT`,
  );
  const input = {
    name: env[`${prefix}NAME`] ?? 'StoryLock',
    version: env[`${prefix}VERSION`],
    chainId: env[`${prefix}CHAIN_ID`],
    verifyingContract: env[`${prefix}VERIFYING_CONTRACT`],
    environment,
  };
  if (environment === 'production') {
    const missing = [
      ['VERSION', input.version],
      ['CHAIN_ID', input.chainId],
      ['VERIFYING_CONTRACT', input.verifyingContract],
    ].filter(([, value]) => value === undefined || value === null || String(value).trim() === '');
    if (missing.length > 0) {
      throw new Error(`production EIP-712 config requires ${missing.map(([name]) => `${prefix}${name}`).join(', ')}`);
    }
  }
  return normalizeEip712Domain(input);
}

export function createProductionEip712Domain({
  name = 'StoryLock',
  version,
  chainId,
  verifyingContract,
} = {}) {
  return normalizeEip712Domain(
    {
      name,
      version,
      chainId,
      verifyingContract,
      environment: 'production',
    },
    {
      name,
      version,
      chainId,
      verifyingContract,
      environment: 'production',
    },
  );
}

export function createDemoEip712Domain({
  name = 'StoryLock',
  version = '1-placeholder',
  chainId = 1,
  verifyingContract = '0x0000000000000000000000000000000000000000',
} = {}) {
  return normalizeEip712Domain({
    name,
    version,
    chainId,
    verifyingContract,
    environment: version.includes('placeholder') ? 'demo' : 'test',
  });
}
