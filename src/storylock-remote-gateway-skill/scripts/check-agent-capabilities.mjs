import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const manifest = JSON.parse(readFileSync(new URL('../assets/agent-capabilities.json', import.meta.url), 'utf8'));
const indexSource = readFileSync(new URL('../index.js', import.meta.url), 'utf8');

assert.equal(manifest.package, 'storylock-remote-gateway-skill');
assert.equal(manifest.agentBoundary.remoteFacing, true);
assert.equal(manifest.agentBoundary.localAuthorizationRequired, true);
assert.equal(manifest.agentBoundary.directSecretAccess, false);

const allowedNames = manifest.allowedCapabilities.map((capability) => capability.name);
assert.deepEqual(allowedNames, ['requestSignature', 'requestPasswordFill']);

const methodNames = [...indexSource.matchAll(/^\s+async\s+(request[A-Za-z0-9_]+)\(/gmu)].map((match) => match[1]);
assert.deepEqual(methodNames, allowedNames, 'agent capability manifest must match StoryLockRemoteGateway request methods');

const signature = manifest.allowedCapabilities.find((capability) => capability.name === 'requestSignature');
assert.equal(signature.requiredLocalStrength, 'high');
assert.equal(signature.localChallengeCells, 9);
assert.deepEqual(signature.allowedAlgorithms, ['ed25519', 'secp256k1']);

const passwordFill = manifest.allowedCapabilities.find((capability) => capability.name === 'requestPasswordFill');
assert.equal(passwordFill.requiredLocalStrength, 'medium');
assert.equal(passwordFill.localChallengeCells, 6);

for (const forbidden of manifest.forbiddenCapabilities) {
  assert.doesNotMatch(
    indexSource,
    new RegExp(`async\\s+${forbidden}\\s*\\(`, 'u'),
    `${forbidden} must not be exposed as a remote gateway method`,
  );
}

for (const field of ['password', 'privateKey', 'signingKeyBytes', 'secretValue', 'rawSecret', 'keyMaterial', 'answers']) {
  assert.ok(manifest.redactedFieldPatterns.includes(field), `${field} must be listed as redacted`);
}

console.log(JSON.stringify({
  status: 'passed',
  package: manifest.package,
  allowedCapabilities: allowedNames,
  forbiddenCapabilities: manifest.forbiddenCapabilities.length,
}, null, 2));
