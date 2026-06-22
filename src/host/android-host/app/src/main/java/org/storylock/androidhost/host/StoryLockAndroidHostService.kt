package org.storylock.androidhost.host

import java.util.UUID
import kotlinx.coroutines.runBlocking
import org.json.JSONArray
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
  private val storyLockPackageRepository: AndroidStoryLockPackageRepository,
  private val connectivityProvider: (() -> JSONObject)? = null,
) : AndroidHostService {
  private val signer = AndroidKeystoreSigner()

  override fun health(): JSONObject {
    val (questionSetReady, strongestBits, requestCount) = runtime.healthSnapshot()
    val permissionSummary = storyLockPackageRepository.loadPermissionSummary()
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
        "storyLockPackage",
        JSONObject()
          .put("resourceCatalogAsset", AndroidStoryLockPackageRepository.DEFAULT_RESOURCE_CATALOG_ASSET_NAME)
          .put("resources", permissionSummary.resources)
          .put("permissionObjects", permissionSummary.permissionObjects),
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

  override fun permissionSummary(): JSONObject {
    val summary = storyLockPackageRepository.loadPermissionSummary()
    return JSONObject()
      .put("requestId", "req-${UUID.randomUUID()}")
      .put("status", "success")
      .put("capability", "permissionSummary")
      .put("executionLocation", "local")
      .put(
        "result",
        JSONObject()
          .put("packageId", summary.packageId ?: JSONObject.NULL)
          .put("resources", summary.resources)
          .put("permissionObjects", summary.permissionObjects)
          .put("permissionSummary", summary.permissionSummary),
      )
      .put("redactionLevel", "audit_meta_only")
      .put("retentionGranted", "audit_meta_only")
      .put("error", JSONObject.NULL)
  }

  override fun authorizationPolicy(request: JSONObject): JSONObject {
    val requestId = request.optString("requestId", "req-${UUID.randomUUID()}")
    val item = findPermissionItem(request)
      ?: return errorResponse(
        requestId = requestId,
        capability = "resolvePermissionObjectPolicy",
        code = "SLG-007",
        type = "object_not_found",
        message = "permission object was not found in permission summary",
        suggestedAction = "Use objectId, or resourceId plus role from /permission-summary.",
      )
    val requiredStrength = request.optString("requiredStrength", strengthForPermissionItem(item))
    val requiredGridCount = request.optInt("requiredGridCount", item.optInt("requiredGridCount", runtime.requiredCells(requiredStrength)))
    val action = request.optString("requestedAction", actionForPermissionItem(item))
    return JSONObject()
      .put("requestId", requestId)
      .put("status", "success")
      .put("capability", "resolvePermissionObjectPolicy")
      .put("executionLocation", "local")
      .put(
        "result",
        JSONObject()
          .put("identityId", request.optString("identityId", runtime.identityId()))
          .put("resourceId", item.optString("resourceId"))
          .put("role", item.optString("role"))
          .put("objectRef", item.optString("objectId"))
          .put("objectId", item.optString("objectId"))
          .put("objectKind", item.optString("objectKind"))
          .put("requestedAction", action)
          .put("requiredStrength", requiredStrength)
          .put("requiredGridCount", requiredGridCount)
          .put(
            "gridPolicy",
            JSONObject()
              .put("gridSize", maxOf(9, requiredGridCount))
              .put("requiredCells", requiredGridCount),
          )
          .put("displayName", item.optString("displayName")),
      )
      .put("redactionLevel", "audit_meta_only")
      .put("retentionGranted", "audit_meta_only")
      .put("error", JSONObject.NULL)
  }

  override fun verify(request: JSONObject): JSONObject {
    val requestId = request.optString("requestId", "req-${UUID.randomUUID()}")
    val policy = authorizationPolicy(request)
    if (policy.optString("status") != "success") {
      return policy
    }
    val result = policy.getJSONObject("result")
    val requiredStrength = result.getString("requiredStrength")
    val requiredGridCount = result.getInt("requiredGridCount")
    val challenge = try {
      runtime.createChallenge(requiredStrength, requiredGridCount)
    } catch (error: ChallengeLockedException) {
      return errorResponse(
        requestId = requestId,
        capability = "createGridVerification",
        code = "SLG-004",
        type = "challenge_locked",
        message = error.message ?: "challenge is locked",
        suggestedAction = "Wait until retryAfter before retrying the local challenge.",
        retryAfter = error.retryAfter,
      )
    }
    return JSONObject()
      .put("requestId", requestId)
      .put("status", "success")
      .put("capability", "createGridVerification")
      .put("executionLocation", "local")
      .put(
        "result",
        JSONObject()
          .put("verificationId", challenge.challengeId)
          .put("identityId", request.optString("identityId", runtime.identityId()))
          .put("objectRef", result.getString("objectRef"))
          .put("requiredStrength", requiredStrength)
          .put("grid", challengeSummary(challenge))
          .put("expiresAt", System.currentTimeMillis() + 5 * 60 * 1000L),
      )
      .put("redactionLevel", "audit_meta_only")
      .put("retentionGranted", "audit_meta_only")
      .put("error", JSONObject.NULL)
  }

  override fun authorize(request: JSONObject): JSONObject {
    val requestId = request.optString("requestId", "req-${UUID.randomUUID()}")
    val verificationId = request.optString("verificationId")
    if (verificationId.isBlank()) {
      return errorResponse(
        requestId = requestId,
        capability = "authorizeLocalAction",
        code = "SLG-001",
        type = "validation_error",
        message = "verificationId is required",
        suggestedAction = "Call /verify first, then submit answers with the returned verificationId.",
      )
    }
    val challenge = runtime.getChallenge(verificationId)
      ?: return errorResponse(
        requestId = requestId,
        capability = "authorizeLocalAction",
        code = "SLG-003",
        type = "challenge_failed",
        message = "challenge verification failed",
        suggestedAction = "Create a fresh grid verification and submit answers before it expires.",
      )
    val verification = runtime.verifyChallengeAnswers(challenge, answersFrom(request.optJSONArray("answers")))
    if (!verification.approved) {
      return errorResponse(
        requestId = requestId,
        capability = "authorizeLocalAction",
        code = if (verification.lockUntil > 0L) "SLG-004" else "SLG-003",
        type = if (verification.lockUntil > 0L) "challenge_locked" else "challenge_failed",
        message = "challenge answer verification failed",
        suggestedAction = "Retry with valid local challenge answers.",
        challenge = challenge,
        retryAfter = verification.lockUntil.takeIf { it > 0L },
        extraErrorFields = mapOf(
          "matchedCount" to verification.matchedCount,
          "requiredThreshold" to verification.requiredThreshold,
        ),
      )
    }
    val objectRef = request.optString("objectRef", request.optString("objectId", "android-object"))
    val allowedAction = request.optString("allowedAction", request.optString("requestedAction", "authorize"))
    val session = runtime.issueSession(allowedAction, objectRef)
    return JSONObject()
      .put("requestId", requestId)
      .put("status", "success")
      .put("capability", "authorizeLocalAction")
      .put("executionLocation", "local")
      .put(
        "result",
        JSONObject()
          .put("approved", true)
          .put("authorizationId", session.authorizationId)
          .put("identityId", request.optString("identityId", runtime.identityId()))
          .put("objectRef", objectRef)
          .put("allowedAction", allowedAction),
      )
      .put("redactionLevel", "audit_meta_only")
      .put("retentionGranted", "audit_meta_only")
      .put("error", JSONObject.NULL)
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
          .put("keyId", signingKey.getString("keyId")),
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
          .put("targetOrigin", credential.getString("targetOrigin"))
          .put("filled", true)
          .put("secretMaterial", "android-keystore-local-only"),
      )
      .put("redactionLevel", "audit_meta_only")
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

  private fun findPermissionItem(request: JSONObject): JSONObject? {
    val summary = request.optJSONObject("permissionSummary")
      ?: storyLockPackageRepository.loadPermissionSummary().permissionSummary
    val items = summary.optJSONArray("items") ?: JSONArray()
    val objectRef = request.optString("objectRef", request.optString("objectId", request.optString("keyId", request.optString("credentialRef"))))
    if (objectRef.isNotBlank()) {
      for (index in 0 until items.length()) {
        val item = items.optJSONObject(index) ?: continue
        if (item.optString("objectId") == objectRef) {
          return item
        }
      }
    }
    val resourceId = request.optString("resourceId")
    val role = request.optString("role")
    if (resourceId.isNotBlank() && role.isNotBlank()) {
      for (index in 0 until items.length()) {
        val item = items.optJSONObject(index) ?: continue
        if (item.optString("resourceId") == resourceId && item.optString("role") == role) {
          return item
        }
      }
    }
    return null
  }

  private fun strengthForPermissionItem(item: JSONObject): String {
    val policy = item.optString("challengePolicy")
    if (policy in setOf("low", "medium", "high", "story_edit")) {
      return policy
    }
    val requiredGridCount = item.optInt("requiredGridCount", 6)
    return when {
      requiredGridCount >= 22 -> "story_edit"
      requiredGridCount >= 12 -> "high"
      requiredGridCount <= 3 -> "low"
      else -> "medium"
    }
  }

  private fun actionForPermissionItem(item: JSONObject): String {
    return when (item.optString("action")) {
      "sign" -> "signature"
      "password_fill" -> "password_fill"
      "story_edit" -> "story_edit"
      "batch_read" -> "batch_read"
      else -> "authorize"
    }
  }

  private fun answersFrom(raw: JSONArray?): Map<String, String> {
    if (raw == null) {
      return emptyMap()
    }
    val answers = linkedMapOf<String, String>()
    for (index in 0 until raw.length()) {
      val value = raw.opt(index)
      if (value is JSONObject) {
        val answer = value.optString("answer")
        if (answer.isNotBlank()) {
          answers[value.optString("cellId", "cell-${index + 1}")] = answer
        }
      } else {
        val answer = value?.toString().orEmpty()
        if (answer.isNotBlank()) {
          answers["cell-${index + 1}"] = answer
        }
      }
    }
    return answers
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
