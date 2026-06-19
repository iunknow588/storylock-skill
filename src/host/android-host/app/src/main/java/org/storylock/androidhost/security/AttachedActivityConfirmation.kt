package org.storylock.androidhost.security

import androidx.fragment.app.FragmentActivity

class AttachedActivityConfirmation : LocalUserConfirmation {
  @Volatile
  private var activity: FragmentActivity? = null

  fun attach(activity: FragmentActivity) {
    this.activity = activity
  }

  override suspend fun confirm(request: LocalConfirmationRequest): LocalConfirmationResult {
    val current = activity ?: return LocalConfirmationResult(
      approved = false,
      reason = "no activity attached for local confirmation",
    )
    if (!request.challengePrompt.isNullOrBlank() && !request.challengeAnswer.isNullOrBlank()) {
      val challengeResult = ChallengePromptLocalUserConfirmation(current).confirm(
        prompt = request.challengePrompt,
        expectedAnswer = request.challengeAnswer,
      )
      if (!challengeResult.approved) {
        return challengeResult
      }
    }
    return BiometricPromptLocalUserConfirmation(current).confirm(request)
  }
}
