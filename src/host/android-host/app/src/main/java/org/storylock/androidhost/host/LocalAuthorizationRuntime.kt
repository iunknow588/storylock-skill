package org.storylock.androidhost.host

import java.util.UUID
import java.util.concurrent.atomic.AtomicInteger

class LocalAuthorizationRuntime(
  private val identityId: String,
  private val questionSetVersion: String,
) {
  private val requestCount = AtomicInteger(0)
  private val challengeBits = 36
  private val questions = List(24) { index ->
    ChallengeCell(
      cellId = "cell-${index + 1}",
      questionId = "q-${index + 1}",
      promptText = "Question ${index + 1}: answer-${index + 1}",
      answer = "answer-${index + 1}",
    )
  }

  fun healthSnapshot(): Triple<Boolean, Int, Int> {
    return Triple(true, challengeBits, requestCount.get())
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

  fun createChallenge(requiredStrength: String): List<ChallengeCell> {
    val count = requiredCells(requiredStrength)
    return questions.take(count)
  }

  fun verifyChallengeAnswers(cells: List<ChallengeCell>, answers: Map<String, String>): Boolean {
    return cells.all { cell ->
      answers[cell.cellId]?.trim()?.equals(cell.answer, ignoreCase = true) == true
    }
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
  fun activeQuestionCount(): Int = questions.size
  fun identityId(): String = identityId
}
