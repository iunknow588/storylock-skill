import { createHash } from "node:crypto";
import { ValidationError } from "../runtime/errors.js";
import { resolveSecretReference } from "../runtime/resource-catalog.js";
import {
  LOGIN_BINDING_MODE,
  resolveLoginFormBindings,
} from "../runtime/templates.js";

function ensureFunction(value, fieldName) {
  if (typeof value !== "function") {
    throw new ValidationError(`${fieldName} must be a function`);
  }
  return value;
}

function ensureNonEmptyString(value, fieldName) {
  if (!value || typeof value !== "string") {
    throw new ValidationError(`${fieldName} must be a non-empty string`);
  }
  return value;
}

function uniqueValues(values) {
  return [...new Set(values)];
}

function ensureHost(host) {
  return {
    createChallenge: ensureFunction(
      host?.createChallenge,
      "host.createChallenge",
    ).bind(host),
    submitChallengeAnswers: ensureFunction(
      host?.submitChallengeAnswers,
      "host.submitChallengeAnswers",
    ).bind(host),
    readSecretObject: ensureFunction(
      host?.readSecretObject,
      "host.readSecretObject",
    ).bind(host),
    recordAudit:
      typeof host?.recordAudit === "function"
        ? host.recordAudit.bind(host)
        : null,
  };
}

function ensureSigner(signer) {
  if (typeof signer === "function") {
    return signer;
  }
  if (typeof signer?.sign === "function") {
    return signer.sign.bind(signer);
  }
  throw new ValidationError("signer must be a function or expose sign()");
}

function determineLoginScope(bindings) {
  return uniqueValues(bindings.map((binding) => binding.secretObjectId)).length > 1
    ? "vault_read_batch"
    : "vault_read_basic";
}

function determineSigningScope({ includeKeyMaterial, secretObjectId, attachments = [] }) {
  const objectIds = uniqueValues([
    secretObjectId,
    ...attachments.map((attachment) => attachment.secretObjectId),
  ]);
  if (includeKeyMaterial || objectIds.length > 1) {
    return "vault_read_batch";
  }
  return "vault_read_basic";
}

function decodeSecretValue(secretBytes) {
  if (!(secretBytes instanceof Uint8Array)) {
    throw new ValidationError("secretBytes must be a Uint8Array");
  }
  return new TextDecoder().decode(secretBytes);
}

function cloneSecretBytes(secretBytes, fieldName) {
  if (!(secretBytes instanceof Uint8Array)) {
    throw new ValidationError(`${fieldName} must be a Uint8Array`);
  }
  return new Uint8Array(secretBytes);
}

function zeroizeBytes(secretBytes) {
  if (secretBytes instanceof Uint8Array) {
    secretBytes.fill(0);
  }
}

function normalizeBinaryPayload(payload, fieldName) {
  if (payload instanceof Uint8Array) {
    return payload;
  }
  if (typeof payload === "string") {
    return new TextEncoder().encode(payload);
  }
  if (Array.isArray(payload)) {
    return new Uint8Array(payload);
  }
  throw new ValidationError(`${fieldName} must be a Uint8Array, string, or byte array`);
}

function stableAuditStringify(value) {
  if (value instanceof Uint8Array) {
    return JSON.stringify(Array.from(value));
  }
  if (Array.isArray(value)) {
    return `[${value.map(stableAuditStringify).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableAuditStringify(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function sha256Hex(value) {
  return createHash("sha256").update(stableAuditStringify(value)).digest("hex");
}

async function readSecretObject(host, identityId, sessionId, secretObjectId) {
  return Promise.resolve(host.readSecretObject(identityId, sessionId, secretObjectId));
}

function resolveAuthorizationSession(authorizationResult) {
  const sessionHolder = authorizationResult?.authorization ?? authorizationResult;
  const sessionId = sessionHolder?.sessionId;
  if (!sessionId || typeof sessionId !== "string") {
    throw new ValidationError("authorization result must include a sessionId");
  }
  return sessionId;
}

export class LoginAuthorizationSkill {
  constructor({ host }) {
    this.host = ensureHost(host);
  }

  skillId() {
    return "login_authorization";
  }

  async run({
    identityId,
    siteId,
    resourceId = null,
    resourceCatalog = null,
    bindings = [],
    bindingMode = LOGIN_BINDING_MODE.TEMPLATE_WITH_OVERRIDES,
    answers = [],
  }) {
    const normalizedIdentityId = ensureNonEmptyString(identityId, "identityId");
    const normalizedSiteId = ensureNonEmptyString(siteId, "siteId");
    const resolvedBindings = resolveLoginFormBindings({
      siteId: normalizedSiteId,
      resourceId,
      resourceCatalog,
      bindings,
      bindingMode,
    });
    const scope = determineLoginScope(resolvedBindings);
    const challenge = await this.host.createChallenge(normalizedIdentityId, scope);
    const authorization = await this.host.submitChallengeAnswers(
      normalizedIdentityId,
      challenge.challengeId,
      Array.isArray(answers) ? answers : [],
    );
    const sessionId = resolveAuthorizationSession(authorization);
    const fields = await Promise.all(
      resolvedBindings.map(async (binding) => ({
        fieldName: binding.fieldName,
        value: decodeSecretValue(
          await readSecretObject(
            this.host,
            normalizedIdentityId,
            sessionId,
            binding.secretObjectId,
          ),
        ),
        secretObjectId: binding.secretObjectId,
      })),
    );

    return {
      challenge,
      authorization,
      siteId: normalizedSiteId,
      scope,
      fields,
    };
  }
}

export class LocalPasswordFillSkill {
  constructor({ host }) {
    this.loginSkill = new LoginAuthorizationSkill({ host });
  }

  skillId() {
    return "local_password_fill";
  }

  async run(input) {
    const result = await this.loginSkill.run(input);
    return {
      mode: "local_password_fill",
      siteId: result.siteId,
      scope: result.scope,
      challenge: result.challenge,
      authorization: result.authorization,
      fields: result.fields,
    };
  }
}

function normalizeSigningAttachment(attachment, resourceId, resourceCatalog) {
  if (!attachment || typeof attachment !== "object") {
    throw new ValidationError("attachment must be an object");
  }
  return {
    attachmentId: ensureNonEmptyString(attachment.attachmentId, "attachmentId"),
    includeMaterial: Boolean(attachment.includeMaterial),
    secretObjectId: resolveSecretReference({
      resourceCatalog,
      resourceId: attachment.resourceId ?? resourceId ?? null,
      role: attachment.role ?? null,
      secretObjectId: attachment.secretObjectId ?? null,
      fieldName: `attachment ${attachment.attachmentId}`,
    }),
  };
}

export class SigningAuthorizationSkill {
  constructor({ host }) {
    this.host = ensureHost(host);
  }

  skillId() {
    return "signing_authorization";
  }

  async run({
    identityId,
    keyId,
    algorithm,
    payload,
    secretObjectId,
    resourceId = null,
    primaryRole = null,
    resourceCatalog = null,
    includeKeyMaterial = false,
    attachments = [],
    answers = [],
  }) {
    const normalizedIdentityId = ensureNonEmptyString(identityId, "identityId");
    const normalizedKeyId = ensureNonEmptyString(keyId, "keyId");
    const normalizedAlgorithm = ensureNonEmptyString(algorithm, "algorithm");
    const normalizedSecretObjectId = resolveSecretReference({
      resourceCatalog,
      resourceId,
      role: primaryRole,
      secretObjectId,
      fieldName: "signing",
    });
    const normalizedAttachments = Array.isArray(attachments)
      ? attachments.map((attachment) =>
          normalizeSigningAttachment(attachment, resourceId, resourceCatalog),
        )
      : [];
    const scope = determineSigningScope({
      includeKeyMaterial,
      secretObjectId: normalizedSecretObjectId,
      attachments: normalizedAttachments,
    });
    const challenge = await this.host.createChallenge(normalizedIdentityId, scope);
    const authorization = await this.host.submitChallengeAnswers(
      normalizedIdentityId,
      challenge.challengeId,
      Array.isArray(answers) ? answers : [],
    );
    const sessionId = resolveAuthorizationSession(authorization);
    const signingKeyBytes = await readSecretObject(
      this.host,
      normalizedIdentityId,
      sessionId,
      normalizedSecretObjectId,
    );

    return {
      challenge,
      authorization,
      keyId: normalizedKeyId,
      algorithm: normalizedAlgorithm,
      payload,
      scope,
      secretReference: normalizedSecretObjectId,
      signingKey: includeKeyMaterial ? signingKeyBytes : null,
      attachments: await Promise.all(
        normalizedAttachments.map(async (attachment) => ({
          attachmentId: attachment.attachmentId,
          secretReference: attachment.secretObjectId,
          secretValue: attachment.includeMaterial
            ? decodeSecretValue(
                await readSecretObject(
                  this.host,
                  normalizedIdentityId,
                  sessionId,
                  attachment.secretObjectId,
                ),
              )
            : null,
        })),
      ),
    };
  }
}

export class ChallengeSigningAuthorizationSkill {
  constructor({ host, signer }) {
    this.host = ensureHost(host);
    this.signer = ensureSigner(signer);
  }

  skillId() {
    return "challenge_signing_authorization";
  }

  async run({
    identityId,
    keyId,
    algorithm,
    payload = null,
    challengeCode = null,
    secretObjectId,
    resourceId = null,
    primaryRole = null,
    resourceCatalog = null,
    attachments = [],
    answers = [],
  }) {
    const normalizedIdentityId = ensureNonEmptyString(identityId, "identityId");
    const normalizedKeyId = ensureNonEmptyString(keyId, "keyId");
    const normalizedAlgorithm = ensureNonEmptyString(algorithm, "algorithm");
    const normalizedSecretObjectId = resolveSecretReference({
      resourceCatalog,
      resourceId,
      role: primaryRole,
      secretObjectId,
      fieldName: "challenge signing",
    });
    const normalizedAttachments = Array.isArray(attachments)
      ? attachments.map((attachment) =>
          normalizeSigningAttachment(attachment, resourceId, resourceCatalog),
        )
      : [];
    const scope = determineSigningScope({
      includeKeyMaterial: true,
      secretObjectId: normalizedSecretObjectId,
      attachments: normalizedAttachments,
    });
    const challengePayload = payload ?? challengeCode;
    const normalizedPayload = normalizeBinaryPayload(
      challengePayload,
      payload == null ? "challengeCode" : "payload",
    );
    const challenge = await this.host.createChallenge(normalizedIdentityId, scope);
    const authorization = await this.host.submitChallengeAnswers(
      normalizedIdentityId,
      challenge.challengeId,
      Array.isArray(answers) ? answers : [],
    );
    const sessionId = resolveAuthorizationSession(authorization);
    const signingKeyBytes = cloneSecretBytes(
      await readSecretObject(
        this.host,
        normalizedIdentityId,
        sessionId,
        normalizedSecretObjectId,
      ),
      "signingKeyBytes",
    );
    const attachmentMaterials = await Promise.all(
      normalizedAttachments.map(async (attachment) => ({
        attachmentId: attachment.attachmentId,
        secretReference: attachment.secretObjectId,
        secretBytes: cloneSecretBytes(
          await readSecretObject(
            this.host,
            normalizedIdentityId,
            sessionId,
            attachment.secretObjectId,
          ),
          `attachment ${attachment.attachmentId}.secretBytes`,
        ),
      })),
    );

    try {
      const signature = await Promise.resolve(
        this.signer({
          keyId: normalizedKeyId,
          algorithm: normalizedAlgorithm,
          payload: normalizedPayload,
          secretReference: normalizedSecretObjectId,
          signingKeyBytes,
          attachments: attachmentMaterials.map((attachment) => ({
            attachmentId: attachment.attachmentId,
            secretReference: attachment.secretReference,
            secretBytes: attachment.secretBytes,
          })),
        }),
      );
      const sessionId = resolveAuthorizationSession(authorization);
      const signatureHash = sha256Hex(signature);
      this.host.recordAudit?.("signature_authorized", {
        identityId: normalizedIdentityId,
        storyObjectId: normalizedSecretObjectId,
        result: "success",
        meta: {
          authorizationId: sessionId,
          keyId: normalizedKeyId,
          algorithm: normalizedAlgorithm,
          scope,
          resource: normalizedSecretObjectId,
          signatureHash,
          attachmentReferences: attachmentMaterials.map((attachment) => ({
            attachmentId: attachment.attachmentId,
            secretReference: attachment.secretReference,
          })),
        },
      });

      return {
        mode: "challenge_signing_authorization",
        challenge,
        authorization,
        authorizationId: sessionId,
        keyId: normalizedKeyId,
        algorithm: normalizedAlgorithm,
        payload: normalizedPayload,
        scope,
        secretReference: normalizedSecretObjectId,
        signature,
        signatureHash,
        attachments: attachmentMaterials.map((attachment) => ({
          attachmentId: attachment.attachmentId,
          secretReference: attachment.secretReference,
        })),
        auditMeta: {
          authorizationId: sessionId,
          scope,
          resource: normalizedSecretObjectId,
          signatureHash,
        },
      };
    } finally {
      zeroizeBytes(signingKeyBytes);
      attachmentMaterials.forEach((attachment) => zeroizeBytes(attachment.secretBytes));
    }
  }
}

export class SignatureAuthorizationSkill extends ChallengeSigningAuthorizationSkill {
  skillId() {
    return "signature_authorization";
  }

  async run(input) {
    const result = await super.run(input);
    return {
      ...result,
      mode: "signature_authorization",
    };
  }
}

export { LOGIN_BINDING_MODE };
