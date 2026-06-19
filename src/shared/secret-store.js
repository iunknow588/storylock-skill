import { randomBytes } from 'node:crypto';
import { execFileSync } from 'node:child_process';

function toBase64(value) {
  return Buffer.from(value).toString('base64');
}

function fromBase64(value) {
  return Buffer.from(String(value).trim(), 'base64');
}

function quotePowerShell(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

export class MemorySecretStore {
  constructor({ developmentMode = false, suppressWarning = false } = {}) {
    if (!developmentMode) {
      throw new Error('MemorySecretStore requires developmentMode=true and must not be used in production.');
    }
    this.developmentMode = developmentMode;
    this.kind = 'memory';
    this.secrets = new Map();
    if (!suppressWarning) {
      console.warn('StoryLock MemorySecretStore is for development only and must not be used in production.');
    }
  }

  getSecret(key) {
    const value = this.secrets.get(key);
    return value ? Buffer.from(value) : null;
  }

  setSecret(key, value) {
    this.secrets.set(key, Buffer.from(value));
  }

  deleteSecret(key) {
    this.secrets.delete(key);
  }

  listKeys(prefix = '') {
    return [...this.secrets.keys()].filter((key) => key.startsWith(prefix));
  }
}

export class WindowsCredentialSecretStore {
  constructor({ service = 'storylock', usernamePrefix = 'storylock' } = {}) {
    this.service = service;
    this.usernamePrefix = usernamePrefix;
  }

  target(key) {
    return `${this.service}/${key}`;
  }

  getSecret(key) {
    const script = [
      '$ErrorActionPreference = "Stop"',
      'if (-not (Get-Command Get-StoredCredential -ErrorAction SilentlyContinue)) { throw "CredentialManager PowerShell module is required" }',
      `$cred = Get-StoredCredential -Target ${quotePowerShell(this.target(key))}`,
      'if ($null -eq $cred) { exit 2 }',
      '$bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($cred.Password)',
      'try { [Console]::Out.Write([Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)) } finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }',
    ].join('; ');
    try {
      const output = execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], {
        encoding: 'utf8',
        windowsHide: true,
      });
      return fromBase64(output);
    } catch (error) {
      if (error.status === 2) {
        return null;
      }
      throw error;
    }
  }

  setSecret(key, value) {
    const script = [
      '$ErrorActionPreference = "Stop"',
      'if (-not (Get-Command New-StoredCredential -ErrorAction SilentlyContinue)) { throw "CredentialManager PowerShell module is required" }',
      `New-StoredCredential -Target ${quotePowerShell(this.target(key))} -UserName ${quotePowerShell(`${this.usernamePrefix}/${key}`)} -Password ${quotePowerShell(toBase64(value))} -Persist LocalMachine | Out-Null`,
    ].join('; ');
    execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], {
      stdio: 'pipe',
      windowsHide: true,
    });
  }

  deleteSecret(key) {
    const script = [
      '$ErrorActionPreference = "Stop"',
      'if (-not (Get-Command Remove-StoredCredential -ErrorAction SilentlyContinue)) { throw "CredentialManager PowerShell module is required" }',
      `Remove-StoredCredential -Target ${quotePowerShell(this.target(key))} -ErrorAction SilentlyContinue | Out-Null`,
    ].join('; ');
    execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], {
      stdio: 'pipe',
      windowsHide: true,
    });
  }

  listKeys() {
    throw new Error('WindowsCredentialSecretStore.listKeys is not supported by CredentialManager cmdlets');
  }

  checkAvailable() {
    const script = 'if (Get-Command Get-StoredCredential -ErrorAction SilentlyContinue) { "ok" } else { throw "CredentialManager PowerShell module is required" }';
    execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], {
      stdio: 'pipe',
      windowsHide: true,
    });
    return true;
  }
}

export class LinuxSecretServiceStore {
  constructor({ schema = 'storylock', service = 'storylock' } = {}) {
    this.schema = schema;
    this.service = service;
  }

  attributes(key) {
    return [this.schema, 'service', this.service, 'key', key];
  }

  getSecret(key) {
    try {
      const output = execFileSync('secret-tool', ['lookup', ...this.attributes(key).slice(1)], {
        encoding: 'utf8',
      });
      return fromBase64(output);
    } catch (error) {
      if (error.status === 1) {
        return null;
      }
      throw error;
    }
  }

  setSecret(key, value) {
    execFileSync('secret-tool', ['store', '--label', `StoryLock ${key}`, ...this.attributes(key).slice(1)], {
      input: toBase64(value),
      encoding: 'utf8',
    });
  }

  deleteSecret(key) {
    try {
      execFileSync('secret-tool', ['clear', ...this.attributes(key).slice(1)], {
        stdio: 'pipe',
      });
    } catch (error) {
      if (error.status !== 1) {
        throw error;
      }
    }
  }

  listKeys() {
    throw new Error('LinuxSecretServiceStore.listKeys is not supported by secret-tool lookup');
  }

  checkAvailable() {
    execFileSync('secret-tool', ['--version'], {
      stdio: 'pipe',
    });
    return true;
  }
}

export function createPlatformSecretStore({ platform = process.platform, allowMemoryFallback = false } = {}) {
  if (platform === 'win32') {
    return new WindowsCredentialSecretStore();
  }
  if (platform === 'linux') {
    return new LinuxSecretServiceStore();
  }
  if (allowMemoryFallback) {
    return new MemorySecretStore({ developmentMode: true });
  }
  throw new Error(`No platform SecretStore adapter configured for ${platform}`);
}

export function ensureMasterSalt(secretStore, key = 'storylock/masterSalt') {
  const existing = secretStore.getSecret(key);
  if (existing) {
    return existing;
  }
  const masterSalt = randomBytes(32);
  secretStore.setSecret(key, masterSalt);
  return masterSalt;
}
