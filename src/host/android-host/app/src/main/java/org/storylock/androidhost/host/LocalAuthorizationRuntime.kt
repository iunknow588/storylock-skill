package org.storylock.androidhost.host

import java.util.UUID
import java.util.concurrent.atomic.AtomicInteger
class LocalAuthorizationRuntime(
  private val identityId: String,
  private val questionSetVersion: String,
  private val normalizationVersion: String,
  private val questions: List<ChallengeCell>,
) {
  private val maxFailuresPerWindow = 3
  private val failureLockMs = 15 * 60 * 1000L
  private val requestCount = AtomicInteger(0)
  private val challengeBits = 36
  private val failureCount = AtomicInteger(0)
  @Volatile
  private var lockUntilMs = 0L
  fun healthSnapshot(): Triple<Boolean, Int, Int> {
    return Triple(questions.isNotEmpty(), challengeBits, requestCount.get())
  }

  fun resolveRequiredStrength(capability: String): String {
    return if (capability == "requestSignature") "high" else "medium"
  }

  fun requiredCells(requiredStrength: String): Int {
    return when (requiredStrength) {
      "high" -> 9
      "medium" -> 6
      else -> 3
    }
  }

  fun createChallenge(requiredStrength: String): ChallengeSession {
    if (isLocked()) {
      throw ChallengeLockedException(lockUntilMs)
    }
    val count = requiredCells(requiredStrength)
    val rotation = requestCount.get() % questions.size
    val selected = (questions.drop(rotation) + questions.take(rotation))
      .take(count)
      .mapIndexed { index, cell ->
        cell.copy(position = index + 1)
      }
    return ChallengeSession(
      challengeId = "chl-${UUID.randomUUID()}",
      requiredStrength = requiredStrength,
      requiredCells = count,
      requiredThreshold = count,
      questionSetVersion = questionSetVersion,
      failureCount = currentFailureCount(),
      maxFailureCount = maxFailuresPerWindow,
      lockUntil = lockUntilMs,
      cells = selected,
    )
  }

  fun verifyChallengeAnswers(challenge: ChallengeSession, answers: Map<String, String>): ChallengeVerificationResult {
    val matchedCount = challenge.cells.count { cell ->
      answers[cell.cellId]?.trim()?.equals(cell.answer, ignoreCase = true) == true
    }
    val approved = matchedCount >= challenge.requiredThreshold
    if (approved) {
      failureCount.set(0)
      lockUntilMs = 0L
      return ChallengeVerificationResult(
        approved = true,
        matchedCount = matchedCount,
        requiredThreshold = challenge.requiredThreshold,
        failureCount = 0,
        maxFailureCount = maxFailuresPerWindow,
        lockUntil = 0L,
      )
    }
    val nextFailureCount = failureCount.incrementAndGet()
    if (nextFailureCount >= maxFailuresPerWindow) {
      lockUntilMs = System.currentTimeMillis() + failureLockMs
    }
    return ChallengeVerificationResult(
      approved = false,
      matchedCount = matchedCount,
      requiredThreshold = challenge.requiredThreshold,
      failureCount = nextFailureCount,
      maxFailureCount = maxFailuresPerWindow,
      lockUntil = lockUntilMs,
    )
  }

  fun issueSession(allowedAction: String, objectRef: String): AuthorizationSession {
    requestCount.incrementAndGet()
    return AuthorizationSession(
      authorizationId = "ses-${UUID.randomUUID()}",
      allowedAction = allowedAction,
      objectRef = objectRef,
    )
  }

  fun questionSetVersion(): String = questionSetVersion
  fun normalizationVersion(): String = normalizationVersion
  fun activeQuestionCount(): Int = questions.size
  fun identityId(): String = identityId

  private fun currentFailureCount(): Int {
    if (isLocked()) {
      return failureCount.get()
    }
    if (lockUntilMs > 0L && System.currentTimeMillis() >= lockUntilMs) {
      failureCount.set(0)
      lockUntilMs = 0L
    }
    return failureCount.get()
  }

  private fun isLocked(): Boolean {
    if (lockUntilMs <= 0L) {
      return false
    }
    if (System.currentTimeMillis() >= lockUntilMs) {
      failureCount.set(0)
      lockUntilMs = 0L
      return false
    }
    return true
  }
}

class ChallengeLockedException(
  val retryAfter: Long,
) : IllegalStateException("challenge is locked")
