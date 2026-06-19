import {
  LocalPasswordFillSkill,
  LOGIN_BINDING_MODE,
  SignatureAuthorizationSkill,
  StoryDraftAssistSkill,
} from "../../../index.js";

const resourceCatalog = {
  version: 1,
  resources: [
    {
      resourceId: "generic-main",
      resourceKind: "website_account",
      providerId: "generic",
      bindings: [
        { role: "username", objectId: "credential/generic/main/username" },
        { role: "password", objectId: "credential/generic/main/password" },
      ],
    },
    {
      resourceId: "eth-main",
      resourceKind: "wallet_account",
      providerId: "ethereum",
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
  async submitChallengeAnswers(_identityId, challengeId, _answers) {
    return { authorization: { sessionId: `session:${challengeId}` } };
  },
  readSecretObject(_identityId, _sessionId, secretObjectId) {
    return new TextEncoder().encode(`value:${secretObjectId}`);
  },
};

async function main() {
  const draftSkill = new StoryDraftAssistSkill({
    async generator(input) {
      return {
        title: "Demo Draft",
        summary: `${input.objective} for ${input.audience}`,
        tone: input.tone,
      };
    },
  });

  const passwordFillSkill = new LocalPasswordFillSkill({ host });
  const signatureSkill = new SignatureAuthorizationSkill({
    host,
    async signer({ algorithm, payload, secretReference, attachments }) {
      return {
        kind: "demo_signature",
        algorithm,
        secretReference,
        attachmentReferences: attachments.map((attachment) => attachment.secretReference),
        signature: `sig:${algorithm}:${Array.from(payload).join("-")}`,
      };
    },
  });

  const draft = await draftSkill.run({
    objective: "Create a memorable founder story",
    audience: "solo entrepreneur",
    tone: "concrete and memorable",
  });

  const passwordFill = await passwordFillSkill.run({
    identityId: "founder_login",
    siteId: "generic_username_password",
    resourceCatalog,
    bindingMode: LOGIN_BINDING_MODE.TEMPLATE_ONLY,
    answers: [],
  });

  const signatureAuthorization = await signatureSkill.run({
    identityId: "founder_signing",
    keyId: "wallet-key",
    algorithm: "ed25519",
    payload: new Uint8Array([4, 5, 6]),
    resourceId: "eth-main",
    primaryRole: "private_key",
    resourceCatalog,
    attachments: [
      {
        attachmentId: "wallet-password",
        role: "keystore_password",
      },
    ],
    answers: [],
  });

  console.log("StoryLock migrated example 04 succeeded.");
  console.log(
    JSON.stringify(
      {
        draft,
        passwordFill,
        signatureAuthorization: {
          ...signatureAuthorization,
          payload: Array.from(signatureAuthorization.payload),
        },
      },
      null,
      2,
    ),
  );
}

main().catch((error) => {
  console.error("StoryLock migrated example 04 failed.");
  console.error(error);
  process.exitCode = 1;
});
