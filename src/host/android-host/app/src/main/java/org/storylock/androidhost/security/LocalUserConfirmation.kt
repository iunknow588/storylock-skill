package org.storylock.androidhost.security

data class LocalConfirmationRequest(
  val title: String,
  val subtitle: String,
  val reason: String,
  val strongConfirmationRequired: Boolean,
  val challengePrompt: String? = null,
  val challengeAnswer: String? = null,
)

data class LocalConfirmationResult(
  val approved: Boolean,
  val reason: String? = null,
)

interface LocalUserConfirmation {
  suspend fun confirm(request: LocalConfirmationRequest): LocalConfirmationResult
}
