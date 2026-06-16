import { createCipheriv, createDecipheriv, createHmac, hkdfSync, randomBytes } from 'node:crypto';

export function hmacSha256Hex(key, value) {
  return createHmac('sha256', key).update(String(value), 'utf8').digest('hex');
}

export function deriveHkdfSha256(keyMaterial, { salt, info, length = 32 }) {
  return Buffer.from(hkdfSync('sha256', keyMaterial, salt, info, length));
}

export function encryptAes256Gcm(plaintext, key, aad = '') {
  if (!Buffer.isBuffer(key) || key.length !== 32) {
    throw new Error('AES-256-GCM key must be a 32-byte Buffer');
  }
  const nonce = randomBytes(12);
  const cipher = createCipheriv('aes-256-gcm', key, nonce);
  if (aad) {
    cipher.setAAD(Buffer.from(aad, 'utf8'));
  }
  const ciphertext = Buffer.concat([
    cipher.update(Buffer.isBuffer(plaintext) ? plaintext : Buffer.from(String(plaintext), 'utf8')),
    cipher.final(),
  ]);
  return {
    algorithm: 'AES-256-GCM',
    nonce: nonce.toString('base64url'),
    ciphertext: ciphertext.toString('base64url'),
    authTag: cipher.getAuthTag().toString('base64url'),
  };
}

export function decryptAes256Gcm(envelope, key, aad = '') {
  if (!Buffer.isBuffer(key) || key.length !== 32) {
    throw new Error('AES-256-GCM key must be a 32-byte Buffer');
  }
  if (envelope?.algorithm !== 'AES-256-GCM') {
    throw new Error('unsupported encryption envelope');
  }
  const decipher = createDecipheriv(
    'aes-256-gcm',
    key,
    Buffer.from(envelope.nonce, 'base64url'),
  );
  if (aad) {
    decipher.setAAD(Buffer.from(aad, 'utf8'));
  }
  decipher.setAuthTag(Buffer.from(envelope.authTag, 'base64url'));
  return Buffer.concat([
    decipher.update(Buffer.from(envelope.ciphertext, 'base64url')),
    decipher.final(),
  ]);
}
