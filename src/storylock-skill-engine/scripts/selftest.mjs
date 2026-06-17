import {
  LocalPasswordFillSkill,
  LOGIN_BINDING_MODE,
  SignatureAuthorizationSkill,
} from "../index.js";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const resourceCatalog = {
  version: 1,
  resources: [
    {
      resourceId: "generic-main",
      bindings: [
        { role: "username", objectId: "credential/generic/main/username" },
        { role: "password", objectId: "credential/generic/main/password" },
      ],
    },
    {
      resourceId: "eth-main",
      bindings: [
        { role: "private_key", objectId: "wallet/ethereum/main/private_key" },
        { role: "keystore_password", objectId: "wallet/ethereum/main/keystore_password" },
      ],
    },
  ],
};

const host = {
  async createChallenge(_identityId, scope) {
    return { challengeId: `challenge:${scope}`, scope };
  },
  async submitChallengeAnswers(_identityId, challengeId) {
    return { authorization: { sessionId: `session:${challengeId}` } };
  },
  readSecretObject(_identityId, _sessionId, secretObjectId) {
    return new TextEncoder().encode(`value:${secretObjectId}`);
  },
};

async function main() {
  const fillSkill = new LocalPasswordFillSkill({ host });
  const fill = await fillSkill.run({
    identityId: "id",
    siteId: "generic_username_password",
    resourceCatalog,
    bindingMode: LOGIN_BINDING_MODE.TEMPLATE_ONLY,
  });
  assert(fill.mode === "local_password_fill", "fill mode mismatch");
  assert(fill.fields.length === 2, "fill fields mismatch");

  const signSkill = new SignatureAuthorizationSkill({
    host,
    async signer({ algorithm, payload }) {
      return `sig:${algorithm}:${payload.length}`;
    },
  });
  const sign = await signSkill.run({
    identityId: "id",
    keyId: "key",
    algorithm: "ed25519",
    payload: new Uint8Array([1, 2, 3]),
    resourceId: "eth-main",
    primaryRole: "private_key",
    resourceCatalog,
  });
  assert(sign.mode === "signature_authorization", "sign mode mismatch");
  assert(sign.signature === "sig:ed25519:3", "signature mismatch");
  assert(typeof sign.signatureHash === "string" && sign.signatureHash.length === 64, "signature hash mismatch");
  assert(sign.auditMeta.authorizationId === sign.authorization.authorization.sessionId, "signature audit authorization mismatch");
  assert(sign.auditMeta.scope === sign.scope, "signature audit scope mismatch");
  assert(sign.auditMeta.resource === "wallet/ethereum/main/private_key", "signature audit resource mismatch");

  console.log("StoryLock skill-engine selftest passed.");
}

main().catch((error) => {
  console.error("StoryLock skill-engine selftest failed.");
  console.error(error);
  process.exitCode = 1;
});
