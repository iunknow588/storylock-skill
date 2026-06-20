package org.storylock.androidhost.host

import java.util.UUID
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.storylock.androidhost.security.AndroidKeystoreSigner
import org.storylock.androidhost.security.LocalChallengeItem
import org.storylock.androidhost.security.LocalConfirmationRequest
import org.storylock.androidhost.security.LocalConfirmationResult
import org.storylock.androidhost.security.LocalUserConfirmation
import org.storylock.androidhost.security.SecretStore

class StoryLockAndroidHostService(
  private val config: AndroidHostConfig,
  private val secretStore: SecretStore,
  private val localConfirmation: LocalUserConfirmation,
  private val runtime: LocalAuthorizationRuntime,
  private val connectivityProvider: (() -> JSONObject)? = null,
) : AndroidHostService {
  private val signer = AndroidKeystoreSigner()

  override fun health(): JSONObject {
    val (questionSetReady, strongestBits, requestCount) = runtime.healthSnapshot()
    return JSONObject()
      .put("status", "ok")
      .put("schemaVersion", "android-host-health-v1")
      .put(
        "layer1",
        JSONObject()
          .put("mode", "local_story_processing")
          .put("questionSetReady", questionSetReady)
          .put("strongestBasicChallengeBits", strongestBits),
      )
      .put(
        "layer2",
        JSONObject()
          .put("identityId", runtime.identityId())
          .put("questionSetVersion", runtime.questionSetVersion())
          .put("normalizationVersion", runtime.normalizationVersion())
          .put("activeQuestionCount", runtime.activeQuestionCount()),
      )
      .put(
        "stats",
        JSONObject()
          .put("requestCount", requestCount),
      )
      .put(
        "executors",
        JSONObject()
          .put("signature", "android_keystore_secret_store")
          .put("credential", "android_keystore_secret_store")
          .put("confirmation", "local_challenge_and_biometric_prompt"),
      )
      .put("connectivity", connectivityProvider?.invoke() ?: JSONObject.NULL)
  }

  override fun execute(request: JSONObject): JSONObject {
    val capability = request.optString("capability")
    val requestId = request.optString("requestId", "req-${UUID.randomUUID()}")

    if (capability !in setOf("requestSignature", "requestPasswordFill")) {
      return errorResponse(
        requestId = requestId,
        capability = if (capability.isBlank()) "requestSignature" else capability,
        code = "SLG-001",
        type = "validation_error",
        message = "unsupported capability",
        suggestedAction = "Use requestSignature or requestPasswordFill.",
      )
    }

    val payload = request.optJSONObject("payload") ?: JSONObject()
    val objectRef = payload.optString(
      if (capability == "requestSignature") "keyId" else "credentialRef",
    )
    val requiredStrength = runtime.resolveRequiredStrength(capability)
    val challenge = try {
      runtime.createChallenge(requiredStrength)
    } catch (error: ChallengeLockedException) {
      return errorResponse(
        requestId = requestId,
        capability = capability,
        code = "SLG-004",
        type = "challenge_locked",
        message = error.message ?: "challenge is locked",
        suggestedAction = "Wait until retryAfter before retrying the local challenge.",
        retryAfter = error.retryAfter,
      )
    }
    val cells = challenge.cells

    val confirmation = runBlocking {
      localConfirmation.confirm(
        LocalConfirmationRequest(
          title = "StoryLock Local Confirmation",
          subtitle = capability,
          reason = "Approve local execution for $capability on Android host",
          strongConfirmationRequired = capability == "requestSignature",
          challengeItems = cells.map { cell ->
            LocalChallengeItem(
              cellId = cell.cellId,
              questionId = cell.questionId,
              promptText = cell.promptText,
              expectedAnswer = cell.answer,
              position = cell.position,
            )
          },
          requiredChallengeAnswers = challenge.requiredThreshold,
        ),
      )
    }

    if (!confirmation.approved) {
      return confirmationErrorResponse(
        requestId = requestId,
        capability = capability,
        challenge = challenge,
        confirmation = confirmation,
      )
    }

    val answers = confirmation.challengeAnswers
    val verification = runtime.verifyChallengeAnswers(challenge, answers)
    if (!verification.approved) {
      return errorResponse(
        requestId = requestId,
        capability = capability,
        code = if (verification.lockUntil > 0L) "SLG-004" else "SLG-003",
        type = if (verification.lockUntil > 0L) "challenge_locked" else "challenge_failed",
        message = if (verification.lockUntil > 0L) {
          "challenge answer verification failed and the local challenge is now locked"
        } else {
          "challenge answer verification failed"
        },
        suggestedAction = if (verification.lockUntil > 0L) {
          "Wait until retryAfter before retrying the local challenge."
        } else {
          "Retry with valid local challenge answers."
        },
        challenge = challenge,
        retryAfter = verification.lockUntil.takeIf { it > 0L },
        extraErrorFields = mapOf(
          "matchedCount" to verification.matchedCount,
          "requiredThreshold" to verification.requiredThreshold,
          "failureCount" to verification.failureCount,
          "maxFailureCount" to verification.maxFailureCount,
        ),
      )
    }

    val session = runtime.issueSession(
      allowedAction = if (capability == "requestSignature") "signature" else "password_fill",
      objectRef = objectRef,
    )

    return when (capability) {
      "requestSignature" -> signatureResponse(requestId, capability, request, session, requiredStrength, challenge)
      "requestPasswordFill" -> passwordFillResponse(requestId, capability, request, session, requiredStrength, challenge)
      else -> errorResponse(
        requestId = requestId,
        capability = capability,
        code = "SLG-001",
        type = "validation_error",
        message = "unsupported capability",
        suggestedAction = "Use requestSignature or requestPasswordFill.",
        challenge = challenge,
      )
    }
  }

  private fun signatureResponse(
    requestId: String,
    capability: String,
    request: JSONObject,
    session: AuthorizationSession,
    requiredStrength: String,
    challenge: ChallengeSession,
  ): JSONObject {
    val keyId = request.optJSONObject("payload")?.optString("keyId").orEmpty()
    val alias = "storylock-signature-$keyId"
    val signingKey = signer.getOrCreateSigningKey(alias, keyId)
    val payload = request.optJSONObject("payload")?.toString() ?: request.toString()
    val signature = signer.sign(alias, payload.encodeToByteArray())
    return JSONObject()
      .put("requestId", requestId)
      .put("status", "success")
      .put("capability", capability)
      .put("executionLocation", "local")
      .put(
        "result",
        JSONObject()
          .put("authorizationId", session.authorizationId)
          .put("requiredStrength", requiredStrength)
          .put("challenge", challengeSummary(challenge))
          .put("algorithm", signingKey.getString("algorithm"))
          .put("requestedAlgorithm", request.optJSONObject("payload")?.optString("algorithm").orEmpty())
          .put("signatureFormat", signingKey.getString("signatureFormat"))
          .put("curve", signingKey.getString("curve"))
          .put("publicKeySpki", signingKey.getString("publicKeySpki"))
          .put("signature", signature)
          .put("keyId", signingKey.getString("keyId"))
          .put("privateKey", "android-keystore-local-only")
          .put("signingKeyBytes", "[android-keystore-protected]"),
      )
      .put("redactionLevel", "result_only")
      .put("retentionGranted", "result_only")
      .put(
        "auditMeta",
        JSONObject()
          .put("authorizationId", session.authorizationId)
          .put("challengeId", challenge.challengeId),
      )
      .put("error", JSONObject.NULL)
  }

  private fun passwordFillResponse(
    requestId: String,
    capability: String,
    request: JSONObject,
    session: AuthorizationSession,
    requiredStrength: String,
    challenge: ChallengeSession,
  ): JSONObject {
    val credentialRef = request.optJSONObject("payload")?.optString("credentialRef").orEmpty()
    val alias = "storylock-credential-$credentialRef"
    val credential = readOrCreateCredential(alias, credentialRef, request)
    return JSONObject()
      .put("requestId", requestId)
      .put("status", "success")
      .put("capability", capability)
      .put("executionLocation", "local")
      .put(
        "result",
        JSONObject()
          .put("authorizationId", session.authorizationId)
          .put("requiredStrength", requiredStrength)
          .put("challenge", challengeSummary(challenge))
          .put("credentialRef", credential.getString("credentialRef"))
          .put("username", credential.getString("username"))
          .put("password", credential.getString("password"))
          .put("targetOrigin", credential.getString("targetOrigin")),
      )
      .put("redactionLevel", "result_only")
      .put("retentionGranted", "audit_meta_only")
      .put(
        "auditMeta",
        JSONObject()
          .put("authorizationId", session.authorizationId)
          .put("challengeId", challenge.challengeId),
      )
      .put("error", JSONObject.NULL)
  }

  private fun readOrCreateCredential(alias: String, credentialRef: String, request: JSONObject): JSONObject {
    val existing = secretStore.getSecret(alias)
    if (existing != null) {
      return JSONObject(existing.toString(Charsets.UTF_8))
    }
    val payload = request.optJSONObject("payload")
    val created = JSONObject()
      .put("credentialRef", credentialRef)
      .put("targetOrigin", payload?.optString("targetOrigin").orEmpty())
      .put("username", "android-user")
      .put("password", "android-keystore-local-only-${UUID.randomUUID()}")
      .put("storage", "android_keystore_secret_store")
      .put("createdAt", System.currentTimeMillis())
    secretStore.setSecret(alias, created.toString().encodeToByteArray())
    return created
  }

  private fun challengeSummary(challenge: ChallengeSession): JSONObject {
    return JSONObject()
      .put("challengeId", challenge.challengeId)
      .put("requiredStrength", challenge.requiredStrength)
      .put("requiredCells", challenge.requiredCells)
      .put("requiredThreshold", challenge.requiredThreshold)
      .put("failureCount", challenge.failureCount)
      .put("maxFailureCount", challenge.maxFailureCount)
      .put("lockUntil", if (challenge.lockUntil > 0L) challenge.lockUntil else JSONObject.NULL)
      .put("questionSetVersion", challenge.questionSetVersion)
      .put(
        "cells",
        challenge.cells.map { cell ->
          JSONObject()
            .put("cellId", cell.cellId)
            .put("position", cell.position)
            .put("questionId", cell.questionId)
            .put("promptText", cell.promptText)
        },
      )
  }

  private fun confirmationErrorResponse(
    requestId: String,
    capability: String,
    challenge: ChallengeSession,
    confirmation: LocalConfirmationResult,
  ): JSONObject {
    val (code, type, suggestedAction) = when (confirmation.failureType) {
      "host_unavailable" -> Triple("SLG-010", "host_unavailable", "Attach a foreground activity before retrying local confirmation.")
      "challenge_cancelled" -> Triple("SLG-003", "challenge_cancelled", "Retry and complete the local challenge.")
      "challenge_failed" -> Triple("SLG-003", "challenge_failed", "Retry and answer every local challenge cell correctly.")
      "biometric_unavailable" -> Triple("SLG-010", "biometric_unavailable", "Enable biometric or device credential confirmation on the Android host.")
      "biometric_cancelled" -> Triple("SLG-003", "biometric_cancelled", "Retry and complete biometric confirmation.")
      "biometric_failed" -> Triple("SLG-003", "biometric_failed", "Retry after successful biometric confirmation.")
      else -> Triple("SLG-003", "authorization_failed", "Retry after local user confirmation.")
    }
    return errorResponse(
      requestId = requestId,
      capability = capability,
      code = code,
      type = type,
      message = confirmation.reason ?: "local confirmation denied",
      suggestedAction = suggestedAction,
      challenge = challenge,
    )
  }

  private fun errorResponse(
    requestId: String,
    capability: String,
    code: String,
    type: String,
    message: String,
    suggestedAction: String,
    challenge: ChallengeSession? = null,
    retryAfter: Long? = null,
    extraErrorFields: Map<String, Any?> = emptyMap(),
  ): JSONObject {
    val errorObject = JSONObject()
      .put("code", code)
      .put("type", type)
      .put("message", message)
      .put("suggestedAction", suggestedAction)
      .put("challenge", challenge?.let { challengeSummary(it) } ?: JSONObject.NULL)
      .put("retryable", code == "SLG-003" || code == "SLG-004")
    if (retryAfter != null) {
      errorObject.put("retryAfter", retryAfter)
    }
    extraErrorFields.forEach { (key, value) ->
      errorObject.put(key, value ?: JSONObject.NULL)
    }
    return JSONObject()
      .put("requestId", requestId)
      .put("status", "error")
      .put("capability", capability)
      .put("executionLocation", "local")
      .put("result", JSONObject.NULL)
      .put("redactionLevel", "full")
      .put("retentionGranted", "audit_meta_only")
      .put(
        "auditMeta",
        JSONObject()
          .put("challengeId", challenge?.challengeId ?: JSONObject.NULL)
          .put("retryAfter", retryAfter ?: JSONObject.NULL)
          .put("timestamp", System.currentTimeMillis()),
      )
      .put("error", errorObject)
  }
}
