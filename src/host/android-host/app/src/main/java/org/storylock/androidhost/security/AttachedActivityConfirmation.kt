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
      failureType = "host_unavailable",
      reason = "no activity attached for local confirmation",
    )
    if (request.challengeItems.isNotEmpty()) {
      val challengeResult = ChallengePromptLocalUserConfirmation(current).confirm(
        title = request.title,
        challengeItems = request.challengeItems,
      )
      if (!challengeResult.approved) {
        return challengeResult
      }
    }
    return BiometricPromptLocalUserConfirmation(current).confirm(request)
  }
}
