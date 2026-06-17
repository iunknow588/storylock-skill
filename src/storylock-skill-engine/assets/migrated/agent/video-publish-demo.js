import { ValidationError } from "../runtime/errors.js";
import {
  LocalPasswordFillSkill,
  SignatureAuthorizationSkill,
} from "../skills/authorization-skills.js";

function ensureNonEmptyString(value, fieldName) {
  if (!value || typeof value !== "string") {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

export class VideoPublishAgentDemo {
  constructor({ host, loginSkill = null, signingSkill = null, signer = null }) {
    this.loginSkill = loginSkill ?? new LocalPasswordFillSkill({ host });
    this.signingSkill =
      signingSkill ??
      new SignatureAuthorizationSkill({
        host,
        signer:
          signer ??
          (({ algorithm, payload, secretReference, attachments }) => ({
            kind: "agent_local_demo_signature",
            algorithm,
            secretReference,
            attachmentReferences: attachments.map((attachment) => attachment.secretReference),
            signature: `${algorithm}:${Array.from(payload)
              .map((byte) => byte.toString(16).padStart(2, "0"))
              .join("")
              .slice(0, 24)}`,
          })),
      });
  }

  async createLoginAuthorization({
    identityId,
    siteId,
    resourceId = null,
    resourceCatalog = null,
    bindings = [],
    bindingMode = "template_with_overrides",
    answers = [],
  }) {
    return this.loginSkill.run({
      identityId,
      siteId: ensureNonEmptyString(siteId, "siteId"),
      resourceId,
      resourceCatalog,
      bindings,
      bindingMode,
      answers,
    });
  }

  async createSigningAuthorization({
    identityId,
    keyId,
    algorithm,
    payload,
    challengeCode = null,
    secretObjectId,
    resourceId = null,
    primaryRole = null,
    resourceCatalog = null,
    attachments = [],
    answers = [],
  }) {
    return this.signingSkill.run({
      identityId,
      keyId: ensureNonEmptyString(keyId, "keyId"),
      algorithm: ensureNonEmptyString(algorithm, "algorithm"),
      payload,
      challengeCode,
      secretObjectId: secretObjectId ?? null,
      resourceId,
      primaryRole,
      resourceCatalog,
      attachments,
      answers,
    });
  }

  async createPublishPlan({ identityId, login, signing }) {
    const loginResult = await this.createLoginAuthorization({
      identityId,
      ...login,
    });
    const signingResult = await this.createSigningAuthorization({
      identityId,
      ...signing,
    });
    return {
      login: loginResult,
      signing: signingResult,
      steps: [
        {
          action: "open_login_page",
          siteId: loginResult.siteId,
        },
        {
          action: "fill_login_form",
          siteId: loginResult.siteId,
          fieldNames: loginResult.fields.map((field) => field.fieldName),
        },
        {
          action: "submit_local_signature_authorization",
          keyId: signingResult.keyId,
          algorithm: signingResult.algorithm,
          signatureKind: signingResult.signature?.kind ?? "opaque_signature",
        },
      ],
    };
  }
}

export function createVideoPublishAgentDemo(options) {
  return new VideoPublishAgentDemo(options);
}
