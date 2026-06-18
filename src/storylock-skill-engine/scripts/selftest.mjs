import assert from "node:assert/strict";
import {
  LocalPasswordFillSkill,
  LOGIN_BINDING_MODE,
  SigningAuthorizationSkill,
  SignatureAuthorizationSkill,
  StoryDraftAssistSkill,
} from "../index.js";

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
  const draftSkill = new StoryDraftAssistSkill({
    async generator({ objective }) {
      return { objective };
    },
  });
  const draft = await draftSkill.run({
    objective: "selftest story draft",
  });
  assert(draft.objective === "selftest story draft", "draft skill export mismatch");

  const fillSkill = new LocalPasswordFillSkill({ host });
  const fill = await fillSkill.run({
    identityId: "id",
    siteId: "generic_username_password",
    resourceCatalog,
    bindingMode: LOGIN_BINDING_MODE.TEMPLATE_ONLY,
  });
  assert(fill.mode === "local_password_fill", "fill mode mismatch");
  assert(fill.fields.length === 2, "fill fields mismatch");

  const signingSkill = new SigningAuthorizationSkill({ host });
  const signingPackage = await signingSkill.run({
    identityId: "id",
    keyId: "key",
    algorithm: "ed25519",
    payload: "payload",
    resourceId: "eth-main",
    primaryRole: "private_key",
    resourceCatalog,
    includeKeyMaterial: true,
    attachments: [
      {
        attachmentId: "keystore-password",
        role: "keystore_password",
        includeMaterial: true,
      },
    ],
  });
  assert(signingPackage.signingKey.materialIncluded === true, "signing key summary mismatch");
  assert(signingPackage.signingKey.byteLength > 0, "signing key byte length mismatch");
  assert(typeof signingPackage.signingKey.sha256 === "string" && signingPackage.signingKey.sha256.length === 64, "signing key digest mismatch");
  assert(!("signingKeyBytes" in signingPackage), "signing key bytes must not be returned");
  assert(signingPackage.attachments[0].materialIncluded === true, "attachment summary mismatch");
  assert(signingPackage.attachments[0].byteLength > 0, "attachment byte length mismatch");
  assert(typeof signingPackage.attachments[0].sha256 === "string" && signingPackage.attachments[0].sha256.length === 64, "attachment digest mismatch");
  assert(!("secretValue" in signingPackage.attachments[0]), "attachment secret value must not be returned");
  assert(!("secretBytes" in signingPackage.attachments[0]), "attachment secret bytes must not be returned");

  await assert.rejects(
    () => signingSkill.run({
      identityId: "id",
      keyId: "key",
      algorithm: "ed25519",
      payload: "payload",
      resourceId: "eth-main",
      primaryRole: "private_key",
      resourceCatalog,
      attachments: "not-array",
    }),
    /attachments must be an array/,
  );

  await assert.rejects(
    () => signingSkill.run({
      identityId: "id",
      keyId: "key",
      algorithm: "ed25519",
      payload: "payload",
      resourceId: "eth-main",
      primaryRole: "private_key",
      resourceCatalog,
      attachments: Array.from({ length: 11 }, (_, index) => ({
        attachmentId: `attachment-${index}`,
        role: "keystore_password",
      })),
    }),
    /attachments must contain 10 items or fewer/,
  );

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

  const observedSecretBytes = new TextEncoder().encode("throwing-signature-key");
  let observedSignerKeyBytes = null;
  const throwingSignSkill = new SignatureAuthorizationSkill({
    host: {
      ...host,
      readSecretObject() {
        return observedSecretBytes;
      },
    },
    signer({ signingKeyBytes }) {
      observedSignerKeyBytes = signingKeyBytes;
      throw new Error("sync signer failure");
    },
  });
  await assert.rejects(
    () => throwingSignSkill.run({
      identityId: "id",
      keyId: "key",
      algorithm: "ed25519",
      payload: new Uint8Array([9, 8, 7]),
      resourceId: "eth-main",
      primaryRole: "private_key",
      resourceCatalog,
    }),
    /sync signer failure/,
  );
  assert(observedSignerKeyBytes instanceof Uint8Array, "signer key bytes should be observed before throw");
  assert(observedSecretBytes.every((byte) => byte === 0), "source signing key bytes must be zeroized");
  assert(observedSignerKeyBytes.every((byte) => byte === 0), "cloned signing key bytes must be zeroized after signer throw");

  console.log("StoryLock skill-engine selftest passed.");
}

main().catch((error) => {
  console.error("StoryLock skill-engine selftest failed.");
  console.error(error);
  process.exitCode = 1;
});
