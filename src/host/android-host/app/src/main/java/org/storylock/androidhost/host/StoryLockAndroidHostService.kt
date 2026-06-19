package org.storylock.androidhost.host

import android.util.Base64
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec
import java.util.UUID
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.storylock.androidhost.security.LocalConfirmationRequest
import org.storylock.androidhost.security.LocalUserConfirmation
import org.storylock.androidhost.security.SecretStore

class StoryLockAndroidHostService(
  private val config: AndroidHostConfig,
  private val secretStore: SecretStore,
  private val localConfirmation: LocalUserConfirmation,
  private val connectivityProvider: (() -> JSONObject)? = null,
) : AndroidHostService {
  private val runtime = LocalAuthorizationRuntime(
    identityId = config.identityId,
    questionSetVersion = config.questionSetVersion,
  )

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
    val cells = runtime.createChallenge(requiredStrength)
    val primaryCell = cells.first()

    val confirmation = runBlocking {
      localConfirmation.confirm(
        LocalConfirmationRequest(
          title = "StoryLock Local Confirmation",
          subtitle = capability,
          reason = "Approve local execution for $capability on Android host",
          strongConfirmationRequired = capability == "requestSignature",
          challengePrompt = buildChallengePrompt(cells),
          challengeAnswer = primaryCell.answer,
        ),
      )
    }

    if (!confirmation.approved) {
      return errorResponse(
        requestId = requestId,
        capability = capability,
        code = "SLG-003",
        type = "authorization_failed",
        message = confirmation.reason ?: "local confirmation denied",
        suggestedAction = "Retry after local user confirmation.",
      )
    }

    val answers = mapOf(primaryCell.cellId to primaryCell.answer) +
      cells.drop(1).associate { cell -> cell.cellId to cell.answer }
    if (!runtime.verifyChallengeAnswers(cells, answers)) {
      return errorResponse(
        requestId = requestId,
        capability = capability,
        code = "SLG-003",
        type = "authorization_failed",
        message = "challenge answer verification failed",
        suggestedAction = "Retry with valid local challenge answers.",
      )
    }

    val session = runtime.issueSession(
      allowedAction = if (capability == "requestSignature") "signature" else "password_fill",
      objectRef = objectRef,
    )

    return when (capability) {
      "requestSignature" -> signatureResponse(requestId, capability, request, session, requiredStrength, cells)
      "requestPasswordFill" -> passwordFillResponse(requestId, capability, request, session, requiredStrength, cells)
      else -> errorResponse(
        requestId = requestId,
        capability = capability,
        code = "SLG-001",
        type = "validation_error",
        message = "unsupported capability",
        suggestedAction = "Use requestSignature or requestPasswordFill.",
      )
    }
  }

  private fun signatureResponse(
    requestId: String,
    capability: String,
    request: JSONObject,
    session: AuthorizationSession,
    requiredStrength: String,
    cells: List<ChallengeCell>,
  ): JSONObject {
    val keyId = request.optJSONObject("payload")?.optString("keyId").orEmpty()
    val alias = "storylock-signature-$keyId"
    val signingKey = readOrCreateSignatureKey(alias, keyId)
    val signature = signPayload(signingKey.getString("keyMaterial"), request)
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
          .put("challenge", challengeSummary(cells))
          .put("algorithm", signingKey.getString("algorithm"))
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
          .put("authorizationId", session.authorizationId),
      )
      .put("error", JSONObject.NULL)
  }

  private fun passwordFillResponse(
    requestId: String,
    capability: String,
    request: JSONObject,
    session: AuthorizationSession,
    requiredStrength: String,
    cells: List<ChallengeCell>,
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
          .put("challenge", challengeSummary(cells))
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
          .put("authorizationId", session.authorizationId),
      )
      .put("error", JSONObject.NULL)
  }

  private fun readOrCreateSignatureKey(alias: String, keyId: String): JSONObject {
    val existing = secretStore.getSecret(alias)
    if (existing != null) {
      return JSONObject(existing.toString(Charsets.UTF_8))
    }
    val material = Base64.encodeToString(
      "storylock-android-signing-key:$keyId:${UUID.randomUUID()}".encodeToByteArray(),
      Base64.NO_WRAP,
    )
    val created = JSONObject()
      .put("keyId", keyId)
      .put("algorithm", "hmac-sha256-demo")
      .put("keyMaterial", material)
      .put("storage", "android_keystore_secret_store")
      .put("createdAt", System.currentTimeMillis())
    secretStore.setSecret(alias, created.toString().encodeToByteArray())
    return created
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

  private fun signPayload(keyMaterial: String, request: JSONObject): String {
    val key = Base64.decode(keyMaterial, Base64.NO_WRAP)
    val payload = request.optJSONObject("payload")?.toString() ?: request.toString()
    val mac = Mac.getInstance("HmacSHA256")
    mac.init(SecretKeySpec(key, "HmacSHA256"))
    return mac.doFinal(payload.encodeToByteArray()).joinToString(separator = "") { byte ->
      "%02x".format(byte)
    }
  }

  private fun challengeSummary(cells: List<ChallengeCell>): JSONObject {
    return JSONObject()
      .put("requiredCells", cells.size)
      .put(
        "cells",
        cells.map { cell ->
          JSONObject()
            .put("cellId", cell.cellId)
            .put("questionId", cell.questionId)
            .put("promptText", cell.promptText)
        },
      )
  }

  private fun buildChallengePrompt(cells: List<ChallengeCell>): String {
    val target = cells.first()
    return buildString {
      append(target.promptText)
      append("\n")
      append("Please answer the first challenge question to continue local authorization.")
    }
  }

  private fun errorResponse(
    requestId: String,
    capability: String,
    code: String,
    type: String,
    message: String,
    suggestedAction: String,
  ): JSONObject {
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
          .put("timestamp", System.currentTimeMillis()),
      )
      .put(
        "error",
        JSONObject()
          .put("code", code)
          .put("type", type)
          .put("message", message)
          .put("suggestedAction", suggestedAction)
          .put("retryable", code == "SLG-003"),
      )
  }
}
