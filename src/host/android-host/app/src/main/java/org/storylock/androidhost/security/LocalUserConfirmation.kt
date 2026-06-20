package org.storylock.androidhost.security

data class LocalConfirmationRequest(
  val title: String,
  val subtitle: String,
  val reason: String,
  val strongConfirmationRequired: Boolean,
  val challengeItems: List<LocalChallengeItem> = emptyList(),
  val requiredChallengeAnswers: Int = 0,
)

data class LocalChallengeItem(
  val cellId: String,
  val questionId: String,
  val promptText: String,
  val expectedAnswer: String,
  val position: Int,
)

data class LocalConfirmationResult(
  val approved: Boolean,
  val failureType: String? = null,
  val reason: String? = null,
  val challengeAnswers: Map<String, String> = emptyMap(),
)

interface LocalUserConfirmation {
  suspend fun confirm(request: LocalConfirmationRequest): LocalConfirmationResult
}
