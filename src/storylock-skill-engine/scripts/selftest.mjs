import {
  ChallengeSigningAuthorizationSkill,
  LocalPasswordFillSkill,
  LOGIN_BINDING_MODE,
  StoryDraftAssistSkill,
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
  const draftSkill = new StoryDraftAssistSkill({
    async generator(input) {
      return { title: input.objective, summary: input.audience, tone: input.tone };
    },
  });
  const draft = await draftSkill.run({ objective: "obj" });
  assert(draft.title === "obj", "draft title mismatch");

  const fillSkill = new LocalPasswordFillSkill({ host });
  const fill = await fillSkill.run({
    identityId: "id",
    siteId: "generic_username_password",
    resourceCatalog,
    bindingMode: LOGIN_BINDING_MODE.TEMPLATE_ONLY,
  });
  assert(fill.mode === "local_password_fill", "fill mode mismatch");
  assert(fill.fields.length === 2, "fill fields mismatch");

  const signSkill = new ChallengeSigningAuthorizationSkill({
    host,
    async signer({ algorithm, payload }) {
      return `sig:${algorithm}:${payload.length}`;
    },
  });
  const sign = await signSkill.run({
    identityId: "id",
    keyId: "key",
    algorithm: "ed25519",
    challengeCode: new Uint8Array([1, 2, 3]),
    resourceId: "eth-main",
    primaryRole: "private_key",
    resourceCatalog,
  });
  assert(sign.mode === "challenge_signing_authorization", "sign mode mismatch");
  assert(sign.signature === "sig:ed25519:3", "signature mismatch");

  console.log("StoryLock skill-engine selftest passed.");
}

main().catch((error) => {
  console.error("StoryLock skill-engine selftest failed.");
  console.error(error);
  process.exitCode = 1;
});
